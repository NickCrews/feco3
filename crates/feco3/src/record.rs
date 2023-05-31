use std::fmt;
use std::hash::Hash;

use crate::schemas::lookup_schema;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Date(chrono::NaiveDate),
    Boolean(bool),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Date(d) => write!(f, "{}", d.format("%Y-%m-%d")),
            Value::Boolean(b) => write!(f, "{}", b),
        }
    }
}

impl Value {
    pub fn typ(&self) -> ValueType {
        match self {
            Value::String(_) => ValueType::String,
            Value::Integer(_) => ValueType::Integer,
            Value::Float(_) => ValueType::Float,
            Value::Date(_) => ValueType::Date,
            Value::Boolean(_) => ValueType::Boolean,
        }
    }
}

/// Similar to Value, but just store the type of the value, not the value itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    String,
    Integer,
    Float,
    Date,
    Boolean,
}

#[derive(Debug, Clone)]
pub struct FieldSchema {
    pub name: String,
    pub typ: ValueType,
}

/// A parsed line of a .FEC file.
#[derive(Debug, Clone)]
pub struct Record {
    pub schema: RecordSchema,
    /// May contain fewer or more values than the schema expects.
    pub values: Vec<Value>,
}

impl Record {
    pub fn get_value(&self, field_name: &str) -> Option<&Value> {
        let field_index = self
            .schema
            .fields
            .iter()
            .position(|f| f.name == field_name)?;
        self.values.get(field_index)
    }
}

#[derive(Debug, Clone)]
pub struct RecordSchema {
    /// Record code, eg "F3" or "SA11"
    pub code: String,
    pub fields: Vec<FieldSchema>,
}

impl Hash for RecordSchema {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.code.hash(state)
    }
}

impl PartialEq for RecordSchema {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl Eq for RecordSchema {}

/// Parse a line of a .FEC file.
///
/// Given a version string like "8.0" and a iterable of byte slices,
/// take the first item as the line code, and the rest as the values.
/// Lookup the schema for the line code and version, and parse the values
/// according to the schema.
///
/// We return a line with the exact number of values seen, which
/// might be different from the expected number of values in the schema.
/// This is because the FEC files are not always consistent with the schema.
/// If we see more values than expected, we don't know what type they are
/// supposed to be, so we just return them as Strings.
pub fn parse<'a>(
    fec_version: &str,
    raw: &mut impl Iterator<Item = &'a str>,
) -> Result<Record, String> {
    let line_code = match raw.next() {
        Some(form_name) => form_name,
        None => return Err("No form name".to_string()),
    };
    let form_schema = lookup_schema(fec_version, &line_code)?;
    let mut schema_fields = form_schema.fields.iter();
    let mut fields = Vec::new();
    fields.push(parse_raw_field_val(line_code, None)?);
    for raw_value in raw {
        fields.push(parse_raw_field_val(raw_value, schema_fields.next())?);
    }
    let extra_schema_fields = schema_fields.count();
    if extra_schema_fields > 0 {
        log::error!("extra_schema_fields: {}", extra_schema_fields);
    }
    Ok(Record {
        schema: form_schema.clone(),
        values: fields,
    })
}

fn parse_raw_field_val(
    raw: &str,
    field_schema: Option<&FieldSchema>,
) -> Result<crate::record::Value, String> {
    // let s = String::from_utf8_lossy(raw_value).to_string();
    let default_field_schema = FieldSchema {
        name: "extra".to_string(),
        typ: ValueType::String,
    };
    let field_schema = field_schema.unwrap_or(&default_field_schema);
    let parsed_val = match field_schema.typ {
        crate::record::ValueType::String => crate::record::Value::String(raw.to_string()),
        crate::record::ValueType::Integer => {
            let i = raw.parse::<i64>().map_err(|e| e.to_string())?;
            crate::record::Value::Integer(i)
        }
        crate::record::ValueType::Float => {
            let f = raw.parse::<f64>().map_err(|e| e.to_string())?;
            crate::record::Value::Float(f)
        }
        crate::record::ValueType::Date => crate::record::Value::Date(parse_date(raw)?),
        crate::record::ValueType::Boolean => {
            let b = raw.parse::<bool>().map_err(|e| e.to_string())?;
            crate::record::Value::Boolean(b)
        }
    };
    Ok(parsed_val)
}

fn parse_date(raw: &str) -> Result<chrono::NaiveDate, String> {
    let date = chrono::NaiveDate::parse_from_str(raw, "%Y%m%d").map_err(|e| e.to_string())?;
    Ok(date)
}
