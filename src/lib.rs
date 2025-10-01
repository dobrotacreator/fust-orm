use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3_async_runtimes::tokio::future_into_py;
use sqlx::{Column, Row, TypeInfo};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Query error: {0}")]
    QueryError(String),
    #[error("Pool error: {0}")]
    PoolError(String),
}

impl From<DatabaseError> for PyErr {
    fn from(err: DatabaseError) -> PyErr {
        pyo3::exceptions::PyValueError::new_err(err.to_string())
    }
}

#[pyclass]
#[derive(Clone)]
struct Database {
    pool: sqlx::sqlite::SqlitePool,
}

#[pymethods]
impl Database {
    #[staticmethod]
    #[pyo3(signature = (db_path))]
    fn connect(py: Python, db_path: String) -> PyResult<Bound<PyAny>> {
        future_into_py(py, async move {
            eprintln!(
                "Attempting to connect to the database at path: {}",
                &db_path
            );

            let pool = sqlx::SqlitePool::connect(&db_path)
                .await
                .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;

            eprintln!("Successfully connected to the database: {}", &db_path);
            Ok(Database { pool })
        })
    }

    fn execute<'py>(&self, py: Python<'py>, query: QueryBuilder) -> PyResult<Bound<'py, PyAny>> {
        let pool = self.pool.clone();

        future_into_py(py, async move {
            let (sql, params) = query.build();
            eprintln!(
                "Executing SQL query: \"{}\" with parameters: {:?}",
                &sql, &params
            );

            let mut sqlx_query = sqlx::query(&sql);
            for param in params {
                sqlx_query = sqlx_query.bind(param);
            }
            let rows = sqlx_query
                .fetch_all(&pool)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

            eprintln!("Query executed successfully, fetched {} rows", rows.len());

            Python::attach(|py| -> PyResult<Py<PyList>> {
                let results = PyList::empty(py);
                let map_db_err = |e: sqlx::Error| DatabaseError::QueryError(e.to_string());
                for row in rows {
                    let dict = PyDict::new(py);
                    for (i, col) in row.columns().iter().enumerate() {
                        let col_name = col.name();
                        let value = match col.type_info().name() {
                            "TEXT" => row
                                .try_get::<Option<String>, _>(i)
                                .map_err(map_db_err)?
                                .into_pyobject(py)?,
                            "INTEGER" => row
                                .try_get::<Option<i64>, _>(i)
                                .map_err(map_db_err)?
                                .into_pyobject(py)?,
                            "REAL" => row
                                .try_get::<Option<f64>, _>(i)
                                .map_err(map_db_err)?
                                .into_pyobject(py)?,
                            "BLOB" => row
                                .try_get::<Option<Vec<u8>>, _>(i)
                                .map_err(map_db_err)?
                                .into_pyobject(py)?,
                            // For NULL and other types, return None
                            _ => py.None().into_pyobject(py)?,
                        };
                        dict.set_item(col_name, value)?;
                    }
                    results.append(dict)?;
                }
                Ok(results.into())
            })
        })
    }
}

#[pyclass]
#[derive(Clone)]
struct QueryBuilder {
    table: String,
    columns: Vec<String>,
    where_clauses: Vec<(String, String)>,
}

#[pymethods]
impl QueryBuilder {
    #[staticmethod]
    #[pyo3(signature = (table, columns))]
    fn select(table: String, columns: Vec<String>) -> Self {
        eprintln!(
            "Creating SELECT query for table '{}' with columns: {:?}",
            &table, &columns
        );
        Self {
            table,
            columns,
            where_clauses: Vec::new(),
        }
    }

    pub fn where_(&mut self, column: String, value: String) -> Self {
        eprintln!("Adding WHERE condition: {} = {}", &column, &value);
        self.where_clauses.push((column, value));
        self.clone()
    }
}

impl QueryBuilder {
    fn build(&self) -> (String, Vec<String>) {
        let cols = self.columns.join(", ");
        let mut sql = format!("SELECT {} FROM {}", cols, self.table);
        let mut params = Vec::new();

        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            let conditions: Vec<String> = self
                .where_clauses
                .iter()
                .map(|(col, val)| {
                    params.push(val.clone());
                    format!("{} = ?", col)
                })
                .collect();
            sql.push_str(&conditions.join(" AND "));
        }

        (sql, params)
    }
}

#[pymodule]
fn fust_orm(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Database>()?;
    m.add_class::<QueryBuilder>()?;
    Ok(())
}
