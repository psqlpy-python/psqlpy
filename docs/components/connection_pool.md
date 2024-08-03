---
title: Connection Pool
---

Connection pool is the main object in the library. It initializes, creates, holds and gives connection to the user side.
Connection pool must be started up before any other operations.

::: important
You cannot set the minimum size for the connection pool, by it is 0.

`ConnectionPool` doesn't create connection on startup. It makes new connection on demand.

So, if you set `max_db_pool_size` to 100, pool will create new connection every time there aren't enough connections to handle the load.
:::

## Connection pool methods

### All available ConnectionPool parameters

- `dsn`: Full dsn connection string.
    `postgres://postgres:postgres@localhost:5432/postgres?target_session_attrs=read-write`
- `username`: Username of the user in the `PostgreSQL`
- `password`: Password of the user in the `PostgreSQL`
- `host`: Host of the `PostgreSQL`
- `hosts`: Hosts of the `PostgreSQL`
- `port`: Port of the `PostgreSQL`
- `ports`: Ports of the `PostgreSQL`
- `db_name`: Name of the database in `PostgreSQL`
- `target_session_attrs`: Specifies requirements of the session.
- `options`: Command line options used to configure the server
- `application_name`: Sets the application_name parameter on the server.
- `connect_timeout_sec`: The time limit in seconds applied to each socket-level
    connection attempt.
    Note that hostnames can resolve to multiple IP addresses,
    and this limit is applied to each address. Defaults to no timeout.
- `connect_timeout_nanosec`: nanosec for connection timeout,
    can be used only with connect_timeout_sec.
- `tcp_user_timeout_sec`: The time limit that
    transmitted data may remain unacknowledged
    before a connection is forcibly closed.
    This is ignored for Unix domain socket connections.
    It is only supported on systems where TCP_USER_TIMEOUT
    is available and will default to the system default if omitted
    or set to 0; on other systems, it has no effect.
- `tcp_user_timeout_nanosec`: nanosec for cp_user_timeout,
    can be used only with tcp_user_timeout_sec.
- `keepalives`: Controls the use of TCP keepalive.
    This option is ignored when connecting with Unix sockets.
    Defaults to on.
- `keepalives_idle_sec`: The number of seconds of inactivity after
    which a keepalive message is sent to the server.
    This option is ignored when connecting with Unix sockets.
    Defaults to 2 hours.
- `keepalives_idle_nanosec`: Nanosec for keepalives_idle_sec.
- `keepalives_interval_sec`: The time interval between TCP keepalive probes.
    This option is ignored when connecting with Unix sockets.
- `keepalives_interval_nanosec`: Nanosec for keepalives_interval_sec.
- `keepalives_retries`: The maximum number of TCP keepalive probes
    that will be sent before dropping a connection.
    This option is ignored when connecting with Unix sockets.
- `load_balance_hosts`: Controls the order in which the client tries to connect
    to the available hosts and addresses.
    Once a connection attempt is successful no other
    hosts and addresses will be tried.
    This parameter is typically used in combination with multiple host names
    or a DNS record that returns multiple IPs.
    If set to disable, hosts and addresses will be tried in the order provided.
    If set to random, hosts will be tried in a random order, and the IP addresses
    resolved from a hostname will also be tried in a random order.
    Defaults to disable.
- `max_db_pool_size`: maximum size of the connection pool.
- `conn_recycling_method`: how a connection is recycled.
- `ssl_mode`: ssl mode.
- `ca_file`: path to ca_file for ssl.

Example of possible `dsn`s:

```
postgresql://user@localhost
postgresql://user:password@%2Fvar%2Flib%2Fpostgresql/mydb?connect_timeout=10
postgresql://user@host1:1234,host2,host3:5678?target_session_attrs=read-write
postgresql:///mydb?user=user&host=/var/lib/postgresql
```

::: important
If `dsn` is specified then `username`, `password`, `host`, `hosts`, `port`, `ports`, `db_name` and `target_session_attrs`
parameters will be ignored.
:::

### Initialize Connection Pool with separate parameters

There are two ways of how to connect to the database. First one is use connection parameters separately:

```python
import asyncio
from typing import Final

from psqlpy import ConnectionPool

db_pool: Final = ConnectionPool(
    username="postgres",
    password="postgres",
    host="localhost",
    port=5432,
    db_name="postgres",
    max_db_pool_size=10,
)

async def main() -> None:

```

### Initialize Connection Pool with DSN

Other way is use DSN:

```python
import asyncio
from typing import Final

from psqlpy import ConnectionPool

db_pool: Final = ConnectionPool(
    dsn="postgres://postgres:postgres@localhost:5432/postgres",
    max_db_pool_size=10,
)

async def main() -> None:

```

### Create Connection Pool with one function
```py
from typing import Final

from psqlpy import connect


db_pool: Final = connect(
    dsn="postgres://postgres:postgres@localhost:5432/postgres",
    max_db_pool_size=10,
)
```
`connect` function has the same parameters as `ConnectionPool`.

### Resize
Resize connection pool capacity.

This change the max_size of the pool dropping excess objects and/or making space for new ones.

#### Parameters:
- `new_max_size`: new size of the pool.

```python
async def main() -> None:
    ...
    db_pool.resize(15)
```

### Status
Retrieve status of the connection pool.

It has 4 parameters:
- `max_size` - maximum possible size of the connection pool.
- `size` - current size of the connection pool.
- `available` - available connection in the connection pool.
- `waiting` - waiting requests to retrieve connection from connection pool.

### Execute

#### Parameters:

- `querystring`: Statement string.
- `parameters`: List of parameters for the statement string.
- `prepared`: Prepare statement before execution or not.

You can execute any query directly from Connection Pool.
This method supports parameters, each parameter must be marked as `$<number>` (number starts with 1).
Parameters must be passed as list after querystring.
::: caution
You must use `ConnectionPool.execute` method in high-load production code wisely!
It pulls connection from the pool each time you execute query.
Preferable way to execute statements with [Connection](./../../introduction/components/connection.md) or [Transaction](./../../introduction/components/transaction.md)
:::

```python
async def main() -> None:
    ...
    results: QueryResult = await db_pool.execute(
        "SELECT * FROM users WHERE id = $1 and username = $2",
        [100, "Alex"],
    )

    dict_results: list[dict[str, Any]] = results.result()
```

### Fetch

#### Parameters:

- `querystring`: Statement string.
- `parameters`: List of parameters for the statement string.
- `prepared`: Prepare statement before execution or not.

The same as the `execute` method, for some people this naming is preferable.

```python
async def main() -> None:
    ...
    results: QueryResult = await db_pool.fetch(
        "SELECT * FROM users WHERE id = $1 and username = $2",
        [100, "Alex"],
    )

    dict_results: list[dict[str, Any]] = results.result()
```

### Acquire

Get single connection for async context manager.
Must be used only in async context manager.

```python
async def main() -> None:
    ...
    async with db_pool.acquire() as connection:
        ...
```

### Connection

To get single connection from the `ConnectionPool` there is method named `connection()`.

```python
async def main() -> None:
    ...
    connection = await db_pool.connection()
```

::: tip Cool tip
This is the preferable way to work with the PostgreSQL.
:::


### Close

To close the connection pool at the stop of your application.
```python
def main() -> None:
    ...
    db_pool.close()
```