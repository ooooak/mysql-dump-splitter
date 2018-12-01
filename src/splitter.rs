use parser::TokenStream;
use tokenizer::Token;
use std::io::Write;
use std::thread;
use std::fs::File;
use std::io::Result;
use std::io::prelude::*;
use helper::dump;



#[derive(Debug)]
pub struct Splitter{
    in_insert_statement: bool,
    collection: Vec<Token>,
    total: usize,
    max_write_size:usize,
    last_insert: Vec<Token>,
    eof: bool,
}

pub enum SplitterState{
    // Reached output limit. send the chunk
    Chunk(Vec<Token>),
    // reached the EOF.
    Done(Vec<Token>),
    // need more data
    Continue, 
}

impl Splitter {
    pub fn new(max_write_size: usize) -> Self {
        Self {
            total: 0,
            collection: vec![],
            last_insert: vec![],
            in_insert_statement: false,
            max_write_size: max_write_size,
            eof: false,
        }
    }

    /**
     * TokenStream::Insert contains values too. 
     * we only need insert statement to pre pend before creating new file. 
     * */ 
    fn copy_insert(&self, tokens: &Vec<Token>) -> Vec<Token> {
        let mut ret = vec![];
        for token in tokens {
            if token.keyword("values") {
                break;
            }
            ret.push(token.clone());
        }
        ret
    }

    fn sum(&self,  tokens: &Vec<Token>) -> usize {
        let mut total = 0;
        for token in tokens {
          total += token.len();
        }
        total
    }

    fn pull_collection(&mut self) -> Vec<Token>{
        let mut collection = self.collection.clone();
        self.collection.clear();
        collection
    }

    fn state(&mut self) -> SplitterState {
        let maxed_out = self.total >= self.max_write_size;
        match (self.eof, maxed_out) {
            (true, _) => {
                SplitterState::Done(self.pull_collection())
            },
            (_, true) => {
                let mut collection = self.pull_collection();
                if self.in_insert_statement {
                    let len  = collection.len();
                    collection[len - 1] = Token::SemiColon;
                }
                SplitterState::Chunk(collection)
            },
            _ => {
                SplitterState::Continue
            }
        }
    }

    // when starting a new collection
    // we need to check if we were in pending values
    // then pre pend last insert statement
    pub fn process(&mut self, token: Option<TokenStream>) -> SplitterState {
        // pre-pend last insert statement
        if self.in_insert_statement && self.collection.len() == 0 {
            self.collection.extend(self.last_insert.clone());
        }

        match token {
            Some(item) => {
                match item {
                    TokenStream::Insert(tokens) => {
                        self.in_insert_statement = match tokens.get(tokens.len() - 1) {
                            Some(Token::Comma) => true,
                            _ => false,
                        };

                        // cache insert statement
                        if self.in_insert_statement {
                            self.last_insert = self.copy_insert(&tokens);
                        }
                        
                        self.total += self.sum(&tokens);
                        self.collection.extend(tokens);
                        self.state()
                    },
                    TokenStream::Values(tokens) => {
                        self.in_insert_statement = match tokens.get(tokens.len() - 1) {
                            Some(Token::Comma) => true,
                            _ => false,
                        };

                        if self.in_insert_statement == false{
                            self.last_insert = vec![];
                        }

                        self.state()
                    },
                    TokenStream::Ignore(token) => {
                        self.collection.push(token);
                        self.state()
                    }
                }
            },
            None => {
                self.eof = true;
                self.state()
            },
        }
    }
}