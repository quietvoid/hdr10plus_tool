use crate::hdr10plus::hdr10plus::*;
use std::io::{stdout, stdin, Write};
use std::path::Path;
use std::process;
use std::env;
use regex::Regex;

mod hdr10plus;

fn main() {

    let mut input = String::new();

    let args: Vec<String> = env::args().collect();

    if args.is_empty(){
        print!("Enter path to HEVC file: ");
        stdout().flush().ok();

        match stdin().read_line(&mut input){
            Ok(_) =>{
                input = input.trim().to_string();
                process_input(input, args);
            }
            Err(error) => println!("Error: {}", error),
        }
    }
    else if args.len() >= 2{
        input = (*args[1].trim()).to_string();

        process_input(input, args);
    }
}

fn process_input(input: String, params: Vec<String>){
    let path = Path::new(&input);
    let parent_dir = path.parent().unwrap();
    let save_str = parent_dir.join(path.file_name().unwrap()).to_str().unwrap().to_string();

    let path_str = path.to_str().unwrap().to_string().to_lowercase();
    let regex = Regex::new(r"\.(hevc|.?265)").unwrap();

    if regex.is_match(&path_str) && path.is_file(){

        let log_file = format!("{}-sei.log", save_str);
        let metadata_file = format!("{}-meta.json", save_str);

        let mut final_metadata: Vec<Metadata> = Vec::new();
        match parse_metadata(path_str, &log_file, params){
            Ok(o) => {
                        println!("{}", o);
                        if o == String::from("Done."){
                            final_metadata = llc_read_metadata(&log_file);
                        }
                     }
            Err(e) => println!("{}", e)
        }

        write_json(metadata_file, final_metadata);
    }
    else{
        if !path.is_file(){
            println!("File path not found.");
        }
        else{
            println!("Invalid file type.");
        }
    }
}