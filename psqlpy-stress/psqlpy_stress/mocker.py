import asyncio
import uuid

import psqlpy

from psqlpy_stress.settings import settings


def get_pool() -> psqlpy.ConnectionPool:
    return psqlpy.ConnectionPool(
        dsn=settings.database_url,
        max_db_pool_size=settings.max_pool_size,
    )


async def fill_users() -> None:
    pool = get_pool()
    conn = await pool.connection()
    users_amount = 1000000
    for _ in range(users_amount):
        await conn.execute(
            "INSERT INTO users (username) VALUES($1)",
            [str(uuid.uuid4())],
        )


async def main() -> None:
    await fill_users()


if __name__ == "__main__":
    asyncio.run(fill_users())
