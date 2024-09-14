use anyhow::{bail, Result};
use hevc_parser::io::IoFormat;

use super::{input_from_either, CliOptions, ExtractArgs};
use crate::core::initialize_progress_bar;
use crate::core::parser::{Parser, ParserOptions};

pub struct Extractor {}

impl Extractor {
    pub fn extract_json(args: ExtractArgs, mut options: CliOptions) -> Result<()> {
        let ExtractArgs {
            input,
            input_pos,
            output,
            skip_reorder,
            limit,
        } = args;
        let input = input_from_either("extract", input, input_pos)?;

        let format = hevc_parser::io::format_from_path(&input)?;

        if format == IoFormat::Matroska {
            bail!("Extractor: Matroska format unsupported");
        }

        if !options.verify && output.is_none() {
            options.verify = true
        };

        let pb = initialize_progress_bar(&format, &input)?;
        let mut parser = Parser::new(
            input,
            output,
            options,
            pb,
            skip_reorder,
            ParserOptions { limit },
        );

        parser.process_input(&format)
    }
}
