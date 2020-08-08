use std::io::{BufReader};
use std::io::prelude::*;
use std::io;

const DEFAULT_BUF_SIZE: usize = 8 * 1024;

pub struct Reader<T>{
    buffer: [u8; DEFAULT_BUF_SIZE],
    index: usize,
    reader: BufReader<T>,
    bytes_read: usize,
}

impl<T> Reader<T> where T: io::Read {
    pub fn new(file: T) -> Self {
        // reader
        let mut reader = Self {
            buffer: [0; DEFAULT_BUF_SIZE],
            reader: BufReader::new(file),
            index: 0,
            bytes_read: 0,
        };

        reader.read_buf();
        reader
    }

    #[inline(always)]
    fn raw_get(&mut self) -> Option<u8> {
        if self.bytes_read == 0 {
            return None
        }

        if let Some(item) = self.buffer.get(self.index) {
            return if self.bytes_read <= self.index {
                None
            }else{
                Some(*item)
            }
        }

        // out of index load next buffer
        self.read_buf();
        self.raw_get()
    }

    pub fn get(&mut self) -> Option<u8> {
        let byte = self.raw_get();
        self.index += 1;
        byte
    }

    pub fn peek(&mut self) -> Option<u8> {
        self.raw_get()
    }

    pub fn peek_next(&mut self) -> Option<u8> {
        self.index += 1;
        let item = self.raw_get();
        self.index -= 1;
        item
    }


    fn read_buf(&mut self) {
        match self.reader.read(&mut self.buffer) {
            Ok(size) => {
                self.bytes_read = size;
                self.index = 0; // reset index
            },
            Err(err) => {
                panic!("{:?}", err);
            }
        }
    }

    pub fn increment_index(&mut self){
        self.index += 1;
    }
}


#[cfg(test)]
mod reader_test{
    use std::fs::File;
    use super::Reader;

    #[test]
    fn empty_file(){
        let file = File::open("./example-files/empty.txt").unwrap();
        let mut reader = Reader::new(file);
        assert_eq!(reader.get(), None);
    }

    #[test]
    fn get(){
        let file = File::open("./example-files/content.txt").unwrap();
        let mut reader = Reader::new(file);

        // let mut col = vec![];
        // loop {                
        //     let item = reader.get();
        //     if !item.is_none() {
        //         col.push(item.unwrap());
        //         continue;
        //     }
        //     println!("buff: {:?} output: {:?}", n, str::from_utf8(&col));
        //     break;
        // }
        
        assert_eq!(reader.get(), Some(b'1'));
        assert_eq!(reader.get(), Some(b'2'));
        assert_eq!(reader.get(), Some(b'3'));
        assert_eq!(reader.get(), Some(b'4'));
        assert_eq!(reader.get(), Some(b'5'));
        assert_eq!(reader.get(), Some(b'6'));
        assert_eq!(reader.get(), Some(b'7'));
        assert_eq!(reader.get(), Some(b'8'));
        assert_eq!(reader.get(), Some(b'9'));
        assert_eq!(reader.get(), Some(b'0'));
        assert_eq!(reader.get().is_none(), true);
        assert_eq!(reader.get().is_none(), true);
    }

    #[test]
    fn peek(){
        let file = File::open("./example-files/content.txt").unwrap();
        let mut reader = Reader::new(file);
        assert_eq!(reader.peek(), Some(b'1'));
        let _skip_it = reader.get();

        assert_eq!(reader.peek(), Some(b'2'));
    }

    #[test]
    fn peek_next(){
        let file = File::open("./example-files/content.txt").unwrap();
        let mut reader = Reader::new(file);
        assert_eq!(reader.peek_next(), Some(b'2'));
        assert_eq!(reader.get(), Some(b'1'));
        assert_eq!(reader.get(), Some(b'2'));
        assert_eq!(reader.peek(), Some(b'3'));
        assert_eq!(reader.peek_next(), Some(b'4'));
    }
}