//! API for writing individual records contained in a FEC file.

use std::{collections::HashMap, fs, path::PathBuf};

use crate::record::{Record, RecordSchema};
use crate::Error;
use std::collections::hash_map::Entry::{Occupied, Vacant};

/// Writes single itemization records.
pub trait RecordWriter {
    fn write_record(&mut self, record: &Record) -> std::io::Result<()>;
    fn finish(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

pub trait RecordWriterFactory {
    fn make(&mut self, schema: &RecordSchema) -> std::io::Result<Box<dyn RecordWriter>>;
}

/// Creates [RecordWriter]s that write to a filea.
pub trait FileRecordWriterFactory {
    fn file_name(&self, form_name: String) -> String;
    fn make(
        &mut self,
        path: &PathBuf,
        schema: &RecordSchema,
    ) -> std::io::Result<Box<dyn RecordWriter>>;

    /// Some forms have a slash in their name, which is not allowed in file names.
    fn norm_form_name(&self, name: &str) -> String {
        name.replace("/", "-")
    }
}

/// A [RecordWriter] that delegates to multiple [RecordWriter]s.
pub struct MultiRecordWriter {
    factory: Box<dyn RecordWriterFactory>,
    writers: HashMap<RecordSchema, Box<dyn RecordWriter>>,
}

impl MultiRecordWriter {
    pub fn new(factory: Box<dyn RecordWriterFactory>) -> Self {
        Self {
            factory,
            writers: HashMap::new(),
        }
    }

    // https://users.rust-lang.org/t/issue-with-hashmap-and-fallible-update/44960/8
    /// Get the existing writer for a schema, or create a new one if it doesn't exist.
    fn get_writer(&mut self, schema: &RecordSchema) -> std::io::Result<&mut Box<dyn RecordWriter>> {
        Ok(match self.writers.entry(schema.clone()) {
            Occupied(e) => e.into_mut(),
            Vacant(e) => e.insert(self.factory.make(schema)?),
        })
    }
}

impl RecordWriter for MultiRecordWriter {
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

pub struct MultiFileRecordWriterFactory {
    base_path: PathBuf,
    factory: Box<dyn FileRecordWriterFactory>,
}

impl MultiFileRecordWriterFactory {
    pub fn new(base_path: PathBuf, factory: Box<dyn FileRecordWriterFactory>) -> Self {
        Self { base_path, factory }
    }
}

impl RecordWriterFactory for MultiFileRecordWriterFactory {
    fn make(&mut self, schema: &RecordSchema) -> std::io::Result<Box<dyn RecordWriter>> {
        let form_name = self.factory.norm_form_name(&schema.code);
        let file_name = self.factory.file_name(form_name);
        let path = self.base_path.join(file_name);
        fs::create_dir_all(&self.base_path)?;
        log::debug!("Creating new FileRecordWriter at: {:?}", path);
        let result = self.factory.make(&path, schema)?;
        Ok(result)
    }
}
