use pyo3::prelude::*;
use tokio_postgres::config::Host;

use std::net::IpAddr;

use super::{connection::Connection, cursor::Cursor, transaction::Transaction};

macro_rules! impl_config_py_methods {
    ($name:ident) => {
        #[pymethods]
        impl $name {
            #[getter]
            fn conn_dbname(&self) -> Option<&str> {
                self.pg_config.get_dbname()
            }

            #[getter]
            fn user(&self) -> Option<&str> {
                self.pg_config.get_user()
            }

            #[getter]
            fn host_addrs(&self) -> Vec<String> {
                let mut host_addrs_vec = vec![];

                let host_addrs = self.pg_config.get_hostaddrs();
                for ip_addr in host_addrs {
                    match ip_addr {
                        IpAddr::V4(ipv4) => {
                            host_addrs_vec.push(ipv4.to_string());
                        }
                        IpAddr::V6(ipv6) => {
                            host_addrs_vec.push(ipv6.to_string());
                        }
                    }
                }

                host_addrs_vec
            }

            #[cfg(unix)]
            #[getter]
            fn hosts(&self) -> Vec<String> {
                let mut hosts_vec = vec![];

                let hosts = self.pg_config.get_hosts();
                for host in hosts {
                    match host {
                        Host::Tcp(host) => {
                            hosts_vec.push(host.to_string());
                        }
                        Host::Unix(host) => {
                            hosts_vec.push(host.display().to_string());
                        }
                    }
                }

                hosts_vec
            }

            #[cfg(not(unix))]
            #[getter]
            fn hosts(&self) -> Vec<String> {
                let mut hosts_vec = vec![];

                let hosts = self.pg_config.get_hosts();
                for host in hosts {
                    match host {
                        Host::Tcp(host) => {
                            hosts_vec.push(host.to_string());
                        }
                        _ => unreachable!(),
                    }
                }

                hosts_vec
            }

            #[getter]
            fn ports(&self) -> Vec<&u16> {
                return self.pg_config.get_ports().iter().collect::<Vec<&u16>>();
            }

            #[getter]
            fn options(&self) -> Option<&str> {
                return self.pg_config.get_options();
            }
        }
    };
}

impl_config_py_methods!(Transaction);
impl_config_py_methods!(Connection);
impl_config_py_methods!(Cursor);
