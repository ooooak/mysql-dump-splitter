use clap::App;
use std::fs::File;
use std::result::Result;
use std::str;
use std::path::Path;

fn parse_size(input: Option<&str>, arg_name: &str) -> Result<usize, String> {
    match input {
        Some(value) => {
            if value.len() < 3 {
                return Err(format!("{} has invalid length.", arg_name))
            }

            let (number, format) = value.split_at(value.len() - 2);
            let offset  = match format {
                "kb" => 1024,
                "mb" => 1024 * 1024,
                "gb" => 1024 * 1024 * 1024,
                _ => 0,
            };

            if offset == 0 {
                return Err(format!(
                    "{} has invalid format. choose from kb, mb or gb.", 
                    arg_name
                ))
            }
            match number.parse::<usize>(){
                Ok(number) => Ok(offset * number),
                Err(_) => {
                    Err(format!(
                        "unable to parse number {} number", 
                        arg_name
                    ))
                }
            }
        },
        None => {
            Err(format!("{} is required", arg_name))
        },
    }

}


pub fn args() -> (Result<File, &'static str>, Result<usize, String>) {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let write_buffer = matches.value_of("OUTPUT_SIZE");
    let file = match matches.value_of("INPUT") {
        Some(file) => {
            let path = Path::new(file);
            if path.exists(){
                match File::open(path) {
                    Ok(file) => Ok(file),
                    Err(_) => Err("Unable to open file"),
                }
            }else{
                Err("File path is invalid")
            }
        },
        None => Err("File name is missing"),
    };

    (file, parse_size(write_buffer, "output-size"))
}