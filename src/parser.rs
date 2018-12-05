use tokenizer::Tokenizer;
use tokenizer::SyntaxErr;
use tokenizer::Token;
use std::io;



/**
 * We can only send tokens that can be written down to file
 * with little modification
 */ 
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
                    return Err(SyntaxErr {
                        text:"invalid end of file"
                    })
                },
                Err(err) => return  Err(err),
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


    /**  
     * Case one: insert into `x` values (1, 4);
     * Case two: insert into x (id, price) values (1, 4), ();
     **/
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
     * 
     *  #Token::Block
     *   Blocks is anything that ends with ;
     *   create sta
     * */
    pub fn token_stream(&mut self) -> Result<Option<TokenStream>, SyntaxErr> {
        let t = self.tokenizer.token();
        // t.clone().unwrap().log();
        match t {
            Ok(Some(token)) => {
                match token {
                    Token::Keyword(_) => {
                        let mut output = vec![];
                        
                        if token.keyword("insert") {
                            // parse insert statement
                            match self.insert() {
                                Ok(value) => {
                                    output.push(token);
                                    output.extend(value);
                                    Ok(Some(TokenStream::Insert(output)))
                                },
                                Err(e) => return Err(e),
                            }
                            
                        }else{
                            // we parse block, thinks like create table, set x
                            output.push(token);
                            match self.read_while(Token::SemiColon) {
                                Ok(val) => {
                                    output.extend(val);
                                    Ok(Some(TokenStream::Block(output)))
                                },
                                Err(e) => return Err(e)   
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

                    // SemiColon can be treated as white space
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