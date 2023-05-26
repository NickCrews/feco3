use std::hash::Hash;

use crate::schemas::lookup_schema;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Date(String),
    Boolean(bool),
}

/// Similar to Value, but just store the type of the value, not the value itself.
#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
pub struct Line {
    pub schema: LineSchema,
    /// May contain fewer or more values than the schema expects.
    pub values: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct LineSchema {
    /// Line code, eg "F3" or "SA11"
    pub code: String,
    pub fields: Vec<FieldSchema>,
}

impl Hash for LineSchema {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.code.hash(state)
    }
}

impl PartialEq for LineSchema {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl Eq for LineSchema {}

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
    fec_version: &String,
    raw: &mut impl Iterator<Item = &'a [u8]>,
) -> Result<Line, String> {
    let line_code = match raw.next() {
        Some(form_name) => form_name,
        None => return Err("No form name".to_string()),
    };
    let line_code_str = String::from_utf8(line_code.to_vec()).map_err(|e| e.to_string())?;
    let form_schema = lookup_schema(fec_version, &line_code_str)?;
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
    Ok(Line {
        schema: form_schema.clone(),
        values: fields,
    })
}

fn parse_raw_field_val(
    raw_value: &[u8],
    field_schema: Option<&FieldSchema>,
) -> Result<crate::line::Value, String> {
    let s = String::from_utf8_lossy(raw_value).to_string();
    let default_field_schema = FieldSchema {
        name: "extra".to_string(),
        typ: ValueType::String,
    };
    let field_schema = field_schema.unwrap_or(&default_field_schema);
    let parsed_val = match field_schema.typ {
        crate::line::ValueType::String => crate::line::Value::String(s),
        crate::line::ValueType::Integer => {
            let i = s.parse::<i64>().map_err(|e| e.to_string())?;
            crate::line::Value::Integer(i)
        }
        crate::line::ValueType::Float => {
            let f = s.parse::<f64>().map_err(|e| e.to_string())?;
            crate::line::Value::Float(f)
        }
        crate::line::ValueType::Date => crate::line::Value::Date(s),
        crate::line::ValueType::Boolean => {
            let b = s.parse::<bool>().map_err(|e| e.to_string())?;
            crate::line::Value::Boolean(b)
        }
    };
    Ok(parsed_val)
}
