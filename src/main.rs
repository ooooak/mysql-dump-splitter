mod parser;
mod splitter;
mod reader;
mod cli;
mod tokenizer;
mod helper;

use std::fs::File;
use std::io::Result;
// use std::io::prelude::*;
use std::str;
// use std::env;
use std::process;


use helper::dump;
use splitter::Splitter;
use splitter::SplitterState;
use parser::Parser;
// use parser::TokenStream;
use reader::Reader;
use tokenizer::Tokenizer;


fn die(text : &str) -> ! {
    println!("{}", text);
    process::exit(0);
}



fn get_file() -> File{
    match cli::file_path() {
        Ok(path) => {
            match File::open(path) {
                Ok(fd) => fd,
                Err(e) => {
                    println!("{:?}", e);
                    process::exit(0)
                },
            }
        },
        Err(e) => die(e),
    }
}



fn main() -> Result<()> {
    let file = get_file();
    let read_buffer: usize = 1 * 1024 * 1024; // 1mb
    let tokenizer = Tokenizer::new(Reader::new(file, read_buffer));
    let mut parser = Parser::new(tokenizer);
    let max_output_buff = 10 * 1024 * 1024; // 10mb
    let mut splitter = Splitter::new(max_output_buff);

    loop {
        match splitter.process(parser.token_stream()) {
            SplitterState::Chunk(tokens) => {
                dump(&tokens);
                break;
            },
            SplitterState::Done(tokens) => {
                dump(&tokens);
                break;
            },
            SplitterState::Continue => continue,
        }
    }

    Ok(())
}
