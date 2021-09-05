use std::path::PathBuf;

use super::hdr10plus::parser::Parser;
use super::input_format;

pub fn extract_json(
    input: Option<PathBuf>,
    stdin: Option<PathBuf>,
    output: Option<PathBuf>,
    verify: bool,
    validate: bool,
) {
    let input = match input {
        Some(input) => input,
        None => match stdin {
            Some(stdin) => stdin,
            None => PathBuf::new(),
        },
    };

    match input_format(&input) {
        Ok(format) => {
            let verify_default = if output.is_none() {
                true
            } else {
                verify
            };

            let parser = Parser::new(format, input, output, verify_default, validate);
            parser.process_input();
        }
        Err(msg) => println!("{}", msg),
    }
}
