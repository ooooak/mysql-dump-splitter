use scanner::Token;
use std::io::Write;
use std::thread;
use std::fs::File;
use std::io::Result;
use std::io::prelude::*;

#[derive(Debug)]
pub struct SQLWriter{
  in_insert: bool,
  file_number: usize, 
  collection: Vec<u8>,
  total: usize,
  max_write_size:usize,
  last_insert: Vec<u8>,
}

impl SQLWriter {
  pub fn new(max_write_size: usize) -> Self {
    SQLWriter{
      total: 0,
      collection: vec![],
      last_insert: vec![],
      in_insert: false,
      file_number: 0,
      max_write_size: max_write_size,
    }
  }

  fn write(&mut self){
    let collection = self.collection.clone();
    self.collection.clear();
    self.total = 0;

    self.file_number += 1;


    let file_name = format!("./tests/sql/{:?}.sql", self.file_number);
    let mut buffer = File::create(file_name).unwrap();
    buffer.write_all(&collection).unwrap();
    

      // thread::spawn(move || {});
  }



  fn push_last_insert_into_collection(&mut self){
    self.collection.extend(&self.last_insert);
    self.collection.extend(" VALUES ".as_bytes().iter().cloned());
  }
  

  pub fn process(&mut self, token: Option<Token>) -> bool {
    match token {
        Some(item) => {
          match item {
            Token::Comment(item) |
            Token::CommentInline(item) |
            Token::Block(item) |
            Token::SpacesOrLineFeeds(item) => {
                self.total += item.len();
                self.collection.extend(item);

                if self.total >= self.max_write_size {
                  self.write();
                }
            },

            Token::Insert(item) => {
              self.total += item.len();
              self.last_insert = item.clone();
              self.collection.extend(item);
              

              if self.total >= self.max_write_size {
                self.collection.push(b';'); // end block
                self.write();
              }
            },
            Token::InsertValues(item) => {
              self.total += item.len();
              if self.total > self.max_write_size {
                // reached max max write size. 
                // close the ending block and write to file 
                let mut cp_item = item.clone();
                let cp_item_len = cp_item.len() -1;
                cp_item[cp_item_len] = b';';
                self.collection.extend(cp_item);  
                self.write();
              }else{
                if self.collection.len() == 0 {
                  // we need to append last insert statement. 
                  self.push_last_insert_into_collection();                                
                }

                self.collection.extend(item);
              }

            }
          }

          true
        },
        None => {
          if self.collection.len() > 0 {
            self.write()
          }
          false
        },
    }
  }
}