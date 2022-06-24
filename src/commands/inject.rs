use std::fs::File;
use std::io::{stdout, BufReader, BufWriter, Write};
use std::path::PathBuf;

use anyhow::{bail, Result};
use indicatif::ProgressBar;

use hevc_parser::io::{processor, FrameBuffer, IoFormat, IoProcessor, NalBuffer};
use hevc_parser::{hevc::*, HevcParser, NALUStartCode};
use processor::{HevcProcessor, HevcProcessorOpts};

use hdr10plus::metadata_json::{Hdr10PlusJsonMetadata, MetadataJsonRoot};

use crate::commands::InjectArgs;
use crate::core::{initialize_progress_bar, is_st2094_40_sei};

use super::{input_from_either, CliOptions};

pub struct Injector {
    input: PathBuf,
    json_in: PathBuf,
    options: CliOptions,

    metadata_list: Vec<Hdr10PlusJsonMetadata>,

    writer: BufWriter<File>,
    progress_bar: ProgressBar,
    already_checked_for_hdr10plus: bool,

    frames: Vec<Frame>,
    nals: Vec<NALUnit>,
    mismatched_length: bool,

    frame_buffer: FrameBuffer,
    last_metadata_written: Option<NalBuffer>,
}

impl Injector {
    pub fn from_args(args: InjectArgs, cli_options: CliOptions) -> Result<Self> {
        let InjectArgs {
            input,
            input_pos,
            json,
            output,
        } = args;

        let input = input_from_either("inject", input, input_pos)?;

        let output = match output {
            Some(path) => path,
            None => PathBuf::from("injected_output.hevc"),
        };

        let chunk_size = 100_000;
        let progress_bar = initialize_progress_bar(&IoFormat::Raw, &input)?;

        let writer = BufWriter::with_capacity(
            chunk_size,
            File::create(&output).expect("Can't create file"),
        );

        let mut injector = Injector {
            input,
            json_in: json,
            options: cli_options,
            metadata_list: Vec::new(),

            writer,
            progress_bar,
            already_checked_for_hdr10plus: false,

            frames: Vec::new(),
            nals: Vec::new(),
            mismatched_length: false,

            frame_buffer: FrameBuffer {
                frame_number: 0,
                nals: Vec::with_capacity(16),
            },
            last_metadata_written: None,
        };

        println!("Parsing JSON file...");
        stdout().flush().ok();

        let metadata_root = MetadataJsonRoot::from_file(&injector.json_in)?;
        injector.metadata_list = metadata_root.scene_info;

        if injector.metadata_list.is_empty() {
            bail!("Empty HDR10+ SceneInfo array");
        }

        Ok(injector)
    }

    pub fn inject_json(args: InjectArgs, cli_options: CliOptions) -> Result<()> {
        let input = input_from_either("inject", args.input.clone(), args.input_pos.clone())?;
        let format = hevc_parser::io::format_from_path(&input)?;

        if let IoFormat::Raw = format {
            let mut injector = Injector::from_args(args, cli_options)?;

            injector.process_input()?;
            injector.interleave_hdr10plus_nals()
        } else {
            bail!("Injector: Must be a raw HEVC bitstream file")
        }
    }

    fn process_input(&mut self) -> Result<()> {
        println!("Processing input video for frame order info...");
        stdout().flush().ok();

        let chunk_size = 100_000;

        let mut processor =
            HevcProcessor::new(IoFormat::Raw, HevcProcessorOpts::default(), chunk_size);

        let file = File::open(&self.input)?;
        let mut reader = Box::new(BufReader::with_capacity(100_000, file));

        processor.process_io(&mut reader, self)
    }

    fn interleave_hdr10plus_nals(&mut self) -> Result<()> {
        let metadata_list = &self.metadata_list;
        self.mismatched_length = if self.frames.len() != metadata_list.len() {
            println!(
                "\nWarning: mismatched lengths. video {}, HDR10+ JSON {}",
                self.frames.len(),
                metadata_list.len()
            );

            if metadata_list.len() < self.frames.len() {
                println!("Metadata will be duplicated at the end to match video length\n");
            } else {
                println!("Metadata will be skipped at the end to match video length\n");
            }

            true
        } else {
            false
        };

        println!("Rewriting file with interleaved HDR10+ SEI NALs..");
        stdout().flush().ok();

        self.progress_bar = initialize_progress_bar(&IoFormat::Raw, &self.input)?;

        let chunk_size = 100_000;

        let mut processor =
            HevcProcessor::new(IoFormat::Raw, HevcProcessorOpts::default(), chunk_size);

        let file = File::open(&self.input)?;
        let mut reader = Box::new(BufReader::with_capacity(chunk_size, file));

        processor.process_io(&mut reader, self)
    }

    fn get_metadata_and_index_to_insert(
        frames: &[Frame],
        metadata_list: &[Hdr10PlusJsonMetadata],
        frame_buffer: &FrameBuffer,
        mismatched_length: bool,
        last_metadata: &Option<NalBuffer>,
        validate: bool,
    ) -> Result<(usize, NalBuffer)> {
        let existing_frame = frames
            .iter()
            .find(|f| f.decoded_number == frame_buffer.frame_number);

        // If we have a metadata buffered frame, write it
        // Otherwise, write the same data as previous
        let hdr10plus_nb = if let Some(frame) = existing_frame {
            if let Some(ref mut meta) = metadata_list.get(frame.presentation_number as usize) {
                let hdr10plus_data = hdr10plus::hevc::encode_hevc_from_json(meta, validate)?;

                Some(NalBuffer {
                    nal_type: NAL_SEI_PREFIX,
                    start_code: NALUStartCode::Length4,
                    data: hdr10plus_data,
                })
            } else if mismatched_length {
                last_metadata.clone()
            } else {
                bail!(
                    "No metadata found for presentation frame {}",
                    frame.presentation_number
                );
            }
        } else if mismatched_length {
            last_metadata.clone()
        } else {
            None
        };

        if let Some(hdr10plus_nb) = hdr10plus_nb {
            // First slice
            let insert_index = frame_buffer
                .nals
                .iter()
                .position(|nb| NALUnit::is_type_slice(nb.nal_type));

            if let Some(idx) = insert_index {
                // we want the SEI before the slice
                Ok((idx, hdr10plus_nb))
            } else {
                bail!(
                    "No slice in decoded frame {}. Cannot insert HDR10+ SEI.",
                    frame_buffer.frame_number
                );
            }
        } else {
            bail!(
                "No HDR10+ SEI data to write for decoded frame {}",
                frame_buffer.frame_number
            );
        }
    }
}

impl IoProcessor for Injector {
    fn input(&self) -> &PathBuf {
        &self.input
    }

    fn update_progress(&mut self, delta: u64) {
        if !self.already_checked_for_hdr10plus {
            self.already_checked_for_hdr10plus = true;
        }

        self.progress_bar.inc(delta);
    }

    fn process_nals(&mut self, _parser: &HevcParser, nals: &[NALUnit], chunk: &[u8]) -> Result<()> {
        // Second pass
        if !self.frames.is_empty() && !self.nals.is_empty() {
            let metadata_list = &self.metadata_list;

            for nal in nals {
                if self.frame_buffer.frame_number != nal.decoded_frame_index {
                    let (idx, hdr10plus_nb) = Self::get_metadata_and_index_to_insert(
                        &self.frames,
                        metadata_list,
                        &self.frame_buffer,
                        self.mismatched_length,
                        &self.last_metadata_written,
                        self.options.validate,
                    )?;

                    self.last_metadata_written = Some(hdr10plus_nb.clone());
                    self.frame_buffer.nals.insert(idx, hdr10plus_nb);

                    // Write NALUs for the frame
                    for nal_buf in &self.frame_buffer.nals {
                        self.writer.write_all(NALUStartCode::Length4.slice())?;
                        self.writer.write_all(&nal_buf.data)?;
                    }

                    self.frame_buffer.frame_number = nal.decoded_frame_index;
                    self.frame_buffer.nals.clear();
                }

                let is_hdr10plus_sei = nal.nal_type == NAL_SEI_PREFIX
                    && is_st2094_40_sei(&chunk[nal.start..nal.end], self.options.validate)?;

                // Ignore existing HDR10+ SEI
                if !is_hdr10plus_sei {
                    self.frame_buffer.nals.push(NalBuffer {
                        nal_type: nal.nal_type,
                        start_code: nal.start_code,
                        data: chunk[nal.start..nal.end].to_vec(),
                    });
                }
            }
        } else if !self.already_checked_for_hdr10plus
            && nals.iter().any(|e| {
                e.nal_type == NAL_SEI_PREFIX
                    && is_st2094_40_sei(&chunk[e.start..e.end], self.options.validate)
                        .unwrap_or(false)
            })
        {
            self.already_checked_for_hdr10plus = true;
            println!("\nWarning: Input file already has HDR10+ SEIs, they will be replaced.");
        }

        Ok(())
    }

    fn finalize(&mut self, parser: &HevcParser) -> Result<()> {
        // First pass
        if self.frames.is_empty() && self.nals.is_empty() {
            self.frames = parser.ordered_frames().clone();
            self.nals = parser.get_nals().clone();
        } else {
            let ordered_frames = parser.ordered_frames();
            let total_frames = ordered_frames.len();

            // Last slice wasn't considered (no AUD/EOS NALU at the end)
            if (self.frame_buffer.frame_number as usize) != total_frames
                && !self.frame_buffer.nals.is_empty()
            {
                let metadata_list = &self.metadata_list;

                let (idx, hdr10plus_nb) = Self::get_metadata_and_index_to_insert(
                    &self.frames,
                    metadata_list,
                    &self.frame_buffer,
                    self.mismatched_length,
                    &self.last_metadata_written,
                    self.options.validate,
                )?;

                self.last_metadata_written = Some(hdr10plus_nb.clone());
                self.frame_buffer.nals.insert(idx, hdr10plus_nb);

                // Write NALUs for the last frame
                for nal_buf in &self.frame_buffer.nals {
                    self.writer.write_all(NALUStartCode::Length4.slice())?;
                    self.writer.write_all(&nal_buf.data)?;
                }

                self.frame_buffer.nals.clear();
            }

            // Second pass
            self.writer.flush()?;
        }

        self.progress_bar.finish_and_clear();

        Ok(())
    }
}
