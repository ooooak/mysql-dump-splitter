#[macro_use] 
extern crate clap;
mod parser;
mod splitter;
mod reader;
mod cli;
mod tokenizer;

use std::str;
use std::process;
use std::fs::File;
use std::io::prelude::*;
use splitter::SplitterSettings;
use splitter::SplitterState;
use splitter::Splitter;

fn log_error(err: &str) -> ! {
    eprintln!("{}", err);
    process::exit(0)
}

fn create_file(name: usize) -> File {
    let file_name = format!("./{:?}.sql", name);
    File::create(file_name).unwrap()
}


fn main(){
    let (file, write_buffer) = cli::args();

    let file = match file {
        Ok(file) => file,
        Err(e) => log_error(e),
    };

    let write_buffer = match write_buffer {
        Ok(file) => file,
        Err(e) => log_error(e.as_str()),
    };

    let mut splitter = Splitter::new(SplitterSettings {
        read: 10 * 1024 * 1024,
        write: write_buffer,
        file: file,
    });

    let mut file_count = 1;
    let mut buffer = create_file(file_count);

    loop {
        match splitter.process() {
            SplitterState::Chunk(file_state, tokens) => {
                if *file_state == splitter::FileState::New {
                    file_count += 1;
                    buffer = create_file(file_count);                    
                }

                buffer.write_all(&tokens).unwrap();
            },
            SplitterState::SyntaxErr(e) => log_error(e.text),
            SplitterState::Done => break,
        }
    }
}
