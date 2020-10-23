use std::fs::File;
use std::io::{stdout, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use ansi_term::Colour::{Blue, Green, Red};
use deku::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use read_byte_slice::{ByteSliceIter, FallibleStreamingIterator};
use serde_json::{json, Value};

use super::metadata::Metadata;

use av_format::{buffer::AccReader, demuxer::Context, demuxer::Event};
use matroska::demuxer::MkvDemuxer;

pub enum Format {
    Raw,
    RawStdin,
    Matroska,
}

pub struct Parser {
    format: Format,
    input: PathBuf,
    output: Option<PathBuf>,
    verify: bool,
    force_single_profile: bool,
}

impl Parser {
    pub fn new(
        format: Format,
        input: PathBuf,
        output: Option<PathBuf>,
        verify: bool,
        force_single_profile: bool,
    ) -> Self {
        Self {
            format,
            input,
            output,
            verify,
            force_single_profile,
        }
    }

    pub fn process_file(&self) {
        println!(
            "{}",
            Blue.paint("Parsing HEVC file for dynamic metadata... ")
        );

        let result = match self.format {
            Format::Matroska => self.parse_matroska(),
            _ => self.parse_raw_hevc(),
        };

        match result {
            Ok(vec) => {
                // Match returned vec to check for --verify
                if vec[0][0] == 1 && vec[0].len() == 1 {
                    println!("{}", Green.paint("Dynamic HDR10+ metadata detected."));
                } else {
                    self.write_json(Self::llc_parse_metadata(vec))
                }
            }
            Err(e) => println!("{}", Red.paint(e)),
        }
    }

    pub fn parse_raw_hevc(&self) -> Result<Vec<Vec<u8>>, &str> {
        //BufReader & BufWriter
        let stdin = std::io::stdin();
        let mut reader = Box::new(stdin.lock()) as Box<dyn BufRead>;
        let bytes_count;

        let pb: ProgressBar;

        if let Format::RawStdin = self.format {
            pb = ProgressBar::hidden();
        } else {
            let file = File::open(&self.input).expect("No file found");

            //Info for indicatif ProgressBar
            let file_meta = file.metadata();
            bytes_count = file_meta.unwrap().len() / 100_000_000;

            reader = Box::new(BufReader::new(file));

            if self.verify {
                pb = ProgressBar::hidden();
            } else {
                pb = ProgressBar::new(bytes_count);
                pb.set_style(
                    ProgressStyle::default_bar()
                        .template("[{elapsed_precise}] {bar:60.cyan} {percent}%"),
                );
            }
        }

        //Byte chunk iterator
        let mut iter = ByteSliceIter::new(reader, 100_000);

        //Bitstream blocks for SMPTE 2094-40
        let header: &[Option<u8>] = &[Some(0), Some(0), Some(1), Some(78), Some(1), Some(4)];
        let mut final_sei_list: Vec<Vec<u8>> = Vec::new();

        let mut cur_byte = 0;

        //Loop over iterator of byte chunks for faster I/O
        while let Ok(Some(chunk)) = iter.next() {
            match self.parse_itu_t35_sei_payload(chunk, header, Some(&mut final_sei_list)) {
                Ok(_) => {
                    if self.verify {
                        return Ok(vec![vec![1]]);
                    }
                }
                Err(e) => return Err(e),
            }

            if cur_byte >= 100_000_000 {
                pb.inc(1);
                cur_byte = 0;
            }
        }

        if !final_sei_list.is_empty() {
            Ok(final_sei_list)
        } else {
            Err("Failed parsing metadata")
        }
    }

    pub fn llc_parse_metadata(input: Vec<Vec<u8>>) -> Vec<Metadata> {
        print!("{}", Blue.paint("Reading parsed dynamic metadata... "));
        stdout().flush().ok();

        let mut complete_metadata: Vec<Metadata> = Vec::new();

        //Loop over lines and read metadata, HDR10+ LLC format
        for data in input.iter() {
            let (_, bytes) = remove_x265_injected_byte(&data);

            // Parse metadata
            let (_rest, metadata) = Metadata::from_bytes((&bytes, 0)).unwrap();

            // Validate values
            metadata.validate();

            // Debug
            // println!("{:?}", metadata);

            complete_metadata.push(metadata);
        }

        println!("{}", Green.paint("Done."));

        complete_metadata
    }

    fn write_json(&self, metadata: Vec<Metadata>) {
        if metadata.is_empty() {
            println!("Failed parsing metadata to JSON");
            return;
        }

        match &self.output {
            Some(path) => {
                let save_file = File::create(path).expect("Can't create file");
                let mut writer = BufWriter::with_capacity(10_000_000, save_file);

                print!("{}", Blue.paint("Writing metadata to JSON file... "));

                let (profile, frame_json_list, warning): (&str, Vec<Value>, Option<String>) =
                    Metadata::json_list(&metadata, self.force_single_profile);

                let json_info = json!({
                    "HDR10plusProfile": profile,
                    "Version": format!("{}.0", &metadata[0].application_version),
                });

                let final_json = json!({
                    "JSONInfo": json_info,
                    "SceneInfo": frame_json_list
                });

                assert!(writeln!(
                    writer,
                    "{}",
                    serde_json::to_string_pretty(&final_json).unwrap()
                )
                .is_ok());

                println!("{}", Green.paint("Done."));

                if warning.is_some() {
                    println!("{}", warning.unwrap());
                }

                writer.flush().ok();
            }
            None => panic!("Output path required!"),
        }
    }

    fn parse_matroska(&self) -> Result<Vec<Vec<u8>>, &str> {
        let reader = File::open(&self.input).expect("No file found");
        let ar = AccReader::with_capacity(1024, reader);
        let mut demuxer = Context::new(Box::new(MkvDemuxer::new()), Box::new(ar));

        demuxer
            .read_headers()
            .expect("Cannot parse the format headers");

        let header: &[Option<u8>] = &[Some(0), Some(0), Some(0), None, Some(78), Some(1), Some(4)];
        let mut final_sei_list: Vec<Vec<u8>> = Vec::new();

        while let Ok(metadata) = match demuxer.read_event() {
            Ok(event) => match event {
                Event::NewPacket(pkt) => self.parse_itu_t35_sei_payload(&pkt.data, header, None),
                Event::NewStream(_) => Err("Stream changed"),
                Event::MoreDataNeeded(_) => Err("ok1"),
                Event::Continue => Err("2"),
                Event::Eof => Err("OK"),
                _ => Err("ok2"),
            },
            Err(e) => panic!("{:?}", e),
        } {
            if self.verify {
                return Ok(vec![vec![1]]);
            }

            final_sei_list.push(metadata);
        }

        if !final_sei_list.is_empty() {
            Ok(final_sei_list)
        } else {
            Err("File doesn't contain dynamic metadata, stopping.")
        }
    }

    fn parse_itu_t35_sei_payload(
        &self,
        data: &[u8],
        header: &[Option<u8>],
        mut final_list: Option<&mut Vec<Vec<u8>>>,
    ) -> Result<Vec<u8>, &str> {
        let mut metadata_detected = false;

        let length = data.len();
        let first = header[0].unwrap();
        for (offset, n) in data.iter().enumerate() {
            if n == &first {
                let all_match_header = header
                    .iter()
                    .enumerate()
                    .map(|(j, v)| {
                        // Matches all header but None (wildcard)
                        if offset + j >= length {
                            false
                        } else if v.is_some() {
                            data[offset + j] == v.unwrap()
                        } else {
                            true
                        }
                    })
                    .all(|v| v);

                if all_match_header {
                    metadata_detected = true;
                    let size = data[offset + header.len()] as usize;

                    let start = offset + header.len() + 1;
                    let end = if start + size > length {
                        length - 1
                    } else {
                        start + size + header.len() + 1
                    };

                    let temp = &data[start..end];

                    let (bytes_removed, _bytes) = remove_x265_injected_byte(&temp);
                    let end = if end + bytes_removed > length {
                        length - 1
                    } else {
                        end + bytes_removed
                    };

                    let payload = data[start..end as usize].to_owned();
                    if let Some(ref mut list) = final_list {
                        list.push(payload);
                    } else {
                        return Ok(payload);
                    }
                }
            }
        }

        if !metadata_detected && final_list.is_none() {
            Err("File doesn't contain dynamic metadata, stopping.")
        } else {
            Ok(Vec::new())
        }
    }

    pub fn _test(&self) -> Option<(usize, Metadata)> {
        if let Ok(vec) = match self.format {
            Format::Matroska => self.parse_matroska(),
            _ => self.parse_raw_hevc(),
        } {
            let results = Parser::llc_parse_metadata(vec);
            Some((results.len(), results.first().cloned().unwrap()))
        } else {
            None
        }
    }
}

pub fn remove_x265_injected_byte(data: &[u8]) -> (usize, Vec<u8>) {
    let mut count = 0;
    // Clear x265's injected 0x03 byte if it is present
    // See https://bitbucket.org/multicoreware/x265_git/src/a82c6c7a7d5f5ef836c82941788a37c6a443e0fe/source/encoder/nal.cpp?at=master#lines-119:136
    let bytes = data
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            if index > 2
                && index < data.len() - 2
                && data[index - 2] == 0
                && data[index - 1] == 0
                && data[index] <= 3
            {
                count += 1;
                None
            } else {
                Some(*value)
            }
        })
        .collect::<Vec<u8>>();

    (count, bytes)
}
