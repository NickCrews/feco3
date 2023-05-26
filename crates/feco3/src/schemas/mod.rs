use std::collections::HashMap;

use crate::line::LineSchema;
use serde_json::Value;
use std::sync::Mutex;

/// Lookup a LineSchema given the .FEC file version and the line code.
///
/// The version is the version of the .FEC file format, like "8.0".
/// This is found in the header of the .FEC file.
/// The line code is the first field in each line of the .FEC file.
/// It is a string like "F3" or "SA11".
pub fn lookup_schema(version: &String, line_code: &String) -> Result<&'static LineSchema, String> {
    let key = (version.clone(), line_code.clone());
    if let Some(schema) = CACHE.lock().unwrap().get(&key) {
        return Ok(schema);
    }
    let schema = do_lookup(version, line_code)?;
    CACHE.lock().unwrap().insert(key, schema);
    Ok(schema)
}

fn do_lookup(version: &String, line_code: &String) -> Result<&'static LineSchema, String> {
    log::debug!(
        "looking up schema for version: '{}', line_code: '{}'",
        version,
        line_code
    );
    for (line_code_regex, versions_and_schemas) in MAPPINGS.iter() {
        if !line_code_regex.is_match(line_code) {
            continue;
        }
        log::debug!("matched line code regex: {:?}", line_code_regex);
        for (version_regex, fields) in versions_and_schemas {
            if !version_regex.is_match(version) {
                continue;
            }
            log::debug!("matched version regex: {:?}", version_regex);
            let mut field_schemas = Vec::new();
            // TODO: Look up the types in types.json
            for field_name in fields {
                field_schemas.push(crate::line::FieldSchema {
                    name: field_name.clone(),
                    typ: crate::line::ValueType::String,
                });
            }
            let schema = LineSchema {
                code: line_code.clone(),
                fields: field_schemas,
            };
            log::debug!("found schema: {:?}", schema);

            // We should only do this once for each schema, so we can leak the Box.
            return Ok(Box::leak(Box::new(schema)));
        }
    }
    Err(format!(
        "Couldn't find schema for form type: {}, version: {}",
        line_code, version
    ))
}

lazy_static! {
    static ref CACHE: Mutex<HashMap<(String, String), &'static LineSchema>> =
        Mutex::new(HashMap::new());
    static ref MAPPINGS: Vec<(FormRegex, Vec<(VersionRegex, Vec<String>)>)> = load_mappings();
}

type VersionRegex = regex::Regex;
type FormRegex = regex::Regex;

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
