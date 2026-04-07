use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write, stdout};
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use hevc_parser::utils::{
    add_start_code_emulation_prevention_3_byte, clear_start_code_emulation_prevention_3_byte,
};
use indicatif::ProgressBar;

use hevc_parser::io::{FrameBuffer, IoFormat, IoProcessor, NalBuffer, processor};
use hevc_parser::{HevcParser, NALUStartCode, hevc::*};
use processor::{HevcProcessor, HevcProcessorOpts};

use hdr10plus::av1::encode_av1_from_json;
use hdr10plus::metadata_json::{Hdr10PlusJsonMetadata, MetadataJsonRoot};

use crate::commands::InjectArgs;
use crate::core::av1_parser::{
    IvfFrameHeader, Obu, OBU_TEMPORAL_DELIMITER, is_hdr10plus_obu,
    read_ivf_frame_header, read_obus_from_ivf_frame, try_read_ivf_file_header,
    write_ivf_frame_header,
};
use crate::core::{initialize_progress_bar, st2094_40_sei_msg};

use super::{CliOptions, input_from_either};

fn is_av1_input(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("av1") | Some("ivf")
    )
}

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

        let writer =
            BufWriter::with_capacity(chunk_size, File::create(output).expect("Can't create file"));

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

        if is_av1_input(&input) {
            Self::inject_json_av1(args, cli_options)
        } else {
            let format = hevc_parser::io::format_from_path(&input)?;

            if let IoFormat::Raw = format {
                let mut injector = Injector::from_args(args, cli_options)?;

                injector.process_input()?;
                injector.interleave_hdr10plus_nals()
            } else {
                bail!("Injector: Must be a raw HEVC bitstream file")
            }
        }
    }

    fn inject_json_av1(args: InjectArgs, cli_options: CliOptions) -> Result<()> {
        let InjectArgs {
            input,
            input_pos,
            json,
            output,
        } = args;
        let input = input_from_either("inject", input, input_pos)?;
        let output = output.unwrap_or_else(|| PathBuf::from("injected_output.av1"));

        println!("Parsing JSON file...");
        stdout().flush().ok();

        let metadata_root = MetadataJsonRoot::from_file(&json)?;
        let metadata_list: Vec<Hdr10PlusJsonMetadata> = metadata_root.scene_info;

        if metadata_list.is_empty() {
            bail!("Empty HDR10+ SceneInfo array");
        }

        let file = File::open(&input)?;
        let mut reader = BufReader::with_capacity(100_000, file);

        let out_file = File::create(&output).expect("Can't create output file");
        let mut writer = BufWriter::with_capacity(100_000, out_file);

        let total_meta = metadata_list.len();

        if let Some(ivf_header) = try_read_ivf_file_header(&mut reader)? {
            writer.write_all(&ivf_header)?;

            let mut tu_index = 0usize;
            let mut last_encoded: Option<Vec<u8>> = None;
            let mut warned_existing = false;
            let mut warned_mismatch = false;

            loop {
                let fh: IvfFrameHeader = match read_ivf_frame_header(&mut reader)? {
                    Some(h) => h,
                    None => break,
                };

                let mut frame_data = vec![0u8; fh.frame_size as usize];
                reader.read_exact(&mut frame_data)?;

                let obus = read_obus_from_ivf_frame(frame_data)?;

                if !warned_existing
                    && obus.iter().any(|o| is_hdr10plus_obu(o, cli_options.validate))
                {
                    warned_existing = true;
                    println!(
                        "\nWarning: Input file already has HDR10+ metadata OBUs; \
                         they will be replaced."
                    );
                }

                let encoded = if tu_index < total_meta {
                    let enc =
                        encode_av1_from_json(&metadata_list[tu_index], cli_options.validate)?;
                    last_encoded = Some(enc.clone());
                    enc
                } else {
                    if !warned_mismatch {
                        warned_mismatch = true;
                        println!(
                            "\nWarning: mismatched lengths. \
                             Metadata has {total_meta} entries but video has more frames. \
                             Last metadata will be duplicated."
                        );
                    }
                    match &last_encoded {
                        Some(enc) => enc.clone(),
                        None => bail!("No HDR10+ metadata available for TU {tu_index}"),
                    }
                };

                let output_frame =
                    Self::build_av1_output_frame(&obus, &encoded, cli_options.validate);

                write_ivf_frame_header(&mut writer, output_frame.len() as u32, fh.timestamp)?;
                writer.write_all(&output_frame)?;

                tu_index += 1;
            }

            if tu_index < total_meta {
                println!(
                    "\nWarning: mismatched lengths. Metadata has {total_meta} entries \
                     but video has {tu_index} frames. Excess metadata was ignored."
                );
            }
        } else {
            // Raw OBU stream
            let mut tu_index = 0usize;
            let mut last_encoded: Option<Vec<u8>> = None;
            let mut warned_existing = false;
            let mut warned_mismatch = false;

            let mut current_td: Option<Obu> = None;
            let mut pending: Vec<Obu> = Vec::new();

            loop {
                let obu_opt = Obu::read_from(&mut reader)?;
                let is_eof = obu_opt.is_none();
                let is_td = obu_opt
                    .as_ref()
                    .map(|o| o.obu_type == OBU_TEMPORAL_DELIMITER)
                    .unwrap_or(false);

                if (is_eof || is_td) && current_td.is_some() {
                    if !warned_existing
                        && pending
                            .iter()
                            .any(|o| is_hdr10plus_obu(o, cli_options.validate))
                    {
                        warned_existing = true;
                        println!(
                            "\nWarning: Input file already has HDR10+ metadata OBUs; \
                             they will be replaced."
                        );
                    }

                    let encoded = if tu_index < total_meta {
                        let enc = encode_av1_from_json(
                            &metadata_list[tu_index],
                            cli_options.validate,
                        )?;
                        last_encoded = Some(enc.clone());
                        enc
                    } else {
                        if !warned_mismatch {
                            warned_mismatch = true;
                            println!(
                                "\nWarning: mismatched lengths. \
                                 Metadata has {total_meta} entries but video has more frames. \
                                 Last metadata will be duplicated."
                            );
                        }
                        match &last_encoded {
                            Some(enc) => enc.clone(),
                            None => bail!("No HDR10+ metadata available for TU {tu_index}"),
                        }
                    };

                    let td = current_td.take().unwrap();
                    writer.write_all(&td.raw_bytes)?;
                    writer.write_all(&encoded)?;
                    for obu in pending.drain(..) {
                        if !is_hdr10plus_obu(&obu, cli_options.validate) {
                            writer.write_all(&obu.raw_bytes)?;
                        }
                    }

                    tu_index += 1;
                }

                match obu_opt {
                    None => break,
                    Some(obu) => {
                        if obu.obu_type == OBU_TEMPORAL_DELIMITER {
                            current_td = Some(obu);
                            pending.clear();
                        } else if current_td.is_some() {
                            pending.push(obu);
                        } else {
                            writer.write_all(&obu.raw_bytes)?;
                        }
                    }
                }
            }

            if tu_index < total_meta {
                println!(
                    "\nWarning: mismatched lengths. Metadata has {total_meta} entries \
                     but video has {tu_index} frames. Excess metadata was ignored."
                );
            }
        }

        println!("Rewriting with interleaved HDR10+ metadata OBUs: Done.");
        writer.flush()?;
        Ok(())
    }

    fn build_av1_output_frame(obus: &[Obu], encoded: &[u8], validate: bool) -> Vec<u8> {
        let mut out = Vec::new();
        let mut injected = false;

        let insert_after_td = obus
            .iter()
            .position(|o| o.obu_type == OBU_TEMPORAL_DELIMITER)
            .map(|i| i + 1)
            .unwrap_or(0);

        for (i, obu) in obus.iter().enumerate() {
            if !injected && i == insert_after_td {
                out.extend_from_slice(encoded);
                injected = true;
            }
            if is_hdr10plus_obu(obu, validate) {
                continue;
            }
            out.extend_from_slice(&obu.raw_bytes);
        }

        if !injected {
            out.extend_from_slice(encoded);
        }

        out
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
        validate: bool,
    ) -> Result<(usize, NalBuffer)> {
        let existing_frame = frames
            .iter()
            .find(|f| f.decoded_number == frame_buffer.frame_number);

        // If we have a metadata buffered frame, write it
        // Otherwise, write the same data as previous
        let hdr10plus_nb = if let Some(frame) = existing_frame {
            let meta = metadata_list
                .get(frame.presentation_number as usize)
                .or_else(|| mismatched_length.then(|| metadata_list.last()).flatten());

            if let Some(meta) = meta {
                let data = hdr10plus::hevc::encode_hevc_from_json(meta, validate)?;

                Some(NalBuffer {
                    nal_type: NAL_SEI_PREFIX,
                    start_code: NALUStartCode::Length4,
                    data,
                })
            } else {
                bail!(
                    "No metadata found for presentation frame {}",
                    frame.presentation_number
                );
            }
        } else if mismatched_length && let Some(meta) = metadata_list.last() {
            let data = hdr10plus::hevc::encode_hevc_from_json(meta, validate)?;

            Some(NalBuffer {
                nal_type: NAL_UNSPEC62,
                start_code: NALUStartCode::Length4,
                data,
            })
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
                        self.options.validate,
                    )?;

                    self.frame_buffer.nals.insert(idx, hdr10plus_nb);

                    // Write NALUs for the frame
                    for nal_buf in &self.frame_buffer.nals {
                        self.writer.write_all(NALUStartCode::Length4.slice())?;
                        self.writer.write_all(&nal_buf.data)?;
                    }

                    self.frame_buffer.frame_number = nal.decoded_frame_index;
                    self.frame_buffer.nals.clear();
                }

                let (st2094_40_msg, payload) = if nal.nal_type == NAL_SEI_PREFIX {
                    let sei_payload =
                        clear_start_code_emulation_prevention_3_byte(&chunk[nal.start..nal.end]);
                    let msg = st2094_40_sei_msg(&sei_payload, self.options.validate)?;

                    (msg, Some(sei_payload))
                } else {
                    (None, None)
                };

                if let (Some(msg), Some(mut payload)) = (st2094_40_msg, payload) {
                    let messages = SeiMessage::parse_sei_rbsp(&payload)?;

                    // Only remove ST2094-40 message if there are others
                    if messages.len() > 1 {
                        let start = msg.msg_offset;
                        let end = msg.payload_offset + msg.payload_size;

                        payload.drain(start..end);
                        add_start_code_emulation_prevention_3_byte(&mut payload);

                        self.frame_buffer.nals.push(NalBuffer {
                            nal_type: nal.nal_type,
                            start_code: nal.start_code,
                            data: payload,
                        });
                    }
                } else {
                    self.frame_buffer.nals.push(NalBuffer {
                        nal_type: nal.nal_type,
                        start_code: nal.start_code,
                        data: chunk[nal.start..nal.end].to_vec(),
                    });
                }
            }
        } else if !self.already_checked_for_hdr10plus {
            let existing_hdr10plus = nals
                .iter()
                .filter(|nal| nal.nal_type == NAL_SEI_PREFIX)
                .any(|nal| {
                    let sei_payload =
                        clear_start_code_emulation_prevention_3_byte(&chunk[nal.start..nal.end]);

                    st2094_40_sei_msg(&sei_payload, self.options.validate)
                        .unwrap_or(None)
                        .is_some()
                });

            if existing_hdr10plus {
                self.already_checked_for_hdr10plus = true;
                println!("\nWarning: Input file already has HDR10+ SEIs, they will be replaced.");
            }
        }

        Ok(())
    }

    fn finalize(&mut self, parser: &HevcParser) -> Result<()> {
        // First pass
        if self.frames.is_empty() && self.nals.is_empty() {
            self.frames.clone_from(parser.ordered_frames());
            self.nals.clone_from(parser.get_nals());
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
                    self.options.validate,
                )?;

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
