use feco3::print_header as _print_header;
use pyo3::prelude::*;

#[pyfunction]
fn print_header(path: &str) -> PyResult<()> {
    Ok(_print_header(path))
}

#[pymodule]
fn _feco3(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(print_header, m)?)?;
    Ok(())
}
