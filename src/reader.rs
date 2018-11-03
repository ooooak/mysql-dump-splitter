use std::process;
use std::fs::File;
use std::io::{BufReader, Result};
use std::io::prelude::*;
use std::str;



pub struct Reader{
    buffer: Vec<u8>,
    index: usize,
    reader: BufReader<File>,
    bytes_read: usize,
}

impl Reader{
    pub fn new(file: File) -> Self {
        let size: usize = 1 * 1024 * 1024; // 100mb 

        // reader
        let mut reader = Self {
            buffer: vec![0; size],
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
            // no buffer data
            None
        }else{
            if let Some(item) = self.buffer.get(self.index) {
                return if *item == 0 as u8{
                    None
                }else{
                    Some(item.clone())
                }
            }
            // ran out of data load next buffer data
            self.read_buf();

            
            // try to read the value from index
            // rec call
            self.get()
        }
    }

    pub fn get(&mut self) -> Option<u8> {
        let byte = self.raw_get();
        self.index += 1;
        byte
    }

    #[inline(always)]
    pub fn peek(&mut self) -> Option<u8> {
        self.raw_get()
    }

    #[inline(always)]
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