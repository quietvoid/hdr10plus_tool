pub mod extract;
pub mod inject;

use std::path::PathBuf;
use structopt::StructOpt;

use super::{hdr10plus, input_format, Format};

#[derive(StructOpt, Debug)]
#[structopt(name = "hdr10plus_tool", about = "CLI utility to work with HDR10+ in HEVC files")]
pub enum Command {
    Extract {
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
            long,
            short = "o",
            help = "Sets the output JSON file to use",
            conflicts_with = "cmd",
            parse(from_os_str)
        )]
        output: Option<PathBuf>,
    },

    Inject {
        #[structopt(
            name = "input",
            long,
            short = "i",
            help = "Sets the input HEVC file to use",
            parse(from_os_str)
        )]
        input: PathBuf,

        #[structopt(
            long,
            short = "j",
            help = "Sets the input JSON file to use",
            parse(from_os_str)
        )]
        json_in: PathBuf,

        #[structopt(
            long,
            short = "o",
            help = "Output HEVC file location",
            parse(from_os_str)
        )]
        output: Option<PathBuf>,
    },
}
