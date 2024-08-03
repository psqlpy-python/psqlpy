# psqlpy-stress

Here you can load test a `psqlpy`, `asycnpg` and `psycopg` drivers in order to compare their performance.

## How to run

1. Run Postgresql database somewhere. We recommend run an external database outside of your local machine.
2. Store `database_dsn` in `settings.py`
3. Install dependencies: `poetry install`
4. Run migrations: `alembic upgrade head`
5. Fill database with mock data: `python psqlpy_stress/mocker.py`
6. Run the application: `gunicorn psqlpy_stress.app:app -b 127.0.0.1:8080 -w 4 -k aiohttp.GunicornWebWorkeron`
7. Install [bombardier](https://github.com/codesenberg/bombardier)
8. Start bombarding application: `bombardier -c 10 -d 60s -l http://127.0.0.1:8080/psqlpy-simple-transaction-select`

You can change driver inside your url in order to test specific driver like:

- `http://127.0.0.1:8080/psqlpy-simple-transaction-select`
- `http://127.0.0.1:8080/asyncpg-simple-transaction-select`
- `http://127.0.0.1:8080/psycopg-simple-transaction-select`

Also you can go and check out all available urls in `psqlpy_stress.api.plain_queries`

## Results interpretation

You would receive such data as output after bombarding

```
Done!
Statistics        Avg      Stdev        Max
  Reqs/sec       485.28    115.61     1065.30
   Latency       20.60ms   5.01ms     67.93ms
   Latency Distribution
      50%   20.89ms
      75%   23.41ms
      90%   24.4ms
      95%   30.76ms
      99%   32.73ms
   HTTP codes:
      1xx - 0, 2xx - 29117, 3xx - 0, 4xx - 0, 5xx - 0
      others - 0
   Throughput: 116.55KB/s
```

You can compare drivers by received percentiles and reqs/sec performance.
