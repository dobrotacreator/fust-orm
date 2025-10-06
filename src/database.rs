use crate::error::FustOrmError;
use crate::query::QueryBuilder;
use log::{debug, info};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3_async_runtimes::tokio::future_into_py;
use sqlx::{AnyPool, Column, Row, TypeInfo};

/// The main class for interacting with a database.
///
/// This class provides an asynchronous interface for connecting to and executing
/// queries against various SQL databases supported by the underlying driver.
#[pyclass]
#[derive(Clone)]
pub struct Database {
    /// A connection pool that can be used with any of the supported database drivers.
    pool: AnyPool,
}

#[pymethods]
impl Database {
    /// Creates an asynchronous connection to the database using a URL.
    ///
    /// The appropriate database driver is selected based on the URL scheme.
    /// For example:
    /// - "sqlite://my_database.db"
    /// - "postgres://user:password@host/database"
    /// - "mysql://user:password@host/database"
    ///
    /// Args:
    ///     db_url (String): The connection string for the database.
    #[staticmethod]
    #[pyo3(signature = (db_url))]
    fn connect(py: Python, db_url: String) -> PyResult<Bound<PyAny>> {
        info!("Attempting to connect to the database at URL: {}", &db_url);
        future_into_py(py, async move {
            let pool = AnyPool::connect(&db_url)
                .await
                .map_err(|e| FustOrmError::ConnectionError(e.to_string()))?;
            info!("Successfully connected to the database: {}", &db_url);
            Ok(Database { pool })
        })
    }

    /// Executes a query against the database.
    ///
    /// The query can be either a raw SQL string or a QueryBuilder instance.
    /// The method returns a list of dictionaries, where each dictionary represents a row.
    fn execute<'py>(
        &self,
        py: Python<'py>,
        query: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let pool = self.pool.clone();

        enum QueryInput {
            Builder(QueryBuilder),
            Raw(String),
        }

        let input = if let Ok(qb) = query.extract::<QueryBuilder>() {
            debug!("Execute called with QueryBuilder");
            QueryInput::Builder(qb)
        } else if let Ok(raw_sql) = query.extract::<String>() {
            debug!("Execute called with raw SQL string");
            QueryInput::Raw(raw_sql)
        } else {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "'query' must be a string or a result of calling select()",
            ));
        };

        future_into_py(py, async move {
            let (sql, params) = Python::attach(|py| -> PyResult<(String, Vec<String>)> {
                match input {
                    QueryInput::Builder(qb) => qb.build(py),
                    QueryInput::Raw(s) => Ok((s, Vec::new())),
                }
            })?;
            info!("Executing SQL: \"{}\"", &sql);
            debug!("With parameters: {:?}", &params);

            let mut sqlx_query = sqlx::query(&sql);
            for param in params {
                // NOTE: Binding all parameters as strings might not be suitable for all databases
                // or column types. For many simple cases, the database driver will handle
                // the type coercion.
                sqlx_query = sqlx_query.bind(param);
            }

            let rows = sqlx_query
                .fetch_all(&pool)
                .await
                .map_err(|e| FustOrmError::QueryError(e.to_string()))?;

            info!("Query executed successfully, fetched {} rows.", rows.len());

            Python::attach(|py| -> PyResult<Py<PyList>> {
                let results = PyList::empty(py);
                let map_db_err = |e: sqlx::Error| FustOrmError::QueryError(e.to_string());

                for row in rows {
                    let dict = PyDict::new(py);
                    for (i, col) in row.columns().iter().enumerate() {
                        let col_name = col.name();
                        // Attempt to decode the value into common types in a specific order.
                        // This generic approach handles various database backends.
                        let value = match col.type_info().name() {
                            "TEXT" | "VARCHAR" => row
                                .try_get::<Option<String>, _>(i)
                                .map_err(map_db_err)?
                                .into_pyobject(py)?,
                            "INTEGER" | "INT" => row
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
                            // Fallback for types that were not successfully decoded above.
                            _ => {
                                if let Ok(None) = row.try_get::<Option<String>, _>(i) {
                                    py.None().into_pyobject(py)?
                                } else if let Ok(val) = row.try_get::<String, _>(i) {
                                    val.into_pyobject(py)?.into_any()
                                } else if let Ok(val) = row.try_get::<i64, _>(i) {
                                    val.into_pyobject(py)?.into_any()
                                } else if let Ok(val) = row.try_get::<f64, _>(i) {
                                    val.into_pyobject(py)?.into_any()
                                } else {
                                    log::warn!(
                                        "Couldn't determine column type of {col_name}, fallback to None"
                                    );
                                    py.None().into_pyobject(py)?
                                }
                            }
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
