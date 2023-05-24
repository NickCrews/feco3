use super::base::{FileFormWriter, FileWriter, FormWriter};
use crate::form::{FormLine, FormSchema};
use std::{fs::File, path::Path};

pub struct CSVFormWriter<W: std::io::Write> {
    csv_writer: csv::Writer<W>,
    schema: FormSchema,
    has_written_header: bool,
}

impl<W: std::io::Write> CSVFormWriter<W> {
    pub fn new(raw_writer: W, schema: &FormSchema) -> Self {
        let writer = csv::Writer::from_writer(raw_writer);
        Self {
            csv_writer: writer,
            schema: schema.clone(),
            has_written_header: false,
        }
    }

    fn write_header(&mut self) -> std::io::Result<()> {
        let fields = &self.schema.fields;
        let field_names = fields.iter().map(|f| f.name.as_str());
        self.csv_writer.write_record(field_names)?;
        Ok(())
    }
}

impl<W: std::io::Write> FormWriter for CSVFormWriter<W> {
    fn write_line(&mut self, line: &FormLine) -> std::io::Result<()> {
        if !self.has_written_header {
            self.write_header()?;
            self.has_written_header = true;
        }
        let string_values = line
            .fields
            .iter()
            .map(|f| f.value.clone())
            .map(|v| match v {
                crate::form::Value::String(s) => s,
                crate::form::Value::Integer(i) => i.to_string(),
                crate::form::Value::Float(f) => f.to_string(),
                crate::form::Value::Date(d) => d.to_string(),
                crate::form::Value::Boolean(b) => b.to_string(),
            });
        self.csv_writer.write_record(string_values)?;
        Ok(())
    }
}

pub struct CSVFileFormWriter(CSVFormWriter<File>);

impl FormWriter for CSVFileFormWriter {
    fn write_line(&mut self, line: &FormLine) -> std::io::Result<()> {
        self.0.write_line(line)
    }
}

impl FileFormWriter for CSVFileFormWriter {
    fn file_name(form_name: String) -> String {
        format!("{}.csv", form_name)
    }

    fn new(path: &Path, schema: &FormSchema) -> std::io::Result<Box<Self>> {
        let file = File::create(path)?;
        let writer = CSVFormWriter::new(file, schema);
        Ok(Box::new(Self(writer)))
    }
}

pub type CSVFileWriter = FileWriter<CSVFileFormWriter>;
