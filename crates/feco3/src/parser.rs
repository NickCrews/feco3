use std::io::Read;
use std::mem::take;

use crate::header::{parse_header, HeaderParseError, HeaderParsing};
use crate::line::{parse, Line};
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

    pub fn next_line(&mut self) -> Result<Option<Result<Line, String>>, String> {
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

    fn next_line(&mut self) -> Option<Result<Line, String>> {
        let record_or_err: Result<csv::ByteRecord, csv::Error> = self.records.next()?;
        log::debug!("raw_record: {:?}", record_or_err);
        let record: csv::ByteRecord = match record_or_err {
            Ok(record) => record,
            Err(e) => return Some(Err(e.to_string())),
        };
        Some(parse(&self.version, &mut record.iter()))
    }
}
