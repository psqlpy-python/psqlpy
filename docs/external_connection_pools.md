---
title: External connection pools
---

PSQLPy supports external connection pools like [PgBouncer](https://www.pgbouncer.org/) or [Supavisor](https://github.com/supabase/supavisor).

Usually, external connection pools have 3 main [modes](https://www.pgbouncer.org/features.html): `Session`, `Transaction` and `Statement`.

If you use `Session` mode, there is nothing you have to do, just use PSQLPy as usual.

But there are a few conditions that must be met to make `Transaction` and `Statement` work.

### Disable statement preparation
Disable statement preparation for any sql statement execution (if a method has `prepared` parameter, set it to `False`).

### Execute statement only in transaction
Each statement must be executed in a transaction.

```python
db_pool = ConnectionPool(...)

async with db_pool.acquire() as conn:
    async with conn.transaction() as transaction:
        await transaction.execute("SELECT 1", prepared=False)
```
