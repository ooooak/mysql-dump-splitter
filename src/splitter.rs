use parser::TokenStream;
use parser::Parser;
use tokenizer::Token;
use tokenizer::Tokenizer;
use reader::Reader;
use std::io;

pub struct SplitterSettings<T>{
    // Bytes to read 
    pub read: usize,
    // bytes to write
    pub write: usize,
    pub file: T,
}

// #[derive(Debug)]
pub struct Splitter<T>{
    parser: Parser<T>,
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

/**
 * Splitter Should only send Valid output
 * */ 
impl<T> Splitter<T> where T: io::Read {
    pub fn new(settings: SplitterSettings<T>) -> Self {
        let tokenizer = Tokenizer::new(Reader::new(settings.file, settings.read));
        Self {
            parser: Parser::new(tokenizer),
            total: 0,
            collection: vec![],
            last_insert: vec![],
            in_insert_statement: false,
            max_write_size: settings.write,
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

    fn reset_collection(&mut self) -> Vec<Token>{
        let collection = self.collection.clone();
        self.collection.clear();
        collection
    }

    fn state(&mut self) -> SplitterState {
        if self.eof {
            self.total = 0;
            return SplitterState::Done(self.reset_collection());
        }

        match self.total >= self.max_write_size {
            true => {
                self.total = 0;
                let mut collection = self.reset_collection();
                if self.in_insert_statement {
                    let len = collection.len();
                    collection[len - 1] = Token::SemiColon;
                }
                
                SplitterState::Chunk(collection)
            },
            _ => SplitterState::Continue
        }
    }

    fn store(&mut self, tokens: Vec<Token>) -> SplitterState {
        // println!("{:?}", self.total);
        self.total += self.sum(&tokens);
        self.collection.extend(tokens);
        self.state()
    }


    pub fn process(&mut self) -> SplitterState {
        // pre-pend last insert statement        
        match self.parser.token_stream() {
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

                        self.store(tokens)
                    },
                    TokenStream::Values(tokens) => {
                        // when starting a new collection
                        // we need to check if we are in insert
                        self.in_insert_statement = match tokens.get(tokens.len() - 1) {
                            Some(Token::SemiColon) => false,
                            _ => true,
                        };

                        if self.in_insert_statement && self.collection.len() == 0 {
                            let last_insert = self.last_insert.clone();
                            self.store(last_insert);
                        }else{
                            self.last_insert = vec![];
                        }

                        self.store(tokens)
                    },
                    TokenStream::Block(tokens) => {
                        self.store(tokens)
                    },
                    TokenStream::Comment(token) |
                    TokenStream::SpaceOrLineFeed(token) => {
                        self.store(vec![token])
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