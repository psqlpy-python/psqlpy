from __future__ import annotations

import asyncio
import typing

import anyio
import pytest
from psqlpy.exceptions import ListenerStartError

if typing.TYPE_CHECKING:
    from psqlpy import Connection, ConnectionPool, Listener

pytestmark = pytest.mark.anyio


TEST_CHANNEL = "test_channel"
TEST_PAYLOAD = "test_payload"

# How long helpers wait for an asynchronous condition before giving up.
# These bound every wait in this module so a lost/late NOTIFY surfaces as a
# fast, explicit failure instead of hanging the whole test session forever
# (which is what used to wedge the GitHub runners).
WAIT_TIMEOUT = 10.0
POLL_INTERVAL = 0.05
# Time we allow a notification to be (not) delivered before asserting absence.
SETTLE_TIMEOUT = 1.0


async def construct_listener(
    psql_pool: ConnectionPool,
    listener_table_name: str,
) -> Listener:
    listener = psql_pool.listener()
    await listener.add_callback(
        channel=TEST_CHANNEL,
        callback=construct_insert_callback(
            listener_table_name=listener_table_name,
        ),
    )
    return listener


def construct_insert_callback(
    listener_table_name: str,
) -> typing.Callable[
    [Connection, str, str, int],
    typing.Awaitable[None],
]:
    async def callback(
        connection: Connection,
        payload: str,
        channel: str,
        process_id: int,
    ) -> None:
        await connection.execute(
            querystring=f"INSERT INTO {listener_table_name} VALUES (1, $1, $2, $3)",
            parameters=(
                payload,
                channel,
                process_id,
            ),
        )

    return callback


async def wait_until_listening(
    listener: Listener,
    *channels: str,
) -> None:
    """Block until the listener's backend session is subscribed to ``channels``.

    ``Listener.listen()`` and async iteration issue the ``LISTEN`` statements
    lazily from a background task, so a ``NOTIFY`` sent immediately afterwards
    can race ahead of the subscription and be lost. Polling
    ``pg_listening_channels()`` on the listener's own connection removes that
    race deterministically.
    """
    wanted = set(channels)
    with anyio.fail_after(WAIT_TIMEOUT):
        while True:
            result = await listener.connection.execute(
                "SELECT pg_listening_channels() AS channel",
            )
            active = {row["channel"] for row in result.result()}
            if wanted <= active:
                return
            await asyncio.sleep(POLL_INTERVAL)


async def notify(
    psql_pool: ConnectionPool,
    channel: str = TEST_CHANNEL,
) -> None:
    connection = await psql_pool.connection()
    try:
        await connection.execute(f"NOTIFY {channel}, '{TEST_PAYLOAD}'")
    finally:
        connection.close()


async def wait_for_callback(
    psql_pool: ConnectionPool,
    listener_table_name: str,
    number_of_data: int = 1,
) -> list[dict[str, typing.Any]]:
    """Poll the result table until the callback has inserted ``number_of_data`` rows."""
    with anyio.fail_after(WAIT_TIMEOUT):
        while True:
            rows = await read_test_table(psql_pool, listener_table_name)
            if len(rows) >= number_of_data:
                return rows
            await asyncio.sleep(POLL_INTERVAL)


async def read_test_table(
    psql_pool: ConnectionPool,
    listener_table_name: str,
) -> list[dict[str, typing.Any]]:
    connection = await psql_pool.connection()
    try:
        return (
            await connection.execute(
                f"SELECT * FROM {listener_table_name}",
            )
        ).result()
    finally:
        connection.close()


async def assert_no_callback(
    psql_pool: ConnectionPool,
    listener_table_name: str,
    settle: float = SETTLE_TIMEOUT,
) -> None:
    """Give a notification time to (not) be delivered, then assert nothing landed."""
    await asyncio.sleep(settle)
    rows = await read_test_table(psql_pool, listener_table_name)
    assert not rows


async def clear_test_table(
    psql_pool: ConnectionPool,
    listener_table_name: str,
) -> None:
    connection = await psql_pool.connection()
    await connection.execute(
        f"DELETE FROM {listener_table_name}",
    )
    connection.close()


@pytest.mark.usefixtures("create_table_for_listener_tests")
async def test_listener_listen(
    psql_pool: ConnectionPool,
    listener_table_name: str,
) -> None:
    """Test that single connection can execute queries."""
    listener = await construct_listener(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
    )
    await listener.startup()
    listener.listen()

    await wait_until_listening(listener, TEST_CHANNEL)
    await notify(psql_pool=psql_pool)

    rows = await wait_for_callback(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
    )
    assert rows[0]["payload"] == TEST_PAYLOAD
    assert rows[0]["channel"] == TEST_CHANNEL

    await listener.shutdown()


@pytest.mark.usefixtures("create_table_for_listener_tests")
async def test_listener_asynciterator(
    psql_pool: ConnectionPool,
    listener_table_name: str,
) -> None:
    listener = await construct_listener(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
    )
    await listener.startup()

    async def trigger() -> None:
        # Iteration subscribes lazily inside ``__anext__``; wait for the
        # subscription before notifying so the message can't be lost.
        await wait_until_listening(listener, TEST_CHANNEL)
        await notify(psql_pool=psql_pool)

    # Bound the iteration: if the notification is never delivered this fails
    # fast instead of blocking the session forever.
    with anyio.fail_after(WAIT_TIMEOUT):
        async with anyio.create_task_group() as task_group:
            task_group.start_soon(trigger)
            async for listener_msg in listener:
                assert listener_msg.channel == TEST_CHANNEL
                assert listener_msg.payload == TEST_PAYLOAD
                break

    await listener.shutdown()


@pytest.mark.usefixtures("create_table_for_listener_tests")
async def test_listener_abort(
    psql_pool: ConnectionPool,
    listener_table_name: str,
) -> None:
    listener = await construct_listener(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
    )
    await listener.startup()
    listener.listen()

    await wait_until_listening(listener, TEST_CHANNEL)
    await notify(psql_pool=psql_pool)

    await wait_for_callback(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
    )

    await listener.shutdown()

    await clear_test_table(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
    )

    await notify(psql_pool=psql_pool)

    await assert_no_callback(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
    )


async def test_listener_start_exc(
    psql_pool: ConnectionPool,
    listener_table_name: str,
) -> None:
    listener = await construct_listener(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
    )

    with pytest.raises(expected_exception=ListenerStartError):
        listener.listen()


async def test_listener_double_start_exc(
    psql_pool: ConnectionPool,
    listener_table_name: str,
) -> None:
    listener = await construct_listener(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
    )
    await listener.startup()

    try:
        with pytest.raises(expected_exception=ListenerStartError):
            await listener.startup()
    finally:
        await listener.shutdown()


@pytest.mark.usefixtures("create_table_for_listener_tests")
async def test_listener_more_than_one_callback(
    psql_pool: ConnectionPool,
    listener_table_name: str,
) -> None:
    additional_channel = "test_channel_2"
    listener = await construct_listener(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
    )
    await listener.add_callback(
        channel=additional_channel,
        callback=construct_insert_callback(
            listener_table_name=listener_table_name,
        ),
    )
    await listener.startup()
    listener.listen()

    await wait_until_listening(listener, TEST_CHANNEL, additional_channel)
    for channel in [TEST_CHANNEL, additional_channel]:
        await notify(
            psql_pool=psql_pool,
            channel=channel,
        )

    await wait_for_callback(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
        number_of_data=2,
    )

    connection = await psql_pool.connection()
    query_result = await connection.execute(
        querystring=(f"SELECT * FROM {listener_table_name} WHERE channel = $1"),
        parameters=(additional_channel,),
    )

    data_result = query_result.result()[0]

    assert data_result["channel"] == additional_channel

    await listener.shutdown()


@pytest.mark.usefixtures("create_table_for_listener_tests")
async def test_listener_clear_callbacks(
    psql_pool: ConnectionPool,
    listener_table_name: str,
) -> None:
    listener = await construct_listener(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
    )

    await listener.startup()
    listener.listen()

    await listener.clear_channel_callbacks(
        channel=TEST_CHANNEL,
    )

    await notify(psql_pool=psql_pool)

    await assert_no_callback(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
    )

    await listener.shutdown()


@pytest.mark.usefixtures("create_table_for_listener_tests")
async def test_listener_clear_all_callbacks(
    psql_pool: ConnectionPool,
    listener_table_name: str,
) -> None:
    listener = await construct_listener(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
    )

    await listener.startup()
    listener.listen()

    await listener.clear_all_channels()

    await notify(psql_pool=psql_pool)

    await assert_no_callback(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
    )

    await listener.shutdown()
