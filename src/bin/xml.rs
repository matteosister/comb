extern crate comb;

use std::fs::File;
use std::io::Read;

use comb::{*, xml::*};

pub fn main() {
    let mut file_content = String::new();
    File::open("src/bin/test.xml").unwrap().read_to_string(&mut file_content).expect("File not found");
    let parsed = element().parse(file_content.as_str()).expect("parser error");
    println!("Document parsed!");
    println!("Parsed: {:?}", parsed.1);
    println!("Remains: {}", if parsed.0 == "" { "-" } else { parsed.0 })
}