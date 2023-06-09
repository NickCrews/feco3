use crate::{
    record::{Record, RecordSchema, Value},
    Error,
};

use super::lookup_schema;

pub trait LineParser<'a> {
    /// Parse the values to a given schema.
    fn parse_values(
        &mut self,
        schema: &RecordSchema,
        line: &mut impl Iterator<Item = &'a String>,
    ) -> Result<Vec<Value>, Error>;

    /// Parse a complete line of a .FEC file.
    ///
    /// Given a version string like "8.0" and a iterable of byte slices,
    /// take the first item as the line code, and the rest as the values.
    /// Lookup the schema for the line code and version, and parse the values
    /// according to the schema.
    fn parse_line(
        &mut self,
        fec_version: &str,
        line: &mut impl Iterator<Item = &'a String>,
    ) -> Result<Record, Error> {
        let (record_type, line) = get_record_type_code(line)?;
        let schema: &RecordSchema = lookup_schema(fec_version, record_type)?;
        let values = self.parse_values(schema, line)?;
        Ok(Record {
            record_type: record_type.to_string(),
            schema: schema.clone(),
            values,
        })
    }
}

/// A LineParser that returns a line with the exact values seen.
///
/// This might be different from the expected number of values in the schema.
/// This is because the FEC files are not always consistent with the schema.
/// If we see more values than expected, we don't know what type they are
/// supposed to be, so we just return them as Strings.
pub struct LiteralLineParser;

impl<'a> LineParser<'a> for LiteralLineParser {
    fn parse_values(
        &mut self,
        schema: &RecordSchema,
        raw: &mut impl Iterator<Item = &'a String>,
    ) -> Result<Vec<Value>, Error> {
        let mut field_schemas = schema.fields.iter();
        let mut values = Vec::new();
        for raw_value in raw {
            let field_schema = field_schemas
                .next()
                .ok_or(Error::RecordParseError("too many values".to_string()))?;
            let rv = match raw_value.trim() {
                "" => None,
                s => Some(s.to_string()),
            };
            let value = field_schema.typ.parse_to_value(rv.as_ref())?;
            values.push(value);
        }
        let extra_schema_fields = field_schemas.count();
        if extra_schema_fields > 0 {
            log::error!("warning: extra_schema_fields: {}", extra_schema_fields);
        }
        Ok(values)
    }
}

/// The first value in each line is the record type code.
fn get_record_type_code<'a, T>(mut line: T) -> Result<(&'a str, T), Error>
where
    T: Iterator<Item = &'a String>,
{
    let record_type = line
        .next()
        .ok_or(Error::RecordParseError("No form name".to_string()))?;
    Ok((record_type, line))
}

pub struct CoercingLineParser;

impl<'a> LineParser<'a> for CoercingLineParser {
    fn parse_values(
        &mut self,
        schema: &RecordSchema,
        line: &mut impl Iterator<Item = &'a String>,
    ) -> Result<Vec<Value>, Error> {
        let mut field_schemas = schema.fields.iter();
        let mut values = Vec::new();
        for raw in line {
            let field_type = match field_schemas.next() {
                Some(field_schema) => field_schema.typ,
                None => {
                    let default_value = Value::String(Some(raw.clone()));
                    values.push(default_value);
                    continue;
                }
            };
            let value = match field_type.parse_to_value(Some(raw)) {
                Ok(value) => value,
                Err(_) => field_type.parse_to_value(None)?,
            };
            values.push(value);
        }
        let not_seen_fields = field_schemas;
        for f in not_seen_fields {
            let value = f.typ.parse_to_value(None)?;
            values.push(value);
        }
        assert!(values.len() == schema.fields.len());
        Ok(values)
    }
}
