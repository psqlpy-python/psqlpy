use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use futures::{stream, FutureExt, StreamExt, TryStreamExt};
use futures_channel::mpsc::UnboundedReceiver;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres_openssl::MakeTlsConnector;
use pyo3::{pyclass, pymethods, Py, PyAny, PyObject, Python};
use tokio::{sync::RwLock, task::AbortHandle};
use tokio_postgres::{AsyncMessage, Client, Config};

use crate::{
    exceptions::rust_errors::{RustPSQLDriverError, RustPSQLDriverPyResult},
    runtime::{rustdriver_future, tokio_runtime},
};

use super::{
    common_options::SslMode,
    utils::{build_tls, ConfiguredTLS},
};

#[pyclass]
pub struct Listener {
    // name: String,
    pg_config: Config,
    ca_file: Option<String>,
    ssl_mode: Option<SslMode>,
    // transaction_config: Option<ListenerTransactionConfig>,
    channel_callbacks: HashMap<String, Vec<Py<PyAny>>>,
    listen_abort_handler: Option<AbortHandle>,
    client: Option<Arc<Client>>,
    receiver: Option<Arc<RwLock<UnboundedReceiver<AsyncMessage>>>>,
    is_listened: Arc<RwLock<bool>>,
}

impl Listener {
    #[must_use] pub fn new(
        // name: String,
        pg_config: Config,
        ca_file: Option<String>,
        ssl_mode: Option<SslMode>,
        // transaction_config: Option<ListenerTransactionConfig>,
    ) -> Self {
        Listener {
            // name: name,
            pg_config,
            ca_file,
            // transaction_config: transaction_config,
            ssl_mode,
            channel_callbacks: Default::default(),
            listen_abort_handler: Default::default(),
            client: Default::default(),
            receiver: Default::default(),
            is_listened: Arc::new(RwLock::new(false)),
        }
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
        Ok(())
    }

    fn __anext__(&self) -> RustPSQLDriverPyResult<Option<PyObject>> {
        let Some(client) = self.client.clone() else {
            return Err(RustPSQLDriverError::BaseConnectionError("test".into()));
        };
        let Some(receiver) = self.receiver.clone() else {
            return Err(RustPSQLDriverError::BaseConnectionError("test".into()));
        };
        let is_listened = self.is_listened.clone();
        let py_future = Python::with_gil(move |gil| {
            rustdriver_future(gil, async move {
                let mut write_is_listened = is_listened.write().await;
                if write_is_listened.eq(&false) {
                    println!("here1");
                    client
                        .batch_execute(
                            "LISTEN test_notifications;
                            LISTEN test_notifications2;",
                        )
                        .await
                        .unwrap();

                    *write_is_listened = true;
                }
                let mut write_receiver = receiver.write().await;
                let next_element = write_receiver.next().await;
                println!("here2");

                match next_element {
                    Some(n) => match n {
                        tokio_postgres::AsyncMessage::Notification(n) => {
                            println!("Notification {n:?}");
                            return Ok(());
                        }
                        _ => {
                            println!("in_in {n:?}");
                        }
                    },
                    _ => {
                        println!("in {next_element:?}");
                    }
                }

                Ok(())
            })
        });

        Ok(Some(py_future?))
    }

    async fn startup(&mut self) -> RustPSQLDriverPyResult<()> {
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
            stream::poll_fn(move |cx| connection.poll_message(cx)).map_err(|e| panic!("{}", e));

        let connection = stream.forward(transmitter).map(|r| r.unwrap());
        tokio_runtime().spawn(connection);

        self.receiver = Some(Arc::new(RwLock::new(receiver)));
        self.client = Some(Arc::new(client));

        Ok(())
    }

    fn add_callback(&mut self, channel: String, callback: Py<PyAny>) -> RustPSQLDriverPyResult<()> {
        match self.channel_callbacks.entry(channel) {
            Entry::Vacant(e) => {
                e.insert(vec![callback]);
            }
            Entry::Occupied(mut e) => {
                e.get_mut().push(callback);
            }
        };

        Ok(())
    }

    async fn listen(&mut self) -> RustPSQLDriverPyResult<()> {
        let Some(client) = self.client.clone() else {
            return Err(RustPSQLDriverError::BaseConnectionError("test".into()));
        };
        let Some(receiver) = self.receiver.clone() else {
            return Err(RustPSQLDriverError::BaseConnectionError("test".into()));
        };

        let jh = tokio_runtime().spawn(async move {
            client
                .batch_execute(
                    "LISTEN test_notifications;
                    LISTEN test_notifications2;",
                )
                .await
                .unwrap();

            loop {
                let mut write_receiver = receiver.write().await;
                let next_element = write_receiver.next().await;
                client
                    .batch_execute("LISTEN test_notifications3;")
                    .await
                    .unwrap();
                match next_element {
                    Some(n) => match n {
                        tokio_postgres::AsyncMessage::Notification(n) => {
                            println!("Notification {n:?}");
                        }
                        _ => {
                            println!("in_in {n:?}");
                        }
                    },
                    _ => {
                        println!("in {next_element:?}");
                    }
                }
            }
        });

        let abj = jh.abort_handle();

        self.listen_abort_handler = Some(abj);

        Ok(())
    }
}
