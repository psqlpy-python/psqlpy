import typing
from urllib.parse import urlparse


def parse_postgres_url(url: str) -> dict[str, typing.Any]:
    result = urlparse(url)
    username = result.username
    password = result.password
    host = result.hostname
    port = result.port
    database = result.path[1:]  # Strip the leading '/'

    return {
        "user": username,
        "password": password,
        "host": host,
        "port": port,
        "database": database,
    }
