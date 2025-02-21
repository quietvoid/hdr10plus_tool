use std::fs::File;
use std::io::{BufWriter, Write, stdout};
use std::path::PathBuf;

use anyhow::{Result, bail, ensure};
use hevc_parser::utils::{
    add_start_code_emulation_prevention_3_byte, clear_start_code_emulation_prevention_3_byte,
};
use indicatif::ProgressBar;

use hevc_parser::HevcParser;
use hevc_parser::hevc::{Frame, NAL_SEI_PREFIX, NALUnit};
use hevc_parser::io::{IoFormat, IoProcessor, processor};
use processor::{HevcProcessor, HevcProcessorOpts};

use hdr10plus::metadata::Hdr10PlusMetadata;
use hdr10plus::metadata_json::generate_json;

use crate::CliOptions;

use super::{ParserError, st2094_40_sei_msg};

pub const TOOL_NAME: &str = env!("CARGO_PKG_NAME");
pub const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Parser {
    input: PathBuf,
    output: Option<PathBuf>,

    options: CliOptions,
    progress_bar: ProgressBar,

    hdr10plus_sei_list: Vec<MetadataFrame>,
    skip_reorder: bool,

    parser_opts: ParserOptions,
}

#[derive(Debug, Clone)]
pub struct MetadataFrame {
    pub decoded_index: u64,
    pub presentation_number: usize,
    pub metadata: Option<Vec<u8>>,
}

#[derive(Default)]
pub struct ParserOptions {
    pub limit: Option<u64>,
}

impl Parser {
    pub fn new(
        input: PathBuf,
        output: Option<PathBuf>,
        options: CliOptions,
        progress_bar: ProgressBar,
        skip_reorder: bool,
        parser_opts: ParserOptions,
    ) -> Self {
        Self {
            input,
            output,
            options,
            progress_bar,
            hdr10plus_sei_list: Vec::new(),
            skip_reorder,
            parser_opts,
        }
    }

    pub fn process_input(&mut self, format: &IoFormat) -> Result<()> {
        let chunk_size = 100_000;

        let processor_opts = HevcProcessorOpts {
            parse_nals: true,
            limit: self.parser_opts.limit,
            ..Default::default()
        };
        let mut processor = HevcProcessor::new(format.clone(), processor_opts, chunk_size);

        let file_path = if let IoFormat::RawStdin = format {
            None
        } else {
            Some(self.input.clone())
        };

        processor.process_file(self, file_path)
    }

    pub fn add_hdr10plus_sei(&mut self, nals: &[NALUnit], chunk: &[u8]) -> Result<()> {
        for nal in nals {
            if let NAL_SEI_PREFIX = nal.nal_type {
                let sei_payload =
                    clear_start_code_emulation_prevention_3_byte(&chunk[nal.start..nal.end]);

                if let Some(msg) = st2094_40_sei_msg(&sei_payload, self.options.validate)? {
                    let start = msg.payload_offset;
                    let end = start + msg.payload_size;

                    // Re-add removed bytes
                    let mut bytes = sei_payload[start..end].to_vec();
                    add_start_code_emulation_prevention_3_byte(&mut bytes);

                    self.hdr10plus_sei_list.push(MetadataFrame {
                        decoded_index: nal.decoded_frame_index,
                        presentation_number: 0,
                        metadata: Some(bytes),
                    });
                }
            }

            if let Some(last_meta) = self.hdr10plus_sei_list.last() {
                // Slice and no metadata for this index, means there was nothing in SEI prefix
                if nal.is_slice() && last_meta.decoded_index < nal.decoded_frame_index {
                    self.hdr10plus_sei_list.push(MetadataFrame {
                        decoded_index: nal.decoded_frame_index,
                        presentation_number: 0,
                        metadata: None,
                    });
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

    pub fn parse_metadata_list(&self, sei_list: &Vec<&Vec<u8>>) -> Result<Vec<Hdr10PlusMetadata>> {
        print!("Reading parsed dynamic metadata... ");
        stdout().flush().ok();

        let mut complete_metadata = Vec::new();

        //Loop over lines and read metadata, HDR10+ LLC format
        for data in sei_list {
            let bytes = hevc_parser::utils::clear_start_code_emulation_prevention_3_byte(data);

            // Parse metadata
            let metadata = Hdr10PlusMetadata::parse(&bytes)?;

            // Validate values
            if self.options.validate {
                metadata.validate()?;
            }

            complete_metadata.push(metadata);
        }

        println!("Done.");

        Ok(complete_metadata)
    }

    fn write_json(&self, metadata: Vec<Hdr10PlusMetadata>) -> Result<()> {
        match &self.output {
            Some(path) => {
                let save_file = File::create(path).expect("Can't create file");
                let mut writer = BufWriter::with_capacity(10_000_000, save_file);

                print!("Generating and writing metadata to JSON file... ");
                stdout().flush().ok();

                let list: Vec<&Hdr10PlusMetadata> = metadata.iter().collect();
                let final_json = generate_json(&list, TOOL_NAME, TOOL_VERSION);

                writeln!(writer, "{}", serde_json::to_string_pretty(&final_json)?)?;

                println!("Done.");

                writer.flush()?;
            }
            None => bail!("Output path required!"),
        }

        Ok(())
    }

    pub fn reorder_metadata(&mut self, frames: &[Frame]) {
        print!("Reordering metadata... ");
        stdout().flush().ok();

        self.hdr10plus_sei_list.sort_by_cached_key(|m| {
            let matching_index = frames
                .iter()
                .position(|f| m.decoded_index == f.decoded_number);

            if let Some(i) = matching_index {
                frames[i].presentation_number
            } else {
                panic!(
                    "Missing frame/slices for metadata! Decoded index {}",
                    m.decoded_index
                );
            }
        });

        self.hdr10plus_sei_list
            .iter_mut()
            .enumerate()
            .for_each(|(idx, m)| {
                m.presentation_number = idx;
            });

        println!("Done.");
    }

    pub fn fill_metadata_gaps(&mut self) {
        print!("Filling metadata gaps... ");
        stdout().flush().ok();

        let present_meta_list: Vec<(usize, Vec<u8>)> = self
            .hdr10plus_sei_list
            .iter()
            .enumerate()
            .filter(|(_, e)| e.metadata.is_some())
            .map(|(idx, e)| (idx, e.metadata.as_ref().unwrap().clone()))
            .collect();

        for (idx, bytes) in present_meta_list {
            self.hdr10plus_sei_list
                .iter_mut()
                .skip(idx + 1)
                .take_while(|e| e.metadata.is_none())
                .for_each(|e| e.metadata = Some(bytes.clone()));
        }

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

        let frames = parser.ordered_frames();

        // Some NALUs may have been added without having parsed the full AU or a slice
        if self.parser_opts.limit.is_some() {
            self.hdr10plus_sei_list.truncate(frames.len());
        }

        ensure!(self.hdr10plus_sei_list.len() == frames.len());

        let has_metadata_gaps = self.hdr10plus_sei_list.iter().any(|e| e.metadata.is_none());

        // Same behaviour as FFmpeg
        // Use metadata from previous SEI by decode order and reorder after
        if has_metadata_gaps {
            self.fill_metadata_gaps();
        }

        // Reorder to display output order
        if !self.skip_reorder {
            self.reorder_metadata(frames);
        }

        let ordered_sei_list = self
            .hdr10plus_sei_list
            .iter()
            .map(|e| e.metadata.as_ref().unwrap())
            .collect();
        let final_metadata = self.parse_metadata_list(&ordered_sei_list)?;

        // Sucessful parse & no --verify
        if !final_metadata.is_empty() {
            self.write_json(final_metadata)?;
        } else {
            bail!("Failed reading parsed metadata.");
        }

        Ok(())
    }
}
