---
title: What is PSQLPy?
---

`PSQLPy` is a new Python driver for PostgreSQL fully written in Rust. It was inspired by `Psycopg3` and `AsyncPG`.

With `PSQLPy` you can:
- Make an interaction with the PostgeSQL in your application much faster (2-3 times).
- Be sure that there won't be any unexpected errors.
- Don't usually go to the documentation to search every question - we have awesome docstrings for every component.
- Use `MyPy` (or any other Python type checker) with confidence that exactly the types specified in the typing will be returned.
- Concentrate on writing your code, not understanding new abstractions in this library, we only have classes which represents PostgreSQL object (transaction, cursor, etc).

::: info
It is extremely important to understand that the library will provide a noticeable acceleration in working with the database only if your queries are optimized.
Otherwise, there will be acceleration, but not so significant
:::

## Important notes
All statements will be prepared by default. You can read more about it here [PostgreSQL Docs](https://www.postgresql.org/docs/current/sql-prepare.html)
But in some situations this behavior can break you application. As an example, if you are using `PGBouncer` with `Transaction Pooling Mode` [Docs](https://devcenter.heroku.com/articles/best-practices-pgbouncer-configuration#transaction-pooling-mode-recommended) or `Statement Pooling Mode` [Docs](https://devcenter.heroku.com/articles/best-practices-pgbouncer-configuration#transaction-pooling-mode-recommended) you need to disable statement preparation. You can read how to do it in the next parts of the documentation.

## Join community!
You can get support from the creators of `PSQLPy` in some social media:
- [Telegram](https://t.me/+f3Y8mYKgXxhmYThi)