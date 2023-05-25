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
    pub form_schema: FormSchema,
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

impl Eq for FormSchema {}
