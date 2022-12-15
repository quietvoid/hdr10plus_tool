use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use anyhow::{bail, Result};
use hevc_parser::hevc::{NALUnit, NAL_SEI_PREFIX};
use hevc_parser::io::processor::{HevcProcessor, HevcProcessorOpts};
use hevc_parser::HevcParser;
use indicatif::ProgressBar;

use hevc_parser::io::{IoFormat, IoProcessor};

use super::{input_from_either, CliOptions, RemoveArgs};
use crate::core::{initialize_progress_bar, prefix_sei_removed_hdr10plus_nalu};

pub struct Remover {
    input: PathBuf,
    progress_bar: ProgressBar,
    writer: BufWriter<File>,
}

impl Remover {
    pub fn remove_sei(args: RemoveArgs, _options: CliOptions) -> Result<()> {
        let RemoveArgs {
            input,
            input_pos,
            output,
        } = args;
        let input = input_from_either("remove", input, input_pos)?;

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
