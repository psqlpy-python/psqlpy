---
title: Contribution guide
---

We love contributions. This guide is for all fellas who want to make psqlpy better together.

There are several rules for contributors:
- Please do not add malware.
- Please make sure that your request solves the problem.

If you struggle with something or feel frustrated, you either create an issue, create a [discussions](https://github.com/qaspen-python/psqlpy/discussions). page or publish a draft PR and ask your question in the description.

We have lots of tests in CI. But since CI runs from first-time contributors should be approved, you better test locally. It just takes less time to prepare PR for merging.

## Setting up environment
Since it's rust-first project you need to install rust on your computer.
The best way to do it is check [official site](https://www.rust-lang.org/tools/install).

After you install rust, you must prepare you python environment.
One of the best ways is follow [maturin offical documentation](https://www.maturin.rs/installation) but below you can find all necessary commands.

```bash
> python3 -m venv .venv
> source .venv/bin/activate
> pip install -U pip maturin
```

Then you need to build `PSQLPy` project.
```bash
maturin develop
```

After this step project is built and installed in your python environment you created in previous step.

## Linting and type checking
We have pre-commit configured with all our settings. We highly recommend you to install it as a git hook using pre-commit install command.

But even without installation, you can run all lints manually:

```bash
pre-commit run -a
```

## Testing
You need to have `PostgreSQL` server somewhere to run `pytest`.

Default credentials for testing `PostgreSQL` and you can configure it with env:
- host: `localhost` (env: POSTGRES_HOST)
- user: `postgres` (env: POSTGRES_USER)
- password: `postgres` (env: POSTGRES_PASSWORD)
- port: `5432` (env: POSTGRES_PORT)
- dbname: `psqlpy_test` (env: POSTGRES_DBNAME)

We have tests with required SSL mode, so, if you don't want
to run PostgreSQL in SSL mode, you could run

```bash
pytest --ignore="./python/tests/test_ssl_mode.py"
```

If you have PostgreSQL with enabled ssl mode, you need to set path to your `ca_file` in `POSTGRES_CERT_FILE` env.
And run
```bash
pytest
```