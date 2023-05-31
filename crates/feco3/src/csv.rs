//! A wrapper around [csv::Reader] that returns raw Vec<&str> records.

use std::io::Read;

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
pub struct CsvReader<R: Read> {
    records: csv::StringRecordsIntoIter<R>,
}

impl<R: Read> CsvReader<R> {
    pub fn new(src: R, sep: &Sep) -> Self {
        let reader = ReaderBuilder::new()
            .delimiter(sep.to_byte())
            .has_headers(false)
            .flexible(true)
            .from_reader(src);
        Self {
            records: reader.into_records(),
        }
    }

    /// Get the next raw line of the CSV file.
    ///
    /// Returns None if there are no more lines.
    /// Returns Some(Err) if there was an error parsing the line.
    /// Returns Some(Ok) if the line was parsed successfully.
    ///
    /// The Ok value is a Vec<&str> of the fields in the line.
    /// The caller is responsible for converting the fields to the correct types.
    pub fn next_line(&mut self) -> Option<Result<Vec<String>, String>> {
        let record_or_err = self.records.next()?;
        log::debug!("raw_record: {:?}", record_or_err);
        let strings: Vec<String> = match record_or_err {
            Err(e) => return Some(Err(e.to_string())),
            Ok(record) => record.iter().map(|s| s.to_string()).collect(),
        };
        Some(Ok(strings))
    }
}
