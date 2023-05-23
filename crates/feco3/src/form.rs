use std::hash::Hash;

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
pub struct Field {
    pub name: String,
    pub value: Value,
}

/// Similar to Field, but don't actually store the values of the fields,
/// just their types.
#[derive(Debug, Clone)]
pub struct FieldSchema {
    pub name: String,
    pub typ: ValueType,
}

#[derive(Debug, Clone)]
pub struct FormLine {
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct FormSchema {
    /// Name of the form type, eg "F3" or "SA11"
    pub name: String,
    pub fields: Vec<FieldSchema>,
}

impl Hash for FormSchema {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl PartialEq for FormSchema {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// Lookup a schema given a form type name.
pub fn lookup_schema(form_type: &[u8]) -> FormSchema {
    // let types = vec![
    //     Value::String,
    //     Value::Integer,
    //     Value::Float,
    //     Value::Date,
    //     Value::Boolean,
    // ];
    let fields = vec![
        FieldSchema {
            name: "a".to_string(),
            typ: ValueType::String,
        },
        FieldSchema {
            name: "b".to_string(),
            typ: ValueType::Integer,
        },
        FieldSchema {
            name: "c".to_string(),
            typ: ValueType::Float,
        },
        FieldSchema {
            name: "d".to_string(),
            typ: ValueType::Date,
        },
        FieldSchema {
            name: "e".to_string(),
            typ: ValueType::Boolean,
        },
    ];
    FormSchema {
        name: String::from_utf8_lossy(form_type).to_string(),
        fields,
    }
}

impl Eq for FormSchema {}
