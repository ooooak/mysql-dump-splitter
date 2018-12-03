use std::process;
use tokenizer::Token;
use std::fs::File;
use std::io::prelude::*;


fn sleep(sec: u64){
    println!("sleep");
    use std::{thread, time};
    
    let from_millis = time::Duration::from_millis(sec * 1000);
    // let now = time::Instant::now();
    thread::sleep(from_millis);
}


pub fn die(text : &str) -> ! {
    println!("{}", text);
    process::exit(0);
}

pub fn write(file_name: String, tokens: Vec<Token>){
    let mut output = vec![];
    for token in tokens {
        token.value(&mut output)
    }
    let mut buffer = File::create(file_name).unwrap();
    buffer.write_all(&output).unwrap();
}