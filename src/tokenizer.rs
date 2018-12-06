use std::str;
use reader::Reader;
use std::io;

#[derive(Debug,PartialEq,Clone)]
pub enum Token{
    String(Vec<u8>),
    Keyword(Vec<u8>),
    Comment(Vec<u8>),
    InlineComment(Vec<u8>),
    // Any thing that starts with `
    Identifier(Vec<u8>),

    // could be /t or /n /r
    LineFeed(u8),
    Space,
    Comma,
    LP, 
    RP,
    SemiColon,
    Ignore(u8),
    Dot,
}

impl Token {
    pub fn keyword(&self, string: &str) -> bool {
        match self {
            Token::Keyword(chunk) => {
                let value = str::from_utf8(&chunk).unwrap();
                value.to_lowercase() == string
            },
            _ => false,
        }
    }

    pub fn value(&self, collection: &mut Vec<u8>) {
        match self {
            Token::String(chunk) => collection.extend(chunk),
            Token::Keyword(chunk) => collection.extend(chunk),
            Token::Comment(chunk) => collection.extend(chunk),
            Token::InlineComment(chunk) => collection.extend(chunk),
            Token::Identifier(chunk) => collection.extend(chunk),
            Token::Ignore(byte) => collection.push(*byte),
            Token::Comma => collection.push(b','),
            Token::LP => collection.push(b'('),
            Token::RP => collection.push(b')'),
            Token::SemiColon => collection.push(b';'),
            Token::Dot => collection.push(b'.'),
            Token::Space => collection.push(b' '),
            Token::LineFeed(byte) => collection.push(*byte), 
        }        
    }

    pub fn len(&self) -> usize {
        match self {
            Token::Keyword(chunk) | 
            Token::Comment(chunk) |
            Token::String(chunk) |
            Token::InlineComment(chunk) |
            Token::Identifier(chunk) => chunk.len(),

            Token::Dot |
            Token::Space |
            Token::LineFeed(_) | 
            Token::Ignore(_) |
            Token::LP | 
            Token::RP |
            Token::SemiColon |
            Token::Comma => 1,
        }
    }

    #[allow(dead_code)]
    pub fn log(&self) {
        match self {
            Token::String(chunk) => self.log_bytes("String",  chunk),
            Token::Keyword(chunk) => self.log_bytes("Keyword",  chunk),
            Token::Comment(chunk) => self.log_bytes("Comment",  chunk),
            Token::InlineComment(chunk) => self.log_bytes("InlineComment",  chunk),
            Token::Identifier(chunk) => self.log_bytes("Identifier",  chunk),
            Token::Ignore(byte) => self.log_byte("Ignore", byte.clone()),
            Token::Comma => self.log_byte("Comma", b','),
            Token::LP => self.log_byte("LP", b'('),
            Token::RP => self.log_byte("RP", b')'),
            Token::SemiColon => self.log_byte("SemiColon", b';'),
            Token::LineFeed(byte) => self.log_byte("SemiColon", byte.clone()),
            Token::Dot => self.log_byte("SemiColon", b'.'),
            Token::Space => self.log_byte("SemiColon", b' '),
        }
    }

    fn log_bytes(&self, index: &str, bytes: &Vec<u8>){
        println!("{}: {:?}", index, str::from_utf8(&bytes).unwrap())
    }

    #[allow(dead_code)]
    fn log_byte(&self, index: &str, byte: u8){
        let vc = vec![byte];
        self.log_bytes(index, &vc);
    }
}


pub struct Tokenizer<T>{
    reader: Reader<T>,
    // line_num: usize,
}

#[derive(Debug)]
pub struct SyntaxErr{
    // line_num: usize,
    pub text: &'static str
}

impl<T> Tokenizer<T> where T: io::Read {
    pub fn new(reader: Reader<T>) -> Self {
        Self {
            reader: reader,
            // line_num: 0
        }
    }

    fn read_till(&mut self, item: u8) -> Result<Vec<u8>, SyntaxErr> {
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
                    return Err(SyntaxErr{
                        text: "Unexpected end of the file."
                    })
                }
            }
        }
        
        Ok(collection)
    }

    fn keyword(&mut self) -> Result<Vec<u8>, SyntaxErr> {
        let mut collection = vec![];
        loop {
            let byte = self.reader.peek();
            match byte {
                Some(item)  => {
                    match item {
                        b'a'...b'z' |
                        b'A'...b'Z' => {
                            self.reader.increment_index();
                            collection.push(item);
                        },
                        _ => break,
                    }
                },
                None => {
                    return Err(SyntaxErr{
                        text:"While parsing keyword."
                    })
                }
            }
        }
        
        Ok(collection)
    }


    fn number(&mut self) -> Token {
        let mut collection = vec![];
        while let Some(byte @ b'0'...b'9') = self.reader.peek() {
            self.reader.increment_index();
            collection.push(byte);
        }
        Token::String(collection)
    }

    fn read_string(&mut self, closing: u8) -> Result<Token, SyntaxErr> {
        let mut collection = vec![];
        let mut last_byte = self.reader.get().unwrap();
        collection.push(last_byte);

        loop {
            let byte = self.reader.get();
            if let Some(item) = byte {
                collection.push(item);
                if item == closing && last_byte != b'\\' {
                    break;
                }
                last_byte = item;
            }else{
                return Err(SyntaxErr{
                    text: "Unclosed string."
                })
            }
        }

        Ok(Token::String(collection))
    }

    fn singular(&mut self, token: Token) -> Result<Option<Token>, SyntaxErr> {
        self.reader.increment_index();
        Ok(Some(token))
    }
    
    pub fn token(&mut self) -> Result<Option<Token>, SyntaxErr> {
        match self.reader.peek() {
            Some(closing @ b'"') |
            Some(closing @ b'\'') => {
                match self.read_string(closing) {
                    Ok(value) => {
                        Ok(Some(value))
                    },
                    Err(err) => {
                        Err(err)
                    },
                }
            },
            Some(byte @ b'/') => {
                if self.reader.peek_next() == Some(b'*') {
                    self.comment()
                }else{
                    self.reader.increment_index();
                    Ok(Some(Token::Ignore(byte)))
                }
            },
            Some(b'0'...b'9') => Ok(Some(self.number())),
            Some(byte @ b'-') => {
                if self.reader.peek_next() == Some(b'-') {
                    match self.read_till(b'\n') {
                        Ok(item) => {
                            Ok(Some(Token::InlineComment(item)))
                        },
                        Err(err) => Err(err),
                    }
                    
                }else{
                    self.reader.increment_index();
                    Ok(Some(Token::Ignore(byte)))
                }
            },
            Some(b'a'...b'z') | 
            Some(b'A'...b'Z') => {
                match self.keyword() {
                    Ok(value) => {
                        Ok(Some(Token::Keyword(value)))
                    },
                    Err(e) => Err(e),
                }                
            },
            Some(byte @ b'`') => {
                self.reader.increment_index(); // skip `
                match self.read_till(b'`') {
                    Ok(value) => {
                        let mut identifier = vec![byte];
                        identifier.extend(value);
                        Ok(Some(Token::Identifier(identifier)))
                    },
                    Err(err) => Err(err),
                }
            },
            Some(b'.') => self.singular(Token::Dot),
            Some(b'(') => self.singular(Token::LP),
            Some(b')') => self.singular(Token::RP),
            Some(b';') => self.singular(Token::SemiColon),
            Some(b',') => self.singular(Token::Comma),
            Some(b' ') => self.singular(Token::Space),
            Some(byte @ b'\r') |  
            Some(byte @ b'\t') | 
            Some(byte @ b'\n') => self.singular(Token::LineFeed(byte)),
            Some(byte) => self.singular(Token::Ignore(byte)),
            None => Ok(None),
        }
    }

    fn comment(&mut self) -> Result<Option<Token>, SyntaxErr> {
        let mut collection = vec![];
        loop {
            let cr = self.reader.get();
            // eof
            if cr.is_none() {
                return Err(SyntaxErr{
                    text: "Incomplete multi-line comment."
                });
            }
            
            collection.push(cr.unwrap());
            if cr == Some(b'*') && self.reader.peek() == Some(b'/') {
                let get_peeked = self.reader.get();
                collection.push(get_peeked.unwrap());
                break
            }
        }
        Ok(Some(Token::Comment(collection)))
    }
}


#[cfg(test)]
mod reader_test{
    // use std::fs::File;
    // use Reader;
    // use super::Tokenizer;

    // #[test]
    // fn tokenizer(){
    //     let read_buffer: usize = 1 * 1024 * 1024; // 1mb 
    //     let file = File::open("./example-files/create-table-with-comments.txt").unwrap();
    //     let reader = Reader::new(file, read_buffer);
    //     let mut tk = Tokenizer::new(reader);
    //     loop {
    //         match tk.token() {
    //             Some(item) => {
    //                 println!("{:?}", item);
    //             },
    //             None => break,
    //         }
    //     }
    // }
}