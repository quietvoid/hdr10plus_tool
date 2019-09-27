pub mod parser {
    use bitreader::BitReader;
    use indicatif::{ProgressBar, ProgressStyle};
    use read_byte_slice::{ByteSliceIter, FallibleStreamingIterator};
    use serde_json::Value;
    use std::fs::File;
    use std::io::{stdout, BufRead, BufReader, BufWriter, Write};

    pub struct Metadata {
        pub bezier_curve_data: Vec<u16>,
        pub knee_x: u16,
        pub knee_y: u16,
        pub average_maxrgb: u16,
        pub maxscl: Vec<u16>,
        pub distribution_index: Vec<u8>,
        pub distribution_values: Vec<u32>,
        pub targeted_system_display_maximum_luminance: u32,
        pub num_windows: u8,
    }

    pub fn parse_metadata(input: &str, verify: bool) -> Result<Vec<Vec<u8>>, std::io::Error> {
        //BufReader & BufWriter
        let stdin = std::io::stdin();
        let mut reader = Box::new(stdin.lock()) as Box<dyn BufRead>;
        let bytes_count;

        let pb: ProgressBar;
        if input != "-" {
            let file = File::open(input).expect("No file found");

            //Info for indicatif ProgressBar
            let file_meta = file.metadata();
            bytes_count = file_meta.unwrap().len() / 100_000_000;

            reader = Box::new(BufReader::new(file));

            pb = ProgressBar::new(bytes_count);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:60.cyan} {percent}%"),
            );
        } else {
            pb = ProgressBar::hidden();
        }

        //Byte chunk iterator
        let mut iter = ByteSliceIter::new(reader, 100_000);

        //Bitstream blocks for SMPTE 2094-40
        let header: Vec<u8> = vec![0, 0, 1, 78, 1, 4];
        let mut current_sei: Vec<u8> = Vec::new();

        println!("Parsing HEVC file for dynamic metadata... ");
        stdout().flush().ok();

        let mut final_metadata: Vec<Vec<u8>> = Vec::new();

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
                    &mut final_metadata,
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

        Ok(final_metadata)
    }

    fn process_bytes(
        header: &Vec<u8>,
        byte: u8,
        current_sei: &mut Vec<u8>,
        mut dynamic_hdr_sei: bool,
        final_metadata: &mut Vec<Vec<u8>>,
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
                final_metadata.push(final_sei.to_vec());

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
        let maxscl_arr = [1, 3, 6];
        let correct_indexes = [1, 5, 10, 25, 50, 75, 90, 95, 99];

        print!("Reading parsed dynamic metadata... ");
        stdout().flush().ok();

        let mut complete_metadata: Vec<Metadata> = Vec::new();

        //Loop over lines and read metadata, HDR10+ LLC format
        for data in input.iter() {
            let bytes = &data[..];

            let mut reader = BitReader::new(bytes);

            reader.read_u8(8).unwrap(); //country_code
            reader.read_u16(16).unwrap(); //terminal_provider_code
            reader.read_u16(16).unwrap(); //terminal_provider_oriented_code
            reader.read_u8(8).unwrap(); //application_identifier
            reader.read_u8(8).unwrap(); //application_version
            let num_windows = reader.read_u8(2).unwrap();

            let targeted_system_display_maximum_luminance = reader.read_u32(27).unwrap();
            let targeted_system_display_actual_peak_luminance_flag = reader.read_u8(1).unwrap();

            /*
                For LLC, when 0, skip 1 byte
            */
            if targeted_system_display_maximum_luminance == 0 {
                reader.read_u8(8).unwrap();
            }

            if targeted_system_display_actual_peak_luminance_flag == 1 {
                println!("Targeted peak flag enabled");
            }

            let mut average_maxrgb: u16 = 0;
            let mut maxscl: Vec<u16> = Vec::new();

            let mut num_distribution_maxrgb_percentiles: u8;
            let mut distribution_index: Vec<u8> = Vec::new();
            let mut distribution_values: Vec<u32> = Vec::new();

            for _w in 0..num_windows {
                for v in &maxscl_arr {
                    reader.read_u16(1).unwrap(); //input maxscl >> 16
                    let maxscl_high = reader.read_u16(16).unwrap();

                    /*
                        For LLC, when maxscl == 1,3 or 6, push next byte
                    */
                    if targeted_system_display_maximum_luminance == 0 && *v == maxscl_high {
                        reader.read_u8(1).unwrap();
                        let x = reader.read_u16(7).unwrap();

                        maxscl.push(x);
                    } else if maxscl_high == 0 {
                        reader.read_u8(8).unwrap();
                        maxscl.push(maxscl_high);
                    } else {
                        maxscl.push(maxscl_high);
                    }
                }

                reader.read_u8(1).unwrap(); //input maxrgb >> 16
                average_maxrgb = reader.read_u16(16).unwrap();

                /*
                    For LLC, AverageRGB < 16 and MaxScl is all 0, use next byte.
                */
                if average_maxrgb < 16 && maxscl == vec![0, 0, 0] {
                    average_maxrgb = reader.read_u16(8).unwrap();
                }

                num_distribution_maxrgb_percentiles = reader.read_u8(4).unwrap();

                for _i in 0..num_distribution_maxrgb_percentiles {
                    distribution_index.push(reader.read_u8(7).unwrap());
                    distribution_values.push(reader.read_u32(17).unwrap());
                }

                reader.read_u16(10).unwrap(); //fraction_bright_pixels, unused for now
            }

            let mastering_display_actual_peak_luminance_flag = reader.read_u8(1).unwrap();

            //0 for now
            if mastering_display_actual_peak_luminance_flag == 1 {
                println!("Mastering peak flag enabled");
            }

            let mut knee_point_x: u16 = 0;
            let mut knee_point_y: u16 = 0;

            let mut bezier_curve_anchors: Vec<u16> = Vec::new();

            for _w in 0..num_windows {
                let tone_mapping_flag = reader.read_u8(1).unwrap();

                if tone_mapping_flag == 1 {
                    knee_point_x = reader.read_u16(12).unwrap();
                    knee_point_y = reader.read_u16(12).unwrap();
                    let num_bezier_curve_anchors = reader.read_u8(4).unwrap();

                    for _i in 0..num_bezier_curve_anchors {
                        bezier_curve_anchors.push(reader.read_u16(10).unwrap());
                    }
                }
            }

            let color_saturation_mapping_flag = reader.read_u8(1).unwrap();

            //0 for now
            if color_saturation_mapping_flag == 1 {
                println!("Color saturation mapping flag enabled");
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

            // Make sure indexes are correct
            assert_eq!(meta.distribution_index, correct_indexes);

            complete_metadata.push(meta);
        }

        println!("Done.");

        complete_metadata
    }

    pub fn write_json(input: &str, metadata: Vec<Metadata>) {
        let save_file = File::create(input).expect("Can't create file");
        let mut writer = BufWriter::with_capacity(10_000_000, save_file);

        assert!(writeln!(writer, "{{\n\t\"SceneInfo\": [").is_ok());

        let max_frames = metadata.len() - 1;

        print!("Writing metadata to JSON file... ");
        stdout().flush().ok();

        for (frame, m) in metadata.iter().enumerate() {
            //Prepare bezier_curve_data JSON string
            let mut anchors_str = String::new();
            let num_bezier_curve_anchors = m.bezier_curve_data.len();
            for a in 0..num_bezier_curve_anchors {
                let anchor_v = m.bezier_curve_data[a as usize];

                let anchor = if a == num_bezier_curve_anchors - 1 {
                    format!("\t{}", anchor_v)
                } else {
                    format!("\t{}, \n", anchor_v)
                };

                anchors_str.push_str(anchor.as_str());
            }

            //Prepare Distribution JSON string
            let mut index_str = String::new();
            let mut values_str = String::new();
            let num_distribution_maxrgb_percentiles = m.distribution_values.len();

            for a in 0..num_distribution_maxrgb_percentiles {
                let index_v = m.distribution_index[a as usize];
                let values_v = m.distribution_values[a as usize];

                let (index, values) = if a == num_distribution_maxrgb_percentiles - 1 {
                    (format!("\t{}", index_v), format!("\t{}", values_v))
                } else {
                    (format!("\t{},\n", index_v), format!("\t{},\n", values_v))
                };

                index_str.push_str(index.as_str());
                values_str.push_str(values.as_str());
            }

            //Prepare MaxScl JSON string
            let mut maxscl_str: String = String::new();
            for a in 0..3 {
                let value = m.maxscl[a as usize];

                let maxscl_l = if a == 2 {
                    format!("\t{}", value)
                } else {
                    format!("\t{},\n", value)
                };

                maxscl_str.push_str(maxscl_l.as_str());
            }

            let bezier_data = format!("\"BezierCurveData\": {{\n\"Anchors\": [\n{}\n],\n\"KneePointX\": {},\n\"KneePointY\": {}\n}},\n", anchors_str, m.knee_x, m.knee_y);
            let luminance_data = format!("\"LuminanceParameters\": {{\n\"AverageRGB\": {},\n\"LuminanceDistributions\": {{\n\"DistributionIndex\": [\n{}\n],\n\"DistributionValues\": [\n{}\n]\n}},\n\"MaxScl\": [\n{}\n]\n}},\n", m.average_maxrgb, index_str, values_str, maxscl_str);
            let windows_data: String = format!(
                "\"NumberOfWindows\": {},\n\"TargetedSystemDisplayMaximumLuminance\": {}",
                m.num_windows, m.targeted_system_display_maximum_luminance
            );

            //Only add bezier_curve_data JSON if it's available, no empty array.
            let final_str = if num_bezier_curve_anchors != 0 {
                format!("{{\n{}{}{}\n}}", bezier_data, luminance_data, windows_data)
            } else {
                format!("{{\n{}{}\n}}", luminance_data, windows_data)
            };

            let json: Value = serde_json::from_str(&final_str).unwrap();

            let mut json_final = serde_json::to_string_pretty(&json).unwrap();

            if frame < max_frames {
                json_final.push(',');
            }

            assert!(writeln!(writer, "{}", json_final).is_ok());
        }

        assert!(writeln!(writer, "]\n}}").is_ok());

        println!("Done.");

        writer.flush().ok();
    }
}
