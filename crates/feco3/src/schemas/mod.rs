use std::collections::HashMap;

use crate::form::FormSchema;
use serde_json::Value;
use std::sync::Mutex;

/// Lookup a schema given the .FEC file version and the form type.
pub fn lookup_schema(version: &String, form_type: &String) -> Result<&'static FormSchema, String> {
    let key = (version.clone(), form_type.clone());
    if let Some(schema) = CACHE.lock().unwrap().get(&key) {
        return Ok(schema);
    }
    let schema = do_lookup(version, form_type)?;
    CACHE.lock().unwrap().insert(key, schema);
    Ok(schema)
}

fn do_lookup(version: &String, form_type: &String) -> Result<&'static FormSchema, String> {
    for (form_regex, versions_and_schemas) in MAPPINGS.iter() {
        log::debug!("form_regex: {}", form_regex);
        if !form_regex.is_match(form_type) {
            continue;
        }
        for (version_regex, fields) in versions_and_schemas {
            log::debug!("version_regex: {}", version_regex);
            if !version_regex.is_match(version) {
                continue;
            }
            log::debug!("fields: {:?}", fields);
            let mut field_schemas = Vec::new();
            for field_name in fields {
                field_schemas.push(crate::form::FieldSchema {
                    name: field_name.clone(),
                    typ: crate::form::ValueType::String,
                });
            }
            let schema = FormSchema {
                name: form_type.clone(),
                fields: field_schemas,
            };
            return Ok(Box::leak(Box::new(schema)));
        }
    }
    Err(format!(
        "Couldn't find schema for form type: {}, version: {}",
        form_type, version
    ))
}

lazy_static! {
    static ref CACHE: Mutex<HashMap<(String, String), &'static FormSchema>> =
        Mutex::new(HashMap::new());
    static ref MAPPINGS: Vec<(FormRegex, Vec<(VersionRegex, Vec<String>)>)> = load_mappings();
}

type VersionRegex = regex::Regex;
type FormRegex = regex::Regex;

// fn load_from_json() -> Vec<(FormRegex, Vec<(VersionRegex, &'static FormSchema)>)> {
//     let mappings = load_mappings();
//     mappings
// }

fn load_mappings() -> Vec<(FormRegex, Vec<(VersionRegex, Vec<String>)>)> {
    let mappings_str = include_str!("mappings.json");
    let value = match serde_json::from_str(mappings_str).unwrap() {
        Value::Object(map) => map,
        _ => panic!("mappings.json is not a map"),
    };
    let mut result = Vec::new();
    for (form_pattern, versions_value) in value {
        let versions = match versions_value {
            Value::Object(map) => map,
            _ => panic!("mappings.json is not a map"),
        };
        let mut versions_vec = Vec::new();
        for (version_pattern, fields_list_value) in versions {
            let fields_value = match fields_list_value {
                Value::Array(fields) => fields,
                _ => panic!("mappings.json is not a map"),
            };
            let mut fields = Vec::new();
            for field_value in fields_value {
                let field = match field_value {
                    Value::String(s) => s,
                    _ => panic!("mappings.json is not a map"),
                };
                fields.push(field);
            }
            versions_vec.push((make_regex(&version_pattern), fields));
        }
        result.push((make_regex(&form_pattern), versions_vec));
    }
    result
}

fn make_regex(s: &str) -> regex::Regex {
    regex::RegexBuilder::new(s)
        .case_insensitive(true)
        .build()
        .unwrap()
}
