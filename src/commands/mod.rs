use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{Args, Parser, ValueHint};
use hdr10plus::metadata::PeakBrightnessSource;

use crate::CliOptions;

pub mod editor;
pub mod extract;
pub mod inject;
pub mod plot;
pub mod remove;

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgPeakBrightnessSource {
    /// The max value from the histogram measurements
    Histogram,
    /// The last percentile in the histogram, usually 99.98% brightness percentile
    Histogram99,
    /// The max value in `maxscl`
    MaxScl,
    /// The luminance calculated from the `maxscl` components
    /// Assumed BT.2020 primaries
    MaxSclLuminance,
}

#[derive(Parser, Debug)]
pub enum Command {
    #[command(about = "Extracts the HDR10+ metadata from HEVC SEI messages to a JSON file")]
    Extract(ExtractArgs),

    #[command(
        about = "Interleaves HDR10+ metadata NAL units before slices in an HEVC encoded bitstream"
    )]
    Inject(InjectArgs),

    #[command(about = "Removes HDR10+ metadata SEI messages in an HEVC encoded bitstream")]
    Remove(RemoveArgs),

    #[command(about = "Plot the HDR10+ dynamic brightness metadata")]
    Plot(PlotArgs),

    #[command(about = "Edit the HDR10+ metadata")]
    Editor(EditorArgs),
}

#[derive(Args, Debug)]
pub struct ExtractArgs {
    #[arg(
        id = "input",
        help = "Sets the input HEVC file to use, or piped with -",
        long,
        short = 'i',
        conflicts_with = "input_pos",
        required_unless_present = "input_pos",
        value_hint = ValueHint::FilePath,
    )]
    pub input: Option<PathBuf>,

    #[arg(
        id = "input_pos",
        help = "Sets the input HEVC file to use, or piped with - (positional)",
        conflicts_with = "input",
        required_unless_present = "input",
        value_hint = ValueHint::FilePath
    )]
    pub input_pos: Option<PathBuf>,

    #[arg(
        long,
        short = 'o',
        help = "Sets the output JSON file to use",
        value_hint = ValueHint::FilePath
    )]
    pub output: Option<PathBuf>,

    #[arg(
        long,
        help = "Skip metadata reordering, workaround for misauthored HEVC files"
    )]
    skip_reorder: bool,

    #[arg(
        id = "limit",
        long,
        short = 'l',
        help = "Stop processing input after N frames"
    )]
    pub limit: Option<u64>,
}

#[derive(Args, Debug)]
pub struct InjectArgs {
    #[arg(
        id = "input",
        help = "Sets the input HEVC file to use",
        long,
        short = 'i',
        conflicts_with = "input_pos",
        required_unless_present = "input_pos",
        value_hint = ValueHint::FilePath,
    )]
    pub input: Option<PathBuf>,

    #[arg(
        id = "input_pos",
        help = "Sets the input HEVC file to use (positional)",
        conflicts_with = "input",
        required_unless_present = "input",
        value_hint = ValueHint::FilePath
    )]
    pub input_pos: Option<PathBuf>,

    #[arg(
        long,
        short = 'j',
        help = "Sets the input JSON file to use",
        value_hint = ValueHint::FilePath
    )]
    pub json: PathBuf,

    #[arg(
        long,
        short = 'o',
        help = "Output HEVC file location",
        value_hint = ValueHint::FilePath
    )]
    pub output: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
    #[arg(
        id = "input",
        help = "Sets the input HEVC file to use, or piped with -",
        long,
        short = 'i',
        conflicts_with = "input_pos",
        required_unless_present = "input_pos",
        value_hint = ValueHint::FilePath,
    )]
    pub input: Option<PathBuf>,

    #[arg(
        id = "input_pos",
        help = "Sets the input HEVC file to use, or piped with - (positional)",
        conflicts_with = "input",
        required_unless_present = "input",
        value_hint = ValueHint::FilePath
    )]
    pub input_pos: Option<PathBuf>,

    #[arg(
        long,
        short = 'o',
        help = "Sets the output HEVC file to use",
        value_hint = ValueHint::FilePath
    )]
    pub output: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct PlotArgs {
    #[arg(
        id = "input",
        help = "Sets the input JSON file to use",
        long,
        short = 'i',
        conflicts_with = "input_pos",
        required_unless_present = "input_pos",
        value_hint = ValueHint::FilePath,
    )]
    pub input: Option<PathBuf>,

    #[arg(
        id = "input_pos",
        help = "Sets the input JSON file to use (positional)",
        conflicts_with = "input",
        required_unless_present = "input",
        value_hint = ValueHint::FilePath
    )]
    pub input_pos: Option<PathBuf>,

    #[arg(
        long,
        short = 'o',
        help = "Output PNG image file location",
        value_hint = ValueHint::FilePath
    )]
    pub output: Option<PathBuf>,

    #[arg(long, short = 't', help = "Title to use at the top")]
    pub title: Option<String>,

    #[arg(
        value_enum,
        short = 'p',
        long,
        help = "How to extract the peak brightness for the metadata",
        default_value = "histogram"
    )]
    pub peak_source: ArgPeakBrightnessSource,

    #[arg(long, short = 's', help = "Set frame range start")]
    pub start: Option<usize>,

    #[arg(long, short = 'e', help = "Set frame range end (inclusive)")]
    pub end: Option<usize>,
}

#[derive(Args, Debug)]
pub struct EditorArgs {
    #[arg(
        id = "input",
        help = "Sets the input JSON file to use",
        long,
        short = 'i',
        conflicts_with = "input_pos",
        required_unless_present = "input_pos",
        value_hint = ValueHint::FilePath,
    )]
    pub input: Option<PathBuf>,

    #[arg(
        id = "input_pos",
        help = "Sets the input JSON file to use (positional)",
        conflicts_with = "input",
        required_unless_present = "input",
        value_hint = ValueHint::FilePath
    )]
    pub input_pos: Option<PathBuf>,

    #[arg(
        id = "json",
        long,
        short = 'j',
        help = "Sets the edit JSON file to use",
        value_hint = ValueHint::FilePath
    )]
    pub edits_json: PathBuf,

    #[arg(
        long,
        short = 'o',
        help = "Modified JSON output file location",
        value_hint = ValueHint::FilePath
    )]
    pub json_out: Option<PathBuf>,
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

impl From<ArgPeakBrightnessSource> for PeakBrightnessSource {
    fn from(e: ArgPeakBrightnessSource) -> Self {
        match e {
            ArgPeakBrightnessSource::Histogram => Self::Histogram,
            ArgPeakBrightnessSource::Histogram99 => Self::Histogram99,
            ArgPeakBrightnessSource::MaxScl => Self::MaxScl,
            ArgPeakBrightnessSource::MaxSclLuminance => Self::MaxSclLuminance,
        }
    }
}
