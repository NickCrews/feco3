use std::io::{BufRead, BufReader, Lines, Read};

use crate::header::{parse_header, Header, HeaderParseError};

#[derive(Debug)]
pub struct Parser<R> {
    lines: Lines<BufReader<R>>,
    /// If parsed yet, contains the header
    header: Option<Header>,
}

impl<R: Read> Parser<R> {
    pub fn from_reader(reader: R) -> Self {
        Self {
            lines: BufReader::new(reader).lines(),
            header: None,
        }
    }

    pub fn parse_header(&mut self) -> Result<Header, HeaderParseError> {
        let header = parse_header(&mut self.lines)?;
        self.header = Some(header.clone());
        Ok(header)
    }
}
