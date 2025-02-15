use pyo3::prelude::*;
use pyo3::exceptions::PyTypeError;

#[pyfunction]
fn to_json_string(json: String) -> PyResult<String> {
    let jsonish_value: json_partial::jsonish::Value =
        json_partial::jsonish::parse(&json, json_partial::jsonish::ParseOptions::default())
            .map_err(|e| {
                make_py_err(
                    &json,
                    format!("Failed to parse input into jsonish::Value: {}", e),
                )
            })?;

    json_partial::jsonish::to_json_string(&jsonish_value).map_err(|e| make_py_err(&json, e))
}

#[pyfunction]
fn to_json_string_pretty(json: String) -> PyResult<String> {
    let jsonish_value: json_partial::jsonish::Value =
        json_partial::jsonish::parse(&json, json_partial::jsonish::ParseOptions::default())
            .map_err(|e| {
                make_py_err(
                    &json,
                    format!("Failed to parse input into jsonish::Value: {}", e),
                )
            })?;

    json_partial::jsonish::to_json_string_pretty(&jsonish_value).map_err(|e| make_py_err(&json, e))
}

/// Unified helper function to convert errors into a PyTypeError.
/// The function takes the original JSON string and an error message.
fn make_py_err<E: std::fmt::Display>(json: &str, err: E) -> PyErr {
    PyErr::new::<PyTypeError, _>(format!("original_string: {} , error: {}", json, err))
}

/// Json Partial Module
#[pymodule]
pub fn jsonish(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(to_json_string, m)?)?;
    m.add_function(wrap_pyfunction!(to_json_string_pretty, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
