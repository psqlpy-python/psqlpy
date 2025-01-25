use std::collections::{hash_map::Entry, HashMap};

use pyo3::{pyclass, pymethods, Py, PyAny, Python};
use pyo3_async_runtimes::TaskLocals;
use tokio_postgres::Notification;

use crate::{
    driver::connection::Connection,
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    runtime::tokio_runtime,
};

#[derive(Default)]
pub struct ChannelCallbacks(HashMap<String, Vec<ListenerCallback>>);

impl ChannelCallbacks {
    pub fn add_callback(&mut self, channel: String, callback: ListenerCallback) {
        match self.0.entry(channel) {
            Entry::Vacant(e) => {
                e.insert(vec![callback]);
            }
            Entry::Occupied(mut e) => {
                e.get_mut().push(callback);
            }
        };
    }

    #[must_use]
    pub fn retrieve_channel_callbacks(&self, channel: &str) -> Option<&Vec<ListenerCallback>> {
        self.0.get(channel)
    }

    pub fn clear_channel_callbacks(&mut self, channel: &str) {
        self.0.remove(channel);
    }

    pub fn clear_all(&mut self) {
        self.0.clear();
    }

    #[must_use]
    pub fn retrieve_all_channels(&self) -> Vec<&String> {
        self.0.keys().collect::<Vec<&String>>()
    }
}

#[derive(Clone, Debug)]
pub struct ListenerNotification {
    pub process_id: i32,
    pub channel: String,
    pub payload: String,
}

impl From<Notification> for ListenerNotification {
    fn from(value: Notification) -> Self {
        ListenerNotification {
            process_id: value.process_id(),
            channel: String::from(value.channel()),
            payload: String::from(value.payload()),
        }
    }
}

#[pyclass]
pub struct ListenerNotificationMsg {
    process_id: i32,
    channel: String,
    payload: String,
    connection: Connection,
}

#[pymethods]
impl ListenerNotificationMsg {
    #[getter]
    fn process_id(&self) -> i32 {
        self.process_id
    }

    #[getter]
    fn channel(&self) -> String {
        self.channel.clone()
    }

    #[getter]
    fn payload(&self) -> String {
        self.payload.clone()
    }

    #[getter]
    fn connection(&self) -> Connection {
        self.connection.clone()
    }
}

impl ListenerNotificationMsg {
    #[must_use]
    pub fn new(value: ListenerNotification, conn: Connection) -> Self {
        ListenerNotificationMsg {
            process_id: value.process_id,
            channel: value.channel,
            payload: value.payload,
            connection: conn,
        }
    }
}

pub struct ListenerCallback {
    task_locals: TaskLocals,
    callback: Py<PyAny>,
}

impl ListenerCallback {
    #[must_use]
    pub fn new(task_locals: TaskLocals, callback: Py<PyAny>) -> Self {
        ListenerCallback {
            task_locals,
            callback,
        }
    }

    /// Dispatch the callback.
    ///
    /// # Errors
    /// May return Err Result if cannot call python future.
    pub async fn call(
        &self,
        lister_notification: ListenerNotification,
        connection: Connection,
    ) -> RustPSQLDriverPyResult<()> {
        let (callback, task_locals) =
            Python::with_gil(|py| (self.callback.clone(), self.task_locals.clone_ref(py)));

        tokio_runtime()
            .spawn(pyo3_async_runtimes::tokio::scope(task_locals, async move {
                let future = Python::with_gil(|py| {
                    let awaitable = callback
                        .call1(
                            py,
                            (
                                connection,
                                lister_notification.payload,
                                lister_notification.channel,
                                lister_notification.process_id,
                            ),
                        )
                        .map_err(|_| RustPSQLDriverError::ListenerCallbackError)?;
                    let aba = pyo3_async_runtimes::tokio::into_future(awaitable.into_bound(py))?;
                    Ok(aba)
                });
                Ok::<Py<PyAny>, RustPSQLDriverError>(
                    future
                        .map_err(|_: RustPSQLDriverError| {
                            RustPSQLDriverError::ListenerCallbackError
                        })?
                        .await?,
                )
            }))
            .await??;

        Ok(())
    }
}
