use crate::form::{FieldSchema, FormSchema, ValueType};
use regex::Regex;
use std::collections::HashMap;

type VersionToSchema = Vec<(Regex, FormSchema)>;
type FormTypeToSchema = HashMap<&'static str, VersionToSchema>;

lazy_static! {
    static ref SCHEMAS_F3N: VersionToSchema = vec![(
        Regex::new("^8.4|8.3|8.2").unwrap(),
        FormSchema {
            name: "F3N".to_string(),
            fields: vec![
                FieldSchema {
                    name: "form_type".to_string(),
                    typ: ValueType::String,
                },
                FieldSchema {
                    name: "filer_committee_id_number".to_string(),
                    typ: ValueType::String,
                },
                FieldSchema {
                    name: "transaction_id_number".to_string(),
                    typ: ValueType::String,
                },
            ],
        }
    )];
    pub static ref SCHEMAS: FormTypeToSchema = HashMap::from([("F3N", SCHEMAS_F3N.clone())]);
}
