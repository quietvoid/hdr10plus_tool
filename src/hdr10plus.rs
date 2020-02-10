use bitreader::BitReader;
use indicatif::{ProgressBar, ProgressStyle};
use read_byte_slice::{ByteSliceIter, FallibleStreamingIterator};
use serde_json::{json, Value};
use std::fs::File;
use std::io::{stdout, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

pub fn process_file(is_stdin: bool, input: &PathBuf, output: PathBuf, verify: bool) {
    let final_metadata: Vec<Metadata>;

    match parse_metadata(is_stdin, input, verify) {
        Ok(vec) => {
            //Match returned vec to check for --verify
            if vec[0][0] == 1 && vec[0].len() == 1 {
                println!("Dynamic HDR10+ metadata detected.");
            } else {
                final_metadata = llc_read_metadata(vec);
                //Sucessful parse & no --verify
                if !final_metadata.is_empty() {
                    write_json(output, final_metadata)
                } else {
                    println!("Failed reading parsed metadata.");
                }
            }
        }
        Err(e) => println!("{}", e),
    }
}

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

    println!("Parsing HEVC file for dynamic metadata... ");
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
            pb.finish();
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "File doesn't contain dynamic metadata, stopping.",
            ));
        } else if verify {
            pb.finish();

            let verified = vec![vec![1]];

            return Ok(verified);
        }

        if cur_byte >= 100_000_000 {
            pb.inc(1);
            cur_byte = 0;
        }
    }

    pb.finish();

    Ok(final_sei_list)
}

fn process_bytes(
    header: &Vec<u8>,
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
                if current_sei == header {
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
    let correct_indexes = [1, 5, 10, 25, 50, 75, 90, 95, 99];
    let expected_num_percentiles = 9;

    print!("Reading parsed dynamic metadata... ");
    stdout().flush().ok();

    let mut complete_metadata: Vec<Metadata> = Vec::new();

    //Loop over lines and read metadata, HDR10+ LLC format
    for data in input.iter() {

        let bytes = &data[..];

        let mut reader = BitReader::new(bytes);
        let mut temp_reader = reader.relative_reader();

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

        // For LLC, when 0, skip 1 byte
        if targeted_system_display_maximum_luminance == 0 || targeted_system_display_maximum_luminance == 8192 {
            temp_reader = reader.relative_reader();
            if temp_reader.read_u8(8).unwrap() != 0 && targeted_system_display_maximum_luminance == 8192 {
                
            } else {
                reader.read_u32(8).unwrap();
            }
        }

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
        let mut original_maxscl: Vec<u32> = Vec::new();

        let mut distribution_index: Vec<u8> = Vec::new();
        let mut distribution_values: Vec<u32> = Vec::new();

        for _w in 0..num_windows {
            for i in 0..3 {
                let mut maxscl_high = reader.read_u32(17).unwrap();

                println!("{}", maxscl_high);

                if i == 0 {
                    temp_reader = reader.relative_reader();
                    let skipped_byte = temp_reader.read_u32(8).unwrap();
                    let first_temp = temp_reader.read_u32(17).unwrap();

                    if maxscl_high == 0 {
                        if skipped_byte == 1 && first_temp >= 65536 && first_temp <= 100_000 {
                            if targeted_system_display_maximum_luminance != 8192 {
                                reader.read_u32(8).unwrap();
                            }
                        }
                    } else if maxscl_high == 1 && (targeted_system_display_maximum_luminance == 0 || targeted_system_display_maximum_luminance % 32 == 0) {
                        reader.read_u32(1).unwrap();
                        maxscl_high = reader.read_u32(7).unwrap();
                    } else if maxscl_high == 32768 || maxscl_high == 65536 || maxscl_high == 98304 {
                        if targeted_system_display_maximum_luminance == 8192 {
                            maxscl_high = maxscl_high - 98304;

                            if skipped_byte == 1 {
                                reader.read_u32(8).unwrap();
                            }
                        } else {
                            reader.read_u32(8).unwrap();
                        }
                    } else if maxscl_high >= 98304 && targeted_system_display_maximum_luminance == 8192 {
                        maxscl_high = maxscl_high - 98304;
                    }

                    maxscl.push(maxscl_high);
                } else if i == 1 {
                    temp_reader = reader.relative_reader();
                    let second_temp = temp_reader.read_u32(8).unwrap();

                    if maxscl[i - 1] == 0 || maxscl[i - 1] == 32768 || maxscl[i - 1] == 65536 || maxscl[i - 1] == 98304 {
                        if maxscl_high >= 65536 {
                            let temp: u32 = maxscl_high - 65536;
    
                            if temp == 0 {
                                if second_temp < 4 {
                                    reader.read_u32(8).unwrap();
                                }
                            }

                            if temp != 0 && targeted_system_display_maximum_luminance == 8192 {
                                maxscl.push(maxscl_high);
                            } else {
                                maxscl.push(temp);
                            }
                        } else {
                            if maxscl_high == 3 && (targeted_system_display_maximum_luminance == 0 || targeted_system_display_maximum_luminance % 32 == 0) {
                                maxscl.push(reader.read_u32(8).unwrap());
                            } else {
                                maxscl.push(maxscl_high);
                            }
                        }
                    } else {
                        if maxscl_high == 0 {
                            if second_temp < 4 {
                                reader.read_u32(8).unwrap();
                            }
                            
                            maxscl.push(maxscl_high);
                        } else {
                            if maxscl_high == 3 && (reader.remaining() == 344 || reader.remaining() == 336 || reader.remaining() == 328) {
                                if maxscl[i - 1] < 2048 {
                                    if maxscl[i - 1] >= 128 && maxscl[i -1] < 2048 && maxscl[i - 1] % 128 == 0 {
                                        maxscl_high = reader.read_u32(8).unwrap();
                                    }

                                    maxscl.push(maxscl_high);
                                } else {
                                    maxscl.push(reader.read_u32(8).unwrap());
                                }
                            } else {
                                maxscl.push(maxscl_high);
                            }
                        }
                    }
                } else if maxscl[i - 1] == 0 {
                    if maxscl_high == 6 && reader.remaining() != 303 {
                        maxscl.push(reader.read_u32(8).unwrap());
                    } else {
                        maxscl.push(maxscl_high);
                    }
                } else {
                    temp_reader = reader.relative_reader();
                    let third_temp = temp_reader.read_u32(8).unwrap();
                    
                    if maxscl_high == 6 && (reader.remaining() == 319 || reader.remaining() == 311) {
                        println!("bad {}", third_temp);
                        if maxscl[i - 1] <= 2048 && third_temp != 6 {
                            maxscl.push(reader.read_u32(8).unwrap());
                        } else {
                            if maxscl[i - 1] > 2048 && maxscl[i - 1] % 256 == 0 {
                                maxscl_high = reader.read_u32(8).unwrap();
                            } else {
                                reader.read_u32(8).unwrap();
                            }

                            maxscl.push(maxscl_high);
                        }
                    } else {
                        if (maxscl_high == 0 && maxscl[i - 1] > 2048) || (maxscl_high == 0 && third_temp == 6 && reader.remaining() == 311) {
                            reader.read_u32(8).unwrap();
                        }

                        maxscl.push(maxscl_high);
                    }
                }

                original_maxscl.push(maxscl_high);
            }

            // Shall be under 100000.
            maxscl.iter().for_each(|&v| assert!(v <= 100_000));

            if maxscl[2] == 0 {
                temp_reader = reader.relative_reader();

                // Skip a byte
                temp_reader.read_u32(8).unwrap();

                // Average max rgb
                let temp_avg_maxrgb = temp_reader.read_u32(17).unwrap();

                let mut future_reader = temp_reader.relative_reader();
                
                // Skip another byte
                let skipped_byte = future_reader.read_u32(8).unwrap();

                if future_reader.read_u8(4).unwrap() == expected_num_percentiles || temp_reader.read_u8(4).unwrap() == expected_num_percentiles {
                    if skipped_byte == 144 && (temp_avg_maxrgb >= 3072 && temp_avg_maxrgb <= 3087) {
                    } else {
                        reader.read_u32(8).unwrap();
                    }
                }
            }

            // Read average max RGB
            average_maxrgb = reader.read_u32(17).unwrap();

            println!("{:?} {}", maxscl, targeted_system_display_maximum_luminance);
            println!("1st {}", average_maxrgb);

            temp_reader = reader.relative_reader();

            // Try to skip a byte
            temp_reader.read_u32(8).unwrap();

            // If the percentiles are correct, go ahead and use next value
            if temp_reader.read_u8(4).unwrap() == expected_num_percentiles {
                average_maxrgb = reader.read_u32(8).unwrap();
            }

            println!("2nd {}", average_maxrgb);

            // Shall be under 100000
            assert!(average_maxrgb <= 100_000);

            let num_distribution_maxrgb_percentiles = reader.read_u8(4).unwrap();

            // The value of num_distribution_maxrgb_percentiles shall be 9
            assert_eq!(num_distribution_maxrgb_percentiles, 9);

            for _i in 0..num_distribution_maxrgb_percentiles {
                distribution_index.push(reader.read_u8(7).unwrap());
                distribution_values.push(reader.read_u32(17).unwrap());
            }

            // Distribution indexes should be equal to  [1, 5, 10, 25, 50, 75, 90, 95, 99]
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

        /* Debug
        println!("NumWindows: {}, Targeted Display Luminance: {}", num_windows, targeted_system_display_maximum_luminance);
        println!("AverageRGB: {}, MaxScl: {:?}", average_maxrgb, maxscl);
        println!("NumPercentiles: {}\nDistributionIndex: {:?}\nDistributionValues: {:?}", num_distribution_maxrgb_percentiles, distribution_index, distribution_values);
        println!("Knee_X: {}, Knee_Y: {}, Anchors: {:?}\n", knee_point_x, knee_point_y, bezier_curve_anchors);
        */

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

        complete_metadata.push(meta);
    }

    println!("Done.");

    complete_metadata
}

fn write_json(output: PathBuf, metadata: Vec<Metadata>) {
    let save_file = File::create(output).expect("Can't create file");
    let mut writer = BufWriter::with_capacity(10_000_000, save_file);

    print!("Writing metadata to JSON file... ");
    stdout().flush().ok();

    let frame_json_list: Vec<Value> = metadata
        .iter()
        .map(|m| {
            if m.targeted_system_display_maximum_luminance == 0 && m.bezier_curve_data.is_empty() {
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
            } else {
                json!({
                    "BezierCurveData": {
                        "Anchors": m.bezier_curve_data,
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

    let final_json = json!({ "SceneInfo": frame_json_list });

    assert!(writeln!(
        writer,
        "{}",
        serde_json::to_string_pretty(&final_json).unwrap()
    )
    .is_ok());

    println!("Done.");

    writer.flush().ok();
}
