#[macro_use] extern crate clap;

mod parser;
mod splitter;
mod reader;
mod cli;
mod tokenizer;
mod helper;

use std::str;
use std::process;use helper::write;
use splitter::SplitterSettings;
use splitter::SplitterState;
use splitter::Splitter;

fn log_error(err: &str) -> ! {
    eprintln!("{}", err);
    process::exit(0)
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

    let mut file_count = 0;
    loop {
        match splitter.process() {
            SplitterState::SyntaxErr(e) => {
                log_error(e.text)
            },
            SplitterState::Chunk(tokens) => {
                file_count += 1;
                let file_name = format!("./{:?}.sql", file_count);
                write(file_name, tokens);
            },
            SplitterState::Done(tokens) => {
                file_count += 1;
                let file_name = format!("./{:?}.sql", file_count);
                write(file_name, tokens);
                break
            },
            SplitterState::Continue => {
                continue
            },
        }
    }
}
