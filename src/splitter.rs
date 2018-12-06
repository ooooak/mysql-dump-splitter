use parser::TokenStream;
use parser::Parser;
use tokenizer::Token;
use tokenizer::Tokenizer;
use tokenizer::SyntaxErr;
use reader::Reader;
use std::io;

pub struct SplitterSettings<T>{
    // Bytes to read 
    pub read: usize,
    // bytes to write
    pub write: usize,
    pub file: T,
}

pub struct Splitter<T>{
    parser: Parser<T>,
    collection: Vec<Token>,
    total: usize,
    max_write_size:usize,
    last_insert: Vec<Token>,
    eof: bool,
}

pub enum SplitterState{
    SyntaxErr(SyntaxErr),
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
            max_write_size: settings.write,
            eof: false,
        }
    }

    /**
     * Copy insert statement till VALUES
     * push extra white space after values
     * */ 
    fn copy_insert_statement(&self, tokens: &[Token]) -> Vec<Token> {
        let mut ret = vec![];
        for token in tokens {
            ret.push(token.clone());
            if token.keyword("values") {
                break;
            }
        }

        ret.push(Token::Space);
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
        if self.eof || self.reached_limit() {
            self.total = 0;
            if self.eof {
                SplitterState::Done(self.reset_collection())
            }else{
                SplitterState::Chunk(self.reset_collection())
            }
        }else{
            SplitterState::Continue
        }
    }

    fn reached_limit(&self) -> bool{
        self.total >= self.max_write_size
    }

    fn store(&mut self, tokens: Vec<Token>) {
        self.total += self.sum(&tokens);
        self.collection.extend(tokens);
    }


    pub fn process(&mut self) -> SplitterState {
        match self.parser.token_stream() {
            Ok(Some(item)) => {
                match item {
                    TokenStream::Insert(tokens) => {
                        self.last_insert = self.copy_insert_statement(&tokens);
                        self.store(tokens);
                        self.state()
                    },
                    TokenStream::ValuesTuple(mut tokens) => {
                        if self.collection.len() == 0 {
                            // starting with fresh collection
                            // push last insert statement
                            let insert_stm = self.last_insert.clone();
                            self.store(insert_stm);
                        }

                        self.store(tokens);

                        
                        if self.reached_limit() {
                            // maxed out in value tuple 
                            // close statement
                            let len = self.collection.len();
                            self.collection[len - 1] = Token::SemiColon;
                        }

                        self.state()
                    },
                    TokenStream::Block(tokens) => {
                        self.store(tokens);
                        self.state()
                    },
                    TokenStream::Comment(token) |
                    TokenStream::SpaceOrLineFeed(token) => {
                        self.store(vec![token]);
                        self.state()
                    }
                }
            },
            Ok(None) => {
                self.eof = true;
                self.state()
            },
            Err(e) => SplitterState::SyntaxErr(e),
        }
    }
}