use crate::column_field::ColumnField;
use heck::ToSnakeCase;
use log::debug;
use pyo3::types::{PyDict, PyType};
use pyo3::{PyTypeInfo, prelude::*};

/// A base class for user-defined models.
///
/// When a new class inherits from `Model`, its `__init_subclass__` method
/// is automatically called. This method inspects the subclass's annotations
/// and sets up `ColumnField` descriptors for each annotated database column.
#[pyclass(subclass)]
pub struct Model;

#[pymethods]
impl Model {
    /// This special class method is called when a class inherits from `Model`.
    ///
    /// It performs the following setup steps:
    /// 1. Determines the database table name, either from a `__table_name__`
    ///    attribute or by converting the class name to snake_case.
    /// 2. Iterates through the class's `__annotations__`.
    /// 3. For each annotation that is a `ColumnField` generic (e.g., `ColumnField[int]`),
    ///    it creates an instance of the `ColumnField` descriptor.
    /// 4. This descriptor instance is then assigned as a class attribute, replacing
    ///    the original annotation. This allows for deferred query operations like `MyModel.id == 5`.
    #[classmethod]
    fn __init_subclass__(cls: &Bound<PyType>) -> PyResult<()> {
        let py = cls.py();
        debug!("Initializing model subclass: {}", cls.name()?);

        let table_name = if let Ok(name) = cls.getattr(pyo3::intern!(py, "__table_name__")) {
            let table_name_str = name.extract::<String>()?;
            debug!("Found explicit `__table_name__`: '{}'", &table_name_str);
            table_name_str
        } else {
            let generated_name = cls.name()?.to_string().to_snake_case();
            debug!(
                "Generating table name '{}' from class name '{}'",
                &generated_name,
                cls.name()?
            );
            cls.setattr(pyo3::intern!(py, "__table_name__"), &generated_name)?;
            generated_name
        };

        let annotations = match cls.getattr(pyo3::intern!(py, "__annotations__")) {
            Ok(ann) => ann.downcast::<PyDict>()?.to_owned(),
            Err(_) => {
                debug!("No `__annotations__` found for model '{}'", cls.name()?);
                PyDict::new(py)
            }
        };

        for (key, value) in annotations.iter() {
            let column_name = key.extract::<String>()?;
            let type_repr = value.to_string();
            if type_repr.starts_with(ColumnField::NAME) {
                debug!(
                    "Creating ColumnField for attribute '{}' on model '{}'",
                    &column_name,
                    cls.name()?
                );
                let column_field = Py::new(
                    py,
                    ColumnField {
                        table_name: table_name.clone(),
                        column_name: column_name.clone(),
                    },
                )?;
                cls.setattr(column_name, column_field)?;
            }
        }

        Ok(())
    }
}
