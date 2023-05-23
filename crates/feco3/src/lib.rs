use std::fs::File;

pub mod form;
pub mod header;
pub mod parser;
pub mod writers;

pub fn add_42(a: i32) -> i32 {
    a + 42
}

pub fn print_header(path: &str) {
    let file = File::open(path).unwrap();
    let mut parser = parser::Parser::from_reader(file);
    let header = parser.parse_header().unwrap();
    println!("Header: {:?}", header);
}
