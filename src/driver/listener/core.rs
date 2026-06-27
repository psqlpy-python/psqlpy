use std::{collections::HashSet, sync::Arc};

use futures::{pin_mut, stream, StreamExt};
use futures_channel::mpsc::UnboundedReceiver;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres_openssl::MakeTlsConnector;
use pyo3::{pyclass, pymethods, Py, PyAny, PyErr, Python};
use tokio::{
    sync::RwLock,
    task::{AbortHandle, JoinHandle},
};
use tokio_postgres::{AsyncMessage, Config};

use crate::{
    connection::{
        structs::{PSQLPyConnection, SingleConnection},
        traits::Connection as _,
    },
    driver::{
        connection::Connection,
        utils::{build_tls, is_coroutine_function, ConfiguredTLS},
    },
    exceptions::rust_errors::{PSQLPyResult, RustPSQLDriverError},
    options::SslMode,
    runtime::{rustdriver_future, tokio_runtime},
};

use super::structs::{
    ChannelCallbacks, ListenerCallback, ListenerNotification, ListenerNotificationMsg,
};

#[pyclass]
pub struct Listener {
    pg_config: Arc<Config>,
    ca_file: Option<String>,
    ssl_mode: Option<SslMode>,
    channel_callbacks: Arc<RwLock<ChannelCallbacks>>,
    listen_abort_handler: Option<AbortHandle>,
    connection: Connection,
    receiver: Option<Arc<RwLock<UnboundedReceiver<AsyncMessage>>>>,
    /// Channels currently subscribed on the backend session. Diffed against the
    /// desired set (`channel_callbacks`) to reconcile LISTEN/UNLISTEN.
    applied_channels: Arc<RwLock<HashSet<String>>>,
    is_listened: Arc<RwLock<bool>>,
    is_started: bool,
}

impl Listener {
    #[must_use]
    pub fn new(
        pg_config: &Arc<Config>,
        ca_file: Option<String>,
        ssl_mode: Option<SslMode>,
    ) -> Self {
        Listener {
            pg_config: pg_config.clone(),
            ca_file,
            ssl_mode,
            channel_callbacks: Arc::default(),
            listen_abort_handler: Option::default(),
            connection: Connection::new(None, None, pg_config.clone()),
            receiver: Option::default(),
            applied_channels: Arc::default(),
            is_listened: Arc::new(RwLock::new(false)),
            is_started: false,
        }
    }

    /// Flag that the backend subscriptions no longer match the desired channel
    /// set, so the next `execute_listen` reconciles them (LISTEN/UNLISTEN).
    ///
    /// Only `is_listened` is taken here, never while holding `channel_callbacks`,
    /// so there is no lock-order cycle with `execute_listen` (which takes
    /// `is_listened` first, then reads `channel_callbacks`).
    async fn mark_subscriptions_dirty(&self) {
        *self.is_listened.write().await = false;
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

    #[allow(clippy::unused_async)]
    async fn __aenter__(slf: Py<Self>) -> PSQLPyResult<Py<Self>> {
        Ok(slf)
    }

    #[allow(clippy::unused_async)]
    async fn __aexit__(
        slf: Py<Self>,
        _exception_type: Py<PyAny>,
        exception: Py<PyAny>,
        _traceback: Py<PyAny>,
    ) -> PSQLPyResult<()> {
        let (client, is_exception_none, py_err) = pyo3::Python::with_gil(|gil| {
            let self_ = slf.borrow(gil);
            (
                self_.connection.db_client(),
                exception.is_none(gil),
                PyErr::from_value(exception.into_bound(gil)),
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

        Err(RustPSQLDriverError::ListenerClosedError)
    }

    fn __anext__(&self) -> PSQLPyResult<Option<Py<PyAny>>> {
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
        let channel_callbacks_clone = self.channel_callbacks.clone();
        let applied_channels_clone = self.applied_channels.clone();
        let connection = self.connection.clone();

        let py_future = Python::with_gil(move |gil| {
            rustdriver_future(gil, async move {
                {
                    execute_listen(
                        &is_listened_clone,
                        &channel_callbacks_clone,
                        &applied_channels_clone,
                        &client,
                    )
                    .await?;
                };
                let next_element = {
                    let mut write_receiver = receiver.write().await;
                    write_receiver.next().await
                };

                let inner_notification = process_message(next_element)?;

                Ok(ListenerNotificationMsg::new(inner_notification, connection))
            })
        });

        Ok(Some(py_future?))
    }

    #[getter]
    fn is_started(&self) -> bool {
        self.is_started
    }

    #[getter]
    fn connection(&self) -> PSQLPyResult<Connection> {
        if !self.is_started {
            return Err(RustPSQLDriverError::ListenerStartError(
                "Listener isn't started up".into(),
            ));
        }

        Ok(self.connection.clone())
    }

    async fn startup(&mut self) -> PSQLPyResult<()> {
        if self.is_started {
            return Err(RustPSQLDriverError::ListenerStartError(
                "Listener is already started".into(),
            ));
        }

        let tls_ = build_tls(&self.ca_file, &self.ssl_mode)?;

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

        let forward_messages = async move {
            let stream = stream::poll_fn(move |cx| connection.poll_message(cx));
            pin_mut!(stream);

            while let Some(message) = stream.next().await {
                match message {
                    // Receiver gone (listener shut down) -> stop forwarding.
                    Ok(async_message) => {
                        if transmitter.unbounded_send(async_message).is_err() {
                            break;
                        }
                    }
                    // Connection closed or errored -> end the task cleanly
                    // instead of panicking the worker thread.
                    Err(_) => break,
                }
            }
        };
        tokio_runtime().spawn(forward_messages);

        self.receiver = Some(Arc::new(RwLock::new(receiver)));
        self.connection = Connection::new(
            Some(Arc::new(RwLock::new(PSQLPyConnection::SingleConnection(
                SingleConnection::new(client, self.pg_config.clone()),
            )))),
            None,
            self.pg_config.clone(),
        );

        self.is_started = true;

        Ok(())
    }

    /// TODO: remove clippy ignore after removing async
    #[allow(clippy::unused_async)]
    async fn shutdown(&mut self) {
        self.abort_listen();
        std::mem::take(&mut self.connection);
        std::mem::take(&mut self.receiver);

        self.is_started = false;
    }

    #[pyo3(signature = (channel, callback))]
    async fn add_callback(&mut self, channel: String, callback: Py<PyAny>) -> PSQLPyResult<()> {
        if !is_coroutine_function(callback.clone())? {
            return Err(RustPSQLDriverError::ListenerCallbackError);
        }

        let task_locals = Python::with_gil(pyo3_async_runtimes::tokio::get_current_locals)?;

        let listener_callback = ListenerCallback::new(task_locals, callback);

        {
            let mut write_channel_callbacks = self.channel_callbacks.write().await;
            write_channel_callbacks.add_callback(channel, listener_callback);
        }

        self.mark_subscriptions_dirty().await;

        Ok(())
    }

    async fn clear_channel_callbacks(&mut self, channel: String) {
        {
            let mut write_channel_callbacks = self.channel_callbacks.write().await;
            write_channel_callbacks.clear_channel_callbacks(&channel);
        }

        self.mark_subscriptions_dirty().await;
    }

    async fn clear_all_channels(&mut self) {
        {
            let mut write_channel_callbacks = self.channel_callbacks.write().await;
            write_channel_callbacks.clear_all();
        }

        self.mark_subscriptions_dirty().await;
    }

    fn listen(&mut self) -> PSQLPyResult<()> {
        let Some(client) = self.connection.db_client() else {
            return Err(RustPSQLDriverError::ListenerStartError(
                "Cannot start listening, underlying connection doesn't exist".into(),
            ));
        };
        let Some(receiver) = self.receiver.clone() else {
            return Err(RustPSQLDriverError::ListenerStartError(
                "Cannot start listening, underlying connection doesn't exist".into(),
            ));
        };

        let connection = self.connection.clone();
        let is_listened_clone = self.is_listened.clone();
        let applied_channels = self.applied_channels.clone();

        let channel_callbacks = self.channel_callbacks.clone();

        let jh: JoinHandle<Result<(), RustPSQLDriverError>> = tokio_runtime().spawn(async move {
            loop {
                {
                    execute_listen(
                        &is_listened_clone,
                        &channel_callbacks,
                        &applied_channels,
                        &client,
                    )
                    .await?;
                };

                let next_element = {
                    let mut write_receiver = receiver.write().await;
                    write_receiver.next().await
                };

                let inner_notification = process_message(next_element)?;

                let read_channel_callbacks = channel_callbacks.read().await;
                let channel = inner_notification.channel.clone();
                let callbacks = read_channel_callbacks.retrieve_channel_callbacks(&channel);

                if let Some(callbacks) = callbacks {
                    for callback in callbacks {
                        dispatch_callback(callback, inner_notification.clone(), connection.clone())
                            .await?;
                    }
                }
            }
        });

        let abj = jh.abort_handle();

        self.listen_abort_handler = Some(abj);

        Ok(())
    }

    fn abort_listen(&mut self) {
        if let Some(listen_abort_handler) = &self.listen_abort_handler {
            listen_abort_handler.abort();
        }

        self.listen_abort_handler = None;
    }
}

async fn dispatch_callback(
    listener_callback: &ListenerCallback,
    listener_notification: ListenerNotification,
    connection: Connection,
) -> PSQLPyResult<()> {
    listener_callback
        .call(listener_notification.clone(), connection)
        .await?;

    Ok(())
}

/// Reconcile the backend subscriptions with the desired channel set.
///
/// When `is_listened` is dirty (`false`) the difference between the desired
/// channels (`channel_callbacks`) and the channels already applied on the
/// backend (`applied_channels`) is turned into `UNLISTEN`/`LISTEN` statements
/// and executed. Re-subscribing is idempotent, so a redundant `LISTEN` is
/// harmless; the `UNLISTEN` half is what stops a cleared channel from delivering.
///
/// Lock order is `client` -> `is_listened` -> `channel_callbacks` ->
/// `applied_channels`. `mark_subscriptions_dirty` only ever takes `is_listened`
/// (never while holding `channel_callbacks`), so the two cannot deadlock.
async fn execute_listen(
    is_listened: &Arc<RwLock<bool>>,
    channel_callbacks: &Arc<RwLock<ChannelCallbacks>>,
    applied_channels: &Arc<RwLock<HashSet<String>>>,
    client: &Arc<RwLock<PSQLPyConnection>>,
) -> PSQLPyResult<()> {
    let read_conn_g = client.read().await;
    let mut write_is_listened = is_listened.write().await;

    if !*write_is_listened {
        let desired: HashSet<String> = {
            let read_channel_callbacks = channel_callbacks.read().await;
            read_channel_callbacks
                .retrieve_all_channels()
                .into_iter()
                .cloned()
                .collect()
        };

        let mut applied = applied_channels.write().await;

        let mut reconcile_query = String::new();
        for channel in applied.difference(&desired) {
            reconcile_query.push_str(format!("UNLISTEN {channel};").as_str());
        }
        for channel in desired.difference(&applied) {
            reconcile_query.push_str(format!("LISTEN {channel};").as_str());
        }

        if !reconcile_query.is_empty() {
            read_conn_g.batch_execute(reconcile_query.as_str()).await?;
        }

        *applied = desired;
    }

    *write_is_listened = true;
    Ok(())
}

fn process_message(message: Option<AsyncMessage>) -> PSQLPyResult<ListenerNotification> {
    let Some(async_message) = message else {
        return Err(RustPSQLDriverError::ListenerError("Wow".into()));
    };
    let AsyncMessage::Notification(notification) = async_message else {
        return Err(RustPSQLDriverError::ListenerError("Wow".into()));
    };

    Ok(ListenerNotification::from(notification))
}
