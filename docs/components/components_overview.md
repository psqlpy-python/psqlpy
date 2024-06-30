---
title: Components
---

## Components
- `ConnectionPool`: holds connections in itself and give them when requested.
- `ConnectionPoolBuilder`: Chainable builder for `ConnectionPool`, for people who prefer it over big initialization.
- `Connection`: represents single database connection, can be retrieved from `ConnectionPool`.
- `Transaction`: represents database transaction, can be made from `Connection`.
- `Cursor`: represents database cursor, can be made from `Transaction`.
- `QueryResult`: represents list of results from database.
- `SingleQueryResult`: represents single result from the database.
- `Exceptions`: we have some custom exceptions.

