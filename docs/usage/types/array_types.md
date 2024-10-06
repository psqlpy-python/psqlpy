---
title: Array Types
---
For type safety and better performance we have predefined array types.

| PSQLPy Array Type | PostgreSQL Array Type |
| :---: | :---: |
| BoolArray | BOOLEAN ARRAY |
| UUIDArray | UUID ARRAY |
| VarCharArray | VarChar ARRAY |
| TextArray | Text ARRAY |
| Int16Array | INT2 ARRAY |
| Int32Array | INT4 ARRAY |
| Int64Array | INT8 ARRAY |
| Float32Array | FLOAT4 ARRAY |
| Float64Array | FLOAT8 ARRAY |
| MoneyArray | MONEY ARRAY |
| IpAddressArray | INET ARRAY |
| JSONBArray | JSONB ARRAY |
| JSONArray | JSON ARRAY |
| DateArray | DATE ARRAY |
| TimeArray | TIME ARRAY |
| DateTimeArray | TIMESTAMP ARRAY |
| DateTimeTZArray | TIMESTAMPTZ ARRAY |
| MacAddr6Array | MACADDR ARRAY |
| MacAddr8Array | MACADDR8 ARRAY |
| NumericArray | NUMERIC ARRAY |
| PointArray | POINT ARRAY |
| BoxArray | BOX ARRAY |
| PathArray | PATH ARRAY |
| LineArray | LINE ARRAY |
| LsegArray | LSEG ARRAY |
| CircleArray | CIRCLE ARRAY |
| IntervalArray | INTERVAL ARRAY |

### Example:

```python
from psqlpy import ConnectionPool
from psqlpy.extra_types import TextArray


async def main() -> None:
    pool = ConnectionPool()
    result = await pool.execute(
        querystring="SELECT * FROM users WHERE name = ANY($1)",
        parameters=[
            TextArray(["Alex", "Dev", "Who"]),
        ]
    )
```
