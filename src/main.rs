use crate::hdr10plus::parser::*;
use regex::Regex;
use std::path::Path;

#[macro_use]
extern crate clap;
use clap::App;

mod hdr10plus;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    process_input(matches);
}

fn process_input(matches: clap::ArgMatches) {
    let input = matches.value_of("input").unwrap(); //Input is required so we can unwrap
    let mut output = "";
    let mut verify = matches.is_present("verify");

    let in_path = Path::new(&input);

    if matches.is_present("output") {
        output = matches.value_of("output").unwrap();
        let out_path = Path::new(output);

        match out_path.parent() {
            Some(parent) => {
                if parent.exists() {
                    match out_path.extension() {
                        Some(_) => {}
                        None => {
                            println!("Output has to be a file.");
                            return;
                        }
                    }
                }
            }
            None => {
                println!("Invalid output path.");
                return;
            }
        }
    } else {
        verify = true;
    }

    let regex = Regex::new(r"\.(hevc|.?265)").unwrap();

    if input == "-" || (regex.is_match(&input) && in_path.is_file()) {
        let final_metadata: Vec<Metadata>;

        match parse_metadata(input, verify) {
            Ok(vec) => {
                //Match returned vec to check for --verify
                if vec[0][0] == 1 && vec[0].len() == 1 {
                    println!("Dynamic HDR10+ metadata detected.");
                    return;
                } else {
                    final_metadata = llc_read_metadata(vec);
                    //Sucessful parse & no --verify
                    if !final_metadata.is_empty() {
                        write_json(output, final_metadata)
                    } else {
                        println!("Failed reading parsed metadata.");
                    }
                }
            }
            Err(e) => println!("{}", e),
        }
    } else if !in_path.is_file() {
        println!("File path not found.");
    } else {
        println!("Invalid file type.");
    }
}
