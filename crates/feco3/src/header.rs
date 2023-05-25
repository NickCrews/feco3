use std::{
    fmt,
    io::{BufReader, Read},
};

use crate::parser::Sep;
use bytelines::ByteLines;
use regex::bytes;

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
    pub sep: Sep,
}

type Result = std::result::Result<HeaderParsing, HeaderParseError>;

type Lines<R> = bytelines::ByteLinesIter<BufReader<R>>;

// Read from src and parse the header.
pub fn parse_header(src: &mut impl Read) -> Result {
    // Only buffer one character at a time so that we don't over-consume
    // the src. As soon as we see every line of the header, we want to stop
    // reading so the rest of src can be used by the RowsParser.
    let mut lines = ByteLines::new(BufReader::with_capacity(1, src)).into_iter();
    let mut read_lines = Vec::new();
    let first_line = next_line(&mut read_lines, &mut lines)?;

    // If the first line contains "/*", its a legacy header.
    if byte_slice_contains(first_line.as_slice(), b"/*") {
        parse_legacy_header(&mut lines, &mut read_lines)
    } else {
        parse_nonlegacy_header(&first_line)
    }
}

// /* Header
// FEC_Ver_# = 2.02
// Soft_Name = FECfile
// Soft_Ver# = 3
// Dec/NoDec = DEC
// Date_Fmat = CCYYMMDD
// NameDelim = ^
// Form_Name = F3XA
// FEC_IDnum = C00101766
// Committee = CONTINENTAL AIRLINES INC EMPLOYEE FUND FOR A BETTER AMERICA (FKA CONTINENTAL HOLDINGS PAC)
// Control_# = K245592Q
// Schedule_Counts:
// SA11A1    = 00139
// SA17      = 00001
// SB23      = 00008
// SB29      = 00003
// /* End Header
// --- Other lines---
fn parse_legacy_header(lines: &mut Lines<impl Read>, read_lines: &mut Vec<Vec<u8>>) -> Result {
    log::debug!("parsing legacy header");
    // read from lines until we hit another "/*" or we've read 100 lines,
    // at which point we error
    let mut header = Header::default();
    let mut num_lines = 0;
    loop {
        let line = next_line(read_lines, lines)?;
        if byte_slice_contains(line.as_slice(), b"/*") {
            break;
        }
        num_lines += 1;
        if num_lines > 100 {
            return Err(HeaderParseError {
                read: read_lines.concat(),
            });
        }
        let parts: Vec<&[u8]> = line.split(|c| *c == b'=').collect();
        if parts.len() != 2 {
            return Err(HeaderParseError {
                read: read_lines.concat(),
            });
        }
        let key = parts[0];
        let value = parts[1];
        match norm_header_key(key).as_str() {
            "fec_ver_#" => header.version = norm_header_value(value),
            "soft_name" => header.software_name = norm_header_value(value),
            "soft_ver#" => header.software_version = norm_header_value(value),
            _ => {}
        }
    }
    // Make sure we've found all the required fields.
    if header.version == "" || header.software_name == "" || header.software_version == "" {
        return Err(HeaderParseError {
            read: read_lines.concat(),
        });
    }
    Ok(HeaderParsing {
        header,
        sep: Sep::Comma,
    })
}

fn norm_header_value(value: &[u8]) -> String {
    bytes_to_string(value).as_str().trim().to_string()
}

fn norm_header_key(key: &[u8]) -> String {
    bytes_to_string(key)
        .to_lowercase()
        .as_str()
        .trim()
        .to_string()
}

/// Parse the header from a non-legacy file.
///
/// This is based on the logic at
/// https://github.com/esonderegger/fecfile/blob/a5ad9af6fc3b408acaf386871e608085f374441e/fecfile/fecparser.py#L134
///
/// This looks like a single line:
/// "HDRFEC8.3NGP8"
/// or
/// "HDR8.3NGP8"
fn parse_nonlegacy_header(line: &Vec<u8>) -> Result {
    log::debug!("parsing non-legacy header");
    let mut header = Header::default();
    let sep = Sep::detect(line);
    log::debug!("separator: {:?}", sep);
    let parts: Vec<&[u8]> = line.split(|c| *c == sep.to_byte()).collect();

    if parts.len() < 2 {
        return Err(HeaderParseError { read: line.clone() });
    }
    if parts[1] == b"FEC" {
        if parts.len() < 5 {
            return Err(HeaderParseError { read: line.clone() });
        }
        header.version = bytes_to_string(parts[2]);
        header.software_name = bytes_to_string(parts[3]);
        header.software_version = bytes_to_string(parts[4]);
    } else {
        if parts.len() < 4 {
            return Err(HeaderParseError { read: line.clone() });
        }
        header.version = bytes_to_string(parts[1]);
        header.software_name = bytes_to_string(parts[2]);
        header.software_version = bytes_to_string(parts[3]);
    }
    Ok(HeaderParsing { header, sep })
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

fn byte_slice_contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn bytes_to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}
