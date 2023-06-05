//! Helpers for converting [Record]s into arrow [RecordBatch]es.
use arrow::array::{
    ArrayBuilder, BooleanBuilder, Date32Builder, Float64Builder, Int64Builder, StringBuilder,
};
use arrow::datatypes::Date32Type;
use arrow::{
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use std::sync::Arc;

use crate::Record;
use crate::{
    record::{FieldSchema, RecordSchema, Value, ValueType},
    writers::base::RecordWriter,
};

/// Convert a [ValueType] into the arrow equivalent, an arrow [DataType].
pub fn value_type_to_arrow_type(vt: &ValueType) -> DataType {
    match vt {
        ValueType::String => DataType::Utf8,
        ValueType::Integer => DataType::Int64,
        ValueType::Float => DataType::Float64,
        ValueType::Date => DataType::Date32,
        ValueType::Boolean => DataType::Boolean,
    }
}

/// Convert a [FieldSchema] into the arrow equivalent, an arrow [Field]
pub fn field_schema_to_arrow_field(fs: &FieldSchema) -> Field {
    Field::new(fs.name.clone(), value_type_to_arrow_type(&fs.typ), true)
}

/// Convert a [RecordSchema] into the arrow equivalent, an arrow [Schema]
pub fn record_schema_to_arrow_schema(rs: &RecordSchema) -> Schema {
    let fields = rs
        .fields
        .iter()
        .map(field_schema_to_arrow_field)
        .collect::<Vec<_>>();
    Schema::new(fields)
}

/// A [RecordWriter] that buffers records into arrow [RecordBatch]es.
///
/// This isn't useful by itself. Users will want to take the buffered
/// batches and write them to a file or stream, or perhaps pass them
/// to Python or R.
pub struct RecordBatchWriter {
    feco3_schema: RecordSchema,
    builders: Vec<Box<dyn ArrayBuilder>>,
}

impl RecordBatchWriter {
    pub fn new(feco3_schema: RecordSchema, capacity: usize) -> Self {
        let builders =
            builders_from_schema(&record_schema_to_arrow_schema(&feco3_schema), capacity);
        Self {
            feco3_schema,
            builders,
        }
    }

    /// Build and return the accumulated [RecordBatch], and reset itself.
    pub fn finish(&mut self) -> RecordBatch {
        let arrays = self
            .builders
            .iter_mut()
            .map(|b| b.finish())
            .collect::<Vec<_>>();
        let schema = record_schema_to_arrow_schema(&self.feco3_schema);
        RecordBatch::try_new(Arc::new(schema), arrays).unwrap()
    }

    /// The number of records buffered.
    pub fn len(&self) -> usize {
        self.builders[0].len()
    }
}

impl RecordWriter for RecordBatchWriter {
    fn write_record(&mut self, record: &Record) -> std::io::Result<()> {
        if record.schema != self.feco3_schema {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "record schema does not match writer schema",
            ));
        }
        for (i, val) in record.values.iter().enumerate() {
            append_value_to_builder(&mut *self.builders[i], val);
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<(), crate::Error> {
        Ok(())
    }
}

fn builders_from_schema(schema: &Schema, capacity: usize) -> Vec<Box<dyn ArrayBuilder>> {
    schema
        .fields
        .iter()
        .map(|fs| arrow::array::make_builder(fs.data_type(), capacity))
        .collect()
}

fn append_value_to_builder(builder: &mut dyn ArrayBuilder, val: &Value) {
    match val {
        Value::Integer(i) => builder
            .as_any_mut()
            .downcast_mut::<Int64Builder>()
            .unwrap()
            .append_option(*i),
        Value::Float(f) => builder
            .as_any_mut()
            .downcast_mut::<Float64Builder>()
            .unwrap()
            .append_option(*f),
        Value::String(s) => builder
            .as_any_mut()
            .downcast_mut::<StringBuilder>()
            .unwrap()
            .append_option(s.clone()),
        Value::Date(d) => builder
            .as_any_mut()
            .downcast_mut::<Date32Builder>()
            .unwrap()
            .append_option(d.map(Date32Type::from_naive_date)),
        Value::Boolean(b) => builder
            .as_any_mut()
            .downcast_mut::<BooleanBuilder>()
            .unwrap()
            .append_option(*b),
    }
}
