# psqlpy-stress

Here you can load test a `psqlpy`, `asycnpg` and `psycopg` drivers in order to compare their performance.

## How to run

1. First of all you have to launch `grafana` and `influxdb` that are present in docker-compose.yaml file.
   You can lanch it via:

```bash
docker compose up
```

2. Log in to IfluxDB. Default credential are `admin`:`admin`. Save token that will be shown to you after login.
3. Create a bucket inside InfluxDB named `psqlpy-stress-bucket`.
4. Connect InfluxDB to grafana.  
   host: `http://influxdb:8086`  
   database: `psqlpy-stress-bucket`  
   user: `admin`  
   password: is your token that you have saved at `step 2`

5. Import a dashboard to a grafana, named `dashboad.yaml`, located in root directory.  
   This dashboard displays certain important parameters:

   - p99 latency
   - p95 latency
   - p90 latency
   - p50 latency
   - mean latency
   - p50 rps

6. Run poetry install in root
7. Apply migrations to database (default database is in docker-compose).  
   You can change `database_url` inside `psqlpy_stress.settings` file in order to connect to external database.

8. Launch application via

```bash
python -m psqlpy_stress.app
```

9. You can start load testing drivers.
