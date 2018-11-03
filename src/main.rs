mod scanner;
mod sql_writer;
mod reader;

use std::fs::File;
use std::io::Result;
use std::io::prelude::*;
use std::str;

fn main() -> Result<()> {
    let file = File::open("C:\\Users\\Akshay Goswami\\Desktop\\test.txt").unwrap();
    let reader = reader::Reader::new(file);
    let mut s = scanner::Scanner::new(reader);
    let max_output_buff = 10 * 1024 * 1024; // 10mb
    let mut sql_writer = sql_writer::SQLWriter::new(max_output_buff);


    loop{
        match s.token() {
            Some(item) => {
                match item {
                    scanner::Token::Eat(i) => {
                        println!("Eat: {:?}", str::from_utf8(&i))
                    },
                    scanner::Token::InsertValues(i) => {
                        println!("InsertValues: {:?}", str::from_utf8(&i))
                    },
                    scanner::Token::Insert(i) => {
                        println!("InsertValues: {:?}", str::from_utf8(&i))
                    },
                }
            },
            None => {
                break;
            },
        }

        // if !sql_writer.process(s.token()) {
        //     break;
        // }
    }
    
    Ok(())
}
