import asyncio
from collections.abc import Generator
import pytest

from fust_orm import Database, Model, ColumnField, select


class User(Model):
    id: ColumnField[int]
    name: ColumnField[str]
    age: ColumnField[int]
    manager_id: ColumnField[int]


class Product(Model):
    id: ColumnField[int]
    product_name: ColumnField[str]


@pytest.fixture(scope="session")
def event_loop() -> Generator[asyncio.AbstractEventLoop]:
    try:
        loop = asyncio.get_running_loop()
    except RuntimeError:
        loop = asyncio.new_event_loop()
    yield loop
    loop.close()


@pytest.fixture
async def db() -> Database:
    database = await Database.connect("sqlite::memory:")
    await database.execute(
        """
        CREATE TABLE user (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            age INTEGER,
            manager_id INTEGER
        );
        """
    )
    await database.execute(
        """
        CREATE TABLE product (
            id INTEGER PRIMARY KEY,
            product_name TEXT NOT NULL
        );
        """
    )
    return database


async def test_db_connection(db: Database) -> None:
    assert db is not None
    result = await db.execute("SELECT 1")
    print(result)
    assert result == [{"1": 1}]


def test_model_structure() -> None:
    assert isinstance(User.id, ColumnField)
    assert isinstance(User.name, ColumnField)


async def test_raw_sql_execution(db: Database) -> None:
    await db.execute(
        """
        INSERT INTO user (id, name, age, manager_id) VALUES
        (1, 'Alice', 30, NULL),
        (2, 'Bob', 25, 1),
        (3, 'Charlie', 35, 1);
        """
    )
    users = await db.execute("SELECT * FROM user WHERE age > 28 ORDER BY id;")
    assert len(users) == 2
    assert users[0]["name"] == "Alice"
    assert users[1]["name"] == "Charlie"


async def test_select_function_with_raw_sql_and_params(db: Database) -> None:
    await db.execute(
        """
        INSERT INTO user (id, name, age, manager_id) VALUES
        (1, 'Alice', 30, NULL),
        (2, 'Bob', 25, 1),
        (3, 'Charlie', 35, 1);
        """
    )
    query = select("SELECT name FROM user WHERE age = ? AND name = ?", 30, "Alice")
    result = await db.execute(query)
    assert len(result) == 1
    assert result[0]["name"] == "Alice"


async def test_select_all_from_model(db: Database) -> None:
    await db.execute(
        """
        INSERT INTO user (id, name, age, manager_id) VALUES
        (1, 'Alice', 30, NULL),
        (2, 'Bob', 25, 1),
        (3, 'Charlie', 35, 1);
        """
    )
    query = select(User)
    users = await db.execute(query)
    assert len(users) == 3
    assert all(key in users[0] for key in ["id", "name", "age", "manager_id"])


async def test_select_specific_columns(db: Database) -> None:
    await db.execute(
        """
        INSERT INTO user (id, name, age, manager_id) VALUES
        (1, 'Alice', 30, NULL),
        (2, 'Bob', 25, 1),
        (3, 'Charlie', 35, 1);
        """
    )
    query = select(User.name, User.age)
    users = await db.execute(query)
    assert len(users) == 3
    assert "name" in users[0]
    assert "age" in users[0]
    assert "id" not in users[0]


async def test_select_with_simple_where_eq(db: Database) -> None:
    await db.execute(
        """
        INSERT INTO user (id, name, age, manager_id) VALUES
        (1, 'Alice', 30, NULL),
        (2, 'Bob', 25, 1),
        (3, 'Charlie', 35, 1);
        """
    )
    query = select(User.name, User.age == 25)
    users = await db.execute(query)
    assert len(users) == 1
    assert users[0]["name"] == "Bob"


async def test_select_with_comparison_operators(db: Database) -> None:
    await db.execute(
        """
        INSERT INTO user (id, name, age, manager_id) VALUES
        (1, 'Alice', 30, NULL),
        (2, 'Bob', 25, 1),
        (3, 'Charlie', 35, 1);
        """
    )

    q_gt = select(User.name, User.age > 30)
    res_gt = await db.execute(q_gt)
    assert len(res_gt) == 1
    assert res_gt[0]["name"] == "Charlie"

    q_ge = select(User.name, User.age >= 30)
    res_ge = await db.execute(q_ge)
    assert len(res_ge) == 2

    q_lt = select(User.name, User.age < 30)
    res_lt = await db.execute(q_lt)
    assert len(res_lt) == 1
    assert res_lt[0]["name"] == "Bob"

    q_ne = select(User.name, User.age != 30)
    res_ne = await db.execute(q_ne)
    assert len(res_ne) == 2


async def test_select_with_is_null_and_is_not_null(db: Database) -> None:
    await db.execute(
        """
        INSERT INTO user (id, name, age, manager_id) VALUES
        (1, 'Alice', 30, NULL),
        (2, 'Bob', 25, 1),
        (3, 'Charlie', 35, 1);
        """
    )

    q_is_null = select(User.name, User.manager_id.is_(None))
    res_is_null = await db.execute(q_is_null)
    assert len(res_is_null) == 1
    assert res_is_null[0]["name"] == "Alice"

    q_is_not_null = select(User.name, User.manager_id.is_not(None))
    res_is_not_null = await db.execute(q_is_not_null)
    assert len(res_is_not_null) == 2


async def test_select_with_unary_plus_for_selection(db: Database) -> None:
    await db.execute(
        """
        INSERT INTO user (id, name, age, manager_id) VALUES
        (1, 'Alice', 30, NULL),
        (2, 'Bob', 25, 1),
        (3, 'Charlie', 35, 1);
        """
    )

    query_combined = select(User.name, +(User.age < 35))
    results_combined = await db.execute(query_combined)
    assert len(results_combined) == 2
    assert "name" in results_combined[0]
    assert "age" in results_combined[0]


async def test_method_aliases_for_operators(db: Database) -> None:
    await db.execute(
        """
        INSERT INTO user (id, name, age, manager_id) VALUES
        (1, 'Alice', 30, NULL),
        (2, 'Bob', 25, 1),
        (3, 'Charlie', 35, 1);
        """
    )

    query_op = select(User.name, User.age > 25)
    query_method = select(User.name, User.age.gt(25))

    res_op = await db.execute(query_op)
    res_method = await db.execute(query_method)

    assert res_op == res_method
    assert len(res_op) == 2


async def test_in_operator(db: Database) -> None:
    await db.execute(
        """
        INSERT INTO user (id, name, age, manager_id) VALUES
        (1, 'Alice', 30, NULL),
        (2, 'Bob', 25, 1),
        (3, 'Charlie', 35, 1);
        """
    )

    target_ages = [25, 35]
    query = select(User.name, User.age.in_(target_ages))
    results = await db.execute(query)

    assert len(results) == 2
    names = sorted([r["name"] for r in results])
    assert names == ["Bob", "Charlie"]


async def test_like_operator(db: Database) -> None:
    await db.execute(
        """
        INSERT INTO user (id, name, age, manager_id) VALUES
        (1, 'Alice', 30, NULL),
        (2, 'Bob', 25, 1),
        (3, 'Charlie', 35, 1);
        """
    )

    query = select(User.id, User.name.like("A%"))
    results = await db.execute(query)

    assert len(results) == 1
    assert results[0]["id"] == 1


async def test_select_from_multiple_tables_raises_error() -> None:
    with pytest.raises(ValueError):
        select(User.id, Product.product_name)


async def test_select_with_no_arguments_raises_error() -> None:
    with pytest.raises(ValueError):
        select()
