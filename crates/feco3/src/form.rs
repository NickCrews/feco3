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
    pub form_schema: LineSchema,
    pub fields: Vec<Field>,
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
