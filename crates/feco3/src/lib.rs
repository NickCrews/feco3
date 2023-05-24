use std::fs::File;

use crate::writers::base::Writer;

pub mod form;
pub mod header;
pub mod parser;
pub mod writers;

pub fn add_42(a: i32) -> i32 {
    a + 42
}

pub fn print_header(path: &str) {
    // TODO Figure out how to reconfigure this, since currently
    // it only configures it on the first call and then never again.
    env_logger::try_init();
    let file = File::open(path).unwrap();
    let mut parser = parser::Parser::from_reader(file);
    let header = parser.parse_header().unwrap();
    let base_out_path = std::path::PathBuf::from("out");
    let mut writer = writers::csv::CSVFileWriter::new(base_out_path);
    println!("Header: {:?}", header);
    while let Some(line) = parser.next_line().unwrap() {
        println!("Line: {:?}", line);
        writer.write_form_line(&line.unwrap()).unwrap();
    }
}
