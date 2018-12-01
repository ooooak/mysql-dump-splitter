use std::str;
use helper::die;
use tokenizer::Tokenizer;
use tokenizer::Token;
use std::io;

pub enum Ast {
    Insert(Vec<Token>),
    Values(Vec<Token>),
    Ignore(Token)
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

    /**
     * case one: 
     *  INSERT INTO distributors (`id`, `name`, `VALUES`)
     *         VALUES (1, 'str', 'none');
     * 
     * case two: 
     *  INSERT INTO `distributors` VALUES (1, 'str');
     * 
     * **/ 


    /**  
     * insert into x
     *  values (1, 2,3 4),
     *  values (1, 2,3 4),
     * max output buffer reached write file 
     * replace ',' with ';'
     * in insert ? get last insert statement
     *  values (1, 2,3 4),
     *  values (1, 2,3 4),
     *  values (1, 2,3 4);
     * fn insert_statement(&mut self) -> Vec<u8> {}
     * */
     pub fn read_while(&mut self, token: Token) -> Vec<Token> {
        let mut collection = vec![];
        loop {
            match self.tokenizer.token() {
                Some(t) => {
                    if t == token {
                        collection.push(t);
                        break
                    }
                },
                None => {
                    die("Error: invalid end of file")
                },
            }
        }

        collection
     }


    /**
     * 
     * Case 1: values (1, 2,3 4),
     * 
     * Caee 2: values (1, 2,3 4);
     * 
     *  
     **/ 
    pub fn values(&mut self, head:Token) -> Vec<Token> {
        let mut collection = vec![head];

        loop {
            match self.tokenizer.token() {
                Some(token) => {
                    match token {
                        Token::LP => {
                            collection.extend(self.read_while(Token::RP));
                        },
                        Token::Comma => {
                            collection.push(token);
                            break;
                        },
                        Token::SemiColon => {
                            collection.push(token);
                            break;
                        }
                        _ => {
                            collection.push(token);                            
                        }
                    }
                },
                None => die("Error: invalid end of file"),
            }
        }

        collection
    }


    /**  
     * Case one: insert into `x` values (1, 4);
     * Case two: insert into x (id, price) values (1, 4);
     **/
    fn insert(&mut self, head:Token) -> Vec<Token>{
        let mut collection = vec![head];

        loop {
            match self.tokenizer.token() {
                Some(token) => {
                    if token.keyword("value") {
                        collection.extend(self.values(token));
                        break;
                    }else{
                        collection.push(token);
                    }
                },
                None => die("Error: invalid end of file"),
            }
        }

        collection
    }


    pub fn ast(&mut self) -> Option<Ast> {        
        match self.tokenizer.token() {
            Some(token) => {
                let mut output = vec![];
                if token.keyword("insert") {
                    output.extend(self.insert(token));
                    Some(Ast::Insert(output))
                }else if token.keyword("value") {
                    output.extend(self.values(token));
                    Some(Ast::Values(output))
                }else{
                    Some(Ast::Ignore(token))
                }
            },
            None => None,
        }
    }
}