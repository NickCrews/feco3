//! Parse the header, the first section of an .FEC file.
//!
//! The header holds meta information on the filing itself, such as
//! the version of the FEC file format, and the software used to
//! generate the file.
//!
//! The header contains

use std::{
    fmt,
    io::{BufReader, Read},
    str::{from_utf8, Utf8Error},
};

use crate::{csv::Sep, record::parse};
use bytelines::ByteLines;
use std::result::Result;

/// The header of a FEC file.
///
/// There might be other bits of information available,
/// but currently we only parse this subset.
/// See the "hdr" section of [mappings.json](mappings.json) to
/// see where these fields come from.
#[derive(Debug, Default, Clone)]
pub struct Header {
    /// The version of the FEC file format.
    pub fec_version: String,
    /// The name of the software used to generate the file.
    pub software_name: String,
    /// The rest of the header fields may be missing,
    /// depending on the version of the FEC file.
    pub software_version: Option<String>,
    pub report_id: Option<String>,
    pub report_number: Option<String>,
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

/// Read from src and parse the header.
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
    if byte_slice_contains(&first_line, b"/*") {
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
        let line_bytes = next_line(read_bytes, lines)?;
        if byte_slice_contains(&line_bytes, b"/*") {
            break;
        }
        num_lines += 1;
        if num_lines > max_lines {
            return Err(format!("more than {} lines in header", max_lines));
        }
        let line = byte_slice_to_string(&line_bytes);
        // TODO: parse the schedule counts like in
        // https://github.com/esonderegger/fecfile/blob/a5ad9af6fc3b408acaf386871e608085f374441e/fecfile/fecparser.py#L134
        if line.to_lowercase().contains("schedule_counts") {
            continue;
        }
        let (key, value) = parse_legacy_kv(&line)?;
        match key.to_lowercase().as_str() {
            "fec_ver_#" => header.fec_version = value,
            "soft_name" => header.software_name = value,
            "soft_ver#" => header.software_version = Some(value),
            _ => {}
        }
    }
    // Make sure we've found all the required fields.
    if header.fec_version == "" {
        return Err("missing FEC_Ver_#".to_string());
    }
    if header.software_name == "" {
        return Err("missing Soft_Name".to_string());
    }
    if header.software_version.is_none() {
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
fn parse_nonlegacy_header(line: &Vec<u8>) -> Result<HeaderParsing, String> {
    log::debug!("parsing non-legacy header");
    let mut header = Header::default();
    let sep = Sep::detect(line);
    log::debug!("separator: {:?}", sep);
    let parts: Result<Vec<&str>, Utf8Error> =
        line.split(|c| *c == sep.to_byte()).map(from_utf8).collect();
    let parts = parts.map_err(|e| e.to_string())?;

    if parts.len() < 2 {
        return Err(format!("less than 2 parts in header: {:?}", parts));
    }
    let version = match parts[1] {
        "FEC" => {
            if parts.len() < 3 {
                return Err(format!("less than 3 parts in header: {:?}", parts));
            }
            parts[2]
        }
        _ => parts[1],
    };
    let line = parse(version, &mut parts.into_iter())?;
    header.fec_version = version.to_string();
    header.software_name = line
        .get_value("soft_name")
        .ok_or("missing soft_name")?
        .to_string();
    header.software_version = line.get_value("soft_ver").map(|s| s.to_string());
    header.report_id = line.get_value("report_id").map(|s| s.to_string());
    header.report_number = line.get_value("report_number").map(|s| s.to_string());
    Ok(HeaderParsing { header, sep })
}

///Get the next line, return an error if we can't.
fn next_line(
    read_bytes: &mut Vec<u8>,
    lines: &mut Lines<impl Read>,
) -> Result<Vec<u8>, &'static str> {
    let line = match lines.next() {
        None => return Err("unexpected end of file"),
        Some(Ok(line)) => line,
        Some(Err(_e)) => return Err("error reading line"),
    };
    if read_bytes.len() > 0 {
        read_bytes.push(b'\n');
    }
    read_bytes.extend_from_slice(&line);
    Ok(line)
}

fn byte_slice_contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn byte_slice_to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}
