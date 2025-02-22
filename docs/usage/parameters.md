---
title: Passing parameters to SQL queries
---

We support two variant of passing parameters to sql queries.

::: tabs
@tab Parameters sequence

You can pass parameters as some python Sequence.

Placeholders in querystring must be marked as `$1`, `$2` and so on,
depending on how many parameters you have.

```python
async def main():
    ...

    await connection.execute(
        querystring="SELECT * FROM users WHERE id = $1",
        parameters=(101,),
    )
```

@tab Parameters mapping

If you prefer use named arguments, we support it too.
Placeholder in querystring must look like `$(parameter)p`.

If you don't pass parameter but have it in querystring, exception will be raised.

```python
async def main():
    ...

    await connection.execute(
        querystring="SELECT * FROM users WHERE id = $(user_id)p",
        parameters=dict(user_id=101),
    )
```

:::