use std::fs::File;
use std::io::{stdout, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

use anyhow::{bail, Result};

//use crate::dovi::get_aud;

const OUT_NAL_HEADER: &[u8] = &[0, 0, 0, 1];
use hdr10plus::metadata_json::{Hdr10PlusJsonMetadata, MetadataJsonRoot};

use crate::core::is_st2094_40_sei;

use super::{initialize_progress_bar, input_format, Format};

use hevc_parser::hevc::*;
use hevc_parser::HevcParser;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

pub struct Injector {
    input: PathBuf,
    json: PathBuf,
    output: PathBuf,

    metadata_list: Option<Vec<Hdr10PlusJsonMetadata>>,
}

impl Injector {
    pub fn run(
        input: PathBuf,
        json: PathBuf,
        output: Option<PathBuf>,
        validate: bool,
    ) -> Result<()> {
        let format = input_format(&input)?;

        if let Format::Raw = format {
            let output = match output {
                Some(path) => path,
                None => PathBuf::from("injected_output.hevc"),
            };

            let mut injector = Injector::new(input, json, output)?;
            let mut parser = HevcParser::default();

            injector.process_input(&mut parser, format)?;
            parser.finish();

            let frames = parser.ordered_frames();
            let nals = parser.get_nals();

            injector.interleave_sei_nalus(nals, frames, validate)?;
        } else {
            bail!("unsupported format");
        }

        Ok(())
    }

    fn process_input(&self, parser: &mut HevcParser, format: Format) -> Result<()> {
        println!("Processing input video for frame order info...");
        stdout().flush().ok();

        let pb = initialize_progress_bar(&format, &self.input)?;

        //BufReader & BufWriter
        let file = File::open(&self.input)?;
        let mut reader = Box::new(BufReader::with_capacity(100_000, file));

        let chunk_size = 100_000;

        let mut main_buf = vec![0; 100_000];

        let mut chunk = Vec::with_capacity(chunk_size);
        let mut end: Vec<u8> = Vec::with_capacity(chunk_size);

        let mut consumed = 0;

        let mut offsets = Vec::with_capacity(2048);

        let mut already_checked_for_hdr10plus = false;

        while let Ok(n) = reader.read(&mut main_buf) {
            let read_bytes = n;
            if read_bytes == 0 && end.is_empty() && chunk.is_empty() {
                break;
            }

            if read_bytes < chunk_size {
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

            if !already_checked_for_hdr10plus {
                let nals = parser.split_nals(&chunk, &offsets, last, true)?;

                let contains_hdr10plus = nals.iter().any(|nal| {
                    nal.nal_type == NAL_SEI_PREFIX
                        && is_st2094_40_sei(&chunk[nal.start..nal.end]).unwrap_or(false)
                });

                if contains_hdr10plus {
                    already_checked_for_hdr10plus = true;
                    println!("\nWarning: Input file already has HDR10+ metadata SEIs, they will be replaced.");
                }
            } else {
                parser.split_nals(&chunk, &offsets, last, true)?;
            }

            chunk.clear();

            if !end.is_empty() {
                chunk.extend_from_slice(&end);
                end.clear();
            }

            consumed += read_bytes;

            if consumed >= 100_000_000 {
                if !already_checked_for_hdr10plus {
                    already_checked_for_hdr10plus = true;
                }

                pb.inc(1);
                consumed = 0;
            }
        }

        pb.finish_and_clear();

        Ok(())
    }

    pub fn new(input: PathBuf, json: PathBuf, output: PathBuf) -> Result<Injector> {
        let mut injector = Injector {
            input,
            json,
            output,
            metadata_list: None,
        };

        let metadata_root = MetadataJsonRoot::from_file(&injector.json)?;
        injector.metadata_list = Some(metadata_root.scene_info);

        Ok(injector)
    }

    fn interleave_sei_nalus(
        &mut self,
        nals: &[NALUnit],
        frames: &[Frame],
        validate: bool,
    ) -> Result<()> {
        if let Some(ref mut metadata_list) = self.metadata_list {
            let mismatched_length = if frames.len() != metadata_list.len() {
                println!(
                    "\nWarning: mismatched lengths. video {}, metadata {}",
                    frames.len(),
                    metadata_list.len()
                );

                if metadata_list.len() < frames.len() {
                    println!("Metadata will be duplicated at the end to match video length\n");
                } else {
                    println!("Metadata will be skipped at the end to match video length\n");
                }

                true
            } else {
                false
            };

            println!("Computing frame indices..");
            stdout().flush().ok();

            let pb_indices = ProgressBar::new(frames.len() as u64);
            pb_indices.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:60.cyan} {percent}%"),
            );

            let first_slice_indices: Vec<usize> = frames
                .par_iter()
                .map(|f| {
                    let index = find_first_slice_nal_index(nals, f);

                    pb_indices.inc(1);

                    index
                })
                .collect();

            pb_indices.finish_and_clear();

            assert_eq!(frames.len(), first_slice_indices.len());

            println!("Rewriting file with interleaved HDR10+ NALUs..");
            stdout().flush().ok();

            let pb = initialize_progress_bar(&Format::Raw, &self.input)?;
            let mut parser = HevcParser::default();

            let chunk_size = 100_000;

            let mut main_buf = vec![0; 100_000];

            let mut chunk = Vec::with_capacity(chunk_size);
            let mut end: Vec<u8> = Vec::with_capacity(chunk_size);

            //BufReader & BufWriter
            let file = File::open(&self.input)?;
            let mut reader = Box::new(BufReader::with_capacity(100_000, file));
            let mut writer = BufWriter::with_capacity(
                chunk_size,
                File::create(&self.output).expect("Can't create file"),
            );

            let mut consumed = 0;
            let mut offsets = Vec::with_capacity(2048);

            let mut nals_parsed = 0;

            // AUDs
            //let first_decoded_index = frames.iter().position(|f| f.decoded_number == 0).unwrap();
            //writer.write_all(&get_aud(&frames[first_decoded_index]))?;

            let mut last_metadata_written: Option<Vec<u8>> = None;

            while let Ok(n) = reader.read(&mut main_buf) {
                let read_bytes = n;
                if read_bytes == 0 && end.is_empty() && chunk.is_empty() {
                    break;
                }

                if read_bytes < chunk_size {
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

                let nals = parser.split_nals(&chunk, &offsets, last, true)?;

                for (cur_index, nal) in nals.iter().enumerate() {
                    // AUDs
                    //if nal.nal_type == NAL_AUD {
                    //    continue;
                    //}

                    if nal.nal_type == NAL_SEI_PREFIX
                        && is_st2094_40_sei(&chunk[nal.start..nal.end])?
                    {
                    } else {
                        writer.write_all(OUT_NAL_HEADER)?;
                        writer.write_all(&chunk[nal.start..nal.end])?;
                    }

                    let global_index = nals_parsed + cur_index;

                    // Slice after interleaved metadata
                    if first_slice_indices.contains(&global_index) {
                        // We can unwrap because parsed indices are the same
                        let metadata_index = first_slice_indices
                            .iter()
                            .position(|i| i == &global_index)
                            .unwrap();

                        // If we have metadata for index, write it
                        // Otherwise, write the same data as previous
                        if metadata_index < metadata_list.len() {
                            let meta = &mut metadata_list[metadata_index];
                            let data = hdr10plus::hevc::encode_hevc_from_json(meta, validate)?;

                            writer.write_all(OUT_NAL_HEADER)?;
                            writer.write_all(&data)?;

                            last_metadata_written = Some(data);
                        } else if mismatched_length {
                            if let Some(data) = &last_metadata_written {
                                writer.write_all(OUT_NAL_HEADER)?;
                                writer.write_all(data)?;
                            }
                        }
                    }
                }

                nals_parsed += nals.len();

                chunk.clear();

                if !end.is_empty() {
                    chunk.extend_from_slice(&end);
                    end.clear()
                }

                consumed += read_bytes;

                if consumed >= 100_000_000 {
                    pb.inc(1);
                    consumed = 0;
                }
            }

            parser.finish();

            writer.flush()?;

            pb.finish_and_clear();
        }

        Ok(())
    }
}

fn find_first_slice_nal_index(nals: &[NALUnit], frame: &Frame) -> usize {
    let slice_nals = frame.nals.iter().filter(|nal| {
        matches!(
            nal.nal_type,
            NAL_TRAIL_R
                | NAL_TRAIL_N
                | NAL_TSA_N
                | NAL_TSA_R
                | NAL_STSA_N
                | NAL_STSA_R
                | NAL_BLA_W_LP
                | NAL_BLA_W_RADL
                | NAL_BLA_N_LP
                | NAL_IDR_W_RADL
                | NAL_IDR_N_LP
                | NAL_CRA_NUT
                | NAL_RADL_N
                | NAL_RADL_R
                | NAL_RASL_N
                | NAL_RASL_R
        )
    });

    let first_slice = slice_nals
        .enumerate()
        .min_by_key(|(idx, _nal)| *idx)
        .unwrap();

    let first_slice_nal = first_slice.1;

    // We want the index of the NAL before the first slice, since we add after
    if let Some(first_slice_index) = nals.iter().position(|n| {
        n.decoded_frame_index == frame.decoded_number && first_slice_nal.nal_type == n.nal_type
    }) {
        first_slice_index - 1
    } else {
        panic!("Could not find a NAL for frame {}", frame.decoded_number);
    }
}
