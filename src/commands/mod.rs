use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::{Args, Parser, ValueHint};

use crate::CliOptions;

pub mod extract;
pub mod inject;
pub mod remove;

#[derive(Parser, Debug)]
pub enum Command {
    #[clap(about = "Extracts the HDR10+ metadata from HEVC SEI messages to a JSON file")]
    Extract(ExtractArgs),

    #[clap(
        about = "Interleaves HDR10+ metadata NAL units before slices in an HEVC encoded bitstream"
    )]
    Inject(InjectArgs),

    #[clap(about = "Removes HDR10+ metadata SEI messages in an HEVC encoded bitstream")]
    Remove(RemoveArgs),
}

#[derive(Args, Debug)]
pub struct ExtractArgs {
    #[clap(
        name = "input",
        help = "Sets the input HEVC file to use, or piped with -",
        long,
        short = 'i',
        conflicts_with = "input_pos",
        required_unless_present = "input_pos",
        value_hint = ValueHint::FilePath,
    )]
    pub input: Option<PathBuf>,

    #[clap(
        name = "input_pos",
        help = "Sets the input HEVC file to use, or piped with - (positional)",
        conflicts_with = "input",
        required_unless_present = "input",
        value_hint = ValueHint::FilePath
    )]
    pub input_pos: Option<PathBuf>,

    #[clap(
        long,
        short = 'o',
        help = "Sets the output JSON file to use",
        value_hint = ValueHint::FilePath
    )]
    pub output: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct InjectArgs {
    #[clap(
        name = "input",
        help = "Sets the input HEVC file to use",
        long,
        short = 'i',
        conflicts_with = "input_pos",
        required_unless_present = "input_pos",
        value_hint = ValueHint::FilePath,
    )]
    pub input: Option<PathBuf>,

    #[clap(
        name = "input_pos",
        help = "Sets the input HEVC file to use (positional)",
        conflicts_with = "input",
        required_unless_present = "input",
        value_hint = ValueHint::FilePath
    )]
    pub input_pos: Option<PathBuf>,

    #[clap(
        long,
        short = 'j',
        help = "Sets the input JSON file to use",
        value_hint = ValueHint::FilePath
    )]
    pub json: PathBuf,

    #[clap(
        long,
        short = 'o',
        help = "Output HEVC file location",
        value_hint = ValueHint::FilePath
    )]
    pub output: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
    #[clap(
        name = "input",
        help = "Sets the input HEVC file to use, or piped with -",
        long,
        short = 'i',
        conflicts_with = "input_pos",
        required_unless_present = "input_pos",
        value_hint = ValueHint::FilePath,
    )]
    pub input: Option<PathBuf>,

    #[clap(
        name = "input_pos",
        help = "Sets the input HEVC file to use, or piped with - (positional)",
        conflicts_with = "input",
        required_unless_present = "input",
        value_hint = ValueHint::FilePath
    )]
    pub input_pos: Option<PathBuf>,

    #[clap(
        long,
        short = 'o',
        help = "Sets the output HEVC file to use",
        value_hint = ValueHint::FilePath
    )]
    pub output: Option<PathBuf>,
}

pub fn input_from_either(cmd: &str, in1: Option<PathBuf>, in2: Option<PathBuf>) -> Result<PathBuf> {
    match in1 {
        Some(in1) => Ok(in1),
        None => match in2 {
            Some(in2) => Ok(in2),
            None => bail!(
                "No input file provided. See `hdr10plus_tool {} --help`",
                cmd
            ),
        },
    }
}
