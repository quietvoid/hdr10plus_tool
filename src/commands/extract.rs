use anyhow::Result;
use std::path::PathBuf;

use super::{input_format, parser::Parser};

pub fn extract_json(
    input: Option<PathBuf>,
    stdin: Option<PathBuf>,
    output: Option<PathBuf>,
    verify: bool,
    validate: bool,
) -> Result<()> {
    let input = match input {
        Some(input) => input,
        None => match stdin {
            Some(stdin) => stdin,
            None => PathBuf::new(),
        },
    };

    let format = input_format(&input)?;
    let verify_default = if output.is_none() { true } else { verify };

    let parser = Parser::new(format, input, output, verify_default, validate);

    parser.process_input()?;

    Ok(())
}
