---
home: true
icon: home
title: PSQLPy documentation
heroImage: ./logo.png
heroText: PSQLPy
tagline: Asynchronous Python PostgreSQL driver written in Rust
actions:
  - text: Let's start
    type: primary
    link: ./introduction/lets_start

  - text: What is PSQLPy?
    link: ./introduction/introduction

highlights:
  -  features:
      - title: Fully Asynchronous
        details: Support native rust/python asynchronous. It's easy as it seems.

      - title: Fully Typed
        details: PSQLPy has type for each class, function, method and etc.

      - title: Blazingly Fast
        details: PSQLPy beats others PostgreSQL drivers in different benchmarks.

      - title: Under active development
        details: PSQLPy is under active development.
---
## What is PSQLPy
`PSQLPy` is a new Python driver for `PostgreSQL` fully written in Rust. It was inspired by `Psycopg3` and `AsyncPG`.
This project has two main goals:
Make a interaction with the database as fast as possible and now `PSQLPy` shows itself to be times faster than the above drivers.
Don't make useless abstractions and make it like a mirror to `PostgreSQL`.

It has all necessary components to create high-load and fault tolerance applications.

## How to install
::: tabs
@tab pip

```bash
pip install psqlpy
```

@tab poetry

```bash
poetry add psqlpy
```

@tab git

```bash
pip install git+https://github.com/qaspen-python/psqlpy
```

:::

## Join community!
You can get support from the creators and users of `PSQLPy` in some social media:
- [Telegram](https://t.me/+f3Y8mYKgXxhmYThi)