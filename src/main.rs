use anyhow::Result;
use clap::Parser;

mod commands;
mod core;

use commands::extract::Extractor;
use commands::inject::Injector;
use commands::remove::Remover;
use commands::Command;

use crate::core::ParserError;

#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"), about = "Parses HDR10+ dynamic metadata in HEVC video files", author = "quietvoid", version = env!("CARGO_PKG_VERSION"))]
struct Opt {
    #[arg(long, help = "Checks if input file contains dynamic metadata")]
    verify: bool,

    #[arg(long, help = "Skip profile conformity validation")]
    skip_validation: bool,

    #[command(subcommand)]
    cmd: Command,
}

#[derive(Default)]
pub struct CliOptions {
    pub verify: bool,
    pub validate: bool,
}

fn main() -> Result<()> {
    let opt = Opt::parse();

    let cli_options = CliOptions {
        verify: opt.verify,
        validate: !opt.skip_validation,
    };

    let res = match opt.cmd {
        Command::Extract(args) => Extractor::extract_json(args, cli_options),
        Command::Inject(args) => Injector::inject_json(args, cli_options),
        Command::Remove(args) => Remover::remove_sei(args, cli_options),
    };

    let actually_errored = if let Err(e) = &res {
        let err_str = e.to_string();

        if err_str == ParserError::MetadataDetected.to_string() {
            println!("{}", err_str);
            false
        } else {
            true
        }
    } else {
        false
    };

    if actually_errored {
        res
    } else {
        Ok(())
    }
}
