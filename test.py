import asyncio
import json

from rustengine import RustEngine


engine = RustEngine(username="postgres", password="postgres", host="localhost", port=5432, db_name="postgres")

async def main():
    await engine.startup()
    return await engine.execute("SELECT * FROM users")


res = asyncio.run(main())
str_res = res.result()

print(type(str_res))
print(str_res)

# json_res = json.loads(str_res)
# print(json_res)