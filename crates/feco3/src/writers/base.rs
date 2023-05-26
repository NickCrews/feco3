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

    /// Some forms have a slash in their name, which is not allowed in file names.
    fn norm_form_name(name: &str) -> String {
        name.replace("/", "-")
    }
}

/// A [LineWriter] that writes to a directory, each form to its own file.
pub struct MultiFileLineWriter<T: FileLineWriter> {
    base_path: PathBuf,
    writers: HashMap<LineSchema, T>,
}

impl<T: FileLineWriter> MultiFileLineWriter<T> {
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
            Vacant(e) => e.insert(*Self::new_writer(&self.base_path, schema)?),
        })
    }

    fn new_writer(base_path: &PathBuf, schema: &LineSchema) -> std::io::Result<Box<T>> {
        let form_name = T::norm_form_name(&schema.code);
        let file_name = T::file_name(form_name);
        let path = base_path.join(file_name);
        fs::create_dir_all(&base_path)?;
        log::debug!("Creating new FileLineWriter at: {:?}", path);
        let result = T::new(&path, schema)?;
        Ok(result)
    }
}

impl<T: FileLineWriter> LineWriter for MultiFileLineWriter<T> {
    fn write_line(&mut self, line: &Line) -> std::io::Result<()> {
        let writer = self.get_form_writer(&line.schema)?;
        writer.write_line(line)
    }
}
