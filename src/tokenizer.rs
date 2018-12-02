use std::str;
use reader::Reader;
use std::io;
use helper::die;

#[derive(Debug,PartialEq,Clone)]
pub enum Token{
    String(Vec<u8>),
    Keyword(Vec<u8>),
    Block(Vec<u8>),
    Comment(Vec<u8>),
    InlineComment(Vec<u8>),
    // Any thing that starts with `
    Identifier(Vec<u8>),  
    Ignore(u8),
    Comma,
    LP, 
    RP,
    SemiColon,
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
            Token::Block(chunk) => collection.extend(chunk),
            Token::Comment(chunk) => collection.extend(chunk),
            Token::InlineComment(chunk) => collection.extend(chunk),
            Token::Identifier(chunk) => collection.extend(chunk),
            Token::Ignore(byte) => collection.push(*byte),
            Token::Comma => collection.push(b','),
            Token::LP => collection.push(b'('),
            Token::RP => collection.push(b')'),
            Token::SemiColon => collection.push(b';'),
        }        
    }
    
    fn log_bytes(&self, index: &str, bytes: &Vec<u8>){
        println!("{}: {:?}", index, str::from_utf8(&bytes).unwrap())
    }

    fn log_byte(&self, index: &str, byte: u8){
        let vc = vec![byte];
        self.log_bytes(index, &vc);
    }

    pub fn len(&self) -> usize {
        match self {
            Token::Keyword(chunk) | 
            Token::Block(chunk) | 
            Token::Comment(chunk) |
            Token::String(chunk) |
            Token::InlineComment(chunk) |
            Token::Identifier(chunk) => chunk.len(),

            Token::Ignore(_) |
            Token::LP | 
            Token::RP |
            Token::SemiColon |
            Token::Comma => 1,
        }
    }

    pub fn log(&self) {
        match self {
            Token::String(chunk) => self.log_bytes("String",  chunk),
            Token::Keyword(chunk) => self.log_bytes("Keyword",  chunk),
            Token::Block(chunk) => self.log_bytes("Block",  chunk),
            Token::Comment(chunk) => self.log_bytes("Comment",  chunk),
            Token::InlineComment(chunk) => self.log_bytes("InlineComment",  chunk),
            Token::Identifier(chunk) => self.log_bytes("Identifier",  chunk),
            Token::Ignore(byte) => self.log_byte("Ignore", byte.clone()),
            Token::Comma => self.log_byte("Comma", b','),
            Token::LP => self.log_byte("LP", b'('),
            Token::RP => self.log_byte("RP", b')'),
            Token::SemiColon => self.log_byte("SemiColon", b';'),
        }
    }
}


pub struct Tokenizer<T>{
    reader: Reader<T>,
}

impl<T> Tokenizer<T> where T: io::Read {
    pub fn new(reader: Reader<T>) -> Self {
        Self {
            reader: reader,
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
                    die("Error: Unexpected end of the file.");
                }
            }
        }
        
        collection
    }

    fn keyword(&mut self) -> Vec<u8> {
        let mut collection = vec![];
        loop {
            let byte = self.reader.peek();
            match byte {
                Some(item)  => {
                    match item{
                        b'a'...b'z' |
                        b'A'...b'Z' => {
                            self.reader.increment_index();
                            collection.push(item);
                        },
                        _ => break,
                    }
                },
                None => {
                    die("Error: While parsing keyword.");
                }
            }
        }
        
        collection
    }

    fn read_string(&mut self, closing: u8) -> Token{
        let mut collection = vec![];
        let mut last_byte = self.reader.get().unwrap();
        collection.push(last_byte);

        loop {
            let byte = self.reader.get();
            if let Some(item) = byte {
                collection.push(item);
                if item == closing && last_byte != b'\\' {
                    // none escaped string 
                    break;
                }
                last_byte = item;
            }else{
                die("Error: Unclosed string.")
            }
        }

        Token::String(collection)
    }

    fn block(&mut self) -> Token{
        Token::Block(self.read_till(b';'))
    }

    pub fn token(&mut self) -> Option<Token> {
        let b = self.reader.peek();
        if b.is_none() {
            return None
        }

        let token = match b.unwrap() {
            closing @ b'"' | 
            closing @ b'\'' => self.read_string(closing),
            b'/' => {
                if self.reader.peek_next() == Some(b'*') {
                    self.comment()
                }else{
                    self.block()
                }
            },
            b'-' => {
                if self.reader.peek_next() == Some(b'-') {
                    Token::InlineComment(self.read_till(b'\n'))
                }else{
                    Token::Block(self.read_till(b';'))
                }
            },
            b'a'...b'z' | b'A'...b'Z' => {
                Token::Keyword(self.keyword())
            },
            b'`' => {
                let mut head = vec![self.reader.get().unwrap()];
                let tail = self.read_till(b'`');
                head.extend(tail);
                Token::Identifier(head)
            },
            b'(' => {
                self.reader.increment_index();
                Token::LP
            },
            b')' => {
                self.reader.increment_index();
                Token::RP
            },
            b';' => {
                self.reader.increment_index();
                Token::SemiColon
            },
            b',' => {
                self.reader.increment_index();
                Token::Comma
            },            
            _ => {
                Token::Ignore(self.reader.get().unwrap())
            },
        };

        Some(token)
    }

    fn comment(&mut self) -> Token {
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

        return Token::Comment(collection)
    }
}


#[cfg(test)]
mod reader_test{
    use std::fs::File;
    use Reader;
    use super::Tokenizer;

    #[test]
    fn tokenizer(){
        let read_buffer: usize = 1 * 1024 * 1024; // 1mb 
        let file = File::open("./example-files/create-table-with-comments.txt").unwrap();
        let reader = Reader::new(file, read_buffer);
        let mut tk = Tokenizer::new(reader);
        loop {
            match tk.token() {
                Some(item) => {
                    item.log();
                },
                None => break,
            }
        }
    }

}