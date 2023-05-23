//! API for writing individual form lines contained in a FEC file.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::form::{FormLine, FormSchema};
use std::collections::hash_map::Entry::{Occupied, Vacant};

pub trait FormWriter {
    fn write_header(&mut self) -> std::io::Result<()>;
    fn write_line(&mut self, line: &FormLine) -> std::io::Result<()>;
}

pub trait FileFormWriter: FormWriter {
    type T;
    fn file_name(form_name: String) -> String;
    fn new(path: &Path, schema: &FormSchema) -> std::io::Result<Self::T>;

    fn new_in_dir(base_path: &Path, schema: &FormSchema) -> std::io::Result<Self::T> {
        let form_name = norm_form_name(&schema.name);
        let file_name = Self::file_name(form_name);
        let path = base_path.join(file_name);
        let result = Self::new(&path, schema)?;
        fs::create_dir_all(base_path)?;
        Ok(result)
    }
}

/// Some forms have a slash in their name, which is not allowed in file names.
pub fn norm_form_name(name: &str) -> String {
    name.replace("/", "-")
}

pub trait Writer {
    fn write_form_line(
        &mut self,
        form_schema: &'static FormSchema,
        line: &FormLine,
    ) -> std::io::Result<()>;
}

pub struct FileWriter<T: FileFormWriter> {
    base_path: PathBuf,
    writers: HashMap<FormSchema, T>,
}

impl<T: FileFormWriter<T = T>> FileWriter<T> {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path: base_path,
            writers: HashMap::new(),
        }
    }

    // https://users.rust-lang.org/t/issue-with-hashmap-and-fallible-update/44960/8
    /// Get the existing form writer for a schema, or create a new one if it doesn't exist.
    fn get_form_writer(&mut self, schema: &FormSchema) -> std::io::Result<&mut T> {
        Ok(match self.writers.entry(schema.clone()) {
            Occupied(e) => e.into_mut(),
            // Vacant(e) => e.insert(faillible_operation(key)?),
            Vacant(e) => e.insert(T::new_in_dir(&self.base_path, schema)?),
        })
        // match self.writers.get(schema) {
        //     Occupied(e) => Ok(e.get()),
        //     Vacant(e) => {
        //         let expensive_data: ExpensiveData = faillible_operation(key)?;
        //         Ok(e.insert(expensive_data))
        //     }
        // }

        // let schema = schema.clone();
        // let writer = self.writers.entry(schema).or_insert_with(make_writer);

        // Ok(writer)
    }
}

impl<T: FileFormWriter<T = T>> Writer for FileWriter<T> {
    fn write_form_line(
        &mut self,
        schema: &'static FormSchema,
        line: &FormLine,
    ) -> std::io::Result<()> {
        let writer = self.get_form_writer(schema)?;
        writer.write_line(line)
    }
}
