use deadpool_postgres::RecyclingMethod;
use pyo3::pyclass;

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
