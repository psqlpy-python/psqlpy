[![PyPI - Python Version](https://img.shields.io/pypi/pyversions/psqlpy?style=for-the-badge)](https://pypi.org/project/psqlpy/)
[![PyPI](https://img.shields.io/pypi/v/psqlpy?style=for-the-badge)](https://pypi.org/project/psqlpy/)
[![PyPI - Downloads](https://img.shields.io/pypi/dm/psqlpy?style=for-the-badge)](https://pypistats.org/packages/psqlpy)

# PSQLPy - Async PostgreSQL driver for Python written in Rust.

Driver for PostgreSQL written fully in Rust and exposed to Python.
Main goals of the library is speed and type safety.

## Documentation
You can find full documentation here - [PSQLPy documentation](https://qaspen-python.github.io/)

## Installation

You can install package with `pip` or `poetry`.

poetry:

```bash
> poetry add psqlpy
```

pip:

```bash
> pip install psqlpy
```

Or you can build it by yourself. To do it, install stable rust and [maturin](https://github.com/PyO3/maturin).

```
> maturin develop --release
```

## Usage

Usage is as easy as possible.
Create new instance of ConnectionPool and start querying.
You don't need to startup connection pool, the connection pool will create connections as needed.

```python
from typing import Any

from psqlpy import ConnectionPool, QueryResult


async def main() -> None:
    db_pool = ConnectionPool(
        username="postgres",
        password="pg_password",
        host="localhost",
        port=5432,
        db_name="postgres",
        max_db_pool_size=2,
    )

    res: QueryResult = await db_pool.execute(
        "SELECT * FROM users",
    )

    print(res.result())
    db_pool.close()

```

## Benchmarks

We have made some benchmark to compare `PSQLPy`, `AsyncPG`, `Psycopg3`.
Main idea is do not compare clear drivers because there are a few situations in which you need to use only driver without any other dependencies.

**So infrastructure consists of:**

1. AioHTTP
2. PostgreSQL driver (`PSQLPy`, `AsyncPG`, `Psycopg3`)
3. PostgreSQL v15. Server is located in other part of the world, because we want to simulate network problems.
4. Grafana (dashboards)
5. InfluxDB
6. JMeter (for load testing)

The results are very promising! `PSQLPy` is faster than `AsyncPG` at best by 2 times, at worst by 45%. `PsycoPG` is 3.5 times slower than `PSQLPy` in the worst case, 60% in the best case.
