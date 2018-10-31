use std::process;
use std::fs::File;
use std::io::{BufReader, Result};
use std::io::prelude::*;
use std::str;



pub struct Scanner{
    buffer: Vec<u8>,
    index: usize,
    reader: BufReader<File>,
    in_insert: bool,
    bytes_read: usize,
}

fn die(text : &str) -> ! {
    println!("{}", text);
    process::exit(0);
}

#[derive(Debug)]
pub enum Token {
    Comment(Vec<u8>),
    CommentInline(Vec<u8>),
    Insert(Vec<u8>),
    InsertValues(Vec<u8>),
    Block(Vec<u8>),
    SpacesOrLineFeeds(Vec<u8>)
}

impl Scanner{
    pub fn new(file: File) -> Scanner {
        let size: usize = 100 * 1024 * 1024; // 100mb 

        // reader
        let mut scanner = Scanner { 
            buffer: vec![0; size],
            reader: BufReader::new(file),
            in_insert: false,
            index: 0,
            bytes_read: 0,
        };

        scanner.read_buf();
        scanner
    }

    #[inline(always)]
    fn get(&mut self) -> Option<u8> {
        if self.bytes_read == 0 {
            // no buffer data
            None
        }else{
            if let Some(item) = self.buffer.get(self.index) {
                return if *item == 0 as u8{
                    None
                }else{
                    Some(item.clone())
                }
            }
            // ran out of data load next buffer data
            self.read_buf();

            
            // try to read the value from index
            // rec call
            self.get()
        }
    }

    fn next(&mut self) -> Option<u8> {
        let byte = self.get();
        self.index += 1;
        byte
    }

    #[inline(always)]
    fn peek(&mut self) -> Option<u8> {
        self.get()
    }

    #[inline(always)]
    fn peek_next(&mut self) -> Option<u8> {
        self.index += 1;
        let item = self.get();
        self.index -= 1;
        item
    }


    fn read_buf(&mut self) {
        match self.reader.read(&mut self.buffer) {
            Ok(size) => {
                self.bytes_read = size;
                self.index = 0; // reset index
            },
            Err(_) => {
                die("Unable to read buffer")
            }
        }
    }

    
    fn multi_line_comment(&mut self) -> Token {
        let mut collection = vec![];
        loop {
            let cr = self.next();
            // eof
            if cr.is_none() {
                die("Unexpected end of the file.")
            }

            collection.push(cr.unwrap());
            // end of the comment 
            if cr == Some(b'*') && self.peek() == Some(b'/') {
                let get_peeked = self.next();
                collection.push(get_peeked.unwrap());
                break
            }
        }

        return Token::Comment(collection)
    }

    fn read_till(&mut self, item: u8) -> Vec<u8> {
        let mut collection = vec![];

        loop {
            let byte = self.next();
            match byte {
                Some(value) => {
                    collection.push(value);
                    if value == item{
                        break;
                    }
                },
                None => {
                    die("Unexpected end of the file.");
                }       
            }
        }
        
        collection
    }


    fn read_until(&mut self, item: u8) -> Vec<u8> {
        let mut collection = vec![];

        loop {
            let byte = self.peek();
            match byte {
                Some(value) => {
                    if value == item {
                        break;
                    }

                    collection.push(value);
                    // skip the index
                    self.next();
                },
                None => {
                    die("Unexpected end of the file.");
                }       
            }
        }
        
        collection
    }

    fn parse_inline_comment(&mut self) -> Token {
         let mut collection = vec![];

        loop {
            let byte = self.next();
            match byte {
                Some(value) => {
                    collection.push(value);
                    if value == b'\n'{
                        break;
                    }
                },
                None => { break }       
            }
        }
        
        Token::CommentInline(collection)
    }

    fn parse_block(&mut self) -> Token {
        Token::Block(self.read_till(b';'))
    }

    fn keyword(&mut self) -> Vec<u8> {
        self.read_until(b' ')
    }

    fn read_string(&mut self, byte: u8) -> Vec<u8> {
        let mut collection = vec![];
        let mut last_byte = byte;
        collection.push(byte);

        loop {
            let byte = self.next();
            collection.push(byte.unwrap());
            if last_byte != b'\\' && byte == Some(b'\n'){                
                break;
            }

            last_byte = byte.unwrap();
        }

        collection
    }

    fn values_statement(&mut self) -> Option<Token> {
        let mut collection = vec![];
        let mut closed = false;

        loop {
            let byte = self.next();
            if byte.is_none(){
                die("Unexpected end of the file.");
            }

            if byte == Some(b'\''){
                let string = self.read_string(byte.unwrap());
                collection.extend(string);
                continue;
            }

            collection.push(byte.unwrap());
            match byte.unwrap() {
                b';' => {
                    self.in_insert = false;    
                    break;    
                },
                b')' => {
                    closed = true;
                },
                b',' => {
                    if closed == true {
                        break;
                    }
                },
                _ => {
                    continue
                }
            }
        }

        Some(Token::InsertValues(collection))
    }


    pub fn token(&mut self) -> Option<Token> {
        let b = self.peek();
        if b.is_none() {
            return None
        }

        if self.in_insert == true{
            return self.values_statement()
        }
        
        let token = match b.unwrap() {
            b'/' => {
                if self.peek_next() == Some(b'*') {
                    self.multi_line_comment()
                }else{
                    self.parse_block()
                }
            },
            b'-' => {
                if self.peek_next() == Some(b'-') {
                    self.parse_inline_comment()
                }else{
                    self.parse_block()
                }
            },
            b'a'...b'z' | 
            b'A'...b'Z' => {
                let mut keyword = self.keyword();
                let mut insert_statement = keyword.clone();
                let str_val = str::from_utf8(&keyword).unwrap();

                match str_val.to_lowercase().as_ref() {
                    "insert" => {
                        let items = self.read_till(b')');
                        insert_statement.extend(items);
                        self.in_insert = true;

                        Token::Insert(insert_statement)
                    },
                    _ => {
                        let items = self.read_till(b';');
                        insert_statement.extend(items);
                        Token::Block(insert_statement)
                    },
                }
            },

            b' ' | b'\n' | b'\r' => {
                let mut collection = vec![];
                loop {
                    match self.peek() {
                        byte @ Some(b' ') | 
                        byte @ Some(b'\n') | 
                        byte @ Some(b'\r') => {
                            collection.push(byte.unwrap());
                            // TODO: we can just up the index
                            self.next();
                        }
                        _ => {
                            break
                        }
                    }
                }
                
                Token::SpacesOrLineFeeds(collection)
            }
            _ => {
                self.parse_block()
            },
        };

        Some(token)
    }
}