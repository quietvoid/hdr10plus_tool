use crate::hdr10plus::hdr10plus::*;
use std::io::{stdout, stdin, Write};
use std::path::Path;
use std::process;
use std::env;

mod hdr10plus;

fn main() {

    let mut input = String::new();

    let mut args: Vec<String> = env::args().collect();

    if args.is_empty(){
        print!("Enter path to HEVC file: ");
        stdout().flush().ok();

        match stdin().read_line(&mut input){
            Ok(_) =>{
                input = input.trim().to_string();
                process_input(input);
            }
            Err(error) => println!("Error: {}", error),
        }
    }
    else if args.len() == 2{
        input = args.pop().unwrap();
        input = input.trim().to_string();

        process_input(input);
    }
}

fn process_input(input: String){
    let path = Path::new(&input);
    let parent_dir = path.parent().unwrap();
    let save_str = parent_dir.join(path.file_name().unwrap()).to_str().unwrap().to_string();


    if !path.is_file(){
        println!("Invalid file path.");
        process::exit(1);
    }

    let path_str = path.to_str().unwrap().to_string();
    if path_str.contains(".h265") || path_str.contains(".hevc"){

        let log_file = format!("{}-sei.log", save_str);
        let metadata_file = format!("{}-meta.json", save_str);


        let mut final_metadata: Vec<Metadata> = Vec::new();
        match parse_metadata(path_str, &log_file){
            Ok(_) => final_metadata = llc_read_metadata(&log_file),
            Err(e) => println!("{}", e)
        }

        write_json(metadata_file, final_metadata);
    }
    else{
        println!("Invalid file type.");
    }
}