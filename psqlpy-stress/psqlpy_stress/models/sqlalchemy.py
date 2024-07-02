from sqlalchemy import Column, Integer, String
from sqlalchemy.dialects.postgresql import ARRAY, JSONB
from sqlalchemy.ext.declarative import declarative_base


Base = declarative_base()


class User(Base):
    __tablename__ = "users"

    user_id = Column(Integer, primary_key=True, autoincrement=True)
    username = Column(String, unique=True, nullable=False)


class SomeBigTable(Base):
    __tablename__ = "big_table"
    big_table_id = Column(Integer, primary_key=True, autoincrement=True)
    string_field = Column(String, unique=False, nullable=False)
    integer_field = Column(Integer, unique=False, nullable=False)
    json_field = Column(JSONB, unique=False, nullable=False)
    array_field = Column(ARRAY(Integer), unique=False, nullable=False)
