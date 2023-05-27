use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass]
struct FecFile(feco3::FecFile);

#[pymethods]
impl FecFile {
    #[staticmethod]
    fn from_path(path: PathBuf) -> PyResult<Self> {
        match feco3::FecFile::from_path(&path) {
            Ok(fec_file) => Ok(FecFile(fec_file)),
            Err(e) => Err(to_py_err(e)),
        }
    }
}

#[pyfunction]
fn parse_from_path(fec_path: PathBuf, out_dir: PathBuf) -> PyResult<()> {
    match feco3::parse_from_path(&fec_path, out_dir) {
        Ok(()) => Ok(()),
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string())),
    }
}

#[pymodule]
fn _feco3(_py: Python, m: &PyModule) -> PyResult<()> {
    // It is important to initialize the Python loggers first,
    // before calling any Rust functions that may log.
    // See https://pyo3.rs/v0.18.3/ecosystem/logging
    pyo3_log::init();
    m.add_function(wrap_pyfunction!(parse_from_path, m)?)?;
    m.add_class::<FecFile>()?;
    Ok(())
}

fn to_py_err(e: feco3::Error) -> PyErr {
    match e {
        feco3::Error::HeaderParseError(e) => {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string())
        }
        feco3::Error::RecordParseError(e) => {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string())
        }
        feco3::Error::IoError(e) => PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()),
    }
}
