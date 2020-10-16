use bitreader::BitReader;
use indicatif::{ProgressBar, ProgressStyle};
use read_byte_slice::{ByteSliceIter, FallibleStreamingIterator};
use serde_json::{json, Value};
use std::fs::File;
use std::io::{stdout, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use ansi_term::Colour::{Blue, Green, Red, Yellow};
use deku::prelude::*;

pub fn process_file(
    is_stdin: bool,
    input: &PathBuf,
    output: PathBuf,
    verify: bool,
    force_single_profile: bool,
) {
    let final_metadata: Vec<Metadata>;

    match parse_metadata(is_stdin, input, verify) {
        Ok(vec) => {
            //Match returned vec to check for --verify
            if vec[0][0] == 1 && vec[0].len() == 1 {
                println!("{}", Blue.paint("Dynamic HDR10+ metadata detected."));
            } else {
                final_metadata = llc_read_metadata(vec);
                //Sucessful parse & no --verify
                if !final_metadata.is_empty() {
                    write_json(output, final_metadata, force_single_profile)
                } else {
                    println!("{}", Red.paint("Failed reading parsed metadata."));
                }
            }
        }
        Err(e) => println!("{}", e),
    }
}

#[derive(Debug)]
pub struct Metadata {
    pub bezier_curve_data: Vec<u16>,
    pub knee_x: u16,
    pub knee_y: u16,
    pub average_maxrgb: u32,
    pub maxscl: Vec<u32>,
    pub distribution_index: Vec<u8>,
    pub distribution_values: Vec<u32>,
    pub targeted_system_display_maximum_luminance: u32,
    pub num_windows: u8,
}

#[derive(Debug, PartialEq, DekuRead)]
pub struct TestMetadata {
    #[deku(bits = "8")]
    country_code: u8,
    #[deku(bits = "16")]
    terminal_provider_code: u16,
    #[deku(bits = "16")]
    terminal_provider_oriented_code: u16,
    #[deku(bits = "8")]
    application_identifier: u8,
    #[deku(bits = "8")]
    application_version: u8,

    #[deku(bits = "2")]
    num_windows: u8,
    #[deku(endian = "big", bits = "27")]
    targeted_system_display_maximum_luminance: u32,
    #[deku(bits = "1")]
    targeted_system_display_actual_peak_luminance_flag: u8,

    #[deku(count = "3", endian = "big", bits = "17")]
    maxscl: Vec<u32>,

    #[deku(endian = "big", bits = "17")]
    average_maxrgb: u32,

    #[deku(bits = "4")]
    num_distribution_maxrgb_percentiles: u8,

    #[deku(count = "num_distribution_maxrgb_percentiles")]
    distribution_maxrgb: Vec<DistributionMaxRgb>
}

#[derive(Debug, PartialEq, DekuRead)]
pub struct DistributionMaxRgb {
    #[deku(bits = "7")]
    percentage: u8,
    #[deku(endian = "big", bits = "17")]
    percentile: u32,
}

pub fn parse_metadata(
    is_stdin: bool,
    input: &PathBuf,
    verify: bool,
) -> Result<Vec<Vec<u8>>, std::io::Error> {
    //BufReader & BufWriter
    let stdin = std::io::stdin();
    let mut reader = Box::new(stdin.lock()) as Box<dyn BufRead>;
    let bytes_count;

    let pb: ProgressBar;

    if is_stdin {
        pb = ProgressBar::hidden();
    } else {
        let file = File::open(input).expect("No file found");

        //Info for indicatif ProgressBar
        let file_meta = file.metadata();
        bytes_count = file_meta.unwrap().len() / 100_000_000;

        reader = Box::new(BufReader::new(file));

        if verify {
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

            let tuple = process_bytes(
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
        } else if verify {
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
    let mut correct_indexes;

    print!("{}", Blue.paint("Reading parsed dynamic metadata... "));
    stdout().flush().ok();

    let mut complete_metadata: Vec<Metadata> = Vec::new();

    //Loop over lines and read metadata, HDR10+ LLC format
    for data in input.iter() {
        // Clear x265's injected 0x03 byte if it is present
        // See https://bitbucket.org/multicoreware/x265_git/src/a82c6c7a7d5f5ef836c82941788a37c6a443e0fe/source/encoder/nal.cpp?at=master#lines-119:136
        let bytes = data.iter()
            .enumerate()
            .filter_map(|(index, value)| {
                if index > 2 && index < data.len() - 2 && data[index - 2] == 0 && data[index - 1] == 0 && data[index] <= 3 {
                    None
                } else {
                    Some(*value)
                }
            })
            .collect::<Vec<u8>>();

        println!("{:?}", bytes);

        //let test_metadata = TestMetadata::from_bytes((bytes, 0)).unwrap();
        //println!("{:?}", test_metadata);

        let mut reader = BitReader::new(&bytes);

        reader.read_u8(8).unwrap(); //country_code
        reader.read_u16(16).unwrap(); //terminal_provider_code
        reader.read_u16(16).unwrap(); //terminal_provider_oriented_code
        let application_identifier = reader.read_u8(8).unwrap(); //application_identifier
        let application_version = reader.read_u8(8).unwrap(); //application_version

        // SMPTE ST-2094 Application 4, Version 1
        assert_eq!(application_identifier, 4);
        assert_eq!(application_version, 1);

        let num_windows = reader.read_u8(2).unwrap();

        // Versions up to 1.2 should be 1
        for _w in 1..num_windows {
            println!("num_windows > 1");
            panic!("The value of num_windows shall be 1 in this version");
        }

        let targeted_system_display_maximum_luminance = reader.read_u32(27).unwrap();
        let targeted_system_display_actual_peak_luminance_flag = reader.read_u8(1).unwrap();

        // The value of targeted_system_display_maximum_luminance shall be in the range of 0 to 10000, inclusive
        assert!(targeted_system_display_maximum_luminance <= 10000);

        let mut targeted_system_display_actual_peak_luminance: Vec<Vec<u8>> = Vec::new();

        // Versions up to 1.2 should be 0
        if targeted_system_display_actual_peak_luminance_flag == 1 {
            let num_rows_targeted_system_display_actual_peak_luminance = reader.read_u8(5).unwrap();
            let num_cols_targeted_system_display_actual_peak_luminance = reader.read_u8(5).unwrap();

            for i in 0..num_rows_targeted_system_display_actual_peak_luminance {
                targeted_system_display_actual_peak_luminance.push(Vec::new());

                for _j in 0..num_cols_targeted_system_display_actual_peak_luminance {
                    targeted_system_display_actual_peak_luminance[i as usize]
                        .push(reader.read_u8(4).unwrap());
                }
            }

            println!("Targeted system display actual peak luminance flag enabled");
            panic!("The value of targeted_system_display_actual_peak_luminances shall be 0 in this version");
        }

        let mut average_maxrgb: u32 = 0;
        let mut maxscl: Vec<u32> = Vec::new();

        let mut distribution_index: Vec<u8> = Vec::new();
        let mut distribution_values: Vec<u32> = Vec::new();

        for _w in 0..num_windows {
            for i in 0..3 {
                let mut maxscl_high = reader.read_u32(17).unwrap();

                maxscl.push(maxscl_high);
            }

            // Shall be under 100000.
            maxscl.iter().for_each(|&v| assert!(v <= 100_000));

            // Read average max RGB
            average_maxrgb = reader.read_u32(17).unwrap();

            // Shall be under 100000
            assert!(average_maxrgb <= 100_000);

            let num_distribution_maxrgb_percentiles = reader.read_u8(4).unwrap();

            // The value of num_distribution_maxrgb_percentiles shall be 9
            // or 10 if your name is Amazon, apparently
            if num_distribution_maxrgb_percentiles == 9 {
                correct_indexes = vec![1, 5, 10, 25, 50, 75, 90, 95, 99];
            } else if num_distribution_maxrgb_percentiles == 10 {
                correct_indexes = vec![1, 5, 10, 25, 50, 75, 90, 95, 98, 99];
            } else {
                panic!(
                    "Invalid number of percentiles: {}",
                    num_distribution_maxrgb_percentiles
                );
            }

            for _i in 0..num_distribution_maxrgb_percentiles {
                distribution_index.push(reader.read_u8(7).unwrap());
                distribution_values.push(reader.read_u32(17).unwrap());
            }

            // Distribution indexes should be equal to:
            // 9 indexes: [1, 5, 10, 25, 50, 75, 90, 95, 99]
            // 10 indexes: [1, 5, 10, 25, 50, 75, 90, 95, 98, 99]
            assert_eq!(distribution_index, correct_indexes);

            reader.read_u16(10).unwrap(); //fraction_bright_pixels, unused for now
        }

        let mastering_display_actual_peak_luminance_flag = reader.read_u8(1).unwrap();
        let mut mastering_display_actual_peak_luminance: Vec<Vec<u8>> = Vec::new();

        // Versions up to 1.2 should be 0
        if mastering_display_actual_peak_luminance_flag == 1 {
            let num_rows_mastering_display_actual_peak_luminance = reader.read_u8(5).unwrap();
            let num_cols_mastering_display_actuak_peak_luminance = reader.read_u8(5).unwrap();

            for i in 0..num_rows_mastering_display_actual_peak_luminance {
                mastering_display_actual_peak_luminance.push(Vec::new());

                for _j in 0..num_cols_mastering_display_actuak_peak_luminance {
                    mastering_display_actual_peak_luminance[i as usize]
                        .push(reader.read_u8(4).unwrap());
                }
            }

            println!("Mastering display actual peak luminance flag enabled");
            panic!("The value of mastering_display_actual_peak_luminance_flag shall be 0 for this version");
        }

        let mut knee_point_x: u16 = 0;
        let mut knee_point_y: u16 = 0;

        let mut bezier_curve_anchors: Vec<u16> = Vec::new();

        for _w in 0..num_windows {
            let tone_mapping_flag = reader.read_u8(1).unwrap();

            if tone_mapping_flag == 1 {
                knee_point_x = reader.read_u16(12).unwrap();
                knee_point_y = reader.read_u16(12).unwrap();

                // The value of knee_point_x shall be in the range of 0 to 1, and in multiples of 1/4095
                assert!(knee_point_x <= 4095);
                assert!(knee_point_y <= 4095);

                let num_bezier_curve_anchors = reader.read_u8(4).unwrap();

                for _i in 0..num_bezier_curve_anchors {
                    bezier_curve_anchors.push(reader.read_u16(10).unwrap());
                }
            }
        }

        let color_saturation_mapping_flag = reader.read_u8(1).unwrap();

        // Versions up to 1.2 should be 0
        if color_saturation_mapping_flag == 1 {
            println!("Color saturation mapping flag enabled");
            panic!("The value of color_saturation_mapping_flag shall be 0 for this version");
        }

        let meta = Metadata {
            num_windows,
            targeted_system_display_maximum_luminance,
            average_maxrgb,
            maxscl,
            distribution_index,
            distribution_values,
            knee_x: knee_point_x,
            knee_y: knee_point_y,
            bezier_curve_data: bezier_curve_anchors,
        };

        // Debug
        println!("{:?}", meta);

        complete_metadata.push(meta);
    }

    println!("{}", Green.paint("Done."));

    complete_metadata
}

fn write_json(output: PathBuf, metadata: Vec<Metadata>, force_single_profile: bool) {
    let save_file = File::create(output).expect("Can't create file");
    let mut writer = BufWriter::with_capacity(10_000_000, save_file);

    print!("{}", Blue.paint("Writing metadata to JSON file... "));
    stdout().flush().ok();

    // Get highest number of anchors (should be constant across frames other than empty)
    let num_bezier_curve_anchors = match metadata.iter().map(|m| m.bezier_curve_data.len()).max() {
        Some(max) => max,
        None => 0,
    };

    // Use max with 0s instead of empty
    let replacement_curve_data = vec![0; num_bezier_curve_anchors];
    let mut warning = None;

    let mut profile = "A";

    let frame_json_list: Vec<Value> = metadata
        .iter()
        .map(|m| {
            // Profile A, no bezier curve data
            if m.targeted_system_display_maximum_luminance == 0 && m.bezier_curve_data.is_empty() && num_bezier_curve_anchors == 0 {
                json!({
                    "LuminanceParameters": {
                        "AverageRGB": m.average_maxrgb,
                        "LuminanceDistributions": {
                            "DistributionIndex": m.distribution_index,
                            "DistributionValues": m.distribution_values,
                        },
                        "MaxScl": m.maxscl
                    },
                    "NumberOfWindows": m.num_windows,
                    "TargetedSystemDisplayMaximumLuminance": m.targeted_system_display_maximum_luminance
                })
            } else { // Profile B
                if profile != "B" {
                    profile = "B";
                }

                // Don't insert empty vec when profile B and forcing single profile
                let bezier_curve_data = if force_single_profile && m.bezier_curve_data.is_empty() && num_bezier_curve_anchors != 0 {
                    if warning.is_none() {
                        warning = Some(format!("{}", Yellow.paint("Forced profile B.")));
                    }

                    &replacement_curve_data
                } else {
                    if warning.is_none() && m.bezier_curve_data.is_empty() && num_bezier_curve_anchors != 0 {
                        warning = Some(format!("{} Different profiles appear to be present in the metadata, this can cause errors when used with x265.\nUse {} to \"fix\".", Yellow.paint("Warning:"), Yellow.paint("--force-single-profile")));
                    }

                    &m.bezier_curve_data
                };

                json!({
                    "BezierCurveData": {
                        "Anchors": bezier_curve_data,
                        "KneePointX": m.knee_x,
                        "KneePointY": m.knee_y
                    },
                    "LuminanceParameters": {
                        "AverageRGB": m.average_maxrgb,
                        "LuminanceDistributions": {
                            "DistributionIndex": m.distribution_index,
                            "DistributionValues": m.distribution_values,
                        },
                        "MaxScl": m.maxscl
                    },
                    "NumberOfWindows": m.num_windows,
                    "TargetedSystemDisplayMaximumLuminance": m.targeted_system_display_maximum_luminance
                })
            }

        })
        .collect::<Vec<Value>>();

    let json_info = json!({
        "HDR10plusProfile": profile,
        "Version": "1.0",
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
