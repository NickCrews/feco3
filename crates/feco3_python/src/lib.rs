use feco3::add_42 as _add_42;
use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pyfunction]
fn add_42(a: i32) -> PyResult<i32> {
    Ok(_add_42(a))
}

/// A Python module implemented in Rust.
#[pymodule]
fn _feco3(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(add_42, m)?)?;
    Ok(())
}
