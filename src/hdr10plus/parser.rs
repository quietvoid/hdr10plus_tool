use std::fs::File;
use std::io::{stdout, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use ansi_term::Colour::{Blue, Green, Red};
use deku::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use read_byte_slice::{ByteSliceIter, FallibleStreamingIterator};
use serde_json::{json, Value};

use super::metadata::Metadata;

pub struct Parser {
    is_stdin: bool,
    input: PathBuf,
    output: Option<PathBuf>,
    verify: bool,
    force_single_profile: bool,
}

impl Parser {
    pub fn new(
        is_stdin: bool,
        input: PathBuf,
        output: Option<PathBuf>,
        verify: bool,
        force_single_profile: bool,
    ) -> Self {
        Self {
            is_stdin,
            input,
            output,
            verify,
            force_single_profile,
        }
    }

    pub fn process_file(&self) {
        let final_metadata: Vec<Metadata>;

        match self.parse_metadata() {
            Ok(vec) => {
                //Match returned vec to check for --verify
                if vec[0][0] == 1 && vec[0].len() == 1 {
                    println!("{}", Blue.paint("Dynamic HDR10+ metadata detected."));
                } else {
                    final_metadata = Self::llc_read_metadata(vec);
                    //Sucessful parse & no --verify
                    if !final_metadata.is_empty() {
                        self.write_json(final_metadata)
                    } else {
                        println!("{}", Red.paint("Failed reading parsed metadata."));
                    }
                }
            }
            Err(e) => println!("{}", e),
        }
    }

    pub fn parse_metadata(&self) -> Result<Vec<Vec<u8>>, std::io::Error> {
        //BufReader & BufWriter
        let stdin = std::io::stdin();
        let mut reader = Box::new(stdin.lock()) as Box<dyn BufRead>;
        let bytes_count;

        let pb: ProgressBar;

        if self.is_stdin {
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
        let header: Vec<u8> = vec![0, 0, 1, 78, 1, 4];
        let mut current_sei: Vec<u8> = Vec::new();

        println!(
            "{}",
            Blue.paint("Parsing HEVC file for dynamic metadata... ")
        );
        stdout().flush().ok();

        let mut final_sei_list: Vec<Vec<u8>> = Vec::new();

        let mut dynamic_hdr_sei = false;
        let mut dynamic_detected = false;
        let mut cur_byte = 0;

        //Loop over iterator of byte chunks for faster I/O
        while let Some(chunk) = iter.next()? {
            for byte in chunk {
                let byte = *byte;

                cur_byte += 1;

                let tuple = Self::process_bytes(
                    &header,
                    byte,
                    &mut current_sei,
                    dynamic_hdr_sei,
                    &mut final_sei_list,
                );
                dynamic_hdr_sei = tuple.0;

                if tuple.1 {
                    dynamic_detected = true;
                }
            }

            if !dynamic_detected {
                pb.finish_and_clear();
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "File doesn't contain dynamic metadata, stopping.",
                ));
            } else if self.verify {
                pb.finish_and_clear();

                let verified = vec![vec![1]];

                return Ok(verified);
            }

            if cur_byte >= 100_000_000 {
                pb.inc(1);
                cur_byte = 0;
            }
        }

        pb.finish_and_clear();

        Ok(final_sei_list)
    }

    fn process_bytes(
        header: &[u8],
        byte: u8,
        current_sei: &mut Vec<u8>,
        mut dynamic_hdr_sei: bool,
        final_sei_list: &mut Vec<Vec<u8>>,
    ) -> (bool, bool) {
        let mut dynamic_detected = false;

        current_sei.push(byte);
        if dynamic_hdr_sei {
            let last = current_sei.len() - 1;

            if current_sei[last - 3] == 128
                && current_sei[last - 2] == 0
                && current_sei[last - 1] == 0
                && (current_sei[last] == 1 || current_sei[last] == 0)
            {
                let final_sei = &current_sei[7..current_sei.len() - 3];

                //Push SEI message to final vec
                final_sei_list.push(final_sei.to_vec());

                //Clear current vec for next pattern match
                current_sei.clear();
                dynamic_hdr_sei = false;
                dynamic_detected = true;
            }
        } else if byte == 0 || byte == 1 || byte == 78 || byte == 4 {
            for i in 0..current_sei.len() {
                if current_sei[i] == header[i] {
                    if current_sei == &header {
                        dynamic_hdr_sei = true;
                        break;
                    }
                } else if current_sei.len() < 3 {
                    current_sei.clear();
                    break;
                } else {
                    current_sei.pop();
                    break;
                }
            }
        } else if !current_sei.is_empty() {
            current_sei.clear();
        }

        (dynamic_hdr_sei, dynamic_detected)
    }

    pub fn llc_read_metadata(input: Vec<Vec<u8>>) -> Vec<Metadata> {
        print!("{}", Blue.paint("Reading parsed dynamic metadata... "));
        stdout().flush().ok();

        let mut complete_metadata: Vec<Metadata> = Vec::new();

        //Loop over lines and read metadata, HDR10+ LLC format
        for data in input.iter() {
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
                        None
                    } else {
                        Some(*value)
                    }
                })
                .collect::<Vec<u8>>();

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
        match &self.output {
            Some(path) => {
                let save_file = File::create(path).expect("Can't create file");
                let mut writer = BufWriter::with_capacity(10_000_000, save_file);

                print!("{}", Blue.paint("Writing metadata to JSON file... "));
                stdout().flush().ok();

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

    pub fn _test(&self) -> Option<Metadata> {
        let mut metadata: Vec<Metadata> = Vec::new();
        match self.parse_metadata() {
            Ok(vec) => metadata = Parser::llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        if !metadata.is_empty() {
            metadata.first().cloned()
        } else {
            None
        }
    }
}
