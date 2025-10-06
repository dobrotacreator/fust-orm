mod column_field;
mod database;
mod error;
mod model;
mod query;
mod where_condition;

use pyo3::prelude::*;

use column_field::ColumnField;
use database::Database;
use model::Model;
use query::{QueryBuilder, select};

use crate::where_condition::WhereCondition;

#[pymodule]
fn fust_orm(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();
    sqlx::any::install_default_drivers();

    m.add_class::<WhereCondition>()?;
    m.add_class::<ColumnField>()?;
    m.add_class::<Database>()?;
    m.add_class::<QueryBuilder>()?;
    m.add_class::<Model>()?;

    m.add_function(wrap_pyfunction!(select, m)?)?;

    Ok(())
}
