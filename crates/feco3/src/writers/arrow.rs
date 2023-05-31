use std::{collections::HashMap, sync::Arc};

use arrow::{
    array::{
        ArrayBuilder, BooleanBuilder, Date32Builder, Float64Builder, Int64Builder, StringBuilder,
    },
    datatypes::{Date32Type, Field, Schema},
    record_batch::RecordBatch,
};

use crate::{
    record::{FieldSchema, RecordSchema, Value, ValueType},
    writers::base::{RecordWriter, RecordWriterFactory},
};

pub struct ArrowWriter {
    feco3_schema: RecordSchema,
    arrow_schema: Arc<Schema>,
    builders: HashMap<String, Box<dyn ArrayBuilder>>,
}

fn value_type_to_builder_type(value_type: ValueType) -> Box<dyn ArrayBuilder> {
    match value_type {
        ValueType::String => Box::new(StringBuilder::new()),
        ValueType::Integer => Box::new(Int64Builder::new()),
        ValueType::Float => Box::new(Float64Builder::new()),
        ValueType::Date => Box::new(Date32Builder::new()),
        ValueType::Boolean => Box::new(BooleanBuilder::new()),
    }
}

fn field_schema_to_arrow_field(fs: &FieldSchema) -> Field {
    match fs.typ {
        ValueType::String => Field::new(fs.name.clone(), arrow::datatypes::DataType::Utf8, true),
        ValueType::Integer => Field::new(fs.name.clone(), arrow::datatypes::DataType::Int64, true),
        ValueType::Float => Field::new(fs.name.clone(), arrow::datatypes::DataType::Float64, true),
        ValueType::Date => Field::new(fs.name.clone(), arrow::datatypes::DataType::Date32, true),
        ValueType::Boolean => {
            Field::new(fs.name.clone(), arrow::datatypes::DataType::Boolean, true)
        }
    }
}

fn record_schema_to_arrow_schema(rs: &RecordSchema) -> Schema {
    let fields = rs
        .fields
        .iter()
        .map(field_schema_to_arrow_field)
        .collect::<Vec<_>>();
    Schema::new(fields)
}

impl ArrowWriter {
    pub fn new(feco3_schema: &RecordSchema) -> std::io::Result<Self> {
        let mut builders = HashMap::new();
        for field in feco3_schema.fields.iter() {
            builders.insert(field.name.clone(), value_type_to_builder_type(field.typ));
        }
        let arrow_schema = record_schema_to_arrow_schema(feco3_schema);
        Ok(Self {
            feco3_schema: feco3_schema.clone(),
            arrow_schema: Arc::new(arrow_schema),
            builders,
        })
    }

    fn get_record_batch(&mut self) -> Result<RecordBatch, String> {
        let arrays = self
            .builders
            .iter_mut()
            .map(|(_field_name, builder)| builder.finish())
            .collect::<Vec<_>>();
        RecordBatch::try_new(self.arrow_schema.clone(), arrays).map_err(|e| e.to_string())
    }
}

impl RecordWriter for ArrowWriter {
    fn write_record(&mut self, record: &crate::Record) -> std::io::Result<()> {
        if record.schema != self.feco3_schema {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Record schema doesn't match writer schema: {:?} != {:?}",
                    record.schema, self.feco3_schema
                ),
            ));
        }
        for (field, value) in record.schema.fields.iter().zip(record.values.iter()) {
            let builder = self.builders.get_mut(&field.name).unwrap();
            match value {
                Value::String(s) => builder
                    .as_any_mut()
                    .downcast_mut::<StringBuilder>()
                    .unwrap()
                    .append_value(s.as_str()),
                Value::Integer(i) => builder
                    .as_any_mut()
                    .downcast_mut::<Int64Builder>()
                    .unwrap()
                    .append_value(*i),

                Value::Float(f) => builder
                    .as_any_mut()
                    .downcast_mut::<Float64Builder>()
                    .unwrap()
                    .append_value(*f),
                Value::Date(d) => builder
                    .as_any_mut()
                    .downcast_mut::<Date32Builder>()
                    .unwrap()
                    .append_value(Date32Type::from_naive_date(*d)),
                Value::Boolean(b) => builder
                    .as_any_mut()
                    .downcast_mut::<BooleanBuilder>()
                    .unwrap()
                    .append_value(*b),
            }
        }
        Ok(())
    }
}

pub struct ArrowWriterFactory;

impl RecordWriterFactory for ArrowWriterFactory {
    fn make(&mut self, schema: &RecordSchema) -> std::io::Result<Box<dyn RecordWriter>> {
        Ok(Box::new(ArrowWriter::new(schema)?))
    }
}
