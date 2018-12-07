use parser::TokenStream;
use parser::Parser;
use tokenizer::Tokenizer;
use tokenizer::SyntaxErr;
use reader::Reader;
use std::io;

pub struct SplitterSettings<T>{
    pub write: usize,
    pub file: T,
}


#[derive(Debug,PartialEq,Clone)]
pub enum FileState{
    New,
    Continue,
}

impl std::fmt::Display for FileState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use splitter::FileState;

        match self{
            FileState::New => write!(f, "{}", "New"),
            FileState::Continue => write!(f, "{}", "Continue")
        }        
    }
}

pub struct Splitter<T>{
    parser: Parser<T>,
    total_bytes: usize,
    max_write_size:usize,
    last_insert: Vec<u8>,
}

pub enum SplitterState{
    SyntaxErr(SyntaxErr),
    // Reached output limit. send the chunk
    Chunk(FileState, Vec<u8>),
    // reached the EOF.
    Done,
}

impl<T> Splitter<T> where T: io::Read {
    pub fn new(settings: SplitterSettings<T>) -> Self {
        let tokenizer = Tokenizer::new(Reader::new(settings.file));
        Self {
            parser: Parser::new(tokenizer),
            total_bytes: 0,
            last_insert: vec![],
            max_write_size: settings.write,
        }
    }

    fn file_state(&self, starting_total: usize) -> FileState {
        if starting_total == 0 {
            FileState::New
        }else{
            FileState::Continue
        }
    }

    fn send(&mut self, tokens: Vec<u8>, starting_total: usize) -> SplitterState {
        self.total_bytes += tokens.len();
        if self.reached_limit(self.total_bytes) {
            self.total_bytes = 0;
        }

        SplitterState::Chunk(self.file_state(starting_total), tokens)
    }

    fn reached_limit(&self, total: usize) -> bool{
        total >= self.max_write_size
    }

    fn copy_last_insert(&self, chunk: &mut Vec<u8>) {
        chunk.extend(&self.last_insert)
    }

    fn close_values_tuple(&self, chunk: &mut Vec<u8>){
        let len = chunk.len() - 1;
        chunk[len] = b';';
    }
    
    pub fn process(&mut self) -> SplitterState {
        let starting_total = self.total_bytes;
        match self.parser.token_stream() {
            Ok(Some(item)) => {
                match item {
                    TokenStream::Insert(insert_with_values, insert_stmt) => {
                        self.last_insert = insert_stmt;                        
                        self.send(insert_with_values, starting_total)
                    },
                    TokenStream::ValuesTuple(tokens) => {
                        let mut ret = vec![];
                        // starting with fresh collection
                        // push last insert statement
                        if starting_total == 0 {
                            self.copy_last_insert(&mut ret);
                        }

                        ret.extend(tokens);
                        // maxed out in value tuple close statement
                        if self.reached_limit(starting_total + ret.len()) {
                            self.close_values_tuple(&mut ret)
                        }
                        
                        self.send(ret, starting_total)
                    },
                    TokenStream::Block(tokens) => self.send(tokens, starting_total),
                    TokenStream::Comment(tokens) |
                    TokenStream::SpaceOrLineFeed(tokens) => self.send(tokens, starting_total),
                }
            },
            Ok(None) => SplitterState::Done,
            Err(e) => SplitterState::SyntaxErr(e),
        }
    }
}