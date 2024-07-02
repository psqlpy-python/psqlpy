import asyncio
import random
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
    users_amount = 10000000
    for _ in range(users_amount):
        await pool.execute(
            querystring="INSERT INTO users (username) VALUES($1)",
            parameters=[str(uuid.uuid4())],
        )


def generate_random_array() -> list[str]:
    return [random.randint(50, 500) for _ in range(random.randint(50, 500))]


def generate_random_dict() -> dict[str, str]:
    return {str(uuid.uuid4()): str(uuid.uuid4()) for _ in range(random.randint(50, 500))}


async def fill_big_table() -> None:
    pool = get_pool()
    big_table_amount = 10000000
    for _ in range(big_table_amount):
        await pool.execute(
            "INSERT INTO big_table (string_field, integer_field, json_field, array_field) VALUES($1, $2, $3, $4)",
            parameters=[
                str(uuid.uuid4()),
                random.randint(1, 99999999),
                generate_random_dict(),
                generate_random_array(),
            ],
        )


async def main() -> None:
    await fill_users()


if __name__ == "__main__":
    asyncio.run(main())
