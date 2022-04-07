use std::io::{stdout, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::{fs::File, path::Path};

use anyhow::{bail, Result};
use indicatif::ProgressBar;

use hevc_parser::hevc::NAL_SEI_PREFIX;
use hevc_parser::hevc::{Frame, NALUnit};
use hevc_parser::HevcParser;

use hdr10plus::metadata::Hdr10PlusMetadata;
use hdr10plus::metadata_json::generate_json;

use super::{is_st2094_40_sei, Format};

pub const TOOL_NAME: &str = env!("CARGO_PKG_NAME");
pub const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Parser {
    pub format: Format,
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub verify: bool,
    pub validate: bool,
}

#[derive(Clone)]
pub struct MetadataFrame {
    pub decoded_index: usize,
    pub presentation_number: usize,
    pub metadata: Hdr10PlusMetadata,
}

impl Parser {
    pub fn new(
        format: Format,
        input: PathBuf,
        output: Option<PathBuf>,
        verify: bool,
        validate: bool,
    ) -> Self {
        Self {
            format,
            input,
            output,
            verify,
            validate,
        }
    }

    pub fn process_input(&self) -> Result<()> {
        let pb = super::initialize_progress_bar(&self.format, &self.input)?;

        let mut parser = HevcParser::default();

        let result = match self.format {
            Format::Matroska => bail!("unsupported format matroska"),
            _ => self.parse_metadata(&self.input, Some(&pb), &mut parser)?,
        };

        pb.finish_and_clear();

        if result.is_empty() {
            bail!("No metadata found in the input.");
        } else if self.verify && result[0][0] == 1 && result[0].len() == 1 {
            //Match returned vec to check for --verify
            println!("Dynamic HDR10+ metadata detected.");
        } else {
            let mut final_metadata = Self::llc_read_metadata(result, self.validate)?;

            //Sucessful parse & no --verify
            if !final_metadata.is_empty() {
                let frames = parser.ordered_frames();

                // Reorder to display output order
                self.reorder_metadata(frames, &mut final_metadata);

                self.write_json(final_metadata)?;
            } else {
                bail!("Failed reading parsed metadata.");
            }
        }

        Ok(())
    }

    pub fn parse_metadata(
        &self,
        input: &Path,
        pb: Option<&ProgressBar>,
        parser: &mut HevcParser,
    ) -> Result<Vec<Vec<u8>>> {
        //BufReader & BufWriter
        let stdin = std::io::stdin();
        let mut reader = Box::new(stdin.lock()) as Box<dyn BufRead>;

        if let Format::Raw = self.format {
            let file = File::open(input)?;
            reader = Box::new(BufReader::with_capacity(100_000, file));
        }

        let chunk_size = 100_000;

        let mut main_buf = vec![0; 100_000];
        let mut sec_buf = vec![0; 50_000];

        let mut chunk = Vec::with_capacity(chunk_size);
        let mut end: Vec<u8> = Vec::with_capacity(100_000);

        let mut consumed = 0;

        let mut offsets = Vec::with_capacity(2048);

        let mut final_sei_list: Vec<Vec<u8>> = Vec::new();

        while let Ok(n) = reader.read(&mut main_buf) {
            let mut read_bytes = n;
            if read_bytes == 0 && end.is_empty() && chunk.is_empty() {
                break;
            }

            if self.format == Format::RawStdin {
                chunk.extend_from_slice(&main_buf[..read_bytes]);

                loop {
                    let num = reader.read(&mut sec_buf)?;

                    if num > 0 {
                        read_bytes += num;

                        chunk.extend_from_slice(&sec_buf[..num]);

                        if read_bytes >= chunk_size {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            } else if read_bytes < chunk_size {
                chunk.extend_from_slice(&main_buf[..read_bytes]);
            } else {
                chunk.extend_from_slice(&main_buf);
            }

            parser.get_offsets(&chunk, &mut offsets);

            if offsets.is_empty() {
                continue;
            }

            let last = if read_bytes < chunk_size {
                *offsets.last().unwrap()
            } else {
                let last = offsets.pop().unwrap();

                end.clear();
                end.extend_from_slice(&chunk[last..]);

                last
            };

            let nals: Vec<NALUnit> = parser.split_nals(&chunk, &offsets, last, true)?;

            let new_sei = self.find_hdr10plus_sei(&chunk, nals)?;

            if final_sei_list.is_empty() && new_sei.is_empty() {
                bail!("File doesn't contain dynamic metadata, stopping.");
            } else if self.verify {
                return Ok(vec![vec![1]]);
            }

            final_sei_list.extend_from_slice(&new_sei);

            chunk.clear();

            if !end.is_empty() {
                chunk.extend_from_slice(&end);
                end.clear();
            }

            consumed += read_bytes;

            if consumed >= 100_000_000 {
                if let Some(pb) = pb {
                    pb.inc(1);
                    consumed = 0;
                }
            }
        }

        parser.finish();

        Ok(final_sei_list)
    }

    pub fn find_hdr10plus_sei(&self, data: &[u8], nals: Vec<NALUnit>) -> Result<Vec<Vec<u8>>> {
        let mut found_list = Vec::new();

        for nal in nals {
            if let NAL_SEI_PREFIX = nal.nal_type {
                let sei_payload = &data[nal.start..nal.end];

                if is_st2094_40_sei(sei_payload)? {
                    found_list.push(sei_payload[4..].to_vec());
                }
            }
        }

        Ok(found_list)
    }

    pub fn llc_read_metadata(input: Vec<Vec<u8>>, validate: bool) -> Result<Vec<MetadataFrame>> {
        print!("Reading parsed dynamic metadata... ");
        stdout().flush().ok();

        let mut complete_metadata: Vec<MetadataFrame> = Vec::new();

        //Loop over lines and read metadata, HDR10+ LLC format
        for (index, data) in input.iter().enumerate() {
            let bytes = hevc_parser::utils::clear_start_code_emulation_prevention_3_byte(data);

            // Parse metadata
            let metadata = Hdr10PlusMetadata::parse(bytes)?;

            // Validate values
            if validate {
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
