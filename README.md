# fust-orm 🚀

An experimental, blazingly fast, and intuitive asynchronous ORM for Python, written in Rust.

[![CI](https://github.com/dobrotacreator/fust-orm/actions/workflows/ci.yml/badge.svg)](https://github.com/dobrotacreator/fust-orm/actions/workflows/ci.yml)
[![PyPI version](https://img.shields.io/pypi/v/fust-orm.svg?color=blue)](https://pypi.org/project/fust-orm/)
[![PyPI - Python Version](https://img.shields.io/pypi/pyversions/fust-orm?color=blue)](https://pypi.org/project/fust-orm)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](./LICENSE)

---

⚠️ **Disclaimer**
`fust-orm` is currently in an early, experimental stage. The API is subject to change, and it is not yet recommended for production use.

## Core Ideas

The goal of `fust-orm` is to provide a Python ORM that combines two key ideas:

*   **Performance**: By leveraging Rust and the excellent `sqlx` library, `fust-orm` aims to deliver performance that significantly surpasses traditional Python ORMs.
*   **Simplicity**: The syntax is designed to be clean, expressive, and feel like native Python, allowing you to focus on your application logic instead of wrestling with the ORM.

## Features

*   **Rust-Powered Core**: All logic is implemented in Rust for maximum speed.
*   **Intuitive Query Building**: Use standard Python comparison operators (`==`, `>`, `!=`, etc.) to build `WHERE` clauses.
*   **Asynchronous from the Ground Up**: Built with `async/await` in mind.
*   **Type-Safe Models**: Define your models using standard Python type annotations.
*   **Raw SQL Fallback**: Drop down to raw SQL for complex queries whenever you need to.
*   **Multi-Database Support**: Thanks to `sqlx`, it's designed to work with various databases like SQLite, PostgreSQL, and MySQL.

## Installation

```sh
pip install fust-orm
```

## Quickstart

Here is a brief overview of how to define models and build queries with fust-orm.

### 1. Define Your Models

Create a class that inherits from `Model`. The table name is either inferred from the class name (converted to snake_case) or explicitly set with `__table_name__`. Define columns using `ColumnField` and standard Python type hints.

```python
from fust_orm import Model, ColumnField

class User(Model):
    __table_name__ = "users"

    id: ColumnField[int]
    name: ColumnField[str]
    age: ColumnField[int]
    is_active: ColumnField[bool]
```

### 2. Connect to the Database

Create an asynchronous connection to your database. `fust-orm` uses a connection URL to determine the driver.

```python
import asyncio
from fust_orm import Database

async def main():
    # Connect to an in-memory SQLite database
    db = await Database.connect("sqlite::memory:")

    # ... execute queries
```

### 3. Build and Execute Queries

The `select()` function is the main entry point for building queries.

#### Selecting Specific Columns

Pass `ColumnField` attributes to `select()` to choose which columns to retrieve.

```python
from fust_orm import select

# SELECT name, age FROM users;
query = select(User.name, User.age)
results = await db.execute(query)
# [{'name': 'Alice', 'age': 30}, ...]
```

#### Filtering Data with `WHERE` clauses

Use standard Python operators on `ColumnField` attributes to create `WHERE` conditions.

```python
# SELECT name FROM users WHERE age > 25;
query = select(User.name, User.age > 25)
users = await db.execute(query)
```

#### Combining Selection and Filtering with `+`

You can use a unary `+` on a condition to automatically include that column in the `SELECT` statement. This avoids repetition.

```python
# SELECT age FROM users WHERE age < 35;
query = select(+(User.age < 35))
results = await db.execute(query)
# [{'age': 30}, {'age': 25}]
```

This can be combined with other columns:

```python
# SELECT name, age FROM users WHERE age < 35;
query = select(User.name, +(User.age < 35))
```

#### Using `IN` and `LIKE`

More complex conditions are also available as methods.

```python
# SELECT name FROM users WHERE id IN (1, 3);
query = select(User.name, User.id.in_([1, 3]))

# SELECT name FROM users WHERE surname LIKE 'A%';
query = select(User.name, User.surname.like("A%"))
```

#### Raw SQL

For complex scenarios, you can always fall back to raw SQL with safe, parameterized queries.

```python
query = select("SELECT name FROM users WHERE age > ? AND is_active = ?", 30, True)
active_users = await db.execute(query)
```

## Roadmap

`fust-orm` is actively developing towards a more complete and powerful feature set. The goal is to enable highly expressive queries like this:

```python
# Dream Syntax
results = await db.execute(
    select(
        User.name,
        +(User.age < 35),
        Users.posts(Post.id, Post.likes >= 0), # Eager-load related posts
        limit=50,
        offset=100
    )
)
```

Here are the features that are planned to be implemented next:

- [ ] Query Modifiers: limit, offset, and order_by.
- [ ] Data Manipulation: insert and update operations.
- [ ] Aggregations: group_by and having clauses.
- [ ] Column Metadata: Define primary_key, foreign_key, index, etc., directly in ColumnField.
- [ ] Model Relationships: Define relations (e.g., one-to-many, many-to-many) directly on models.
- [ ] Automatic Joins: A resolver that uses relationship info to automatically perform JOIN or SELECT IN queries.
- [ ] Migration Tool: A simple, powerful tool for automatically creating and managing database migrations.

## License

This project is licensed under the MIT License.
