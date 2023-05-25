use std::{
    fmt,
    io::{BufReader, Read},
};

use bytelines::ByteLines;

#[derive(Debug, Default, Clone)]
pub struct Header {
    pub version: String,
    pub software_name: String,
    pub software_version: String,
}

#[derive(Debug, Clone)]
pub struct HeaderParseError {
    pub read: Vec<u8>,
}

impl fmt::Display for HeaderParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HeaderParseError: {:?}", self.read)
    }
}

impl std::error::Error for HeaderParseError {}

#[derive(Debug, Clone)]
pub struct HeaderParsing {
    pub header: Header,
    pub uses_ascii28: bool,
}

type Result = std::result::Result<HeaderParsing, HeaderParseError>;

type Lines<R> = bytelines::ByteLinesIter<BufReader<R>>;

// Read from src and parse the header.
pub fn parse_header(src: &mut impl Read) -> Result {
    // Only buffer one character at a time so that we don't over-consume
    // the src. As soon as we see every line of the header, we want to stop
    // reading so the rest of src can be used by the RowsParser.
    let mut lines = ByteLines::new(BufReader::with_capacity(1, src)).into_iter();
    // let mut lines = BufReader::with_capacity(1, src).lines();
    let mut read_lines = Vec::new();
    let first_line = next_line(&mut read_lines, &mut lines)?;
    // let first_line = lines.next()()

    // If the first line contains "/*", its a legacy header.
    if byte_slice_contains(first_line.as_slice(), b"/*") {
        log::debug!("legacy header");
        return parse_legacy_header(&mut lines, &mut read_lines);
    } else {
        log::debug!("non-legacy header");
        return parse_nonlegacy_header(&first_line);
    }
}

fn parse_legacy_header(lines: &mut Lines<impl Read>, read_lines: &mut Vec<Vec<u8>>) -> Result {
    let mut header = Header::default();
    header.version = String::from_utf8_lossy(&next_line(read_lines, lines)?).to_string();
    Ok(HeaderParsing {
        header,
        uses_ascii28: false,
    })
}

/// Parse the header from a non-legacy file.
///
/// This looks like a single line:
/// "HDRFEC8.3NGP8"
fn parse_nonlegacy_header(line: &Vec<u8>) -> Result {
    let mut header = Header::default();
    let uses_ascii28 = line.contains(&b'\x1c');
    log::debug!("uses_ascii28: {}", uses_ascii28);
    let sep = if uses_ascii28 { b'\x1c' } else { b',' };
    let mut parts = line.split(|c| *c == sep);
    if parts.next() != Some(b"HDR") {
        return Err(HeaderParseError { read: line.clone() });
    }
    if parts.next() != Some(b"FEC") {
        return Err(HeaderParseError { read: line.clone() });
    }
    header.version = String::from_utf8_lossy(parts.next().unwrap()).to_string();
    header.software_name = String::from_utf8_lossy(parts.next().unwrap()).to_string();
    header.software_version = String::from_utf8_lossy(parts.next().unwrap()).to_string();
    Ok(HeaderParsing {
        header,
        uses_ascii28,
    })
}

///Get the next line, return an error if we can't.
fn next_line(
    read_lines: &mut Vec<Vec<u8>>,
    lines: &mut Lines<impl Read>,
) -> std::result::Result<Vec<u8>, HeaderParseError> {
    let line = match lines.next() {
        Some(Ok(line)) => line,
        // We errored when reading a line.
        Some(Err(_e)) => {
            return Err(HeaderParseError {
                read: read_lines.concat(),
            })
        }
        // We didn't read a line, but we expected to.
        None => {
            return Err(HeaderParseError {
                read: read_lines.concat(),
            })
        }
    };
    read_lines.push(line.clone());
    Ok(line)
}

/// Return true if haystack contains needle.
fn byte_slice_contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}
