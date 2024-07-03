---
title: Connection Pool Builder
---

Component allows you to create `ConnectionPool` with chainable methods. It doesn't have any difference from classic python-way initialization.

Every method has robust docstring but you can check all available methods here.

```python
from psqlpy import ConnectionPoolBuilder


database_pool = (
    ConnectionPoolBuilder()
    .max_pool_size(10)
    .user("psqlpy")
    .password("psqlpy")
    .dbname("psqlpy")
    .host("192.0.0.1")
    .port(5432)
    .build()
)
```

## ConnectionPoolBuilder methods

### build
Create new `ConnectionPool` from `ConnectionPoolBuilder`.

### max_pool_size
Set maximum connection pool size.

### conn_recycling_method
Set connection recycling method.

Connection recycling method is how a connection is recycled.

### user
Set username to `PostgreSQL`.

### password
Set password for `PostgreSQL`.

### dbname
Set database name for the `PostgreSQL`.

### options
Set command line options used to configure the server.

### application_name
Set the value of the `application_name` runtime parameter.

### ssl_mode
Set the SSL configuration.

### ca_file
Set ca_file for SSL.

### host
Add a host to the configuration.

Multiple hosts can be specified by calling this method multiple times,
and each will be tried in order.
On Unix systems, a host starting with a `/` is interpreted
as a path to a directory containing Unix domain sockets.
There must be either no hosts,
or the same number of hosts as hostaddrs.

### hostaddr
Add a hostaddr to the configuration.

Multiple hostaddrs can be specified by calling
this method multiple times, and each will be tried in order.
There must be either no hostaddrs,
or the same number of hostaddrs as hosts.

### port
Add a port to the configuration.

Multiple ports can be specified by calling this method multiple times.
There must either be no ports,
in which case the default of 5432 is used,
a single port, in which it is used for all hosts,
or the same number of ports as hosts.

### connect_timeout
Set the timeout applied to socket-level connection attempts.

Note that hostnames can resolve to multiple IP addresses,
and this timeout will apply to each address of each
host separately. Defaults to no limit.

### tcp_user_timeout
Set the TCP user timeout.

This is ignored for Unix domain socket connections.
It is only supported on systems where TCP_USER_TIMEOUT is available
and will default to the system default if omitted or set to 0;
on other systems, it has no effect.

### target_session_attrs
Set the requirements of the session.

This can be used to connect to the primary server in a
clustered database rather than one of the read-only
secondary servers. Defaults to `Any`.

### load_balance_hosts
Set the host load balancing behavior.

Defaults to `disable`.

### keepalives
Control the use of TCP keepalive.

This is ignored for Unix domain socket connections.

Defaults to `true`.

### keepalives_idle
Set the amount of idle time before a keepalive packet is sent on the connection.

This is ignored for Unix domain sockets,
or if the `keepalives` option is disabled.

Defaults to 2 hours.

### keepalives_interval
Sets the time interval between TCP keepalive probes.

On Windows, this sets the value of the
tcp_keepalive struct keepalive interval field.

This is ignored for Unix domain sockets,
or if the `keepalives` option is disabled.

### keepalives_retries
Sets the maximum number of TCP keepalive probes that will be sent before dropping a connection.

This is ignored for Unix domain sockets, or if the `keepalives` option is disabled.