

from typing import Protocol, Sequence


class DbModel(Protocol):
    def make_request(self) -> Sequence:
        ...


class Postgres(DbModel):
    def make_request(self) -> Sequence:
        return ["123", "456"]


class MySQL(DbModel):
    def make_request(self) -> Sequence:
        return ("123", "456")


class Requester:

    def __init__(self, database: DbModel) -> None:
        self.database: DbModel = database


req = Requester()
req.database.make_request()
