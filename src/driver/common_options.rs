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
