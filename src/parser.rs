use helper::die;
use tokenizer::Tokenizer;
use tokenizer::Token;
use std::io;

/**
 * We can only send tokens that can be written down to file
 * with little modification
 */ 
pub enum TokenStream {
    Insert(Vec<Token>),
    Values(Vec<Token>),
    Block(Vec<Token>),
    Comment(Token),
    SpaceOrLineFeed(Token),
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
                },
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
    
    /**
     * We only send streams that we can write with little modification
     * 
     *  #TokenStream::Insert 
     *   We are sending insert statement with first value.
     *   reason to do that is we only want to send stream that we can write to
     *   file with less modifications 
     *   
     *   insert can end with , or ;
     *   TokenStream::Insert("insert into xyz values (),")
     *   TokenStream::Insert("insert into xyz values ();")
     * 
     *  #TokenStream::Values
     *   Value is only create when we are in insert statement with multiple values.
     *    
     *   values can end with , or ;
     *   we might have white space at start
     *   TokenStream::Insert(" values (),")
     *   TokenStream::Insert("  values ();")
     * 
     *  #Token::Block
     *   Blocks is anything that ends with ;
     *   create sta
     * */
    pub fn token_stream(&mut self) -> Option<TokenStream> {
        let t = self.tokenizer.token();
        t.clone().unwrap().log();
        match t {
            Some(token) => {
                match token {
                    Token::Keyword(_) => {
                        let mut output = vec![];
                        if token.keyword("insert") {
                            // parse insert statement
                            output.extend(self.insert(token));
                            Some(TokenStream::Insert(output))
                        }else if token.keyword("values") {
                            // parse values statement
                            output.extend(self.values(token));
                            Some(TokenStream::Values(output))
                        }else{
                            // we parse block, thinks like create table, set x 
                            output.extend(self.read_while(Token::SemiColon));
                            Some(TokenStream::Block(output))
                        }
                    },
                    Token::Comment(_) | 
                    Token::InlineComment(_) => {
                        Some(TokenStream::Comment(token))
                    },
                    Token::LP |
                    Token::RP |
                    Token::Dot |
                    Token::String(_) |
                    Token::Identifier(_) |
                    Token::Comma |
                    Token::Ignore(_) => {
                        token.log();
                        die("We cannot start with following set of tokens.")
                    },

                    // SemiColon can be treated as white space
                    Token::SemiColon |
                    Token::Space |
                    Token::LineFeed(_) =>{
                        Some(TokenStream::SpaceOrLineFeed(token))
                    }
                }
            },
            None => None,
        }
    }
}