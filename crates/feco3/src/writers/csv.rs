use super::base::{
    FileRecordWriterFactory, MultiFileRecordWriterFactory, MultiRecordWriter, RecordWriter,
};
use crate::record::{Record, RecordSchema};
use std::{fs::File, path::PathBuf};

/// A [RecordWriter] that writes to CSV format.
struct CSVFormWriter<W: std::io::Write> {
    csv_writer: csv::Writer<W>,
    schema: RecordSchema,
    has_written_header: bool,
}

impl<W: std::io::Write> CSVFormWriter<W> {
    fn new(raw_writer: W, schema: &RecordSchema) -> Self {
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

impl<W: std::io::Write> RecordWriter for CSVFormWriter<W> {
    fn write_record(&mut self, record: &Record) -> std::io::Result<()> {
        self.maybe_write_header()?;
        // TODO: Check the length of values vs the schema
        let string_values = record.values.iter().map(|v| v.to_string());
        self.csv_writer.write_record(string_values)?;
        Ok(())
    }
}

struct CSVFileWriterFactory;

impl FileRecordWriterFactory for CSVFileWriterFactory {
    fn file_name(&self, form_name: String) -> String {
        format!("{}.csv", form_name)
    }

    fn make(
        &mut self,
        path: &PathBuf,
        schema: &RecordSchema,
    ) -> std::io::Result<Box<dyn RecordWriter>> {
        let file = File::create(path)?;
        let writer = CSVFormWriter::new(file, schema);
        Ok(Box::new(writer))
    }
}

pub fn csv_files_writer(out_dir: PathBuf) -> MultiRecordWriter {
    let factory = Box::new(CSVFileWriterFactory);
    let f2 = MultiFileRecordWriterFactory::new(out_dir, factory);
    MultiRecordWriter::new(Box::new(f2))
}
