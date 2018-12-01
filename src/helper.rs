use std::process;
use tokenizer::Token;

fn sleep(sec: u64){
    println!("sleep");
    use std::{thread, time};
    
    let from_millis = time::Duration::from_millis(sec * 1000);
    // let now = time::Instant::now();
    thread::sleep(from_millis);
}


pub fn die(text : &str) -> ! {
    println!("{}", text);
    process::exit(0);
}

pub fn dump(tokens: &Vec<Token>) {
    for token in tokens {
        token.log()
    }
}

    
pub fn write(){
    // let file_name = format!("./example-files/output/{:?}.sql", self.file_number);
    // let mut buffer = File::create(file_name).unwrap();
    // buffer.write_all(&collection).unwrap();
}