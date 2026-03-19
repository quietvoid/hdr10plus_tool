use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use hevc_parser::HevcParser;
use hevc_parser::hevc::{NAL_SEI_PREFIX, NALUnit};
use hevc_parser::io::processor::{HevcProcessor, HevcProcessorOpts};
use indicatif::ProgressBar;

use hevc_parser::io::{IoFormat, IoProcessor};

use super::{CliOptions, RemoveArgs, input_from_either};
use crate::core::{initialize_progress_bar, prefix_sei_removed_hdr10plus_nalu};

fn is_av1_input(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("av1") | Some("ivf")
    )
}

pub struct Remover {
    input: PathBuf,
    progress_bar: ProgressBar,
    writer: BufWriter<File>,
}

impl Remover {
    pub fn remove_sei(args: RemoveArgs, options: CliOptions) -> Result<()> {
        let RemoveArgs {
            input,
            input_pos,
            output,
        } = args;
        let input = input_from_either("remove", input, input_pos)?;

        if is_av1_input(&input) {
            Self::remove_sei_av1(input, output, options)
        } else {
            let format = hevc_parser::io::format_from_path(&input)?;

            if format == IoFormat::Matroska {
                bail!("Remover: Matroska format unsupported");
            }

            let hevc_out = match output {
                Some(path) => path,
                None => PathBuf::from("hdr10plus_removed_output.hevc"),
            };

            let pb = initialize_progress_bar(&format, &input)?;

            let mut remover = Remover {
                input,
                progress_bar: pb,
                writer: BufWriter::with_capacity(
                    100_000,
                    File::create(hevc_out).expect("Can't create file"),
                ),
            };

            remover.process_input(&format)
        }
    }

    fn remove_sei_av1(
        input: PathBuf,
        output: Option<PathBuf>,
        options: CliOptions,
    ) -> Result<()> {
        use crate::core::av1_parser::{
            Obu, is_hdr10plus_obu, read_ivf_frame_header, read_obus_from_ivf_frame,
            try_read_ivf_file_header, write_ivf_frame_header,
        };

        let out_path = output.unwrap_or_else(|| PathBuf::from("hdr10plus_removed_output.av1"));

        let file = File::open(&input)?;
        let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);
        let mut reader = BufReader::with_capacity(100_000, file);

        let out_file = File::create(&out_path).expect("Can't create output file");
        let mut writer = BufWriter::with_capacity(100_000, out_file);

        let pb = initialize_progress_bar(&IoFormat::Raw, &input)?;
        let mut bytes_read = 0u64;

        if let Some(ivf_header) = try_read_ivf_file_header(&mut reader)? {
            writer.write_all(&ivf_header)?;
            bytes_read += ivf_header.len() as u64;

            loop {
                let fh = match read_ivf_frame_header(&mut reader)? {
                    Some(h) => h,
                    None => break,
                };
                bytes_read += 12;
                pb.set_position(bytes_read * 100 / file_len.max(1));

                let mut frame_data = vec![0u8; fh.frame_size as usize];
                reader.read_exact(&mut frame_data)?;
                bytes_read += fh.frame_size as u64;

                let obus = read_obus_from_ivf_frame(frame_data)?;

                let output_frame: Vec<u8> = obus
                    .iter()
                    .filter(|o| !is_hdr10plus_obu(o, options.validate))
                    .flat_map(|o| o.raw_bytes.iter().copied())
                    .collect();

                write_ivf_frame_header(&mut writer, output_frame.len() as u32, fh.timestamp)?;
                writer.write_all(&output_frame)?;
            }
        } else {
            loop {
                match Obu::read_from(&mut reader) {
                    Ok(Some(obu)) => {
                        bytes_read += obu.raw_bytes.len() as u64;
                        pb.set_position(bytes_read * 100 / file_len.max(1));

                        if !is_hdr10plus_obu(&obu, options.validate) {
                            writer.write_all(&obu.raw_bytes)?;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => return Err(e),
                }
            }
        }

        pb.finish_and_clear();
        writer.flush()?;
        Ok(())
    }

    pub fn process_input(&mut self, format: &IoFormat) -> Result<()> {
        let chunk_size = 100_000;

        let processor_opts = HevcProcessorOpts {
            parse_nals: false,
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
}

impl IoProcessor for Remover {
    fn input(&self) -> &std::path::PathBuf {
        &self.input
    }

    fn update_progress(&mut self, delta: u64) {
        self.progress_bar.inc(delta);
    }

    fn process_nals(&mut self, _parser: &HevcParser, nals: &[NALUnit], chunk: &[u8]) -> Result<()> {
        for nal in nals {
            let mut nalu_data_override = None;

            if nal.nal_type == NAL_SEI_PREFIX {
                let (has_st2094_40, data) = prefix_sei_removed_hdr10plus_nalu(chunk, nal)?;

                // Drop NALUs containing only one SEI message
                if has_st2094_40 && data.is_none() {
                    continue;
                } else {
                    nalu_data_override = data;
                }
            }

            let data = nalu_data_override
                .as_ref()
                .map(|e| e.as_ref())
                .unwrap_or(&chunk[nal.start..nal.end]);

            NALUnit::write_with_preset(
                &mut self.writer,
                data,
                hevc_parser::io::StartCodePreset::Four,
                nal.nal_type,
                false,
            )?;
        }

        Ok(())
    }

    fn finalize(&mut self, _parser: &HevcParser) -> Result<()> {
        self.progress_bar.finish_and_clear();
        self.writer.flush()?;

        Ok(())
    }
}
