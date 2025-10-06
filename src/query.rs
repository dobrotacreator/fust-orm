use std::collections::HashSet;
use std::sync::Arc;

use crate::column_field::ColumnField;
use crate::error::FustOrmError;
use crate::model::Model;
use crate::where_condition::WhereCondition;
use log::debug;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyFloat, PyInt, PyList, PyString, PyTuple, PyType};

/// Represents the two modes for building a query.
#[derive(Debug, Clone)]
enum QueryType {
    /// A query built programmatically from Model fields and conditions.
    Structured {
        table: String,
        columns: Vec<String>,
        where_clauses: Vec<WhereCondition>,
    },
    /// A raw SQL string with its associated parameters.
    Raw {
        sql: String,
        params: Arc<Vec<Py<PyAny>>>,
    },
}

/// A builder object that accumulates parts of a SQL query.
/// It can operate in two modes: building a query from structured components
/// (tables, columns, conditions) or holding a raw SQL string with parameters.
#[pyclass]
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    query_type: QueryType,
}

impl QueryBuilder {
    /// Constructs the final SQL string and a vector of parameter values.
    ///
    /// This method inspects the `query_type` and either:
    /// 1.  Builds a structured query, automatically adding columns marked with `select_column`.
    /// 2.  Processes a raw query, converting its Python parameters into strings.
    pub fn build(&self, py: Python) -> PyResult<(String, Vec<String>)> {
        match &self.query_type {
            QueryType::Structured {
                table,
                columns,
                where_clauses,
            } => self.build_structured(py, table, columns, where_clauses),
            QueryType::Raw { sql, params } => self.build_raw(py, sql, params),
        }
    }

    /// Helper to build a query from structured components.
    fn build_structured(
        &self,
        py: Python,
        table: &str,
        columns: &[String],
        where_clauses: &[WhereCondition],
    ) -> PyResult<(String, Vec<String>)> {
        debug!(
            "Building structured query for table '{}' with {} explicit columns and {} where clauses.",
            table,
            columns.len(),
            where_clauses.len()
        );

        // Use a HashSet to automatically handle duplicate column names.
        let mut all_columns: HashSet<String> = columns.iter().cloned().collect();

        // Add columns from where-clauses that have the `select_column` flag.
        for cond in where_clauses {
            if cond.select_column {
                all_columns.insert(cond.column_name.clone());
            }
        }

        let cols = if all_columns.is_empty() {
            "*".to_string()
        } else {
            all_columns.into_iter().collect::<Vec<_>>().join(", ")
        };

        let mut sql = format!("SELECT {} FROM {}", cols, table);
        let mut params = Vec::new();

        if !where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            let conditions: Result<Vec<String>, PyErr> = where_clauses
                .iter()
                .map(|cond| {
                    if cond.value.is_none(py) {
                        return Ok(format!("{} {} NULL", cond.column_name, cond.operator));
                    }
                    match py_any_to_string(py, &cond.value, &cond.column_name)? {
                        SqlParam::Single(s) => {
                            params.push(s);
                            Ok(format!("{} {} ?", cond.column_name, cond.operator))
                        }
                        SqlParam::List(vec) => {
                            let placeholders: Vec<&str> = vec.iter().map(|_| "?").collect();
                            params.extend(vec);
                            Ok(format!(
                                "{} {} ({})",
                                cond.column_name,
                                cond.operator,
                                placeholders.join(", ")
                            ))
                        }
                    }
                })
                .collect();

            sql.push_str(&conditions?.join(" AND "));
        }
        Ok((sql, params))
    }

    /// Helper to process a raw SQL query and its parameters.
    fn build_raw(
        &self,
        py: Python,
        sql: &str,
        params: &[Py<PyAny>],
    ) -> PyResult<(String, Vec<String>)> {
        debug!("Building raw query with {} parameters.", params.len());
        let string_params: Vec<String> = params
            .iter()
            .map(|p| match py_any_to_string(py, p, "raw query parameter")? {
                SqlParam::Single(s) => Ok(s),
                SqlParam::List(vec) => Ok(vec.join(", ")),
            })
            .collect::<PyResult<Vec<String>>>()?;
        Ok((sql.to_string(), string_params))
    }
}

/// Entry point for creating a database query.
///
/// This function is highly flexible and can be called in two ways:
///
/// 1.  **With Model components (ORM-style):**
///     Pass `ColumnField` instances and `WhereCondition` objects to build a query.
///     Example: `select(User.id, User.name, User.age > 18)`
///
/// 2.  **With a raw SQL string:**
///     Pass a SQL string as the first argument, followed by any parameters.
///     Example: `select("SELECT * FROM users WHERE age > ?", 18)`
#[pyfunction]
#[pyo3(signature = (*args))]
pub fn select(_py: Python, args: &Bound<'_, PyTuple>) -> PyResult<QueryBuilder> {
    if args.is_empty() {
        return Err(FustOrmError::InvalidQueryArgument(
            "select() cannot be called with no arguments.".to_string(),
        )
        .into());
    }

    let first_arg = args.get_item(0)?;

    // Mode 1: Raw SQL Query
    if let Ok(sql_str) = first_arg.extract::<String>() {
        debug!("Creating a new raw SQL query.");
        let params: Vec<Py<PyAny>> = args.iter().skip(1).map(|item| item.into()).collect();
        return Ok(QueryBuilder {
            query_type: QueryType::Raw {
                sql: sql_str,
                params: Arc::new(params),
            },
        });
    }

    // Mode 2: Structured Query
    debug!(
        "Creating a new structured query from {} arguments.",
        args.len()
    );
    let mut table_name = None;
    let mut columns = Vec::new();
    let mut where_clauses = Vec::new();

    for arg in args.iter() {
        if let Ok(py_type) = arg.downcast::<PyType>()
            && py_type.is_subclass_of::<Model>()?
        {
            // If a Model class is passed, select all its columns (represented by '*')
            // and determine the table name from it.
            if table_name.is_none() {
                let name_attr = py_type.getattr("__table_name__")?;
                table_name = Some(name_attr.extract::<String>()?);
            } else {
                return Err(FustOrmError::InvalidQueryArgument(
                        "Cannot select from multiple tables by passing multiple Model classes. For joins, use raw SQL.".to_string(),
                    )
                    .into());
            }
            continue; // Continue to next arg
        }

        if let Ok(col_field) = arg.extract::<PyRef<ColumnField>>() {
            if table_name.is_none() {
                table_name = Some(col_field.table_name.clone());
            } else if table_name.as_ref() != Some(&col_field.table_name) {
                return Err(FustOrmError::InvalidQueryArgument(
                    "Cannot select columns from multiple tables in one query.".to_string(),
                )
                .into());
            }
            columns.push(col_field.column_name.clone());
        } else if let Ok(where_cond) = arg.extract::<PyRef<WhereCondition>>() {
            where_clauses.push(where_cond.clone());
        } else {
            return Err(FustOrmError::InvalidQueryArgument(format!(
                "Unsupported argument type in select(): {}",
                arg.get_type().name()?
            ))
            .into());
        }
    }

    // Determine the final table name.
    let final_table_name = if let Some(name) = table_name {
        name
    } else {
        // We still don't have a table name, try to infer it from where_clauses.
        // This is not yet implemented as it can be ambiguous.
        return Err(FustOrmError::InvalidQueryArgument(
            "Cannot determine table name. Please select at least one column or pass a Model class."
                .to_string(),
        )
        .into());
    };

    Ok(QueryBuilder {
        query_type: QueryType::Structured {
            table: final_table_name,
            columns,
            where_clauses,
        },
    })
}

pub enum SqlParam {
    Single(String),
    List(Vec<String>),
}

/// Converts a Python object (`Py<PyAny>`) into a String for SQL parameter binding.
fn py_any_to_string(py: Python, value: &Py<PyAny>, context: &str) -> PyResult<SqlParam> {
    let bound_val = value.bind(py);
    if let Ok(s) = bound_val.downcast::<PyString>() {
        Ok(SqlParam::Single(s.to_string()))
    } else if let Ok(i) = bound_val.downcast::<PyInt>() {
        Ok(SqlParam::Single(i.to_string()))
    } else if let Ok(f) = bound_val.downcast::<PyFloat>() {
        Ok(SqlParam::Single(f.to_string()))
    } else if let Ok(b) = bound_val.downcast::<PyBool>() {
        Ok(SqlParam::Single(
            (if b.is_true() { "1" } else { "0" }).to_string(),
        ))
    } else if let Ok(list) = bound_val.downcast::<PyList>() {
        let mut string_vec: Vec<String> = Vec::with_capacity(list.len());
        for item in list.iter() {
            if let Ok(s) = item.downcast::<PyString>() {
                string_vec.push(s.to_string());
            } else if let Ok(i) = item.downcast::<PyInt>() {
                string_vec.push(i.to_string());
            } else if let Ok(f) = item.downcast::<PyFloat>() {
                string_vec.push(f.to_string());
            } else if let Ok(b) = item.downcast::<PyBool>() {
                string_vec.push(b.to_string());
            } else {
                return Err(FustOrmError::BuildError(format!(
                    "Unsupported parameter type in list for '{}': {}",
                    context,
                    item.get_type().name()?
                ))
                .into());
            }
        }
        Ok(SqlParam::List(string_vec))
    } else {
        Err(FustOrmError::BuildError(format!(
            "Unsupported parameter type for '{}': {}",
            context,
            bound_val.get_type().name()?
        ))
        .into())
    }
}
