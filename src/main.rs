use regex::Regex;
use std::path::Path;
use structopt::StructOpt;

mod commands;
mod core;

use commands::{extract, inject, Command};
use extract::extract_json;
use inject::Injector;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "hdr10plus_tool",
    about = "Parses HDR10+ dynamic metadata in HEVC video files"
)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,

    #[structopt(long, help = "Checks if input file contains dynamic metadata")]
    verify: bool,

    #[structopt(long, help = "Skip profile conformity validation")]
    skip_validation: bool,
}

#[derive(Debug, PartialEq)]
pub enum Format {
    Raw,
    RawStdin,
    Matroska,
}

fn main() {
    let opt = Opt::from_args();

    let verify = opt.verify;
    let validate = !opt.skip_validation;

    match opt.cmd {
        Command::Extract {
            input,
            stdin,
            output,
        } => extract_json(input, stdin, output, verify, validate),
        Command::Inject {
            input,
            json,
            output,
        } => Injector::run(input, json, output, validate),
    }
}

fn input_format(input: &Path) -> Result<Format, &str> {
    let regex = Regex::new(r"\.(hevc|.?265|mkv)").unwrap();
    let file_name = match input.file_name() {
        Some(file_name) => file_name.to_str().unwrap(),
        None => "",
    };

    if file_name == "-" {
        Ok(Format::RawStdin)
    } else if regex.is_match(file_name) && input.is_file() {
        if file_name.ends_with(".mkv") {
            Ok(Format::Matroska)
        } else {
            Ok(Format::Raw)
        }
    } else if file_name.is_empty() {
        Err("Missing input.")
    } else if !input.is_file() {
        Err("Input file doesn't exist.")
    } else {
        Err("Invalid input file type.")
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Format::Matroska => write!(f, "Matroska file"),
            Format::Raw => write!(f, "HEVC file"),
            Format::RawStdin => write!(f, "HEVC pipe"),
        }
    }
}
