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

/// Lookup a schema given the .FEC file version and the form type.
pub fn lookup_schema(version: &String, form_type: &String) -> Result<&'static FormSchema, String> {
    let versions_and_schemas = crate::schemas::SCHEMAS
        .get(form_type.as_str())
        .ok_or(format!("Couldn't find form type: {}", form_type))?;
    for (version_regex, schema) in versions_and_schemas {
        if version_regex.is_match(version) {
            return Ok(schema);
        }
    }
    Err(format!(
        "Couldn't find schema for form type: {}, version: {}",
        form_type, version
    ))
}

impl Eq for FormSchema {}
