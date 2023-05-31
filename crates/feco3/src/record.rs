use std::fmt;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub enum Value {
    String(Option<String>),
    Integer(Option<i64>),
    Float(Option<f64>),
    Date(Option<chrono::NaiveDate>),
    Boolean(Option<bool>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::String(Some(s)) => write!(f, "{}", s),
            Value::String(None) => write!(f, ""),
            Value::Integer(Some(i)) => write!(f, "{}", i),
            Value::Integer(None) => write!(f, ""),
            Value::Float(Some(fl)) => write!(f, "{}", fl),
            Value::Float(None) => write!(f, ""),
            Value::Date(Some(d)) => write!(f, "{}", d.format("%Y-%m-%d")),
            Value::Date(None) => write!(f, ""),
            Value::Boolean(Some(b)) => write!(f, "{}", b),
            Value::Boolean(None) => write!(f, ""),
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

impl ValueType {
    pub fn parse_to_value(&self, raw: Option<&String>) -> Result<Value, String> {
        let parsed_val = match raw {
            None => match self {
                ValueType::String => Value::String(None),
                ValueType::Integer => Value::Integer(None),
                ValueType::Float => Value::Float(None),
                ValueType::Date => Value::Date(None),
                ValueType::Boolean => Value::Boolean(None),
            },
            Some(raw) => match self {
                ValueType::String => Value::String(Some(raw.clone())),
                ValueType::Integer => {
                    let i = raw.parse::<i64>().map_err(|e| e.to_string())?;
                    Value::Integer(Some(i))
                }
                ValueType::Float => {
                    let f = raw.parse::<f64>().map_err(|e| e.to_string())?;
                    Value::Float(Some(f))
                }
                ValueType::Date => Value::Date(Some(parse_date(raw)?)),
                ValueType::Boolean => {
                    let b = raw.parse::<bool>().map_err(|e| e.to_string())?;
                    Value::Boolean(Some(b))
                }
            },
        };
        Ok(parsed_val)
    }
}

#[derive(Debug, Clone)]
pub struct FieldSchema {
    pub name: String,
    pub typ: ValueType,
}

/// A parsed line of a .FEC file.
#[derive(Debug, Clone)]
pub struct Record {
    pub line_code: String,
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

fn parse_date(raw: &str) -> Result<chrono::NaiveDate, String> {
    let date = chrono::NaiveDate::parse_from_str(raw, "%Y%m%d").map_err(|e| e.to_string())?;
    Ok(date)
}
