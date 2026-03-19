use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write, stdout};
use std::path::Path;

use anyhow::{Result, bail};

use hdr10plus::metadata::Hdr10PlusMetadata;
use hdr10plus::metadata_json::generate_json;

use super::{CliOptions, ExtractArgs, input_from_either};
use crate::core::ParserError;
use crate::core::av1_parser::{
    Av1NaluParser, Obu, OBU_METADATA, extract_hdr10plus_t35_bytes,
    read_ivf_frame_header, read_obus_from_ivf_frame, try_read_ivf_file_header,
};
use crate::core::initialize_progress_bar;
use crate::core::parser::{Parser, ParserOptions, TOOL_NAME, TOOL_VERSION};

pub struct Extractor {}

fn is_av1_input(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("av1") | Some("ivf")
    )
}

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

        if is_av1_input(&input) {
            Self::extract_json_av1(input, output, options, limit)
        } else {
            let format = hevc_parser::io::format_from_path(&input)?;

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

    fn extract_json_av1(
        input: std::path::PathBuf,
        output: Option<std::path::PathBuf>,
        options: CliOptions,
        limit: Option<u64>,
    ) -> Result<()> {
        let file = File::open(&input)?;
        let mut reader = BufReader::with_capacity(100_000, file);

        let mut av1_nalu_parser = Av1NaluParser::new();
        let mut t35_frames: Vec<Vec<u8>> = Vec::new();
        let mut obu_count = 0u64;

        let is_ivf = try_read_ivf_file_header(&mut reader)?.is_some();

        if is_ivf {
            loop {
                let fh = match read_ivf_frame_header(&mut reader)? {
                    Some(h) => h,
                    None => break,
                };
                let mut frame_data = vec![0u8; fh.frame_size as usize];
                reader.read_exact(&mut frame_data)?;

                let obus = read_obus_from_ivf_frame(frame_data)?;
                for obu in &obus {
                    av1_nalu_parser.process_obu(obu)?;
                    if obu.obu_type == OBU_METADATA {
                        if let Some(t35) =
                            extract_hdr10plus_t35_bytes(&obu.payload, options.validate)
                        {
                            t35_frames.push(t35);
                        }
                    }
                    obu_count += 1;
                    if limit.map(|l| obu_count >= l).unwrap_or(false) {
                        break;
                    }
                }

                if limit.map(|l| obu_count >= l).unwrap_or(false) {
                    break;
                }
            }
        } else {
            loop {
                match Obu::read_from(&mut reader) {
                    Ok(Some(obu)) => {
                        av1_nalu_parser.process_obu(&obu)?;

                        if obu.obu_type == OBU_METADATA {
                            if let Some(t35) =
                                extract_hdr10plus_t35_bytes(&obu.payload, options.validate)
                            {
                                t35_frames.push(t35);
                            }
                        }

                        obu_count += 1;
                        if limit.map(|l| obu_count >= l).unwrap_or(false) {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => return Err(e),
                }
            }
        }

        if t35_frames.is_empty() {
            bail!(ParserError::NoMetadataFound);
        }

        if options.verify {
            bail!(ParserError::MetadataDetected);
        }

        print!("Reading parsed dynamic metadata... ");
        stdout().flush().ok();

        let mut complete_metadata: Vec<Hdr10PlusMetadata> = Vec::new();
        for t35_bytes in &t35_frames {
            let meta = Hdr10PlusMetadata::parse(t35_bytes)?;
            if options.validate {
                meta.validate()?;
            }
            complete_metadata.push(meta);
        }

        println!("Done.");

        match output {
            Some(path) => {
                let save_file = File::create(&path).expect("Can't create file");
                let mut writer = BufWriter::with_capacity(10_000_000, save_file);

                print!("Generating and writing metadata to JSON file... ");
                stdout().flush().ok();

                let list: Vec<&Hdr10PlusMetadata> = complete_metadata.iter().collect();
                let final_json = generate_json(&list, TOOL_NAME, TOOL_VERSION);

                writeln!(writer, "{}", serde_json::to_string_pretty(&final_json)?)?;
                writer.flush()?;

                println!("Done.");
            }
            None => bail!("Output path required for AV1 extraction"),
        }

        Ok(())
    }
}
