use std::process;
use std::fs::File;
use std::io::{BufReader, Result};
use std::io::prelude::*;
use std::str;
use reader::Reader;

pub struct Scanner{
    reader: Reader,
    in_insert: bool,
}

fn die(text : &str) -> ! {
    println!("{}", text);
    process::exit(0);
}

#[derive(Debug)]
pub enum Token {
    Eat(Vec<u8>),
    Insert(Vec<u8>),
    InsertValues(Vec<u8>),
}

impl Scanner{
    pub fn new(reader: Reader) -> Self {
        Self {
            reader: reader,
            in_insert: false,
        }
    }

    fn read_till(&mut self, item: u8) -> Vec<u8> {
        let mut collection = vec![];

        loop {
            let byte = self.reader.get();
            match byte {
                Some(value) => {
                    collection.push(value);
                    if value == item {
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
            let byte = self.reader.peek();
            match byte {
                Some(value) => {
                    if value == item {
                        break;
                    }
                    collection.push(value);
                    // skip the index
                    self.reader.get();
                },
                None => {
                    die("Unexpected end of the file.");
                }
            }
        }
        
        collection
    }

    

    fn keyword(&mut self) -> Vec<u8> {
        self.read_until(b' ')
    }

    fn read_string(&mut self) -> Vec<u8> {
        let mut collection = vec![];
        let mut last_byte = self.reader.get();
        collection.push(last_byte.unwrap());

        loop {
            let byte = self.reader.get();
            if let Some(item) = byte {
                collection.push(item);
                if byte == Some(b'\'')  && last_byte != Some(b'\\') {
                    // none escaped string 
                    break;
                }

                last_byte = byte;
            }else{
                die("Error: Unable to close the string")
            }
        }

        collection
    }

    fn values_statement(&mut self) -> Option<Token> {
        let mut collection = vec![];
        let mut closed = false;

        loop {
            let byte = self.reader.peek();
            match byte {
                Some(b'\'' ) => {
                    let string = self.read_string();
                    collection.extend(string);
                },
                Some(byte) => {
                    self.reader.increment_index();
                    collection.push(byte);
                    if  byte == b';' {
                            self.in_insert = false;    
                            break;    
                    }else if byte == b')' {
                        // values statements are closed by tow items `)` and `,`.
                        closed = true;
                    }else if byte == b','{
                        // `,` could be inside values, so we need to make sure 
                        // we are at end of the tuple
                        if closed == true {
                            break;
                        }
                    }
                }
                None => {
                    die("Error: unfinished values stream.");
                },
            }
        }

        Some(Token::InsertValues(collection))
    }

    /**
     * we should have built a proper parser
     * case one: 
     *  INSERT INTO distributors (`id`, `name`, `VALUES`)
     *         VALUES (1, 'str', 'none');
     * 
     * case two: 
     *  INSERT INTO `distributors` VALUES (1, 'str');
     * 
     * **/ 
    fn insert_statement(&mut self) -> Vec<u8> {
        // read till we value as key word 
        // we will have empty
        let mut keyword_count = 0;
        let mut collection = vec![];

        loop {
            if keyword_count == 2 {
                match self.reader.peek() {
                    Some(item) => {
                        match item {
                            b'(' => {
                                let tuple = self.read_till(b')');
                                collection.extend(tuple);
                                die("1");
                                break ;
                            },
                            b'A'...b'Z' |
                            b'a'...b'z' => {
                               break;
                            }, 
                            _ => {
                                collection.push(item);
                                self.reader.increment_index();
                                continue;
                            }
                        }
                    },
                    None => {
                        println!("{:?}", &collection);
                        die("Error: Unexpected end of the file")                      
                    },
                }
            }

            match self.reader.peek() {
                Some(item) => {
                    match item {
                        b'a'...b'z' |
                        b'A'...b'Z' => {
                            keyword_count += 1;
                            let keyword = self.keyword();
                            collection.extend(keyword);                            
                        },
                        _  => {
                            collection.push(item);
                            self.reader.increment_index();
                        },
                    }
                },
                None => {
                    println!("{:?}", &collection);
                    die("Error: Unexpected end of the file")
                },
            }
        }

        collection
    }


    pub fn token(&mut self) -> Option<Token> {
        let b = self.reader.peek();
        if b.is_none() {
            return None
        }

        if self.in_insert {
            return self.values_statement()
        }
        
        let token = match b.unwrap() {
            b'/' => {
                if self.reader.peek_next() == Some(b'*') {
                    self.multi_line_comment()
                }else{
                    self.parse_block()
                }
            },
            b'-' => {
                if self.reader.peek_next() == Some(b'-') {
                    self.parse_inline_comment()
                }else{
                    self.parse_block()
                }
            },
            b'a'...b'z' |
            b'A'...b'Z' => {
                let mut keyword = self.keyword();
                let str_val = str::from_utf8(&keyword).unwrap();
                let mut insert_statement = vec![];
                insert_statement.extend(&keyword);

                match str_val.to_lowercase().as_ref() {
                    "insert" => {
                        self.in_insert = true;
                        let statement = self.insert_statement();
                        insert_statement.extend(statement);
                        Token::Insert(insert_statement)
                    },
                    _ => {
                        let items = self.read_till(b';');

                        insert_statement.extend(&items);
                        Token::Eat(insert_statement)
                    },
                }
            },

            b' ' | b'\n' | b'\r' => {
                let mut collection = vec![];
                loop {
                    match self.reader.peek() {
                        byte @ Some(b' ') | 
                        byte @ Some(b'\n') | 
                        byte @ Some(b'\r') => {
                            collection.push(byte.unwrap());
                            // TODO: we can just up the index
                            self.reader.increment_index();
                        }
                        _ => {
                            break
                        }
                    }
                }
                
                Token::Eat(collection)
            }
            _ => {
                self.parse_block()
            },
        };

        Some(token)
    }

    fn parse_inline_comment(&mut self) -> Token {
         let mut collection = vec![];

        loop {
            let byte = self.reader.get();
            match byte {
                Some(value) => {
                    collection.push(value);
                    if value == b'\n'{
                        break;
                    }
                },
                None => {
                    break
                }
            }
        }
        
        Token::Eat(collection)
    }

    fn parse_block(&mut self) -> Token {
        Token::Eat(self.read_till(b';'))
    }

    fn multi_line_comment(&mut self) -> Token {
        let mut collection = vec![];
        loop {
            let cr = self.reader.get();
            // eof
            if cr.is_none() {
                die("Error: Incomplete multi line comment .")
            }

            collection.push(cr.unwrap());
            // end of the comment 
            if cr == Some(b'*') && self.reader.peek() == Some(b'/') {
                let get_peeked = self.reader.get();
                collection.push(get_peeked.unwrap());
                break
            }
        }

        return Token::Eat(collection)
    }
}