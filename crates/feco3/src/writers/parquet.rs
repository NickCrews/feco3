use parquet::{arrow::ArrowWriter, file::properties::WriterProperties};
use std::{fs::File, path::PathBuf, sync::Arc};

use crate::schemas::{CoercingLineParser, LineParser};
use crate::{record::RecordSchema, writers::base::RecordWriter};
use crate::{Error, FecFile};

use super::arrow::{record_schema_to_arrow_schema, RecordBatchWriter};
use super::base::{FileRecordWriterFactory, MultiFileRecordWriterFactory, MultiRecordWriter};

pub struct ParquetWriter {
    batcher: RecordBatchWriter,
    writer: Option<ArrowWriter<File>>,
    /// The number of records to buffer before writing a batch.
    batch_size: usize,
}

impl ParquetWriter {
    pub fn new(
        file: File,
        feco3_schema: &RecordSchema,
        props: Option<WriterProperties>,
    ) -> std::io::Result<Self> {
        let arrow_schema = Arc::new(record_schema_to_arrow_schema(feco3_schema));
        let props = props.unwrap_or_else(|| WriterProperties::builder().build());
        let batch_size = props.max_row_group_size();
        let batcher = RecordBatchWriter::new(feco3_schema.clone(), batch_size);
        let writer = ArrowWriter::try_new(file, arrow_schema, Some(props.clone())).unwrap();
        Ok(Self {
            batcher,
            writer: Some(writer),
            batch_size,
        })
    }

    fn write_batch(&mut self) -> std::io::Result<()> {
        let writer = self.writer.as_mut().expect("writing to a closed writer");
        writer.write(&self.batcher.finish())?;
        Ok(())
    }
}

impl RecordWriter for ParquetWriter {
    fn write_record(&mut self, record: &crate::Record) -> std::io::Result<()> {
        self.batcher.write_record(record)?;
        if self.batcher.len() < self.batch_size {
            return self.write_batch();
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Error> {
        self.write_batch()?;
        let writer = self.writer.take().expect("writing to a closed writer");
        writer
            .close()
            // FIXME
            .map_err(|e| Error::RecordParseError(e.to_string()))?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ParquetWriterFactory {
    pub props: Option<WriterProperties>,
}

impl FileRecordWriterFactory for ParquetWriterFactory {
    fn file_name(&self, form_name: String) -> String {
        format!("{}.parquet", form_name)
    }
    fn make(
        &mut self,
        path: &PathBuf,
        schema: &RecordSchema,
    ) -> std::io::Result<Box<dyn RecordWriter>> {
        let file = File::create(path)?;
        Ok(Box::new(ParquetWriter::new(
            file,
            schema,
            self.props.clone(),
        )?))
    }
}

/// Processes an entire FEC file, writing each form to a separate file.
pub struct ParquetProcessor {
    writer: MultiRecordWriter,
}

impl ParquetProcessor {
    pub fn new(out_dir: PathBuf, writer_props: Option<WriterProperties>) -> Self {
        let factory = ParquetWriterFactory {
            props: writer_props,
        };
        let f2 = MultiFileRecordWriterFactory::new(out_dir, Box::new(factory));
        let writer = MultiRecordWriter::new(Box::new(f2));
        Self { writer }
    }

    pub fn process(&mut self, fec: &mut FecFile) -> Result<(), Error> {
        let fec_version = fec.get_header()?.fec_version.clone();
        let mut parser = CoercingLineParser;
        for line in fec.lines() {
            let line = line?;
            let record = parser.parse_line(&fec_version, &mut line.iter())?;
            self.writer.write_record(&record)?;
        }
        self.writer.finish()?;
        Ok(())
    }
}
