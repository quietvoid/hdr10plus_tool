use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::{fs::File, path::Path};

use super::Format;

pub mod parser;

#[cfg(test)]
mod tests;

pub fn initialize_progress_bar(format: &Format, input: &Path) -> Result<ProgressBar> {
    let pb: ProgressBar;
    let bytes_count;

    if let Format::RawStdin = format {
        pb = ProgressBar::hidden();
    } else {
        let file = File::open(input).expect("No file found");

        //Info for indicatif ProgressBar
        let file_meta = file.metadata()?;
        bytes_count = file_meta.len() / 100_000_000;

        pb = ProgressBar::new(bytes_count);
        pb.set_style(
            ProgressStyle::default_bar().template("[{elapsed_precise}] {bar:60.cyan} {percent}%"),
        );
    }

    Ok(pb)
}
