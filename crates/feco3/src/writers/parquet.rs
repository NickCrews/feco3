use arrow::array::{Array, BooleanArray};
use arrow::array::{Date32Array, Float64Array, Int64Array, StringArray};
use arrow::datatypes::Date32Type;
use parquet::{arrow::ArrowWriter, file::properties::WriterProperties};
use std::vec;
use std::{fs::File, path::PathBuf, sync::Arc};

// use arrow::datatypes::DataType::{Boolean, Date32, Float64, Int64, Utf8};
use arrow::{
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};

use crate::{
    record::{FieldSchema, RecordSchema, Value, ValueType},
    writers::base::RecordWriter,
    Record,
};

use super::base::{FileRecordWriterFactory, MultiFileRecordWriterFactory, MultiRecordWriter};

fn value_type_to_arrow_type(vt: &ValueType) -> DataType {
    match vt {
        ValueType::String => DataType::Utf8,
        ValueType::Integer => DataType::Int64,
        ValueType::Float => DataType::Float64,
        ValueType::Date => DataType::Date32,
        ValueType::Boolean => DataType::Boolean,
    }
}

fn field_schema_to_arrow_field(fs: &FieldSchema) -> Field {
    Field::new(fs.name.clone(), value_type_to_arrow_type(&fs.typ), true)
}

fn record_schema_to_arrow_schema(rs: &RecordSchema) -> Schema {
    let fields = rs
        .fields
        .iter()
        .map(field_schema_to_arrow_field)
        .collect::<Vec<_>>();
    Schema::new(fields)
}

fn value_to_arrow_array(val: &Value) -> Arc<dyn Array> {
    match val {
        Value::Integer(i) => Arc::new(Int64Array::from(vec![*i])),
        Value::Float(f) => Arc::new(Float64Array::from(vec![*f])),
        Value::String(s) => Arc::new(StringArray::from(vec![s.clone()])),
        Value::Date(d) => Arc::new(Date32Array::from(vec![d.map(Date32Type::from_naive_date)])),
        Value::Boolean(b) => Arc::new(BooleanArray::from(vec![*b])),
    }
}

fn record_to_arrow_record(record: &Record) -> RecordBatch {
    let arrays = record.values.iter().map(value_to_arrow_array);
    let names = record.schema.fields.iter().map(|fs| fs.name.clone());
    let is_null = vec![true; record.values.len()];
    let together = names
        .zip(arrays)
        .zip(is_null)
        .map(|((name, array), is_null)| (name, array, is_null));
    RecordBatch::try_from_iter_with_nullable(together).unwrap()
}

pub struct ParquetWriter {
    writer: Option<ArrowWriter<File>>,
}

impl ParquetWriter {
    pub fn new(
        file: File,
        feco3_schema: &RecordSchema,
        props: Option<WriterProperties>,
    ) -> std::io::Result<Self> {
        let arrow_schema = record_schema_to_arrow_schema(feco3_schema);
        let arrow_schema = Arc::new(arrow_schema);
        println!("using schema: {:?}", arrow_schema);
        let writer = ArrowWriter::try_new(file, arrow_schema, props).unwrap();
        Ok(Self {
            // arrow_schema,
            writer: Some(writer),
        })
    }
}

impl RecordWriter for ParquetWriter {
    fn write_record(&mut self, record: &crate::Record) -> std::io::Result<()> {
        let writer = self.writer.as_mut().expect("writing to a closed writer");
        let arrow_record = record_to_arrow_record(record);
        println!("writing record: {:?}", arrow_record.schema());
        writer.write(&arrow_record)?;
        Ok(())
    }

    fn finish(&mut self) -> Result<(), crate::Error> {
        let writer = self.writer.take().expect("writing to a closed writer");
        writer
            .close()
            // FIXME
            .map_err(|e| crate::Error::RecordParseError(e.to_string()))?;
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

pub fn parquet_files_writer(
    out_dir: PathBuf,
    writer_props: Option<WriterProperties>,
) -> MultiRecordWriter {
    let factory = ParquetWriterFactory {
        props: writer_props,
    };
    let f2 = MultiFileRecordWriterFactory::new(out_dir, Box::new(factory));
    MultiRecordWriter::new(Box::new(f2))
}
