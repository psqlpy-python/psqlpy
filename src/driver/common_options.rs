use std::time::Duration;

use deadpool_postgres::RecyclingMethod;
use pyo3::{pyclass, pymethods};

#[pyclass]
#[derive(Clone, Copy)]
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

#[pyclass]
#[derive(Clone, Copy)]
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

#[pyclass]
#[derive(Clone, Copy)]
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

#[pyclass]
#[derive(Clone, Copy)]
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
    fn build_config(idle: u64, interval: Option<u64>, retries: Option<u32>) -> Self {
        let interval_internal = interval.map(Duration::from_secs);
        KeepaliveConfig {
            idle: Duration::from_secs(idle),
            interval: interval_internal,
            retries,
        }
    }
}
