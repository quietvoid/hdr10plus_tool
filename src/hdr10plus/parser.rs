use std::io::{stdout, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::{fs::File, path::Path};

use indicatif::ProgressBar;
use serde_json::{json, Value};

use hevc_parser::hevc::NAL_SEI_PREFIX;
use hevc_parser::hevc::{Frame, NALUnit};
use hevc_parser::HevcParser;

use super::{metadata::Hdr10PlusMetadata, Format, TOOL_NAME, TOOL_VERSION};

const HDR10PLUS_SEI_HEADER: &[u8] = &[78, 1, 4];

pub struct Parser {
    format: Format,
    input: PathBuf,
    output: Option<PathBuf>,
    verify: bool,
    validate: bool,
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

    pub fn process_input(&self) {
        let pb = super::initialize_progress_bar(&self.format, &self.input);

        let mut parser = HevcParser::default();

        let result = match self.format {
            Format::Matroska => panic!("unsupported format matroska"),
            _ => self.parse_metadata(&self.input, Some(&pb), &mut parser),
        };

        pb.finish_and_clear();

        match result {
            Ok(vec) => {
                if vec.is_empty() {
                    println!("No metadata found in the input.");
                } else if self.verify && vec[0][0] == 1 && vec[0].len() == 1 {
                    //Match returned vec to check for --verify
                    println!("Dynamic HDR10+ metadata detected.");
                } else {
                    let mut final_metadata = Self::llc_read_metadata(vec, self.validate);

                    //Sucessful parse & no --verify
                    if !final_metadata.is_empty() {
                        let frames = parser.ordered_frames();

                        // Reorder to display output order
                        self.reorder_metadata(frames, &mut final_metadata);

                        self.write_json(final_metadata)
                    } else {
                        println!("Failed reading parsed metadata.");
                    }
                }
            }
            Err(e) => println!("{}", e),
        }
    }

    pub fn parse_metadata(
        &self,
        input: &Path,
        pb: Option<&ProgressBar>,
        parser: &mut HevcParser,
    ) -> Result<Vec<Vec<u8>>, std::io::Error> {
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
                    match reader.read(&mut sec_buf) {
                        Ok(num) => {
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
                        Err(e) => panic!("{:?}", e),
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

            let nals: Vec<NALUnit> = parser.split_nals(&chunk, &offsets, last, true);

            let new_sei = self.find_hdr10plus_sei(&chunk, nals);

            if final_sei_list.is_empty() && new_sei.is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "File doesn't contain dynamic metadata, stopping.",
                ));
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

    pub fn find_hdr10plus_sei(&self, data: &[u8], nals: Vec<NALUnit>) -> Vec<Vec<u8>> {
        let mut found_list = Vec::new();

        for nal in nals {
            if let NAL_SEI_PREFIX = nal.nal_type {
                if let HDR10PLUS_SEI_HEADER = &data[nal.start..nal.start + 3] {
                    found_list.push(data[nal.start + 4..nal.end].to_vec());
                }
            }
        }

        found_list
    }

    pub fn llc_read_metadata(input: Vec<Vec<u8>>, validate: bool) -> Vec<MetadataFrame> {
        print!("Reading parsed dynamic metadata... ");
        stdout().flush().ok();

        let mut complete_metadata: Vec<MetadataFrame> = Vec::new();

        //Loop over lines and read metadata, HDR10+ LLC format
        for (index, data) in input.iter().enumerate() {
            let bytes = hevc_parser::utils::clear_start_code_emulation_prevention_3_byte(data);

            // Parse metadata
            let metadata = Hdr10PlusMetadata::parse(bytes);

            // Validate values
            if validate {
                metadata.validate();
            }

            let metadata_frame = MetadataFrame {
                decoded_index: index,
                presentation_number: 0,
                metadata,
            };

            complete_metadata.push(metadata_frame);
        }

        println!("Done.");

        complete_metadata
    }

    pub fn generate_json(metadata: &[MetadataFrame]) -> Value {
        let (profile, frame_json_list): (&str, Vec<Value>) = Hdr10PlusMetadata::json_list(metadata);

        let json_info = json!({
            "HDR10plusProfile": profile,
            "Version": format!("{}.0", &metadata[0].metadata.application_version),
        });

        let first_frames: Vec<u64> = frame_json_list
            .iter()
            .filter_map(|meta| {
                if meta.get("SceneFrameIndex").unwrap().as_u64().unwrap() == 0 {
                    meta.get("SequenceFrameIndex").unwrap().as_u64()
                } else {
                    None
                }
            })
            .collect();

        let mut scene_lengths: Vec<u64> = Vec::with_capacity(first_frames.len());

        for i in 0..first_frames.len() {
            if i < first_frames.len() - 1 {
                scene_lengths.push(first_frames[i + 1] - first_frames[i]);
            } else {
                scene_lengths.push(frame_json_list.len() as u64 - first_frames[i]);
            }
        }

        let scene_info_json = json!({
            "SceneFirstFrameIndex": first_frames,
            "SceneFrameNumbers": scene_lengths,
        });

        let final_json = json!({
            "JSONInfo": json_info,
            "SceneInfo": frame_json_list,
            "SceneInfoSummary": scene_info_json,
            "ToolInfo": json!({
                "Tool": TOOL_NAME,
                "Version": TOOL_VERSION,
            })
        });

        final_json
    }

    fn write_json(&self, metadata: Vec<MetadataFrame>) {
        match &self.output {
            Some(path) => {
                let save_file = File::create(path).expect("Can't create file");
                let mut writer = BufWriter::with_capacity(10_000_000, save_file);

                print!("Generating and writing metadata to JSON file... ");
                stdout().flush().ok();

                let final_json = Self::generate_json(&metadata);

                assert!(writeln!(
                    writer,
                    "{}",
                    serde_json::to_string_pretty(&final_json).unwrap()
                )
                .is_ok());

                println!("Done.");

                writer.flush().ok();
            }
            None => panic!("Output path required!"),
        }
    }

    fn reorder_metadata(&self, frames: &[Frame], metadata: &mut Vec<MetadataFrame>) {
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

    pub fn _test(&self) -> Option<(Hdr10PlusMetadata, Value)> {
        let mut metadata: Vec<MetadataFrame> = Vec::new();
        let mut parser = HevcParser::default();

        match self.parse_metadata(&self.input, None, &mut parser) {
            Ok(vec) => metadata = Parser::llc_read_metadata(vec, false),
            Err(e) => println!("{}", e),
        }

        if !metadata.is_empty() {
            let first_decoded_metadata = metadata[0].metadata.clone();

            let frames = parser.ordered_frames();

            // Reorder to display output order
            self.reorder_metadata(frames, &mut metadata);

            let final_json = Self::generate_json(&metadata);

            Some((first_decoded_metadata, final_json))
        } else {
            None
        }
    }
}
