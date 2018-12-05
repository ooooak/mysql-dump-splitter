use tokenizer::Token;
use std::fs::File;
use std::io::prelude::*;

pub fn write(file_name: String, tokens: Vec<Token>){
    let mut output = vec![];
    for token in tokens {
        token.value(&mut output)
    }
    let mut buffer = File::create(file_name).unwrap();
    buffer.write_all(&output).unwrap();
}