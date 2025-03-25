use pyo3::prelude::*;
use ::free_range_rust::spaces::Discrete;
use ::free_range_rust::Space;

#[pyclass(name = "Discrete")]
pub struct PyDiscrete {
    pub inner: Discrete,
}

#[pymethods]
impl PyDiscrete {
    #[new]
    pub fn new(n: i32, start: i32) -> Self {
        PyDiscrete {
            inner: Discrete { n, start },
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[getter]
    pub fn n(&self) -> i32 {
        self.inner.n
    }

    #[getter]
    pub fn start(&self) -> i32 {
        self.inner.start
    }
}

#[pymodule]
fn free_range_rust(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDiscrete>()?;
    Ok(())
}
