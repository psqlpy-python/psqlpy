from piccolo.columns import JSONB, Array, Integer, Serial, Varchar
from piccolo.table import Table


class User(Table, tablename="users"):
    user_id = Serial(primary_key=True)
    username = Varchar(null=False)


class SomeBigTable(Table, tablename="big_table"):
    big_table_id = Integer(primary_key=True)
    string_field = Varchar(null=False)
    integer_field = Integer(null=False)
    json_field = JSONB(null=False)
    array_field = Array(base_column=Integer(), null=False)
