import asyncio
import json
import time

from rustengine import RustEngine


engine = RustEngine(username="postgres", password="postgres", host="localhost", port=5432)

async def main():
    await engine.startup()
    start = time.time()
    for _ in range(100000):
        await engine.execute("SELECT * FROM users", [])
    print("Took: ", time.time() - start)
    


asyncio.run(main())
# res = asyncio.run(main())
# str_res = res.result()

# print(type(str_res))
# print(str_res)

# json_res = json.loads(str_res)
# print(json_res)