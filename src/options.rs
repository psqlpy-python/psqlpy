use std::time::Duration;

use deadpool_postgres::RecyclingMethod;
use pyo3::{pyclass, pymethods};

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq)]
pub enum ConnRecyclingMethod {
    Fast,
    Verified,
    Clean,
}

impl ConnRecyclingMethod {
    #[must_use]
    pub fn to_internal(&self) -> RecyclingMethod {
        match self {
            ConnRecyclingMethod::Fast => RecyclingMethod::Fast,
            ConnRecyclingMethod::Verified => RecyclingMethod::Verified,
            ConnRecyclingMethod::Clean => RecyclingMethod::Clean,
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq)]
pub enum LoadBalanceHosts {
    /// Make connection attempts to hosts in the order provided.
    Disable,
    /// Make connection attempts to hosts in a random order.
    Random,
}

impl LoadBalanceHosts {
    #[must_use]
    pub fn to_internal(&self) -> tokio_postgres::config::LoadBalanceHosts {
        match self {
            LoadBalanceHosts::Disable => tokio_postgres::config::LoadBalanceHosts::Disable,
            LoadBalanceHosts::Random => tokio_postgres::config::LoadBalanceHosts::Random,
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq)]
pub enum TargetSessionAttrs {
    /// No special properties are required.
    Any,
    /// The session must allow writes.
    ReadWrite,
    /// The session allow only reads.
    ReadOnly,
}

impl TargetSessionAttrs {
    #[must_use]
    pub fn to_internal(&self) -> tokio_postgres::config::TargetSessionAttrs {
        match self {
            TargetSessionAttrs::Any => tokio_postgres::config::TargetSessionAttrs::Any,
            TargetSessionAttrs::ReadWrite => tokio_postgres::config::TargetSessionAttrs::ReadWrite,
            TargetSessionAttrs::ReadOnly => tokio_postgres::config::TargetSessionAttrs::ReadOnly,
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SslMode {
    /// Do not use TLS.
    Disable,
    /// Pay the overhead of encryption if the server insists on it.
    Allow,
    /// Attempt to connect with TLS but allow sessions without.
    Prefer,
    /// Require the use of TLS.
    Require,
    /// I want my data encrypted,
    /// and I accept the overhead.
    /// I want to be sure that I connect to a server that I trust.
    VerifyCa,
    /// I want my data encrypted,
    /// and I accept the overhead.
    /// I want to be sure that I connect to a server I trust,
    /// and that it's the one I specify.
    VerifyFull,
}

impl SslMode {
    #[must_use]
    pub fn to_internal(&self) -> tokio_postgres::config::SslMode {
        match self {
            SslMode::Disable => tokio_postgres::config::SslMode::Disable,
            SslMode::Allow => tokio_postgres::config::SslMode::Allow,
            SslMode::Prefer => tokio_postgres::config::SslMode::Prefer,
            SslMode::Require => tokio_postgres::config::SslMode::Require,
            SslMode::VerifyCa => tokio_postgres::config::SslMode::VerifyCa,
            SslMode::VerifyFull => tokio_postgres::config::SslMode::VerifyFull,
        }
    }
}

#[pyclass]
#[derive(Clone, Copy)]
pub struct KeepaliveConfig {
    pub idle: Duration,
    pub interval: Option<Duration>,
    pub retries: Option<u32>,
}

#[pymethods]
impl KeepaliveConfig {
    #[new]
    #[pyo3(signature = (idle, interval=None, retries=None))]
    fn build_config(idle: u64, interval: Option<u64>, retries: Option<u32>) -> Self {
        let interval_internal = interval.map(Duration::from_secs);
        KeepaliveConfig {
            idle: Duration::from_secs(idle),
            interval: interval_internal,
            retries,
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq)]
pub enum CopyCommandFormat {
    TEXT,
    CSV,
    BINARY,
}

impl CopyCommandFormat {
    #[must_use]
    pub fn to_internal(&self) -> String {
        match self {
            CopyCommandFormat::TEXT => "text".into(),
            CopyCommandFormat::CSV => "csv".into(),
            CopyCommandFormat::BINARY => "binary".into(),
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl IsolationLevel {
    /// Return isolation level as String literal.
    #[must_use]
    pub fn to_str_level(&self) -> String {
        match self {
            IsolationLevel::ReadUncommitted => "READ UNCOMMITTED".into(),
            IsolationLevel::ReadCommitted => "READ COMMITTED".into(),
            IsolationLevel::RepeatableRead => "REPEATABLE READ".into(),
            IsolationLevel::Serializable => "SERIALIZABLE".into(),
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ReadVariant {
    ReadOnly,
    ReadWrite,
}

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq)]
pub enum SynchronousCommit {
    /// As the name indicates, the commit acknowledgment can come before
    /// flushing the records to disk.
    /// This is generally called as an asynchronous commit.
    /// If the `PostgreSQL` instance crashes,
    /// the last few asynchronous commits might be lost.
    Off,
    /// WAL records are written and flushed to local disks.
    /// In this case, the commit will be acknowledged after the
    /// local WAL Write and WAL flush completes.
    Local,
    /// WAL records are successfully handed over to
    /// remote instances which acknowledged back
    /// about the write (not flush).
    RemoteWrite,
    /// The meaning may change based on whether you have
    /// a synchronous standby or not.
    /// If there is a synchronous standby,
    /// setting the value to on will result in waiting till “remote flush”.
    On,
    /// This will result in commits waiting until replies from the
    /// current synchronous standby(s) indicate they have received
    /// the commit record of the transaction and applied it so
    /// that it has become visible to queries on the standby(s).
    RemoteApply,
}

impl SynchronousCommit {
    /// Return isolation level as String literal.
    #[must_use]
    pub fn to_str_level(&self) -> String {
        match self {
            SynchronousCommit::Off => "off".into(),
            SynchronousCommit::Local => "local".into(),
            SynchronousCommit::RemoteWrite => "remote_write".into(),
            SynchronousCommit::On => "on".into(),
            SynchronousCommit::RemoteApply => "remote_apply".into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct ListenerTransactionConfig {
    isolation_level: Option<IsolationLevel>,
    read_variant: Option<ReadVariant>,
    deferrable: Option<bool>,
    synchronous_commit: Option<SynchronousCommit>,
}
