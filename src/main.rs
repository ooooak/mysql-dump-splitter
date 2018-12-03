mod parser;
mod splitter;
mod reader;
mod cli;
mod tokenizer;
mod helper;

use std::fs::File;
use std::io::Result;
use std::str;
// use std::env;
use std::process;



use helper::write;
use splitter::SplitterSettings;
use splitter::SplitterState;
use splitter::Splitter;
use parser::Parser;
use reader::Reader;
use tokenizer::Tokenizer;

fn die(text : &str) -> ! {
    println!("{}", text);
    process::exit(0);
}


// Unsafe 
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

    let mut splitter = Splitter::new(SplitterSettings {
        read: 1 * 1024 * 1024,
        write: 1 * 1024 * 1024,
        file: file,
    });

    let mut chunk_count = 0;
    loop {
        match splitter.process() {
            SplitterState::Chunk(tokens) => {
                println!("{:?}", tokens.last().unwrap());
                chunk_count += 1;
                let file_name = format!("./example-files/output/{:?}.sql", chunk_count);
                write(file_name, tokens);
            },
            SplitterState::Done(tokens) => {
                chunk_count += 1;
                let file_name = format!("./example-files/output/{:?}.sql", chunk_count);
                write(file_name, tokens);
                break
            },
            SplitterState::Continue => {
                continue
            },
        }
    }

    Ok(())
}
