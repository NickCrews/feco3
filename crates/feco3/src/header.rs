//! Parse the header of a fec file.
//!
//! The header holds meta information on the filing itself.
//! This includes the version of the FEC file format, the software used to
//! generate the file, and the version of that software.

use std::{
    fmt,
    io::{BufReader, Read},
};

use crate::parser::Sep;
use bytelines::ByteLines;
use std::result::Result;

#[derive(Debug, Default, Clone)]
pub struct Header {
    pub version: String,
    pub software_name: String,
    pub software_version: String,
}

#[derive(Debug, Clone)]
pub struct HeaderParseError {
    pub message: String,
    pub read: Vec<u8>,
}

impl fmt::Display for HeaderParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "HeaderParseError: {} (read: '{}')",
            self.message,
            String::from_utf8_lossy(&self.read)
        )
    }
}

impl std::error::Error for HeaderParseError {}

#[derive(Debug, Clone)]
pub struct HeaderParsing {
    pub header: Header,
    pub sep: Sep,
}

type Lines<R> = bytelines::ByteLinesIter<BufReader<R>>;

// Read from src and parse the header.
pub fn parse_header(src: &mut impl Read) -> Result<HeaderParsing, HeaderParseError> {
    // Only buffer one character at a time so that we don't over-consume
    // the src. As soon as we see every line of the header, we want to stop
    // reading so the rest of src can be used by the RowsParser.
    let mut lines = ByteLines::new(BufReader::with_capacity(1, src)).into_iter();
    let mut read_bytes = Vec::new();
    let first_line = next_line(&mut read_bytes, &mut lines).map_err(|e| HeaderParseError {
        message: e.to_string(),
        read: read_bytes.clone(),
    })?;

    // If the first line contains "/*", its a legacy header.
    let header;
    if first_line.contains("/*") {
        header = parse_legacy_header(&mut lines, &mut read_bytes)
    } else {
        header = parse_nonlegacy_header(&first_line)
    }
    header.map_err(|e| HeaderParseError {
        message: e,
        read: read_bytes.clone(),
    })
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
fn parse_legacy_header(
    lines: &mut Lines<impl Read>,
    read_bytes: &mut Vec<u8>,
) -> Result<HeaderParsing, String> {
    log::debug!("parsing legacy header");
    // read from lines until we hit another "/*" or we've read 100 lines,
    // at which point we error
    let mut header = Header::default();
    let mut num_lines = 0;
    let max_lines = 100;
    loop {
        let line = next_line(read_bytes, lines)?;
        if line.contains("/*") {
            break;
        }
        num_lines += 1;
        if num_lines > max_lines {
            return Err(format!("more than {} lines in header", max_lines));
        }
        // TODO: parse the schedule counts like in
        // https://github.com/esonderegger/fecfile/blob/a5ad9af6fc3b408acaf386871e608085f374441e/fecfile/fecparser.py#L134
        if line.to_lowercase().contains("schedule_counts") {
            continue;
        }
        let (key, value) = parse_legacy_kv(&line)?;
        match key.to_lowercase().as_str() {
            "fec_ver_#" => header.version = value,
            "soft_name" => header.software_name = value,
            "soft_ver#" => header.software_version = value,
            _ => {}
        }
    }
    // Make sure we've found all the required fields.
    if header.version == "" {
        return Err("missing FEC_Ver_#".to_string());
    }
    if header.software_name == "" {
        return Err("missing Soft_Name".to_string());
    }
    if header.software_version == "" {
        return Err("missing Soft_Ver#".to_string());
    }
    Ok(HeaderParsing {
        header,
        sep: Sep::Comma,
    })
}

fn parse_legacy_kv(line: &str) -> std::result::Result<(String, String), String> {
    let parts = line.split('=').collect::<Vec<&str>>();
    if parts.len() != 2 {
        return Err(format!("more than one '=' in header k=v line: {:?}", line));
    }
    let key = parts[0].trim().to_string();
    let value = parts[1].trim().to_string();
    Ok((key, value))
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
fn parse_nonlegacy_header(line: &String) -> Result<HeaderParsing, String> {
    log::debug!("parsing non-legacy header");
    let mut header = Header::default();
    let sep = Sep::detect(line);
    log::debug!("separator: {:?}", sep);
    let parts: Vec<&str> = line.split(&sep.to_byte().to_string()).collect();

    if parts.len() < 2 {
        return Err("less than 2 parts in header".to_string());
    }
    if parts[1] == "FEC" {
        if parts.len() < 5 {
            return Err("less than 5 parts in header".to_string());
        }
        header.version = parts[2].to_string();
        header.software_name = parts[3].to_string();
        header.software_version = parts[4].to_string();
    } else {
        if parts.len() < 4 {
            return Err("less than 4 parts in header".to_string());
        }
        header.version = parts[1].to_string();
        header.software_name = parts[2].to_string();
        header.software_version = parts[3].to_string();
    }
    Ok(HeaderParsing { header, sep })
}

///Get the next line, return an error if we can't.
fn next_line(
    read_bytes: &mut Vec<u8>,
    lines: &mut Lines<impl Read>,
) -> Result<String, &'static str> {
    let line = match lines.next() {
        None => return Err("unexpected end of file"),
        Some(Ok(line)) => line,
        Some(Err(_e)) => return Err("error reading line"),
    };
    if read_bytes.len() > 0 {
        read_bytes.push(b'\n');
    }
    read_bytes.extend_from_slice(&line);
    Ok(String::from_utf8_lossy(&line).to_string())
}
