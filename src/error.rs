use pyo3::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FustOrmError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Query execution error: {0}")]
    QueryError(String),

    #[error("Invalid query argument: {0}")]
    InvalidQueryArgument(String),

    #[error("Failed to build query: {0}")]
    BuildError(String),
}

impl From<FustOrmError> for PyErr {
    fn from(err: FustOrmError) -> PyErr {
        pyo3::exceptions::PyValueError::new_err(err.to_string())
    }
}
