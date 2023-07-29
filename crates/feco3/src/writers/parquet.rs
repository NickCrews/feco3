use parquet::{arrow::ArrowWriter, file::properties::WriterProperties};
use std::{fs::File, path::PathBuf, sync::Arc};

use crate::record::Record;
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
        writer.write(&self.batcher.build_batch())?;
        Ok(())
    }
}

impl RecordWriter for ParquetWriter {
    fn write_record(&mut self, record: &Record) -> std::io::Result<()> {
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
    type Writer = ParquetWriter;
    fn file_name(&self, form_name: String) -> String {
        format!("{}.parquet", form_name)
    }
    fn make(&mut self, path: &PathBuf, schema: &RecordSchema) -> std::io::Result<Self::Writer> {
        let file = File::create(path)?;
        Ok(ParquetWriter::new(file, schema, self.props.clone())?)
    }
}

/// Writes forms to a directory of CSV files.
///
/// Each form type gets its own file. If the form type contains a "/"
/// (which would result in a subdirectory), it is replaced with a "-".
/// For example, "SC/10" would be written to "SC-10.csv".
pub struct ParquetProcessor {
    writer: MultiRecordWriter<MultiFileRecordWriterFactory<ParquetWriterFactory>>,
}

impl ParquetProcessor {
    /// Create a new CSVProcessor that writes to the given directory.
    ///
    /// `writer_props` can be used to configure the parquet writer used for
    /// each file. If None, the default writer properties are used.
    pub fn new(out_dir: PathBuf, writer_props: Option<WriterProperties>) -> Self {
        let factory = ParquetWriterFactory {
            props: writer_props,
        };
        let f2 = MultiFileRecordWriterFactory::new(out_dir, factory);
        let writer = MultiRecordWriter::new(f2);
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
