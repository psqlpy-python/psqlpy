use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use futures::{stream, FutureExt, StreamExt, TryStreamExt};
use futures_channel::mpsc::UnboundedReceiver;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres_openssl::MakeTlsConnector;
use pyo3::{pyclass, pymethods, Py, PyAny, PyErr, PyObject, Python};
use pyo3_async_runtimes::TaskLocals;
use tokio::{sync::RwLock, task::{AbortHandle, JoinHandle}};
use tokio_postgres::{AsyncMessage, Config, Notification};

use crate::{
    driver::utils::is_coroutine_function, exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult}, runtime::{rustdriver_future, tokio_runtime}
};

use super::{
    common_options::SslMode, connection::{Connection, InnerConnection}, utils::{build_tls, ConfiguredTLS}
};

struct ChannelCallbacks(HashMap<String, Vec<ListenerCallback>>);

impl Default for ChannelCallbacks {
    fn default() -> Self {
        ChannelCallbacks(Default::default())
    }
}

impl ChannelCallbacks {
    fn add_callback(&mut self, channel: String, callback: ListenerCallback) {
        match self.0.entry(channel) {
            Entry::Vacant(e) => {
                e.insert(vec![callback]);
            }
            Entry::Occupied(mut e) => {
                e.get_mut().push(callback);
            }
        };
    }

    fn retrieve_channel_callbacks(&self, channel: String) -> Option<&Vec<ListenerCallback>> {
        self.0.get(&channel)
    }

    fn clear_channel_callbacks(&mut self, channel: String) {
        self.0.remove(&channel);
    }

    fn retrieve_all_channels(&self) -> Vec<&String> {
        self.0.keys().collect::<Vec<&String>>()
    }
}


#[derive(Clone, Debug)]
#[pyclass]
pub struct ListenerNotification {
    process_id: i32,
    channel: String,
    payload: String,
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
struct ListenerNotificationMsg {
    process_id: i32,
    channel: String,
    payload: String,
    connection: Connection,
}

struct ListenerCallback {
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

    async fn call(
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
                    let awaitable = callback.call1(py, (lister_notification, connection)).unwrap();
                    pyo3_async_runtimes::tokio::into_future(awaitable.into_bound(py)).unwrap()
                });
                future.await.unwrap();
            })).await?;
        };

        Ok(())
    }
}

#[pyclass]
pub struct Listener {
    pg_config: Config,
    ca_file: Option<String>,
    ssl_mode: Option<SslMode>,
    channel_callbacks: Arc<RwLock<ChannelCallbacks>>,
    listen_abort_handler: Option<AbortHandle>,
    connection: Connection,
    receiver: Option<Arc<RwLock<UnboundedReceiver<AsyncMessage>>>>,
    listen_query: Arc<RwLock<String>>,
    is_listened: Arc<RwLock<bool>>,
    is_started: bool,
}

impl Listener {
    #[must_use] pub fn new(
        pg_config: Config,
        ca_file: Option<String>,
        ssl_mode: Option<SslMode>,
    ) -> Self {
        Listener {
            pg_config,
            ca_file,
            ssl_mode,
            channel_callbacks: Default::default(),
            listen_abort_handler: Default::default(),
            connection: Connection::new(None, None),
            receiver: Default::default(),
            listen_query: Default::default(),
            is_listened: Arc::new(RwLock::new(false)),
            is_started: false,
        }
    }

    async fn update_listen_query(&self) -> () {
        let read_channel_callbacks = self.channel_callbacks.read().await;

        let channels = read_channel_callbacks.retrieve_all_channels();

        let mut final_query: String = String::default();

        for channel_name in channels {
            final_query.push_str(
                format!("LISTEN {};", channel_name).as_str()
            );
        }

        let mut write_listen_query = self.listen_query.write().await;
        let mut write_is_listened = self.is_listened.write().await;

        write_listen_query.clear();
        write_listen_query.push_str(&final_query);
        *write_is_listened = false;
    }
}

#[pymethods]
impl Listener {
    #[must_use]
    fn __aiter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __await__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    async fn __aenter__<'a>(slf: Py<Self>) -> RustPSQLDriverPyResult<Py<Self>> {
        Ok(slf)
    }

    async fn __aexit__<'a>(
        slf: Py<Self>,
        _exception_type: Py<PyAny>,
        exception: Py<PyAny>,
        _traceback: Py<PyAny>,
    ) -> RustPSQLDriverPyResult<()> {
        let (client, is_exception_none, py_err) =
            pyo3::Python::with_gil(|gil| {
                let self_ = slf.borrow(gil);
                (
                    self_.connection.db_client(),
                    exception.is_none(gil),
                    PyErr::from_value_bound(exception.into_bound(gil)),
                )
            });

        if client.is_some() {
            pyo3::Python::with_gil(|gil| {
                let mut self_ = slf.borrow_mut(gil);
                std::mem::take(&mut self_.connection);
                std::mem::take(&mut self_.receiver);
            });

            if !is_exception_none {
                return Err(RustPSQLDriverError::RustPyError(py_err));
            }

            return Ok(());
        }

        return Err(RustPSQLDriverError::ListenerClosedError)
    }

    #[getter]
    fn connection(&self) -> Connection {
        self.connection.clone()
    }

    async fn startup(&mut self) -> RustPSQLDriverPyResult<()> {
        if self.is_started {
            return Err(RustPSQLDriverError::ListenerStartError(
                "Listener is already started".into(),
            ));
        }

        let tls_ = build_tls(&self.ca_file.clone(), self.ssl_mode)?;

        let mut builder = SslConnector::builder(SslMethod::tls())?;
        builder.set_verify(SslVerifyMode::NONE);

        let pg_config = self.pg_config.clone();
        let connect_future = async move {
            match tls_ {
                ConfiguredTLS::NoTls => {
                    return pg_config
                        .connect(MakeTlsConnector::new(builder.build()))
                        .await;
                }
                ConfiguredTLS::TlsConnector(connector) => {
                    return pg_config.connect(connector).await;
                }
            }
        };

        let (client, mut connection) = tokio_runtime().spawn(connect_future).await??;

        let (transmitter, receiver) = futures_channel::mpsc::unbounded::<AsyncMessage>();

        let stream =
            stream::poll_fn(
                move |cx| connection.poll_message(cx)).map_err(|e| panic!("{}", e),
            );

        let connection = stream.forward(transmitter).map(|r| r.unwrap());
        tokio_runtime().spawn(connection);

        self.receiver = Some(Arc::new(RwLock::new(receiver)));
        self.connection = Connection::new(
            Some(Arc::new(InnerConnection::SingleConn(client))),
            None,
        );

        self.is_started = true;

        Ok(())
    }

    fn __anext__(&self) -> RustPSQLDriverPyResult<Option<PyObject>> {
        let Some(client) = self.connection.db_client() else {
            return Err(RustPSQLDriverError::ListenerStartError(
                "Listener doesn't have underlying client, please call startup".into(),
            ));
        };
        let Some(receiver) = self.receiver.clone() else {
            return Err(RustPSQLDriverError::ListenerStartError(
                "Listener doesn't have underlying receiver, please call startup".into(),
            ));
        };

        let is_listened_clone = self.is_listened.clone();
        let listen_query_clone = self.listen_query.clone();

        let py_future = Python::with_gil(move |gil| {
            rustdriver_future(gil, async move {
                {
                    call_listen(&is_listened_clone, &listen_query_clone, &client).await?;
                };
                let next_element = {
                    let mut write_receiver = receiver.write().await;
                    write_receiver.next().await
                };

                let inner_notification = process_message(next_element)?;

                Ok(inner_notification)
            })
        });

        Ok(Some(py_future?))
    }

    #[pyo3(signature = (channel, callback))]
    async fn add_callback(
        &mut self,
        channel: String,
        callback: Py<PyAny>,
    ) -> RustPSQLDriverPyResult<()> {
        let callback_clone = callback.clone();

        let is_coro = is_coroutine_function(callback_clone)?;

        if !is_coro {
            return Err(RustPSQLDriverError::ListenerCallbackError)
        }

        let task_locals = Python::with_gil(|py| {
            pyo3_async_runtimes::tokio::get_current_locals(py)}
        )?;

        let listener_callback = ListenerCallback::new(
            Some(task_locals),
            callback,
        );

        // let awaitable = callback.call1(()).unwrap();
        // println!("8888888 {:?}", awaitable);
        // let bbb = pyo3_async_runtimes::tokio::into_future(awaitable).unwrap();
        // println!("999999");
        {
            let mut write_channel_callbacks = self.channel_callbacks.write().await;
            write_channel_callbacks.add_callback(channel, listener_callback);
        }

        self.update_listen_query().await;

        Ok(())
    }

    async fn listen(&mut self) -> RustPSQLDriverPyResult<()> {
        let Some(client) = self.connection.db_client() else {
            return Err(RustPSQLDriverError::BaseConnectionError("test".into()));
        };
        let connection = self.connection.clone();
        let Some(receiver) = self.receiver.clone() else {
            return Err(RustPSQLDriverError::BaseConnectionError("test".into()));
        };
        let listen_query_clone = self.listen_query.clone();
        let is_listened_clone = self.is_listened.clone();

        let channel_callbacks = self.channel_callbacks.clone();

        let jh: JoinHandle<Result<(), RustPSQLDriverError>> = tokio_runtime().spawn(async move {
            loop {
                {
                    call_listen(&is_listened_clone, &listen_query_clone, &client).await?;
                };

                let next_element = {
                    let mut write_receiver = receiver.write().await;
                    write_receiver.next().await
                };

                let inner_notification = process_message(next_element)?;

                let read_channel_callbacks = channel_callbacks.read().await;
                let channel = inner_notification.channel.clone();
                let callbacks = read_channel_callbacks.retrieve_channel_callbacks(
                    channel,
                );

                if let Some(callbacks) = callbacks {
                    for callback in callbacks {
                        dispatch_callback(
                            callback,
                            inner_notification.clone(),
                            connection.clone(),
                        ).await?;
                    }
                }
            }
        });

        let abj = jh.abort_handle();

        self.listen_abort_handler = Some(abj);

        Ok(())
    }
}

async fn dispatch_callback(
    listener_callback: &ListenerCallback,
    listener_notification: ListenerNotification,
    connection: Connection,
) -> RustPSQLDriverPyResult<()> {
    listener_callback.call(
        listener_notification.clone(),
        connection,
    ).await?;

    Ok(())
}

async fn call_listen(
    is_listened: &Arc<RwLock<bool>>,
    listen_query: &Arc<RwLock<String>>,
    client: &Arc<InnerConnection>,
) -> RustPSQLDriverPyResult<()> {
    let mut write_is_listened = is_listened.write().await;

    if !write_is_listened.eq(&true) {
        let listen_q = {
            let read_listen_query = listen_query.read().await;
            String::from(read_listen_query.as_str())
        };

        client
            .batch_execute(listen_q.as_str())
            .await?;
    }

    *write_is_listened = true;
    Ok(())
}

fn process_message(
    message: Option<AsyncMessage>,
) -> RustPSQLDriverPyResult<ListenerNotification> {
    let Some(async_message) = message else {
        return Err(RustPSQLDriverError::ListenerError("Wow".into()))
    };
    let AsyncMessage::Notification(notification) = async_message else {
        return Err(RustPSQLDriverError::ListenerError("Wow".into()))
    };

    Ok(ListenerNotification::from(notification))
}
