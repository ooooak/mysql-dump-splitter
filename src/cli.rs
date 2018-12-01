use std::result::Result;
use std::str;
use std::env;
use std::path::Path;

pub fn file_path() -> Result<String, &'static str> {
  match env::args().nth(1) {
    Some(ref file) => {
      let path = Path::new(file);
      if path.exists() {
        Ok(file.clone())
      }else{
        Err("File path is invalid")
      }
    },
    None => {
      Err( "File path is invalid")
    },
  }
}
