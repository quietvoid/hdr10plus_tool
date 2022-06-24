use std::fs::File;
use std::io::{stdout, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use anyhow::{bail, Result};
use indicatif::ProgressBar;

use hevc_parser::hevc::{Frame, NALUnit, NAL_SEI_PREFIX};
use hevc_parser::io::{processor, IoFormat, IoProcessor};
use hevc_parser::HevcParser;
use processor::{HevcProcessor, HevcProcessorOpts};

use hdr10plus::metadata::Hdr10PlusMetadata;
use hdr10plus::metadata_json::generate_json;

use crate::CliOptions;

use super::{is_st2094_40_sei, ParserError};

pub const TOOL_NAME: &str = env!("CARGO_PKG_NAME");
pub const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Parser {
    input: PathBuf,
    output: Option<PathBuf>,

    options: CliOptions,
    progress_bar: ProgressBar,

    hdr10plus_sei_list: Vec<Vec<u8>>,
}

#[derive(Clone)]
pub struct MetadataFrame {
    pub decoded_index: usize,
    pub presentation_number: usize,
    pub metadata: Hdr10PlusMetadata,
}

impl Parser {
    pub fn new(
        input: PathBuf,
        output: Option<PathBuf>,
        options: CliOptions,
        progress_bar: ProgressBar,
    ) -> Self {
        Self {
            input,
            output,
            options,
            progress_bar,
            hdr10plus_sei_list: Vec::new(),
        }
    }

    pub fn process_input(&mut self, format: &IoFormat) -> Result<()> {
        let chunk_size = 100_000;

        let processor_opts = HevcProcessorOpts {
            parse_nals: true,
            ..Default::default()
        };
        let mut processor = HevcProcessor::new(format.clone(), processor_opts, chunk_size);

        let stdin = std::io::stdin();
        let mut reader = Box::new(stdin.lock()) as Box<dyn BufRead>;

        if let IoFormat::Raw = format {
            let file = File::open(&self.input)?;
            reader = Box::new(BufReader::with_capacity(100_000, file));
        }

        processor.process_io(&mut reader, self)
    }

    pub fn add_hdr10plus_sei(&mut self, nals: &[NALUnit], chunk: &[u8]) -> Result<()> {
        for nal in nals {
            if let NAL_SEI_PREFIX = nal.nal_type {
                let sei_payload = &chunk[nal.start..nal.end];

                if is_st2094_40_sei(sei_payload, self.options.validate)? {
                    self.hdr10plus_sei_list.push(sei_payload[4..].to_vec());
                }
            }
        }

        if self.hdr10plus_sei_list.is_empty() {
            bail!(ParserError::NoMetadataFound);
        } else if self.options.verify {
            bail!(ParserError::MetadataDetected);
        }

        Ok(())
    }

    pub fn parse_metadata_list(&mut self) -> Result<Vec<MetadataFrame>> {
        print!("Reading parsed dynamic metadata... ");
        stdout().flush().ok();

        let mut complete_metadata: Vec<MetadataFrame> = Vec::new();

        //Loop over lines and read metadata, HDR10+ LLC format
        for (index, data) in self.hdr10plus_sei_list.iter().enumerate() {
            let bytes = hevc_parser::utils::clear_start_code_emulation_prevention_3_byte(data);

            // Parse metadata
            let metadata = Hdr10PlusMetadata::parse(bytes)?;

            // Validate values
            if self.options.validate {
                metadata.validate()?;
            }

            let metadata_frame = MetadataFrame {
                decoded_index: index,
                presentation_number: 0,
                metadata,
            };

            complete_metadata.push(metadata_frame);
        }

        println!("Done.");

        Ok(complete_metadata)
    }

    fn write_json(&self, metadata: Vec<MetadataFrame>) -> Result<()> {
        match &self.output {
            Some(path) => {
                let save_file = File::create(path).expect("Can't create file");
                let mut writer = BufWriter::with_capacity(10_000_000, save_file);

                print!("Generating and writing metadata to JSON file... ");
                stdout().flush().ok();

                let list: Vec<&Hdr10PlusMetadata> =
                    metadata.iter().map(|mf| &mf.metadata).collect();
                let final_json = generate_json(&list, TOOL_NAME, TOOL_VERSION);

                writeln!(writer, "{}", serde_json::to_string_pretty(&final_json)?)?;

                println!("Done.");

                writer.flush()?;
            }
            None => bail!("Output path required!"),
        }

        Ok(())
    }

    pub fn reorder_metadata(&self, frames: &[Frame], metadata: &mut [MetadataFrame]) {
        print!("Reordering metadata... ");
        stdout().flush().ok();

        metadata.sort_by_cached_key(|m| {
            let matching_index = frames
                .iter()
                .position(|f| m.decoded_index == f.decoded_number as usize);

            if let Some(i) = matching_index {
                frames[i].presentation_number
            } else {
                panic!(
                    "Missing frame/slices for metadata! Decoded index {}",
                    m.decoded_index
                );
            }
        });

        metadata
            .iter_mut()
            .enumerate()
            .for_each(|(idx, m)| m.presentation_number = idx);

        println!("Done.");
    }
}

impl IoProcessor for Parser {
    fn input(&self) -> &PathBuf {
        &self.input
    }

    fn update_progress(&mut self, delta: u64) {
        self.progress_bar.inc(delta);
    }

    fn process_nals(&mut self, _parser: &HevcParser, nals: &[NALUnit], chunk: &[u8]) -> Result<()> {
        self.add_hdr10plus_sei(nals, chunk)
    }

    fn finalize(&mut self, parser: &HevcParser) -> Result<()> {
        self.progress_bar.finish_and_clear();

        if self.hdr10plus_sei_list.is_empty() {
            bail!(ParserError::NoMetadataFound);
        }

        let mut final_metadata = self.parse_metadata_list()?;

        // Sucessful parse & no --verify
        if !final_metadata.is_empty() {
            let frames = parser.ordered_frames();

            // Reorder to display output order
            self.reorder_metadata(frames, &mut final_metadata);

            self.write_json(final_metadata)?;
        } else {
            bail!("Failed reading parsed metadata.");
        }

        Ok(())
    }
}
