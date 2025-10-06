use std::sync::Arc;

use pyo3::prelude::*;

/// Represents a single condition in a SQL WHERE clause (e.g., "id = 5").
///
/// Instances of this struct are typically created by applying comparison
/// operators to `ColumnField` objects (e.g., `User.id == 5`). They are then
/// collected by a `QueryBuilder` to construct the final query.
#[pyclass]
#[derive(Debug, Clone)]
pub struct WhereCondition {
    pub column_name: String,
    pub operator: String,
    pub value: Arc<Py<PyAny>>, // wrapped to be passed between Rust and Python
    /// A flag indicating if the column in this condition should also be implicitly
    /// added to the `SELECT` clause of the query.
    /// This is toggled by the unary `+` operator.
    /// For example, in `select(User.id, +(User.age > 30))`, `select_column` for the
    /// resulting condition will be `true`.
    pub select_column: bool,
}

#[pymethods]
impl WhereCondition {
    /// Overloads the unary plus (+) operator.
    ///
    /// Returns a new `WhereCondition` instance with a flag indicating that
    /// this column should also be included in the final SELECT statement.
    /// This allows for expressive queries like `select(User.id, +(User.age > 30))`,
    /// which both filters by `age` and selects it.
    fn __pos__(&self) -> Self {
        let mut new_condition = self.clone();
        new_condition.select_column = true;
        new_condition
    }
}
