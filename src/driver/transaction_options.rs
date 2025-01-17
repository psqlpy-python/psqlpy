use pyo3::pyclass;

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq)]
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
#[derive(Clone, Copy, PartialEq)]
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
    /// If the PostgreSQL instance crashes,
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
