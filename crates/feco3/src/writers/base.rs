//! API for writing individual form lines contained in a FEC file.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::line::{Line, LineSchema};
use std::collections::hash_map::Entry::{Occupied, Vacant};

/// Writes single itemization lines.
pub trait LineWriter {
    fn write_line(&mut self, line: &Line) -> std::io::Result<()>;
}

/// A specialization of [LineWriter] that writes to a file.
pub trait FileLineWriter: LineWriter {
    fn file_name(form_name: String) -> String;
    fn new(path: &Path, schema: &LineSchema) -> std::io::Result<Box<Self>>;

    fn new_in_dir(base_path: &Path, schema: &LineSchema) -> std::io::Result<Box<Self>> {
        let form_name = Self::norm_form_name(&schema.code);
        let file_name = Self::file_name(form_name);
        let path = base_path.join(file_name);
        log::debug!("Creating base dir at: {:?}", base_path);
        fs::create_dir_all(base_path)?;
        log::debug!("Creating file: {:?}", path);
        let result = Self::new(&path, schema)?;
        Ok(result)
    }

    /// Some forms have a slash in their name, which is not allowed in file names.
    fn norm_form_name(name: &str) -> String {
        name.replace("/", "-")
    }
}

/// A [LineWriter] that writes to a directory, each form to its own file.
pub struct MultiFileWriter<T: FileLineWriter> {
    base_path: PathBuf,
    writers: HashMap<LineSchema, T>,
}

impl<T: FileLineWriter> MultiFileWriter<T> {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path: base_path,
            writers: HashMap::new(),
        }
    }

    // https://users.rust-lang.org/t/issue-with-hashmap-and-fallible-update/44960/8
    /// Get the existing form writer for a schema, or create a new one if it doesn't exist.
    fn get_form_writer(&mut self, schema: &LineSchema) -> std::io::Result<&mut T> {
        Ok(match self.writers.entry(schema.clone()) {
            Occupied(e) => e.into_mut(),
            Vacant(e) => e.insert(*T::new_in_dir(&self.base_path, schema)?),
        })
    }
}

impl<T: FileLineWriter> LineWriter for MultiFileWriter<T> {
    fn write_line(&mut self, line: &Line) -> std::io::Result<()> {
        let writer = self.get_form_writer(&line.schema)?;
        writer.write_line(line)
    }
}
