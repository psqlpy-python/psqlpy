use deadpool::managed::{self, Metrics, RecycleResult};
use deadpool_postgres::{ClientWrapper, Manager, ManagerConfig, StatementCaches};
use postgres_openssl::MakeTlsConnector;
use tokio_postgres::{error::SqlState, Config as PgConfig, NoTls};

use crate::options::SslMode;

use super::utils::ConfiguredTLS;

/// `deadpool::managed::Pool` parameterized on [`PsqlpyManager`].
///
/// All `ConnectionPool` plumbing in psqlpy goes through this alias rather than
/// `deadpool_postgres::Pool` (which is hard-bound to
/// `deadpool_postgres::Manager`). Both share the same `ClientWrapper` object
/// type, so the rest of the `deadpool_postgres` ergonomics (`Client = Object`,
/// pool status, builder pattern, etc.) still works.
pub type PsqlpyPool = deadpool::managed::Pool<PsqlpyManager>;

/// Pool-checked-out client (`Object<PsqlpyManager>`). Replaces the
/// `deadpool_postgres::Object` / `deadpool_postgres::Client` alias at the
/// boundaries that handed those out (`PoolConnection`, `retrieve_connection`,
/// `listener.start`) — same underlying `ClientWrapper`, different generic
/// witness.
pub type PsqlpyClient = deadpool::managed::Object<PsqlpyManager>;

/// Connection manager that owns a `deadpool_postgres::Manager` for the primary
/// TLS configuration and, only for `SslMode::Allow`, a second manager for the
/// plaintext fallback path.
///
/// The two-manager shape is what makes psqlpy's `Allow` mode libpq-faithful.
/// libpq tries plaintext first and silently retries over TLS when the server
/// rejects the plaintext attempt with the `INVALID_AUTHORIZATION_SPECIFICATION`
/// `no encryption` diagnostic pair. We mirror that here. The `primary`
/// manager is the plaintext (`SslMode::Disable` `NoTls`) side, and the
/// `tls_fallback` manager is the TLS side that picks up when the server
/// requires encryption. For every other `SslMode`, `tls_fallback` is `None`
/// and `create()` simply delegates straight to `primary`.
#[derive(Debug)]
pub struct PsqlpyManager {
    primary: Manager,
    tls_fallback: Option<Manager>,
}

impl PsqlpyManager {
    /// Expose the primary manager's `StatementCaches` for the legacy
    /// `pool.manager().statement_caches.remove(...)` call sites that predate
    /// this wrapper. The fallback manager's cache (if any) is intentionally
    /// not surfaced here — Allow-mode TLS fallback connections are rare and
    /// transient, and double-bookkeeping a second cache for them would only
    /// matter if a caller invalidated a prepared statement and we wanted that
    /// invalidation to ride through both inner caches.
    #[must_use]
    pub fn statement_caches(&self) -> &StatementCaches {
        &self.primary.statement_caches
    }
}

impl PsqlpyManager {
    /// Plain wrapper: only the primary manager, no Allow-style retry.
    pub fn single(primary: Manager) -> Self {
        Self {
            primary,
            tls_fallback: None,
        }
    }

    /// Allow-mode wrapper: plaintext primary + TLS fallback. `primary` is
    /// expected to be a `Disable+NoTls` manager and `tls_fallback` a
    /// `Require+MakeTlsConnector` manager — the inverse pair would still
    /// type-check but would lose libpq-`Allow`'s plaintext-first semantics.
    pub fn with_tls_fallback(primary: Manager, tls_fallback: Manager) -> Self {
        Self {
            primary,
            tls_fallback: Some(tls_fallback),
        }
    }
}

/// Returns `true` when the error chain matches the postgres-side "this server
/// requires encryption" rejection that libpq's `sslmode=allow` is built
/// around.
///
/// `PostgreSQL` surfaces this as SQLSTATE `INVALID_AUTHORIZATION_SPECIFICATION`
/// (28000) with a message that contains the literal "no encryption" substring
/// — empirically present in `auth.c:ClientAuthentication` on pg 12–17. We walk
/// the `source()` chain so callers can pass a `PoolError`, `DeadpoolError`, or
/// raw `tokio_postgres::Error` and get the same answer.
#[must_use]
pub fn is_ssl_required_rejection(err: &(dyn std::error::Error + 'static)) -> bool {
    let mut current: Option<&(dyn std::error::Error + 'static)> = Some(err);
    while let Some(e) = current {
        if let Some(pg_err) = e.downcast_ref::<tokio_postgres::Error>() {
            if let Some(db_err) = pg_err.as_db_error() {
                if db_err.code() == &SqlState::INVALID_AUTHORIZATION_SPECIFICATION
                    && db_err.message().contains("no encryption")
                {
                    return true;
                }
            }
        }
        current = e.source();
    }
    false
}

impl managed::Manager for PsqlpyManager {
    type Type = ClientWrapper;
    type Error = tokio_postgres::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        match self.primary.create().await {
            Ok(client) => Ok(client),
            Err(err) => {
                if let Some(fallback) = &self.tls_fallback {
                    if is_ssl_required_rejection(&err) {
                        return fallback.create().await;
                    }
                }
                Err(err)
            }
        }
    }

    async fn recycle(
        &self,
        client: &mut Self::Type,
        metrics: &Metrics,
    ) -> RecycleResult<Self::Error> {
        // Recycling is TLS-agnostic — same SQL probe regardless of which inner
        // manager originally produced the client. Delegate to `primary` for
        // both branches; the `Allow` fallback only matters at create-time.
        self.primary.recycle(client, metrics).await
    }

    fn detach(&self, client: &mut Self::Type) {
        self.primary.detach(client);
    }
}

/// Build the two-manager pair an [`SslMode::Allow`] connection requires.
///
/// Caller supplies the base `pg_config` and `mgr_config` (recycling method
/// etc.) plus the already-built `MakeTlsConnector` to use on the TLS side.
/// Both inner configs are clones with only the `ssl_mode` field differing.
#[must_use]
pub fn build_allow_pair(
    mut pg_config: PgConfig,
    mgr_config: ManagerConfig,
    tls_connector: MakeTlsConnector,
) -> PsqlpyManager {
    let mut plaintext_config = pg_config.clone();
    plaintext_config.ssl_mode(tokio_postgres::config::SslMode::Disable);
    pg_config.ssl_mode(tokio_postgres::config::SslMode::Require);

    let plaintext = Manager::from_config(plaintext_config, NoTls, mgr_config.clone());
    let tls = Manager::from_config(pg_config, tls_connector, mgr_config);
    PsqlpyManager::with_tls_fallback(plaintext, tls)
}

/// Bridge from the legacy `(ManagerConfig, Config, ConfiguredTLS)` builder
/// shape to a [`PsqlpyManager`].
///
/// - `SslMode::Allow` + a TLS connector → `Allow`-pair (plaintext-first).
/// - `SslMode::Allow` + `NoTls` (no `ca_file`, no upstream cert) → degrades
///   to a plain `Disable` manager. Without a TLS connector there is no
///   meaningful retry target and libpq's behavior in that situation is itself
///   plaintext-only.
/// - Any other `SslMode` → straight pass-through to a single inner
///   `Manager`, no retry.
#[must_use]
pub fn build_psqlpy_manager(
    mgr_config: ManagerConfig,
    pg_config: PgConfig,
    configured_tls: ConfiguredTLS,
    ssl_mode: Option<SslMode>,
) -> PsqlpyManager {
    if matches!(ssl_mode, Some(SslMode::Allow)) {
        if let ConfiguredTLS::TlsConnector(connector) = configured_tls {
            return build_allow_pair(pg_config, mgr_config, connector);
        }
        // Allow + no TLS connector available: behaves as Disable.
        return PsqlpyManager::single(Manager::from_config(pg_config, NoTls, mgr_config));
    }

    let inner = match configured_tls {
        ConfiguredTLS::NoTls => Manager::from_config(pg_config, NoTls, mgr_config),
        ConfiguredTLS::TlsConnector(connector) => {
            Manager::from_config(pg_config, connector, mgr_config)
        }
    };
    PsqlpyManager::single(inner)
}
