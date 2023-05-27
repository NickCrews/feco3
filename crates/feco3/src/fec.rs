use std::fs::File;
use std::io::Read;
use std::mem::take;
use std::path::PathBuf;

use crate::cover::{parse_cover_record, Cover};
use crate::csv::{CsvParser, Sep};
use crate::header::{parse_header, Header};
use crate::record::Record;
use crate::Error;

/// A FEC file, the core data structure of this crate.
///
/// You create a FecFile from a stream of bytes (e.g. a file, an HTTP stream,
/// a python callback function, or some other custom source).
///
/// All methods are lazy and streaming, so nothing is read from the source
/// until you call a method that requires it.
pub struct FecFile {
    /// The source of raw bytes
    reader: Option<Box<dyn Read + Send>>,
    header: Option<Header>,
    cover: Option<Cover>,
    sep: Option<Sep>,
    /// After reading the header, this contains the CSV reader
    /// that will be used to read the rest of the file.
    csv_parser: Option<CsvParser<Box<dyn Read + Send>>>,
}

impl FecFile {
    pub fn from_reader(reader: Box<dyn Read + Send>) -> Self {
        Self {
            reader: Some(reader),
            header: None,
            cover: None,
            sep: None,
            csv_parser: None,
        }
    }

    pub fn from_path(path: &PathBuf) -> Result<Self, Error> {
        let file = File::open(path)?;
        Ok(Self::from_reader(Box::new(file)))
    }

    pub fn get_header(&mut self) -> Result<&Header, Error> {
        self.parse_header()?;
        Ok(self.header.as_ref().expect("header should be set"))
    }

    // TODO: should this not return a reference?
    pub fn get_cover(&mut self) -> Result<&Cover, Error> {
        self.parse_cover()?;
        Ok(self.cover.as_ref().expect("cover should be set"))
    }

    pub fn next_record(&mut self) -> Option<Result<Record, Error>> {
        match self.parse_cover() {
            Err(e) => return Some(Err(e)),
            Ok(_) => (),
        }
        let p = self.csv_parser.as_mut().expect("No row parser");
        match p.next_record() {
            None => return None,
            Some(Ok(record)) => Some(Ok(record)),
            Some(Err(e)) => Some(Err(Error::RecordParseError(e.to_string()))),
        }
    }

    pub fn records(&mut self) -> RecordIter {
        RecordIter { fec_file: self }
    }

    fn parse_header(&mut self) -> Result<(), Error> {
        if self.header.is_some() {
            return Ok(());
        }
        let reader = self.reader.as_mut().expect("no reader");
        let header_parsing =
            parse_header(reader).map_err(|e| Error::HeaderParseError(e.to_string()))?;
        self.header = Some(header_parsing.header.clone());
        self.sep = Some(header_parsing.sep.clone());
        Ok(())
    }

    fn parse_cover(&mut self) -> Result<(), Error> {
        if self.cover.is_some() {
            return Ok(());
        }
        self.make_csv_parser()?;
        let p = self.csv_parser.as_mut().expect("No row parser");
        let record = match p.next_record() {
            None => return Err(Error::HeaderParseError("no cover record".to_string())),
            Some(Ok(record)) => record,
            Some(Err(e)) => return Err(Error::HeaderParseError(e.to_string())),
        };
        self.cover = Some(parse_cover_record(&record).map_err(Error::HeaderParseError)?);
        Ok(())
    }

    fn make_csv_parser(&mut self) -> Result<(), Error> {
        if self.csv_parser.is_some() {
            return Ok(());
        }
        self.parse_header()
            .map_err(|e| Error::HeaderParseError(e.to_string()))?;
        let header = self.header.as_ref().expect("No header");
        let sep = self.sep.as_ref().expect("No sep");
        if self.csv_parser.is_none() {
            // Hand off the reader ownership to the row parser.
            let reader = take(&mut self.reader).expect("no reader");
            self.csv_parser = Some(CsvParser::new(reader, header.fec_version.clone(), &sep));
        }
        Ok(())
    }
}

pub struct RecordIter<'a> {
    fec_file: &'a mut FecFile,
}

impl<'a> Iterator for RecordIter<'a> {
    type Item = Result<Record, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.fec_file.next_record()
    }
}
