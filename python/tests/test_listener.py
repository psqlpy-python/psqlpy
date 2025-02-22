from __future__ import annotations

import asyncio
import typing

import pytest
from psqlpy.exceptions import ListenerStartError

if typing.TYPE_CHECKING:
    from psqlpy import Connection, ConnectionPool, Listener

pytestmark = pytest.mark.anyio


TEST_CHANNEL = "test_channel"
TEST_PAYLOAD = "test_payload"


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


async def notify(
    psql_pool: ConnectionPool,
    channel: str = TEST_CHANNEL,
    with_delay: bool = False,
) -> None:
    if with_delay:
        await asyncio.sleep(0.5)

    connection = await psql_pool.connection()
    await connection.execute(f"NOTIFY {channel}, '{TEST_PAYLOAD}'")
    connection.back_to_pool()


async def check_insert_callback(
    psql_pool: ConnectionPool,
    listener_table_name: str,
    is_insert_exist: bool = True,
    number_of_data: int = 1,
) -> None:
    connection = await psql_pool.connection()
    test_data_seq = (
        await connection.execute(
            f"SELECT * FROM {listener_table_name}",
        )
    ).result()

    if is_insert_exist:
        assert len(test_data_seq) == number_of_data
    else:
        assert not len(test_data_seq)
        return

    data_record = test_data_seq[0]

    assert data_record["payload"] == TEST_PAYLOAD
    assert data_record["channel"] == TEST_CHANNEL

    connection.back_to_pool()


async def clear_test_table(
    psql_pool: ConnectionPool,
    listener_table_name: str,
) -> None:
    connection = await psql_pool.connection()
    await connection.execute(
        f"DELETE FROM {listener_table_name}",
    )
    connection.back_to_pool()


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

    await notify(psql_pool=psql_pool)
    await asyncio.sleep(0.5)

    await check_insert_callback(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
    )

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

    asyncio.create_task(  # noqa: RUF006
        notify(
            psql_pool=psql_pool,
            with_delay=True,
        ),
    )

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

    await notify(psql_pool=psql_pool)
    await asyncio.sleep(0.5)

    await check_insert_callback(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
    )

    await listener.shutdown()

    await clear_test_table(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
    )

    await notify(psql_pool=psql_pool)
    await asyncio.sleep(0.5)

    await check_insert_callback(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
        is_insert_exist=False,
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

    with pytest.raises(expected_exception=ListenerStartError):
        await listener.startup()


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

    for channel in [TEST_CHANNEL, additional_channel]:
        await notify(
            psql_pool=psql_pool,
            channel=channel,
        )

    await asyncio.sleep(0.5)
    await check_insert_callback(
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
    await asyncio.sleep(0.5)

    await check_insert_callback(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
        is_insert_exist=False,
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
    await asyncio.sleep(0.5)

    await check_insert_callback(
        psql_pool=psql_pool,
        listener_table_name=listener_table_name,
        is_insert_exist=False,
    )

    await listener.shutdown()
