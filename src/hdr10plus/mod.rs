use indicatif::{ProgressBar, ProgressStyle};
use std::{fs::File, path::Path};

pub mod metadata;
pub mod parser;

const TOOL_NAME: &str = env!("CARGO_PKG_NAME");
const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq)]
pub enum Format {
    Raw,
    RawStdin,
    Matroska,
}

#[derive(Debug)]
pub struct RpuOptions {
    pub mode: Option<u8>,
    pub crop: bool,
}

pub fn initialize_progress_bar(format: &Format, input: &Path) -> ProgressBar {
    let pb: ProgressBar;
    let bytes_count;

    if let Format::RawStdin = format {
        pb = ProgressBar::hidden();
    } else {
        let file = File::open(input).expect("No file found");

        //Info for indicatif ProgressBar
        let file_meta = file.metadata();
        bytes_count = file_meta.unwrap().len() / 100_000_000;

        pb = ProgressBar::new(bytes_count);
        pb.set_style(
            ProgressStyle::default_bar().template("[{elapsed_precise}] {bar:60.cyan} {percent}%"),
        );
    }

    pb
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
