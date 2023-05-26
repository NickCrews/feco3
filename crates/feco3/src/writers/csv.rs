use super::base::{FileLineWriter, LineWriter, MultiFileWriter};
use crate::line::{Line, LineSchema};
use std::{fs::File, path::Path};

/// A [LineWriter] that writes to CSV format.
pub struct CSVLineWriter<W: std::io::Write> {
    csv_writer: csv::Writer<W>,
    schema: LineSchema,
    has_written_header: bool,
}

impl<W: std::io::Write> CSVLineWriter<W> {
    pub fn new(raw_writer: W, schema: &LineSchema) -> Self {
        let writer = csv::WriterBuilder::new()
            .has_headers(false) // We'll write the header ourselves
            .flexible(true)
            .from_writer(raw_writer);
        Self {
            csv_writer: writer,
            schema: schema.clone(),
            has_written_header: false,
        }
    }

    /// Write the header if it hasn't been written yet.
    fn maybe_write_header(&mut self) -> std::io::Result<()> {
        if self.has_written_header {
            return Ok(());
        }
        self.has_written_header = true;
        let fields = &self.schema.fields;
        let field_names = fields.iter().map(|f| f.name.as_str());
        self.csv_writer.write_record(field_names)?;
        Ok(())
    }
}

impl<W: std::io::Write> LineWriter for CSVLineWriter<W> {
    fn write_line(&mut self, line: &Line) -> std::io::Result<()> {
        self.maybe_write_header()?;
        // TODO: Check the length of values vs the schema
        let string_values = line.values.iter().map(|v| v.to_string());
        self.csv_writer.write_record(string_values)?;
        Ok(())
    }
}

/// A [CSVLineWriter] that writes to a file.
pub type CSVFileLineWriter = CSVLineWriter<File>;

impl FileLineWriter for CSVFileLineWriter {
    fn file_name(form_name: String) -> String {
        format!("{}.csv", form_name)
    }

    fn new(path: &Path, schema: &LineSchema) -> std::io::Result<Box<Self>> {
        let file = File::create(path)?;
        let writer = CSVLineWriter::new(file, schema);
        Ok(Box::new(writer))
    }
}

/// A [MultiFileWriter] that writes to CSV files.
pub type CSVMultiFileWriter = MultiFileWriter<CSVFileLineWriter>;
