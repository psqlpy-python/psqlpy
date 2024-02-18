use pyo3::pyclass;

#[pyclass]
#[derive(Clone, Copy)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
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

#[pyclass]
#[derive(Clone, Copy)]
pub enum ReadVariant {
    ReadOnly,
    ReadWrite,
}

impl ReadVariant {
    /// Return Read variant as String literal.
    pub fn to_str_option(&self) -> String {
        match self {
            ReadVariant::ReadOnly => "READ ONLY".into(),
            ReadVariant::ReadWrite => "READ WRITE".into(),
        }
    }
}
