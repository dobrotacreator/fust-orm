use std::sync::Arc;

use pyo3::{
    exceptions::PyTypeError,
    prelude::*,
    types::{PyList, PyString},
};

use crate::where_condition::WhereCondition;

/// Represents a database column as a Python object.
///
/// This struct acts as a descriptor on a `Model` subclass. It doesn't hold data itself;
/// instead, it captures operations (like comparisons) to build SQL `WHERE` clauses.
/// When you write `MyModel.id == 5`, an instance of `WhereCondition` is created,
/// representing the expression `id = 5`.
#[pyclass(generic)]
#[derive(Debug, Clone)]
pub struct ColumnField {
    pub table_name: String,
    pub column_name: String,
}

/// A helper function to reduce boilerplate when creating WhereCondition instances.
fn create_where_condition(
    column_field: &ColumnField,
    operator: &str,
    value: Py<PyAny>,
) -> PyResult<WhereCondition> {
    Ok(WhereCondition {
        column_name: column_field.column_name.clone(),
        operator: operator.to_string(),
        value: Arc::new(value),
        select_column: false,
    })
}

#[pymethods]
impl ColumnField {
    // --- Standard Comparison Operators ---

    /// Creates an equality condition (`=` or `IS`).
    /// Handles `None` by translating `column == None` to SQL `column IS NULL`.
    fn __eq__(&self, py: Python, other: Py<PyAny>) -> PyResult<WhereCondition> {
        let op = if other.is_none(py) {
            "IS".to_string()
        } else {
            "=".to_string()
        };
        create_where_condition(self, &op, other)
    }

    /// Creates an inequality condition (`!=` or `IS NOT`).
    /// Handles `None` by translating `column != None` to SQL `column IS NOT NULL`.
    fn __ne__(&self, py: Python, other: Py<PyAny>) -> PyResult<WhereCondition> {
        let op = if other.is_none(py) {
            "IS NOT".to_string()
        } else {
            "!=".to_string()
        };
        create_where_condition(self, &op, other)
    }

    /// Creates a "greater than" condition (`>`).
    fn __gt__(&self, _py: Python, other: Py<PyAny>) -> PyResult<WhereCondition> {
        create_where_condition(self, ">", other)
    }

    /// Creates a "greater than or equal to" condition (`>=`).
    fn __ge__(&self, _py: Python, other: Py<PyAny>) -> PyResult<WhereCondition> {
        create_where_condition(self, ">=", other)
    }

    /// Creates a "less than" condition (`<`).
    fn __lt__(&self, _py: Python, other: Py<PyAny>) -> PyResult<WhereCondition> {
        create_where_condition(self, "<", other)
    }

    /// Creates a "less than or equal to" condition (`<=`).
    fn __le__(&self, _py: Python, other: Py<PyAny>) -> PyResult<WhereCondition> {
        create_where_condition(self, "<=", other)
    }

    // --- Method Aliases for Operators ---

    /// Method alias for `==`. Creates an equality condition (`=` or `IS`).
    #[pyo3(text_signature = "($self, value)")]
    fn eq(&self, py: Python, other: Py<PyAny>) -> PyResult<WhereCondition> {
        self.__eq__(py, other)
    }

    /// Method alias for `!=`. Creates an inequality condition (`!=` or `IS NOT`).
    #[pyo3(text_signature = "($self, value)")]
    fn ne(&self, py: Python, other: Py<PyAny>) -> PyResult<WhereCondition> {
        self.__ne__(py, other)
    }

    /// Method alias for `>`. Creates a "greater than" condition.
    #[pyo3(text_signature = "($self, value)")]
    fn gt(&self, py: Python, other: Py<PyAny>) -> PyResult<WhereCondition> {
        self.__gt__(py, other)
    }

    /// Method alias for `>=`. Creates a "greater than or equal to" condition.
    #[pyo3(text_signature = "($self, value)")]
    fn ge(&self, py: Python, other: Py<PyAny>) -> PyResult<WhereCondition> {
        self.__ge__(py, other)
    }

    /// Method alias for `<`. Creates a "less than" condition.
    #[pyo3(text_signature = "($self, value)")]
    fn lt(&self, py: Python, other: Py<PyAny>) -> PyResult<WhereCondition> {
        self.__lt__(py, other)
    }

    /// Method alias for `<=`. Creates a "less than or equal to" condition.
    #[pyo3(text_signature = "($self, value)")]
    fn le(&self, py: Python, other: Py<PyAny>) -> PyResult<WhereCondition> {
        self.__le__(py, other)
    }

    // --- Custom SQL-specific Methods ---

    /// Creates a `LIKE` condition (case-sensitive pattern matching).
    /// Example: `User.name.like("J%")`
    fn like(&self, _py: Python, pattern: Py<PyString>) -> PyResult<WhereCondition> {
        create_where_condition(self, "LIKE", pattern.into())
    }

    /// Creates an `ILIKE` condition (case-insensitive pattern matching).
    /// Note: `ILIKE` is specific to databases like PostgreSQL. For others,
    /// you might need to use `LOWER(column) LIKE LOWER(pattern)`.
    /// Example: `User.name.ilike("j%")`
    fn ilike(&self, _py: Python, pattern: Py<PyString>) -> PyResult<WhereCondition> {
        create_where_condition(self, "ILIKE", pattern.into())
    }

    /// Creates an `IN` condition to check for a value within any iterable.
    /// Example: `User.status.in_(["active", "pending"])`
    /// Example: `User.status.in_({"active", "pending"})`
    fn in_(&self, py: Python, values: &Bound<PyAny>) -> PyResult<WhereCondition> {
        let py_iterator = values
            .try_iter()
            .map_err(|_| PyTypeError::new_err("Argument to `in_` must be an iterable."))?;
        let list_values =
            PyList::new(py, &py_iterator.collect::<PyResult<Vec<Bound<PyAny>>>>()?)?.into();
        create_where_condition(self, "IN", list_values)
    }

    /// Creates an explicit `IS` condition.
    /// This provides a more readable alternative to `==` for some cases.
    /// Example: `User.manager_id.is_(None)`
    #[pyo3(text_signature = "($self, value)")]
    fn is_(&self, py: Python, value: Py<PyAny>) -> PyResult<WhereCondition> {
        self.__eq__(py, value)
    }

    /// Creates an explicit `IS NOT` condition.
    /// This provides a more readable alternative to `!=` for some cases.
    /// Example: `User.manager_id.is_not(None)`
    #[pyo3(text_signature = "($self, value)")]
    fn is_not(&self, _py: Python, value: Py<PyAny>) -> PyResult<WhereCondition> {
        create_where_condition(self, "IS NOT", value)
    }

    /// Provides a developer-friendly representation of the ColumnField object.
    fn __repr__(&self) -> String {
        format!("<ColumnField: {}.{}>", self.table_name, self.column_name)
    }
}
