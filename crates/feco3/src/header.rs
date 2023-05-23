use std::{
    fmt,
    io::{BufRead, BufReader, Lines, Read},
};

#[derive(Debug, Default, Clone)]
pub struct Header {
    pub version: String,
    pub software_name: String,
    pub software_version: String,
}

#[derive(Debug, Clone)]
pub struct HeaderParseError {
    lines: Vec<String>,
}

impl fmt::Display for HeaderParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HeaderParseError: {:?}", self.lines)
    }
}

impl std::error::Error for HeaderParseError {}

type Result<T> = std::result::Result<T, HeaderParseError>;

// Create a Header given Lines
pub fn parse_header(lines: &mut impl Read) -> Result<Header> {
    let mut lines = BufReader::new(lines).lines();
    let mut header = Header::default();
    let mut read_lines = Vec::new();
    next_line(&mut read_lines, &mut lines)?;
    header.version = next_line(&mut read_lines, &mut lines)?;
    header.software_name = next_line(&mut read_lines, &mut lines)?;
    header.software_version = next_line(&mut read_lines, &mut lines)?;
    Ok(header)
}

fn next_line(read_lines: &mut Vec<String>, lines: &mut Lines<impl BufRead>) -> Result<String> {
    let line = lines
        .next()
        .ok_or_else(|| HeaderParseError {
            lines: read_lines.clone(),
        })?
        .map_err(|_| HeaderParseError {
            lines: read_lines.clone(),
        })?;
    read_lines.push(line.clone());
    Ok(line)
}
