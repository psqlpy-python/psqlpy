---
title: Let's Start
---

## Installation

You can install psqlpy with pip, poetry or directly from git using pip:

::: tabs
@tab pip

```bash
pip install psqlpy
```

@tab poetry

```bash
poetry add psqlpy
```

@tab git

```bash
pip install git+https://github.com/psqlpy-python/psqlpy
```

:::

After installation you are ready to start querying!

## First request to the database

There is a minimal example of what you need to do to send your first query and receive result.
Let's assume that we have table `users`:
| id | name | username |
| :---: | :---: | :---: |
| 1 | Aleksandr | chandr-andr |
| 2 | Michail | insani7y |

```python
import asyncio
from typing import Final, Any

from psqlpy import ConnectionPool, QueryResult


async def main() -> None:
    # It uses default connection parameters
    db_pool: Final = ConnectionPool()

    async with db_pool.acquire() as conn:
        results: Final[QueryResult] = await conn.execute(
            "SELECT * FROM users WHERE id = $1",
            [2],
        )

    dict_results: Final[list[dict[Any, Any]]] = results.result()
    db_pool.close()
```

::: tip
It's better to call `close()` on database pool when you application is shutting down.
:::
