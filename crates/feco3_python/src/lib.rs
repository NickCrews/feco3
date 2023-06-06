use arrow::pyarrow::PyArrowType;
use arrow::record_batch::RecordBatch;
use pyo3::{
    exceptions::{PyIOError, PyValueError},
    prelude::*,
};
use std::path::PathBuf;

#[pyclass]
struct Header(feco3::Header);

#[pymethods]
impl Header {
    #[getter]
    fn fec_version(&self) -> PyResult<String> {
        Ok(self.0.fec_version.clone())
    }

    #[getter]
    fn software_name(&self) -> PyResult<String> {
        Ok(self.0.software_name.clone())
    }

    #[getter]
    fn software_version(&self) -> PyResult<Option<String>> {
        Ok(self.0.software_version.clone())
    }

    #[getter]
    fn report_id(&self) -> PyResult<Option<String>> {
        Ok(self.0.report_id.clone())
    }

    #[getter]
    fn report_number(&self) -> PyResult<Option<String>> {
        Ok(self.0.report_number.clone())
    }
}

#[pyclass]
struct Cover(feco3::Cover);

#[pymethods]
impl Cover {
    #[getter]
    fn form_type(&self) -> PyResult<String> {
        Ok(self.0.form_type.clone())
    }

    #[getter]
    fn filer_committee_id(&self) -> PyResult<String> {
        Ok(self.0.filer_committee_id.clone())
    }
}

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

    #[getter]
    fn header(&mut self) -> PyResult<Header> {
        match self.0.get_header() {
            Ok(header) => Ok(Header(header.clone())),
            Err(e) => Err(to_py_err(e)),
        }
    }

    #[getter]
    fn cover(&mut self) -> PyResult<Cover> {
        match self.0.get_cover() {
            Ok(cover) => Ok(Cover(cover.clone())),
            Err(e) => Err(to_py_err(e)),
        }
    }
}

#[pyclass]
struct ParquetProcessor(feco3::writers::parquet::ParquetProcessor);

#[pymethods]
impl ParquetProcessor {
    #[new]
    fn new(out_dir: PathBuf) -> Self {
        let writer_props = None;
        let processor = feco3::writers::parquet::ParquetProcessor::new(out_dir, writer_props);
        Self(processor)
    }

    fn process(&mut self, fec_file: &mut FecFile) -> PyResult<()> {
        match self.0.process(&mut fec_file.0) {
            Ok(()) => Ok(()),
            Err(e) => Err(to_py_err(e)),
        }
    }
}

#[pyclass]
struct PyarrowProcessor(feco3::writers::arrow::RecordBatchProcessor);

#[pymethods]
impl PyarrowProcessor {
    #[new]
    fn new(batch_size: usize) -> Self {
        let processor = feco3::writers::arrow::RecordBatchProcessor::new(batch_size);
        Self(processor)
    }

    // Gets the next pyarrow record batch or None if there are no more records.
    fn next_batch(&mut self, fec_file: &mut FecFile) -> PyResult<Option<PyArrowType<RecordBatch>>> {
        match self.0.next_batch(&mut fec_file.0) {
            Ok(Some(batch)) => Ok(Some(batch.into())),
            Ok(None) => Ok(None),
            Err(e) => Err(to_py_err(e)),
        }
    }
}

#[pymodule]
fn _feco3(_py: Python, m: &PyModule) -> PyResult<()> {
    // It is important to initialize the Python loggers first,
    // before calling any Rust functions that may log.
    // See https://pyo3.rs/v0.18.3/ecosystem/logging
    pyo3_log::init();
    m.add_class::<FecFile>()?;
    m.add_class::<ParquetProcessor>()?;
    m.add_class::<PyarrowProcessor>()?;
    Ok(())
}

fn to_py_err(e: feco3::Error) -> PyErr {
    match e {
        feco3::Error::HeaderParseError(e) => PyErr::new::<PyValueError, _>(e.to_string()),
        feco3::Error::RecordParseError(e) => PyErr::new::<PyValueError, _>(e.to_string()),
        feco3::Error::IoError(e) => PyErr::new::<PyIOError, _>(e.to_string()),
        feco3::Error::SchemaError(e, f) => PyErr::new::<PyValueError, _>(format!(
            "Failed to find schema for fec version {} and line code {}",
            e, f
        )),
        feco3::Error::CoverParseError(e) => PyErr::new::<PyValueError, _>(e.to_string()),
    }
}
