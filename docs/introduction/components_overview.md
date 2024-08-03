---
title: Components Overview
---

## Components

- `Connection pool`: holds connections in itself and give them when requested.
- `Connection`: represents single database connection, can be retrieved from `Connection pool`.
- `Transaction`: represents database transaction, can be made from `Connection`.
- `Cursor`: represents database cursor, can be made from `Transaction`.
- `Results`: represents data returned from driver.
- `Exceptions`: we have some custom exceptions. (Section in development)

## Connection pool

Connection pool is the main object in the library. It initializes, creates, holds and gives connection to the user side.
Connection pool must be started up before any other operations.
