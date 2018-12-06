use tokenizer::Tokenizer;
use tokenizer::SyntaxErr;
use tokenizer::Token;
use std::io;


#[derive(Debug,PartialEq)]
pub enum TokenStream {
    Insert(Vec<Token>),
    ValuesTuple(Vec<Token>),
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

   
    pub fn read_while(&mut self, token: Token) -> Result<Vec<Token>, SyntaxErr> {
        let mut collection = vec![];
        loop {
            match self.tokenizer.token() {
                Ok(Some(t)) => {
                    if t == token {
                        collection.push(t);
                        break
                    }else{
                        collection.push(t);
                    }
                },
                Ok(None) => {
                    return Err(SyntaxErr{
                        text:"invalid end of file"
                    })
                },
                Err(err) => {
                    return Err(err)
                },
            }
        }

        Ok(collection)
    }

    pub fn values(&mut self) -> Result<Vec<Token>, SyntaxErr> {
        let mut collection = vec![];
        loop {
            match self.tokenizer.token() {
                Ok(Some(token @ Token::LP)) => {
                    collection.push(token);
                    match self.read_while(Token::RP) {
                        Ok(val) => {
                            collection.extend(val);
                        },
                        Err(e) => return Err(e),
                    }                    
                },
                Ok(Some(token @ Token::Comma)) => {
                    collection.push(token);
                    break;
                },
                Ok(Some(token @ Token::SemiColon)) => {
                    collection.push(token);
                    break;
                },
                Ok(Some(token @ _)) => {
                    collection.push(token);
                },
                Ok(None) => {
                    return Err(SyntaxErr{
                        text: "Unable to parse values."
                    })
                },
                Err(e) => return Err(e),
            }
        }

        Ok(collection)
    }

    pub fn values_tuple(&mut self) -> Result<Vec<Token>, SyntaxErr> {
        let mut collection = vec![];
        match self.read_while(Token::RP) {
            Ok(val) => {
                collection.extend(val);
            },
            Err(e) => return Err(e),
        }
        

        loop {
            match self.tokenizer.token() {                    
                Ok(Some(token @ Token::Comma)) => {
                    collection.push(token);
                    break;
                },
                Ok(Some(token @ Token::SemiColon)) => {
                    collection.push(token);
                    break;
                },
                Ok(Some(token @ _)) => {
                    collection.push(token);
                },
                Ok(None) => {
                    return Err(SyntaxErr{
                        text: "Unable to parse values."
                    })
                },
                Err(e) => return Err(e),
            }
        }

        Ok(collection)
    }

    fn insert(&mut self) ->  Result<Vec<Token>, SyntaxErr> {
        let mut collection = vec![];
        loop {
            match self.tokenizer.token() {
                Ok(Some(token)) => {
                    if token.keyword("values") {
                        collection.push(token);
                        match self.values() {
                            Ok(val) => {
                                collection.extend(val);
                            },
                            Err(e) =>return Err(e),
                        }
                        break;
                    }else{
                        collection.push(token);
                    }
                },
                Ok(None) => {
                    return Err(SyntaxErr{
                        text: "Incomplete Insert statement."
                    })
                },
                Err(e) => return Err(e),
            }
        }

        Ok(collection)
    }

    pub fn token_stream(&mut self) -> Result<Option<TokenStream>, SyntaxErr> {
        let t = self.tokenizer.token();
        // t.clone().unwrap().log();
        match t {
            Ok(Some(token)) => {
                match token {
                    Token::Keyword(_) => { 
                        if token.keyword("insert") {
                            // parse insert statement
                            // should end with with , or ;
                            // example: "insert into xyz values (),"
                            // example: "insert into xyz values ();"
                            match self.insert() {
                                Ok(value) => {
                                    let mut output = vec![token];
                                    output.extend(value);
                                    Ok(Some(TokenStream::Insert(output)))
                                },
                                Err(e) => Err(e),
                            }
                        }else{
                            // we assume its a block handle blocks
                            // anything that ends with `;` and 
                            // start with create, drop or set etc etc
                            match self.read_while(Token::SemiColon) {
                                Ok(val) => {
                                    let mut output = vec![];
                                    output.push(token);
                                    output.extend(val);
                                    Ok(Some(TokenStream::Block(output)))
                                },
                                Err(e) => Err(e)   
                            }
                        }
                    },
                    Token::LP => {
                        match self.values_tuple() {
                            Ok(val) => {
                                let mut output = vec![Token::LP];
                                output.extend(val);
                                Ok(Some(TokenStream::ValuesTuple(output)))
                            },
                            Err(e) => return Err(e),
                        }
                    }
                    Token::Comment(_) | 
                    Token::InlineComment(_) => {
                        Ok(Some(TokenStream::Comment(token)))
                    },
                    Token::RP |
                    Token::Dot |
                    Token::String(_) |
                    Token::Identifier(_) |
                    Token::Comma |
                    Token::Ignore(_) => {
                        Err(SyntaxErr{
                            text: "Invalid sql file."
                        })
                    },
                    Token::SemiColon |
                    Token::Space |
                    Token::LineFeed(_) =>{
                        Ok(Some(TokenStream::SpaceOrLineFeed(token)))
                    }
                }
            },
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}




#[cfg(test)]
mod reader_test{
    use std::fs::File;
    use reader::Reader;
    use tokenizer::Token;
    use tokenizer::Tokenizer;
    use tokenizer::SyntaxErr;
    use super::Parser;
    use super::TokenStream;

    type TS = Result<Option<TokenStream>, SyntaxErr>;    
    fn is_space(value: TS) -> bool {
        match value {
            Ok(Some(TokenStream::SpaceOrLineFeed(_))) => {
                true
            },
            _ => false,
        }
    }

    fn is_comment(value: TS) -> bool {
        match value {
            Ok(Some(TokenStream::Comment(_))) => {
                true
            },
            _ => false,
        }
    }

    fn valid_values_tuple(value: TS) -> (bool, &'static str) {
        match value {
            Ok(Some(TokenStream::ValuesTuple(tokens))) => {
                match tokens[tokens.len() - 1] {
                    Token::SemiColon => (true, ""),
                    _ => (false, "Last token should be semicolon or comma"),
                }
            },
            _ => (false, "expected ValuesTuple"),
        }
    }

    fn valid_block(value: TS) -> (bool, &'static str) {
        match value {
            Ok(Some(TokenStream::Block(tokens))) => {
                match tokens[tokens.len() - 1] {
                    Token::SemiColon => (true, ""),
                    _ => (false, "Last token should be semicolon"),
                }
            },
            _ => (false, "expected block"),
        }
    }

    fn valid_insert(value: TS) -> (bool, &'static str) {
        match value {
            Ok(Some(TokenStream::Insert(tokens))) => {
                match tokens[tokens.len() - 1] {
                    Token::SemiColon => (true, ""),
                    Token::Comma => (true, ""),
                    _ => (false, "Last token should be semicolon or Comma"),
                }
            },
            _ => (false, "expected insert statement"),
        }
    }
    
    #[test]
    fn tokenizer(){
        let read_buffer: usize = 1 * 1024 * 1024;
        let file = File::open("./example-files/1.txt").unwrap();
        let tokenizer = Tokenizer::new(Reader::new(file, read_buffer));
        
        let mut parser = Parser::new(tokenizer);
                
        // inline comment
        assert!(is_comment(parser.token_stream()), "Expecting a comment");
        assert!(is_comment(parser.token_stream()), "Expecting a comment");
        
        // comment ends with "\n"
        // so we only expect one new line 
        assert!(is_space(parser.token_stream()), "white space");
        assert!(is_space(parser.token_stream()), "white space");

        // create table
        let (state, msg) = valid_block(parser.token_stream());
        assert!(state, msg);

        // white space or line feed
        assert!(is_space(parser.token_stream()), "white space");
        assert!(is_space(parser.token_stream()), "white space");
        assert!(is_space(parser.token_stream()), "white space");
        assert!(is_space(parser.token_stream()), "white space");

        // insert
        let (state, msg) = valid_insert(parser.token_stream());
        assert!(state, msg);

        assert!(is_space(parser.token_stream()), "white space");
        assert!(is_space(parser.token_stream()), "white space");

        let (state, msg) = valid_insert(parser.token_stream());
        assert!(state, msg);

        // line feeds
        assert!(is_space(parser.token_stream()), "white space");
        assert!(is_space(parser.token_stream()), "white space");
        assert!(is_space(parser.token_stream()), "white space");
        assert!(is_space(parser.token_stream()), "white space");

        // set FOREIGN_KEY_CHECKS block
        let (state, msg) = valid_block(parser.token_stream());
        assert!(state, msg);
        
        // line feed
        assert!(is_space(parser.token_stream()), "white space");
        assert!(is_space(parser.token_stream()), "white space");
        assert!(is_space(parser.token_stream()), "white space");
        assert!(is_space(parser.token_stream()), "white space");

        // create table block
        let (state, msg) = valid_block(parser.token_stream());
        assert!(state, msg);

        // line feed
        assert!(is_space(parser.token_stream()), "white space");
        assert!(is_space(parser.token_stream()), "white space");
        assert!(is_space(parser.token_stream()), "white space");
        assert!(is_space(parser.token_stream()), "white space");

        // insert 
        let (state, msg) = valid_insert(parser.token_stream());
        assert!(state, msg);

        // value tuple
        let (state, msg) = valid_values_tuple(parser.token_stream());
        assert!(state, msg);
    }


}