---
title: Listener
---

`Listener` object allows users to work with [LISTEN](https://www.postgresql.org/docs/current/sql-listen.html)/[NOTIFY](https://www.postgresql.org/docs/current/sql-notify.html) functionality.

## Usage

There are two ways of using `Listener` object:
- Async iterator
- Background task

::: tabs

@tab Background task
```python
from psqlpy import ConnectionPool, Connection, Listener


db_pool = ConnectionPool(
    dsn="postgres://postgres:postgres@localhost:5432/postgres",
)

async def test_channel_callback(
    connection: Connection,
    payload: str,
    channel: str,
    process_id: int,
) -> None:
    # do some important staff
    ...

async def main() -> None:
    # Create listener object
    listener: Listener = db_pool.listener()

    # Add channel to listen and callback for it.
    await listener.add_callback(
        channel="test_channel",
        callback=test_channel_callback,
    )

    # Startup the listener
    await listener.startup()

    # Start listening.
    # `listen` method isn't blocking, it returns None and starts background
    # task in the Rust event loop.
    listener.listen()

    # You can stop listening.
    listener.abort_listen()
```

@tab Async Iterator
```python
from psqlpy import (
    ConnectionPool,
    Connection,
    Listener,
    ListenerNotificationMsg,
)


db_pool = ConnectionPool(
    dsn="postgres://postgres:postgres@localhost:5432/postgres",
)

async def main() -> None:
    # Create listener object
    listener: Listener = db_pool.listener()

    # Startup the listener
    await listener.startup()

    listener_msg: ListenerNotificationMsg
    async for listener_msg in listener:
        print(listener_msg)
```

:::

## Listener attributes

- `connection`: Instance of `Connection`.
If `startup` wasn't called, raises `ListenerStartError`.

- `is_started`: Flag that shows whether the `Listener` is running or not.

## Listener methods

### Startup

Startup `Listener` instance and can be called once or again only after `shutdown`.

::: important
`Listener` must be started up.
:::

```python
async def main() -> None:
    listener: Listener = db_pool.listener()

    await listener.startup()
```

### Shutdown
Abort listen (if called) and release underlying connection.

```python
async def main() -> None:
    listener: Listener = db_pool.listener()

    await listener.startup()
    await listener.shutdown()
```

### Add Callback

#### Parameters:
- `channel`: name of the channel to listen.
- `callback`: coroutine callback.

Add new callback to the channel, can be called multiple times (before or after `listen`).

Callback signature is like this:
```python
from psqlpy import Connection

async def callback(
    connection: Connection,
    payload: str,
    channel: str,
    process_id: int,
) -> None:
    ...
```

Parameters for callback are based like `args`, so this signature is correct to:
```python
async def callback(
    connection: Connection,
    *args,
) -> None:
    ...
```

**Example:**
```python
async def test_channel_callback(
    connection: Connection,
    payload: str,
    channel: str,
    process_id: int,
) -> None:
    ...

async def main() -> None:
    listener = db_pool.listener()

    await listener.add_callback(
        channel="test_channel",
        callback=test_channel_callback,
    )
```

### Clear Channel Callbacks

#### Parameters:
- `channel`: name of the channel

Remove all callbacks for the channel

```python
async def main() -> None:
    listener = db_pool.listener()
    await listener.clear_channel_callbacks()
```

### Clear All Channels
Clear all channels and callbacks.

```python
async def main() -> None:
    listener = db_pool.listener()
    await listener.clear_all_channels()
```

### Listen
Start listening.

It's a non-blocking operation.
In the background it creates task in Rust event loop.

```python
async def main() -> None:
    listener = db_pool.listener()
    await listener.startup()
    listener.listen()
```

### Abort Listen
Abort listen.
If `listen()` method was called, stop listening, else don't do anything.

```python
async def main() -> None:
    listener = db_pool.listener()
    await listener.startup()
    listener.listen()
    listener.abort_listen()
```
