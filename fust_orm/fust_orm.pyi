"""
Stub file for the 'fust_orm' rust module.

This file provides type hints for IDEs and static type checkers like mypy,
enabling autocompletion and better code analysis.
"""

from collections.abc import Awaitable
from typing import Any

class QueryBuilder[T]:
    """
    A builder for creating SQL SELECT queries.

    This class allows for the programmatic and chainable construction of queries.
    """

    # Although __init__ is not directly called, it's useful for type checkers
    # to understand the object's structure.
    def __init__(self, table: str, columns: list[str]): ...
    @staticmethod
    def select(table: str, columns: list[str]) -> T:
        """
        Creates a new SELECT query.

        Args:
            table: The name of the table to select from.
            columns: A list of column names to retrieve.

        Returns:
            A new QueryBuilder instance.
        """
        ...

    def where_(self: T, column: str, value: str) -> T:
        """
        Adds a 'WHERE column = ?' clause to the query.

        Args:
            column: The name of the column for the condition.
            value: The value to match against the column.

        Returns:
            The QueryBuilder instance to allow for method chaining.
        """
        ...

class Database:
    """
    Represents an asynchronous connection to a SQLite database.
    """

    # __init__ is not exposed to Python. Instances are created via the async
    # static method 'connect'.
    def __init__(self) -> None: ...
    @staticmethod
    def connect(db_path: str) -> Awaitable["Database"]:
        """
        Asynchronously connects to a SQLite database file.

        This is the factory function to create Database instances.

        Args:
            db_path: The file path to the SQLite database.

        Returns:
            An awaitable that resolves to a new Database instance.
        """
        ...

    async def execute(self, query: QueryBuilder) -> list[dict[str, Any]]:
        """
        Asynchronously executes a query built with QueryBuilder.

        Args:
            query: A QueryBuilder instance representing the query to execute.

        Returns:
            A list of dictionaries, where each dictionary represents a row.
        """
        ...
