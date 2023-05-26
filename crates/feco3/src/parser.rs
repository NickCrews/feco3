use std::io::Read;
use std::mem::take;

use crate::form::{FieldSchema, FormLine, ValueType};
use crate::header::{parse_header, HeaderParseError, HeaderParsing};
use crate::schemas::lookup_schema;
use crate::summary::Summary;
// use csv::Reader;
use csv::ReaderBuilder;

#[derive(Debug, Clone)]
pub enum Sep {
    Comma,
    Ascii28,
}

impl Sep {
    /// Return the byte value of the separator.
    /// e.g. b',' or b'\x1c'
    pub fn to_byte(&self) -> u8 {
        match self {
            Sep::Comma => b',',
            Sep::Ascii28 => b'\x1c',
        }
    }

    /// Detect the separator from a string.
    /// If the slice contains b'\x1c', return Ascii28.
    pub fn detect(s: &String) -> Self {
        if s.contains('\x1c') {
            Self::Ascii28
        } else {
            Self::Comma
        }
    }
}

pub struct Parser<R: Read> {
    /// If parsed yet, contains the header
    pub header_parsing: Option<HeaderParsing>,
    /// The source of raw bytes
    reader: Option<R>,
    /// After reading the header, this contains the CSV reader
    /// that will be used to read the rest of the file.
    row_parser: Option<RowsParser<R>>,
}

impl<R: Read> Parser<R> {
    pub fn from_reader(reader: R) -> Self {
        Self {
            reader: Some(reader),
            header_parsing: None,
            row_parser: None,
        }
    }

    pub fn parse_header(&mut self) -> Result<&HeaderParsing, HeaderParseError> {
        if self.reader.is_none() {
            panic!("No reader")
        }
        let header_parsing = parse_header(self.reader.as_mut().unwrap())?;
        self.header_parsing = Some(header_parsing);
        let result = self.header_parsing.as_ref().unwrap();
        Ok(result)
    }

    pub fn parse_summary(&mut self) -> Result<Summary, String> {
        Err("Not implemented".to_string())
    }

    pub fn next_line(&mut self) -> Result<Option<Result<FormLine, String>>, String> {
        if self.row_parser.is_none() {
            // Hand off the reader ownership to the row parser.
            let reader = take(&mut self.reader).ok_or("No reader")?;
            let hp = self.header_parsing.as_ref().ok_or("No header")?;
            self.row_parser = Some(RowsParser::new(reader, hp.header.version.clone(), &hp.sep));
        }
        let rp = self.row_parser.as_mut().ok_or("No row parser")?;
        let line = rp.next_line();
        Ok(line)
    }
}

struct RowsParser<R: Read> {
    /// The version of the FEC file format
    version: String,
    records: csv::ByteRecordsIntoIter<R>,
}

impl<R: Read> RowsParser<R> {
    fn new(src: R, version: String, sep: &Sep) -> Self {
        let reader = ReaderBuilder::new()
            .delimiter(sep.to_byte())
            .has_headers(false)
            .flexible(true)
            .from_reader(src);
        Self {
            version,
            records: reader.into_byte_records(),
        }
    }

    fn next_line(&mut self) -> Option<Result<FormLine, String>> {
        let record_or_err: Result<csv::ByteRecord, csv::Error> = self.records.next()?;
        log::debug!("raw_record: {:?}", record_or_err);
        let record: csv::ByteRecord = match record_or_err {
            Ok(record) => record,
            Err(e) => return Some(Err(e.to_string())),
        };
        Some(self.parse_csv_record(record))
    }

    fn parse_csv_record(&self, record: csv::ByteRecord) -> Result<FormLine, String> {
        let mut record_fields = record.iter();
        let line_code = match record_fields.next() {
            Some(form_name) => form_name,
            None => return Err("No form name".to_string()),
        };
        let line_code_str = String::from_utf8(line_code.to_vec()).map_err(|e| e.to_string())?;
        let form_schema = lookup_schema(&self.version, &line_code_str)?;
        let mut schema_fields = form_schema.fields.iter();
        let mut fields = Vec::new();
        fields.push(parse_raw_field_val(line_code, None)?);
        for raw_value in record_fields {
            fields.push(parse_raw_field_val(raw_value, schema_fields.next())?);
        }
        let extra_schema_fields = schema_fields.count();
        if extra_schema_fields > 0 {
            log::error!("extra_schema_fields: {}", extra_schema_fields);
        }
        Ok(FormLine {
            form_schema: form_schema.clone(),
            fields,
        })
    }
}

fn parse_raw_field_val(
    raw_value: &[u8],
    field_schema: Option<&FieldSchema>,
) -> Result<crate::form::Field, String> {
    let s = String::from_utf8_lossy(raw_value).to_string();
    let default_field_schema = FieldSchema {
        name: "extra".to_string(),
        typ: ValueType::String,
    };
    let field_schema = field_schema.unwrap_or(&default_field_schema);
    let parsed_val = match field_schema.typ {
        crate::form::ValueType::String => crate::form::Value::String(s),
        crate::form::ValueType::Integer => {
            let i = s.parse::<i64>().map_err(|e| e.to_string())?;
            crate::form::Value::Integer(i)
        }
        crate::form::ValueType::Float => {
            let f = s.parse::<f64>().map_err(|e| e.to_string())?;
            crate::form::Value::Float(f)
        }
        crate::form::ValueType::Date => crate::form::Value::Date(s),
        crate::form::ValueType::Boolean => {
            let b = s.parse::<bool>().map_err(|e| e.to_string())?;
            crate::form::Value::Boolean(b)
        }
    };
    Ok(crate::form::Field {
        name: field_schema.name.clone(),
        value: parsed_val,
    })
}
