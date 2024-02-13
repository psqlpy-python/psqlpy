use pyo3::pyclass;

#[pyclass]
#[derive(Clone, Copy)]
pub enum IsolationLevel {
    ReadUncommitted = 1,
    ReadCommitted = 2,
    RepeatableRead = 3,
    Serializable = 4,
}

impl IsolationLevel {
    /// Return isolation level as String literal.
    pub fn to_str_level(&self) -> String {
        match self {
            IsolationLevel::ReadUncommitted => "READ UNCOMMITTED".into(),
            IsolationLevel::ReadCommitted => "READ COMMITTED".into(),
            IsolationLevel::RepeatableRead => "REPEATABLE READ".into(),
            IsolationLevel::Serializable => "SERIALIZABLE".into(),
        }
    }
}
