pub mod extract;
pub mod inject;

use clap::Parser;
use std::path::PathBuf;

use super::core::{initialize_progress_bar, parser};
use super::{input_format, Format};

#[derive(Parser, Debug)]
pub enum Command {
    Extract {
        #[clap(
            short = 'i',
            long,
            help = "Sets the input file to use",
            long,
            conflicts_with = "stdin",
            parse(from_os_str)
        )]
        input: Option<PathBuf>,

        #[clap(help = "Uses stdin as input data", parse(from_os_str))]
        stdin: Option<PathBuf>,

        #[clap(
            long,
            short = 'o',
            help = "Sets the output JSON file to use",
            parse(from_os_str)
        )]
        output: Option<PathBuf>,
    },

    Inject {
        #[clap(
            long,
            short = 'i',
            help = "Sets the input HEVC file to use",
            parse(from_os_str)
        )]
        input: PathBuf,

        #[clap(
            long,
            short = 'j',
            help = "Sets the input JSON file to use",
            parse(from_os_str)
        )]
        json: PathBuf,

        #[clap(
            long,
            short = 'o',
            help = "Output HEVC file location",
            parse(from_os_str)
        )]
        output: Option<PathBuf>,
    },
}
