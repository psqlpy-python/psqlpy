use deadpool_postgres::RecyclingMethod;
use pyo3::pyclass;
use tokio_postgres::config::{LoadBalanceHosts, TargetSessionAttrs};

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
pub enum ConnLoadBalanceHosts {
    /// Make connection attempts to hosts in the order provided.
    Disable,
    /// Make connection attempts to hosts in a random order.
    Random,
}

impl ConnLoadBalanceHosts {
    #[must_use]
    pub fn to_internal(&self) -> LoadBalanceHosts {
        match self {
            ConnLoadBalanceHosts::Disable => LoadBalanceHosts::Disable,
            ConnLoadBalanceHosts::Random => LoadBalanceHosts::Random,
        }
    }
}

#[pyclass]
#[derive(Clone, Copy)]
pub enum ConnTargetSessionAttrs {
    /// No special properties are required.
    Any,
    /// The session must allow writes.
    ReadWrite,
    /// The session allow only reads.
    ReadOnly,
}

impl ConnTargetSessionAttrs {
    #[must_use]
    pub fn to_internal(&self) -> TargetSessionAttrs {
        match self {
            ConnTargetSessionAttrs::Any => TargetSessionAttrs::Any,
            ConnTargetSessionAttrs::ReadWrite => TargetSessionAttrs::ReadWrite,
            ConnTargetSessionAttrs::ReadOnly => TargetSessionAttrs::ReadOnly,
        }
    }
}
