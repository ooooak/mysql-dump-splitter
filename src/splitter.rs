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


#[derive(Debug,PartialEq,Clone)]
pub enum FileState {
    New,
    Continue,
}

pub struct Splitter<T>{
    parser: Parser<T>,
    total_bytes: usize,
    max_write_size:usize,
    last_insert: Vec<Token>,
    last_file_state: FileState,
}

pub enum SplitterState<'a>{
    SyntaxErr(SyntaxErr),
    // Reached output limit. send the chunk
    Chunk(&'a FileState, Vec<u8>),
    // reached the EOF.
    Done,
}

/**
 * Splitter Should only send Valid output
 * */ 
impl<T> Splitter<T> where T: io::Read {
    pub fn new(settings: SplitterSettings<T>) -> Self {
        let tokenizer = Tokenizer::new(Reader::new(settings.file, settings.read));
        Self {
            parser: Parser::new(tokenizer),
            total_bytes: 0,
            last_insert: vec![],
            max_write_size: settings.write,
            last_file_state: FileState::New,
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

    fn get_bytes(&self, tokens: Vec<Token>) -> Vec<u8> {
        let mut collection = vec![];
        for token in tokens {
          collection.extend(token.value());
        }

        collection
    }
    fn sum(&self,  tokens: &Vec<Token>) -> usize {
        let mut total = 0;
        for token in tokens {
          total += token.len();
        }
        total
    }

    fn send(&mut self, tokens: Vec<Token>) -> SplitterState {
        
        if self.reached_limit() {
            self.reset_limit();
            self.last_file_state = FileState::New;            
        }else{
            self.last_file_state = FileState::Continue;            
        }
        
        
        
        SplitterState::Chunk(&self.last_file_state, self.get_bytes(tokens))
    }

    fn reset_limit(&mut self){
        self.total_bytes = 0;
    }

    fn reached_limit(&self) -> bool{
        self.total_bytes >= self.max_write_size
    }


    pub fn process(&mut self) -> SplitterState {

        match self.parser.token_stream() {
            Ok(Some(item)) => {
                match item {
                    TokenStream::Insert(tokens) => {
                        self.last_insert = self.copy_insert_statement(&tokens);
                        self.total_bytes += self.sum(&tokens);
                        self.send(tokens)
                    },
                    TokenStream::ValuesTuple(tokens) => {
                        let mut ret = vec![];
                        if self.last_file_state == FileState::New {
                            // starting with fresh collection
                            // push last insert statement
                            ret = self.last_insert.clone();
                        }

                        ret.extend(tokens);
                        self.total_bytes += self.sum(&ret);
                        
                        if self.reached_limit() {
                            // maxed out in value tuple 
                            // close statement
                            let len = ret.len() - 1;
                            ret[len] = Token::SemiColon;
                        }

                        self.send(ret)
                    },
                    TokenStream::Block(tokens) => {
                        self.total_bytes += self.sum(&tokens);
                        self.send(tokens)
                    },
                    TokenStream::Comment(token) |
                    TokenStream::SpaceOrLineFeed(token) => {
                        self.total_bytes += 1;
                        self.send(vec![token])
                    }
                }
            },
            Ok(None) => SplitterState::Done,
            Err(e) => SplitterState::SyntaxErr(e),
        }
    }
}