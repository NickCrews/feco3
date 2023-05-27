//! A wrapper around csv::Reader that returns Line objects.

use std::io::Read;
use std::str::{from_utf8, Utf8Error};

use crate::line::{parse, Line};
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
    pub fn detect(s: &[u8]) -> Self {
        if s.contains(&b'\x1c') {
            Self::Ascii28
        } else {
            Self::Comma
        }
    }
}

/// A convenience wrapper around a csv::Reader.
pub struct CsvParser<R: Read> {
    fec_version: String,
    records: csv::ByteRecordsIntoIter<R>,
}

impl<R: Read> CsvParser<R> {
    pub fn new(src: R, fec_version: String, sep: &Sep) -> Self {
        let reader = ReaderBuilder::new()
            .delimiter(sep.to_byte())
            .has_headers(false)
            .flexible(true)
            .from_reader(src);
        Self {
            fec_version,
            records: reader.into_byte_records(),
        }
    }

    pub fn next_line(&mut self) -> Option<Result<Line, String>> {
        let record_or_err: Result<csv::ByteRecord, csv::Error> = self.records.next()?;
        log::debug!("raw_record: {:?}", record_or_err);
        let record: csv::ByteRecord = match record_or_err {
            Ok(record) => record,
            Err(e) => return Some(Err(e.to_string())),
        };
        let raw_fields: Result<Vec<&str>, Utf8Error> = record.into_iter().map(from_utf8).collect();
        let fields = match raw_fields {
            Ok(fields) => fields,
            Err(e) => return Some(Err(e.to_string())),
        };
        let line = parse(&self.fec_version, &mut fields.into_iter());
        Some(line)
    }
}
