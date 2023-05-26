use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

use crate::writers::base::Writer;

pub mod fec;
pub mod header;
pub mod line;
mod schemas;
mod summary;
pub mod writers;

#[macro_use]
extern crate lazy_static;

pub fn parse_from_path(fec_path: &PathBuf, out_dir: PathBuf) -> Result<(), Box<dyn Error>> {
    // TODO Figure out how to reconfigure this, since currently
    // it only configures it on the first call and then never again.
    let file = File::open(fec_path)?;
    let mut parser = fec::FecFile::from_reader(file);
    let mut writer = writers::csv::CSVFileWriter::new(out_dir);
    while let Some(line) = parser.next_line()? {
        writer.write_form_line(&line?)?;
    }
    Ok(())
}
