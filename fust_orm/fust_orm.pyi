from typing import (
    Any,
    ClassVar,
    Coroutine,
    Dict,
    Generic,
    Iterable,
    List,
    Optional,
    Type,
    TypeVar,
    Union,
    overload,
)

# A generic type variable to represent the column's data type (e.g., int, str).
T = TypeVar("T")

class WhereCondition:
    """Represents a single condition in a SQL WHERE clause (e.g., "id = 5").

    Instances are created by applying operators to `ColumnField` objects.
    They can be marked for inclusion in the SELECT clause using the unary `+` operator.
    """

    column_name: str
    operator: str
    value: Any
    select_column: bool

    def __pos__(self) -> "WhereCondition":
        """Marks the condition's column for inclusion in the SELECT clause.

        Returns:
            A new `WhereCondition` instance with the `select_column` flag set to True.
        """

class ColumnField(Generic[T]):
    """A descriptor representing a database column on a Model.

    It translates Python operations into `WhereCondition` objects for building SQL queries.
    """

    def __eq__(self, other: Any) -> "WhereCondition":  # type: ignore[override]
        """Creates an equality condition (`=` or `IS`).

        Args:
            other: The value to compare with. Handles `None` correctly.

        Returns:
            A `WhereCondition` object.
        """

    def __ne__(self, other: Any) -> "WhereCondition":  # type: ignore[override]
        """Creates an inequality condition (`!=` or `IS NOT`).

        Args:
            other: The value to compare with. Handles `None` correctly.

        Returns:
            A `WhereCondition` object.
        """

    def __gt__(self, other: Any) -> "WhereCondition":
        """Creates a "greater than" condition (`>`).

        Args:
            other: The value to compare with.

        Returns:
            A `WhereCondition` object.
        """

    def __ge__(self, other: Any) -> "WhereCondition":
        """Creates a "greater than or equal to" condition (`>=`).

        Args:
            other: The value to compare with.

        Returns:
            A `WhereCondition` object.
        """

    def __lt__(self, other: Any) -> "WhereCondition":
        """Creates a "less than" condition (`<`).

        Args:
            other: The value to compare with.

        Returns:
            A `WhereCondition` object.
        """

    def __le__(self, other: Any) -> "WhereCondition":
        """Creates a "less than or equal to" condition (`<=`).

        Args:
            other: The value to compare with.

        Returns:
            A `WhereCondition` object.
        """

    def eq(self, other: Any) -> "WhereCondition":
        """Creates an equality condition (`=` or `IS`). Alias for `__eq__`."""

    def ne(self, other: Any) -> "WhereCondition":
        """Creates an inequality condition (`!=` or `IS NOT`). Alias for `__ne__`."""

    def gt(self, other: Any) -> "WhereCondition":
        """Creates a "greater than" condition (`>`). Alias for `__gt__`."""

    def ge(self, other: Any) -> "WhereCondition":
        """Creates a "greater than or equal to" condition (`>=`). Alias for `__ge__`."""

    def lt(self, other: Any) -> "WhereCondition":
        """Creates a "less than" condition (`<`). Alias for `__lt__`."""

    def le(self, other: Any) -> "WhereCondition":
        """Creates a "less than or equal to" condition (`<=`). Alias for `__le__`."""

    def like(self, pattern: str) -> "WhereCondition":
        """Creates a `LIKE` condition (case-sensitive pattern matching).

        Args:
            pattern: The SQL pattern (e.g., "J%").

        Returns:
            A `WhereCondition` object.
        """

    def ilike(self, pattern: str) -> "WhereCondition":
        """Creates an `ILIKE` condition (case-insensitive pattern matching).

        Note: `ILIKE` is specific to databases like PostgreSQL.

        Args:
            pattern: The SQL pattern (e.g., "j%").

        Returns:
            A `WhereCondition` object.
        """

    def in_(self, values: Iterable[Any]) -> "WhereCondition":
        """Creates an `IN` condition to check for a value within an iterable.

        Args:
            values: An iterable (list, tuple, set, etc.) of values.

        Returns:
            A `WhereCondition` object.
        """

    def is_(self, value: Any) -> "WhereCondition":
        """Creates an `IS` condition. A more readable alternative to `==`."""

    def is_not(self, value: Any) -> "WhereCondition":
        """Creates an `IS NOT` condition. A more readable alternative to `!=`."""

    def __repr__(self) -> str: ...

class Database:
    """The main class for interacting with a database.

    This class provides an asynchronous interface for connecting to and executing
    queries against various SQL databases supported by the underlying driver.
    """

    @staticmethod
    def connect(db_url: str) -> Coroutine[Any, Any, "Database"]:
        """Asynchronously connects to a database using a connection URL.

        The appropriate database driver is selected based on the URL scheme.
        Examples:
        - "sqlite://path/to/my_database.db"
        - "postgres://user:password@host/database"
        - "mysql://user:password@host/database"

        Args:
            db_url: The full connection string for the database.

        Returns:
            An awaitable that resolves to a new Database instance.
        """

    @overload
    def execute(
        self, query: "QueryBuilder"
    ) -> Coroutine[Any, Any, List[Dict[str, Any]]]:
        """Executes an SQL query asynchronously.

        The query can be either a QueryBuilder instance (e.g., from a `select()` call)
        or a raw SQL string.

        Args:
            query: The QueryBuilder instance or the raw SQL string to execute.

        Returns:
            An awaitable that resolves to a list of dictionaries, where each
            dictionary represents a row from the query result.
        """

    @overload
    def execute(self, query: str) -> Coroutine[Any, Any, List[Dict[str, Any]]]: ...

class Model:
    """A base class for user-defined models.

    Inheriting from this class triggers `__init_subclass__` to automatically
    set up `ColumnField` descriptors for all annotated attributes, linking the
    Python class to a database table.
    """

    __table_name__: ClassVar[Optional[str]]

    def __init_subclass__(cls) -> None: ...

class QueryBuilder:
    """An opaque object representing a query to be executed.

    Instances of this class are created by the `select()` function and are
    passed to `Database.execute()`. You do not need to interact with its
    methods or attributes directly.
    """

@overload
def select(sql_query: str, *params: Any) -> "QueryBuilder":
    """Creates a query builder instance from a raw SQL string.

    This is one of two ways to use the `select` function.

    1.  **ORM-style query building:**
        Pass Model classes, `ColumnField` instances, or `WhereCondition` objects.
        Example: `select(User, where(User.age > 18))`

    2.  **Raw SQL execution:**
        Pass a SQL string as the first argument, followed by its parameters
        which will be safely bound to the query.
        Example: `select("SELECT * FROM users WHERE name = ?", "John")`

    Args:
        sql_query: The raw SQL string with placeholders (`?`).
        *params: Values to be safely bound to the placeholders in the query.

    Returns:
        A `QueryBuilder` instance to be passed to `database.execute()`.
    """

@overload
def select(
    *clauses: Union[ColumnField[Any], WhereCondition, Type[Model]],
) -> "QueryBuilder":
    """Creates a query builder instance using an ORM-style syntax.

    This is the recommended and most common way to use `select()`. You can
    pass Model classes, `ColumnField` instances, or `WhereCondition` objects.

    Args:
        *clauses: The components of your query, including models,
                  columns, and where conditions.

    Returns:
        A `QueryBuilder` instance for executing the query.
    """
