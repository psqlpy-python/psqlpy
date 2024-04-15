import datetime
import typing

import influxdb_client
import pytz

from psqlpy_stress.settings import DriversEnum, settings


if typing.TYPE_CHECKING:
    from aiohttp.web import Request
    from influxdb_client.client.influxdb_client_async import InfluxDBClientAsync


P = typing.ParamSpec("P")
R = typing.TypeVar("R")


def write_timings_to_influx(
    measurment_tag: DriversEnum,
) -> typing.Callable[[typing.Callable[P, R]], typing.Callable[P, R]]:
    def wrapper(function: typing.Callable[P, R]) -> typing.Callable[P, R]:
        async def inner(*args: P.args, **kwargs: P.kwargs) -> R:
            t1 = datetime.datetime.now(tz=pytz.UTC)
            result = await function(*args, **kwargs)
            execution_time = datetime.datetime.now(tz=pytz.UTC) - t1
            request: Request = args[0]

            async_influx_db_client: InfluxDBClientAsync = request.app[
                settings.influx_db_client_app_key
            ]
            write_api = async_influx_db_client.write_api()
            record = (
                influxdb_client.Point(settings.influx_db_measurment)
                .tag(settings.influx_db_measurment_tag, measurment_tag)
                .field("time-took", execution_time.total_seconds())
            )
            await write_api.write(
                bucket=settings.influx_db_bucket,
                org=settings.influx_db_organization,
                record=record,
            )

            return result

        return inner

    return wrapper
