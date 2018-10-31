mod scanner;
mod sql_writer;
use std::fs::File;
use std::io::Result;
use std::io::prelude::*;



fn main() -> Result<()> {

    let file_name = "C:\\Users\\Akshay Goswami\\Desktop\\test.txt";
    let file = File::open(file_name).unwrap();
    let mut s = scanner::Scanner::new(file);
    let max_output_buff = 10 * 1024 * 1024; // 10mb
    let mut sql_writer = sql_writer::SQLWriter::new(max_output_buff);


    loop{

        if !sql_writer.process(s.token()) {
            break;
        }
    }
    
    Ok(())
}
