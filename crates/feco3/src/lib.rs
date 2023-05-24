use std::fs::File;
use std::path::PathBuf;

use crate::writers::base::Writer;

pub mod form;
pub mod header;
pub mod parser;
pub mod writers;

pub fn parse_from_path(fec_path: &PathBuf, out_dir: PathBuf) {
    // TODO Figure out how to reconfigure this, since currently
    // it only configures it on the first call and then never again.
    env_logger::try_init();
    let file = File::open(fec_path).unwrap();
    let mut parser = parser::Parser::from_reader(file);
    let header = parser.parse_header().unwrap();

    let mut writer = writers::csv::CSVFileWriter::new(out_dir);
    println!("Header: {:?}", header);
    while let Some(line) = parser.next_line().unwrap() {
        println!("Line: {:?}", line);
        writer.write_form_line(&line.unwrap()).unwrap();
    }
}
