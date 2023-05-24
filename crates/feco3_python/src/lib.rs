use pyo3::prelude::*;
use std::path::PathBuf;

#[pyfunction]
fn parse_from_path(fec_path: PathBuf, out_dir: PathBuf) -> PyResult<()> {
    Ok(feco3::parse_from_path(&fec_path, out_dir))
}

#[pymodule]
fn _feco3(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_from_path, m)?)?;
    Ok(())
}
