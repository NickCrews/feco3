//! API for writing individual records contained in a FEC file.

use std::{collections::HashMap, fs, path::PathBuf};

use crate::record::{Record, RecordSchema};
use crate::Error;
use std::collections::hash_map::Entry::{Occupied, Vacant};

/// Writes single itemization records.
pub trait RecordWriter: Send {
    fn write_record(&mut self, record: &Record) -> std::io::Result<()>;
    fn finish(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

/// Creates [RecordWriter]s given a schema.
pub trait RecordWriterFactory: Send {
    type Writer: RecordWriter;
    /// Create a new [RecordWriter] for a given schema.
    fn make_writer(&mut self, schema: &RecordSchema) -> std::io::Result<Self::Writer>;
}

/// Creates [RecordWriter]s that write to a file.
pub trait FileRecordWriterFactory: Send {
    type Writer: RecordWriter;
    fn file_name(&self, form_name: String) -> String;
    /// Make a new [RecordWriter] for a given schema that writes to the given path.
    fn make(&mut self, path: &PathBuf, schema: &RecordSchema) -> std::io::Result<Self::Writer>;

    /// Some forms have a slash in their name, which is not allowed in file names.
    fn norm_form_name(&self, name: &str) -> String {
        name.replace("/", "-")
    }
}

/// A [RecordWriter] that delegates to multiple [RecordWriter]s.
pub struct MultiRecordWriter<F: RecordWriterFactory> {
    factory: F,
    pub writers: HashMap<RecordSchema, F::Writer>,
}

impl<F: RecordWriterFactory> MultiRecordWriter<F> {
    pub fn new(factory: F) -> Self {
        Self {
            factory,
            writers: HashMap::new(),
        }
    }

    // https://users.rust-lang.org/t/issue-with-hashmap-and-fallible-update/44960/8
    /// Get the existing writer for a schema, or create a new one if it doesn't exist.
    pub fn get_writer(&mut self, schema: &RecordSchema) -> std::io::Result<&mut F::Writer> {
        Ok(match self.writers.entry(schema.clone()) {
            Occupied(e) => e.into_mut(),
            Vacant(e) => e.insert(self.factory.make_writer(schema)?),
        })
    }
}

impl<F: RecordWriterFactory> RecordWriter for MultiRecordWriter<F> {
    fn write_record(&mut self, record: &Record) -> std::io::Result<()> {
        let writer = self.get_writer(&record.schema)?;
        writer.write_record(record)
    }
    fn finish(&mut self) -> Result<(), Error> {
        for (_, writer) in self.writers.iter_mut() {
            writer.finish()?;
        }
        Ok(())
    }
}

/// A [RecordWriterFactory] that uses a new [FileRecordWriterFactory] for each new form.
pub struct MultiFileRecordWriterFactory<F: FileRecordWriterFactory> {
    base_path: PathBuf,
    factory: F,
}

impl<F: FileRecordWriterFactory> MultiFileRecordWriterFactory<F> {
    pub fn new(base_path: PathBuf, factory: F) -> Self {
        Self { base_path, factory }
    }
}

impl<F: FileRecordWriterFactory> RecordWriterFactory for MultiFileRecordWriterFactory<F> {
    type Writer = F::Writer;
    fn make_writer(&mut self, schema: &RecordSchema) -> std::io::Result<F::Writer> {
        let form_name = self.factory.norm_form_name(&schema.code);
        let file_name = self.factory.file_name(form_name);
        let path = self.base_path.join(file_name);
        fs::create_dir_all(&self.base_path)?;
        log::debug!("Creating new FileRecordWriter at: {:?}", path);
        let result = self.factory.make(&path, schema)?;
        Ok(result)
    }
}
