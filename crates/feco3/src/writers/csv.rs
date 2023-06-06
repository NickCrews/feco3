use super::base::{
    FileRecordWriterFactory, MultiFileRecordWriterFactory, MultiRecordWriter, RecordWriter,
};
use crate::{
    record::{Record, RecordSchema},
    schemas::{CoercingLineParser, LineParser},
    Error, FecFile,
};
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

impl<W: std::io::Write + Send> RecordWriter for CSVFormWriter<W> {
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
    type Writer = CSVFormWriter<File>;
    fn file_name(&self, form_name: String) -> String {
        format!("{}.csv", form_name)
    }

    fn make(&mut self, path: &PathBuf, schema: &RecordSchema) -> std::io::Result<Self::Writer> {
        let file = File::create(path)?;
        let writer = CSVFormWriter::new(file, schema);
        Ok(writer)
    }
}

pub struct CSVProcessor {
    multi_writer: MultiRecordWriter<MultiFileRecordWriterFactory<CSVFileWriterFactory>>,
}

impl CSVProcessor {
    pub fn new(out_dir: PathBuf) -> Self {
        let factory = CSVFileWriterFactory;
        let f2 = MultiFileRecordWriterFactory::new(out_dir, factory);
        let multi_writer = MultiRecordWriter::new(f2);
        Self { multi_writer }
    }

    // TODO: factor this out with ParquetProcessor.process()
    pub fn process(&mut self, fec: &mut FecFile) -> Result<(), Error> {
        let fec_version = fec.get_header()?.fec_version.clone();
        let mut parser = CoercingLineParser;
        for line in fec.lines() {
            let line = line?;
            let record = parser.parse_line(&fec_version, &mut line.iter())?;
            self.multi_writer.write_record(&record)?;
        }
        self.multi_writer.finish()?;
        Ok(())
    }
}
