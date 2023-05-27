//! FECo3 is a library for parsing .fec files from the US Federal Election Commission.
//!
//! .fec files are binary files that contain campaign finance data.
//! This library provides an efficient, flexible, stream-oriented parser
//! for these files.
//!
//! The parser takes a byte stream as input, which makes it flexible enough
//! to parse files from a variety of sources, including local files, HTTP
//! streams, or a custom source.
//!
//! FECo3 includes a framework for writing the parsed data. Currently,
//! the only supported output format is CSV, but the framework is designed
//! to be extensible to other formats.
//!
//! There are bindings for python available
//! [on the repo](https://github.com/NickCrews/feco3).

use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

mod csv;
pub mod fec;
pub mod header;
pub mod line;
mod schemas;
mod summary;
pub mod writers;

#[macro_use]
extern crate lazy_static;

pub fn parse_from_path(fec_path: &PathBuf, out_dir: PathBuf) -> Result<(), Box<dyn Error>> {
    let file = File::open(fec_path)?;
    let mut fec = fec::FecFile::from_reader(file);
    println!("header: {:?}", fec.get_header()?);
    println!("summary: {:?}", fec.get_summary()?);
    let mut writer = writers::csv::CSVMultiFileWriter::new(out_dir);
    while let Some(line) = fec.next_line()? {
        writers::base::LineWriter::write_line(&mut writer, &line?)?;
    }
    Ok(())
}
