use std::collections::{hash_map::Entry, HashMap};

use pyo3::{pyclass, pymethods, Py, PyAny, Python};
use pyo3_async_runtimes::TaskLocals;
use tokio_postgres::Notification;

use crate::{
    driver::connection::Connection, exceptions::rust_errors::RustPSQLDriverPyResult, runtime::tokio_runtime
};


pub struct ChannelCallbacks(HashMap<String, Vec<ListenerCallback>>);

impl Default for ChannelCallbacks {
    fn default() -> Self {
        ChannelCallbacks(Default::default())
    }
}

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

    pub fn retrieve_channel_callbacks(&self, channel: String) -> Option<&Vec<ListenerCallback>> {
        self.0.get(&channel)
    }

    pub fn clear_channel_callbacks(&mut self, channel: String) {
        self.0.remove(&channel);
    }

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

impl From::<Notification> for ListenerNotification {
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
    pub fn new(value: ListenerNotification, conn: Connection) -> Self {
        ListenerNotificationMsg {
            process_id: value.process_id,
            channel: String::from(value.channel),
            payload: String::from(value.payload),
            connection: conn,
        }
    }
}

pub struct ListenerCallback {
    task_locals: Option<TaskLocals>,
    callback: Py<PyAny>,
}

impl ListenerCallback {
    pub fn new(
        task_locals: Option<TaskLocals>,
        callback: Py<PyAny>,
    ) -> Self {
        ListenerCallback {
            task_locals,
            callback,
        }
    }

    pub async fn call(
        &self,
        lister_notification: ListenerNotification,
        connection: Connection,
    ) -> RustPSQLDriverPyResult<()> {
        let (callback, task_locals) = Python::with_gil(|py| {
            if let Some(task_locals) = &self.task_locals {
                return (self.callback.clone(), Some(task_locals.clone_ref(py)));
            }
            (self.callback.clone(), None)
        });
        
        if let Some(task_locals) = task_locals {
            tokio_runtime().spawn(pyo3_async_runtimes::tokio::scope(task_locals, async move {
                let future = Python::with_gil(|py| {
                    let awaitable = callback.call1(
                        py,
                        (
                            lister_notification.channel,
                            lister_notification.payload,
                            lister_notification.process_id,
                            connection,
                        )
                    ).unwrap();
                    pyo3_async_runtimes::tokio::into_future(awaitable.into_bound(py)).unwrap()
                });
                future.await.unwrap();
            })).await?;
        };

        Ok(())
    }
}
