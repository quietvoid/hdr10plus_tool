use regex::Regex;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

mod commands;
mod hdr10plus;

use commands::{extract, inject, Command};
use extract::extract_json;
use inject::Injector;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "hdr10plus_parser",
    about = "Parses HDR10+ dynamic metadata in HEVC video files"
)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Option<Command>,

    #[structopt(
        name = "input",
        short = "i",
        long,
        help = "Sets the input file to use",
        long,
        conflicts_with = "stdin",
        conflicts_with = "cmd",
        parse(from_os_str)
    )]
    input: Option<PathBuf>,

    #[structopt(
        help = "Uses stdin as input data",
        conflicts_with = "input",
        conflicts_with = "cmd",
        parse(from_os_str)
    )]
    stdin: Option<PathBuf>,

    #[structopt(
        short = "o",
        long,
        help = "Sets the output JSON file to use",
        conflicts_with = "cmd",
        parse(from_os_str)
    )]
    output: Option<PathBuf>,

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

    let verify = opt.verify || opt.output.is_none();
    let validate = !opt.skip_validation;

    if let Some(cmd) = opt.cmd {
        match cmd {
            Command::Extract {
                input,
                stdin,
                output,
            } => extract_json(input, stdin, output, verify, validate),
            Command::Inject {
                input,
                json_in,
                output,
            } => Injector::run(input, json_in, output, validate),
        }
    } else {
        extract_json(opt.input, opt.stdin, opt.output, verify, validate);
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
