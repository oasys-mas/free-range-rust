use pyo3::prelude::*;
#[pyfunction]
fn hello_py() -> &'static str {
    "Hello from bindings!"
}
#[pymodule]
fn bindings(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hello_py, m)?)?;
    Ok(())
}
