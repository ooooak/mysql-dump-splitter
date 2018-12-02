use std::str;
use helper::die;
use helper::dump;
use tokenizer::Tokenizer;
use tokenizer::Token;
use std::io;

/**
 * As long as we are not doing anything crazy with
 * ast we are going to keep it as list of tokens.
 */ 
pub enum TokenStream {
    Insert(Vec<Token>),
    Values(Vec<Token>),
    Ignore(Token),
}

pub struct Parser<T> {
    tokenizer: Tokenizer<T>,
}

impl<T> Parser<T> where T: io::Read{
    pub fn new(tokenizer: Tokenizer<T>) -> Self {
        Self {
            tokenizer: tokenizer
        }
    }
    
    pub fn read_while(&mut self, token: Token) -> Vec<Token> {
        let mut collection = vec![];
        loop {
            match self.tokenizer.token() {
                Some(t) => {
                    if t == token {
                        collection.push(t);
                        break
                    }else{
                        collection.push(t);
                    }
                },
                None => {
                    die("Error: invalid end of file")
                },
            }
        }
        collection
    }

    pub fn values(&mut self, head:Token) -> Vec<Token> {
        let mut collection = vec![head];
        loop {
            match self.tokenizer.token() {
                Some(token @ Token::LP) => {
                    collection.push(token);
                    collection.extend(self.read_while(Token::RP));
                },
                Some(token @ Token::Comma) => {
                    collection.push(token);
                    break;
                },
                Some(token @ Token::SemiColon) => {
                    collection.push(token);
                    break;
                },
                Some(token @ _) => {
                    collection.push(token);
                }
                None => {
                    die("Error: Unable to parse values.")
                },
            }
        }

        collection
    }


    /**  
     * Case one: insert into `x` values (1, 4);
     * Case two: insert into x (id, price) values (1, 4);
     **/
    fn insert(&mut self, head:Token) -> Vec<Token> {
        let mut collection = vec![head];
        loop {
            match self.tokenizer.token() {
                Some(token) => {
                    if token.keyword("values") {
                        collection.extend(self.values(token));
                        break;
                    }else{
                        collection.push(token);
                    }
                },
                None => {
                    die("Error: Incomplete Insert statement.");
                },
            }
        }
        collection
    }
    
    pub fn token_stream(&mut self) -> Option<TokenStream> {
        match self.tokenizer.token() {
            Some(token) => {
                let mut output = vec![];
                if token.keyword("insert") {
                    output.extend(self.insert(token));
                    Some(TokenStream::Insert(output))
                }else if token.keyword("values") {
                    output.extend(self.values(token));
                    Some(TokenStream::Values(output))
                }else{
                    Some(TokenStream::Ignore(token))
                }
            },
            None => None,
        }
    }
}