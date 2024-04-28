use deadpool_postgres::RecyclingMethod;
use pyo3::pyclass;
use tokio_postgres::config::LoadBalanceHosts;

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
