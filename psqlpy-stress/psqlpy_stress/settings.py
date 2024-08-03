import enum

from pydantic_settings import BaseSettings


class DriversEnum(enum.StrEnum):
    PSQLPY: str = "psqlpy"
    ASYNCPG: str = "asyncpg"
    PSYCOPG: str = "psycopg"


class Settings(BaseSettings):
    database_url: str = "postgresql://postgres:postgres@127.0.0.1:5432/postgres"
    max_pool_size: int = 20

    app_port: int = 8080


settings = Settings()
