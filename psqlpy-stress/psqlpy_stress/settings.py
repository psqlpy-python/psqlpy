import enum

from pydantic_settings import BaseSettings


class DriversEnum(enum.StrEnum):
    PSQLPY: str = "psqlpy"
    ASYNCPG: str = "asyncpg"
    PSYCOPG: str = "psycopg"


class Settings(BaseSettings):
    database_url: str = "postgres://postgres:postgres@127.0.0.1:5432/postgres"
    max_pool_size: int = 20

    app_port: int = 8080

    influx_db_address: str = "http://127.0.0.1:8086"
    influx_db_token: str = "J9A2-ZrrxJLA6pmOmCvJlqc913BbXFbJJA-HmG7cUm8epwHe32Yv-V_MtE2xTZT9j_hIy064ZwF6cZ30Hm2mQw=="
    influx_db_organization: str = "psqlpy-stress-test"
    influx_db_bucket: str = "psqlpy-stress-bucket"
    influx_db_measurment: str = "stress-test-timings"
    influx_db_measurment_tag: str = "stress-test-timings-tag"
    influx_db_client_app_key: str = "influxdb"


settings = Settings()
