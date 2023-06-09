use std::fs::File;
use std::io::Read;
use std::mem::take;
use std::path::PathBuf;

use crate::cover::{parse_cover_line, Cover};
use crate::csv::{CsvReader, Sep};
use crate::header::{parse_header, Header};
use crate::Error;

/// A FEC file, the low-level core data structure of this crate.
///
/// You create a FecFile from a stream of bytes (e.g. a file, an HTTP stream,
/// a python callback function, or some other custom source).
/// Then, you can query the various parts of the file, such as the header,
/// the cover, or the itemizations.
///
/// See https://github.com/NickCrews/feco3/wiki for more info.
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
    csv_reader: Option<CsvReader<Box<dyn Read + Send>>>,
}

impl FecFile {
    pub fn from_reader(reader: Box<dyn Read + Send>) -> Self {
        Self {
            reader: Some(reader),
            header: None,
            cover: None,
            sep: None,
            csv_reader: None,
        }
    }

    pub fn from_path(path: &PathBuf) -> Result<Self, Error> {
        let file = File::open(path)?;
        Ok(Self::from_reader(Box::new(file)))
    }

    pub fn from_https(url: &str) -> Result<Self, Error> {
        log::debug!("fetching {}", url);
        let resp = ureq::get(url)
            .set("User-Agent", "Mozilla/5.0")
            .call()
            .map_err(|e| Error::HttpError(e.to_string()))?;
        if resp.status() >= 400 {
            return Err(Error::HttpError(resp.status_text().to_string()));
        }
        let reader = resp.into_reader();
        Ok(Self::from_reader(reader))
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

    // panics if the header hasn't been parsed yet
    fn fec_version(&self) -> String {
        self.header.as_ref().expect("No header").fec_version.clone()
    }

    pub fn next_line(&mut self) -> Option<Result<Vec<String>, Error>> {
        match self.parse_cover() {
            Err(e) => return Some(Err(e)),
            Ok(_) => (),
        }
        let p = self.csv_reader.as_mut().expect("No row parser");
        match p.next_line() {
            None => return None,
            Some(Ok(line)) => Some(Ok(line)),
            Some(Err(e)) => return Some(Err(Error::RecordParseError(e.to_string()))),
        }
    }

    pub fn lines(&mut self) -> LineIter {
        LineIter { fec_file: self }
    }

    fn parse_header(&mut self) -> Result<(), Error> {
        if self.header.is_some() {
            return Ok(());
        }
        let reader = self.reader.as_mut().expect("no reader");
        let header_parsing = parse_header(reader).map_err(Error::HeaderParseError)?;
        self.header = Some(header_parsing.header.clone());
        self.sep = Some(header_parsing.sep.clone());
        Ok(())
    }

    fn parse_cover(&mut self) -> Result<(), Error> {
        if self.cover.is_some() {
            return Ok(());
        }
        self.make_csv_parser()?;
        let fec_version = &self.fec_version().clone();
        let p = self.csv_reader.as_mut().expect("No row parser");
        let line = match p.next_line() {
            None => return Err(Error::CoverParseError("no cover record".to_string())),
            Some(Ok(record)) => record,
            Some(Err(e)) => return Err(Error::CoverParseError(e.to_string())),
        };
        self.cover = Some(parse_cover_line(fec_version, &mut line.iter())?);
        Ok(())
    }

    fn make_csv_parser(&mut self) -> Result<(), Error> {
        if self.csv_reader.is_some() {
            return Ok(());
        }
        self.parse_header()?;
        let sep = self.sep.as_ref().expect("No sep");
        if self.csv_reader.is_none() {
            // Hand off the reader ownership to the row parser.
            let reader = take(&mut self.reader).expect("no reader");
            self.csv_reader = Some(CsvReader::new(reader, sep));
        }
        Ok(())
    }
}

pub struct LineIter<'a> {
    fec_file: &'a mut FecFile,
}

impl<'a> Iterator for LineIter<'a> {
    type Item = Result<Vec<String>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.fec_file.next_line()
    }
}
