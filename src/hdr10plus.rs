pub mod hdr10plus{
    use std::io::{stdout, Write, BufRead, BufReader, BufWriter};
    use indicatif::{ProgressBar, ProgressStyle};
    use read_byte_slice::{ByteSliceIter, FallibleStreamingIterator};
    use serde_json::Value;
    use std::fs::File;
    use bitreader::BitReader;

    pub struct Metadata{
        pub bezier_curve_data: Vec<u16>,
        pub knee_x: u16,
        pub knee_y: u16,
        pub average_maxrgb: u16,
        pub maxscl: Vec<u16>,
        pub distribution_index: Vec<u8>,
        pub distribution_values: Vec<u32>,
        pub targeted_system_display_maximum_luminance: u32,
        pub num_windows: u8
    }

    pub fn parse_metadata(input: String, log: &String) -> Result<String, std::io::Error>{

        //Input
        let f = File::open(input).expect("No file found");

        //Info for indicatif ProgressBar
        let file_meta = f.metadata();
        let bytes_count = file_meta.unwrap().len() / 100000000;
        let mut cur_byte = 0;

        let pb = ProgressBar::new(bytes_count);
        pb.set_style(ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:60.cyan} {percent}%"));

        //BufReader & BufWriter
        let reader = BufReader::new(f);
        let save_file = File::create(log).expect("Can't create file");
        let mut writer = BufWriter::with_capacity(10000000, save_file);

        //Bitstream blocks for SMPTE 2094-40
        let header: Vec<u8> = vec![0, 0, 1, 78, 1, 4];
        let mut current_sei: Vec<u8> = Vec::new();
        let mut dynamic_hdr_sei = false;


        println!("Parsing HEVC file for dynamic metadata... ");
        stdout().flush().ok();

        let mut iter = ByteSliceIter::new(reader, 100000);
        let mut dynamic_detected = false;

        //Loop over iterator of byte chunks for faster I/O
        while let Some(chunk) = iter.next()? {
            for byte in chunk {
                let byte = *byte;

                cur_byte += 1;
                current_sei.push(byte);

                if dynamic_hdr_sei{
                    let last = current_sei.len() - 1;

                    if current_sei[last-3] == 128 && current_sei[last-2] == 0 && current_sei[last-1] == 0 && current_sei[last] == 1{

                        let final_sei = &current_sei[7 .. current_sei.len() - 3];

                        if let Err(_) = writeln!(writer, "{:?}", final_sei){
                            eprintln!("Couldn't write to file");
                        }

                        current_sei.clear();
                        dynamic_hdr_sei = false;

                        dynamic_detected = true;
                    }
                }
                else if byte == 0 || byte == 1 || byte == 78 || byte == 4{
                    for i in 0..current_sei.len(){
                        if current_sei[i] == header[i]{
                            if current_sei == header{
                                dynamic_hdr_sei = true;
                                break;
                            }
                        }
                        else{
                            current_sei.clear();
                            break;
                        }
                    }
                }
                else if current_sei.len() != 0{
                    current_sei.clear();
                }
            }

            if !dynamic_detected{
                pb.finish();
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "File doesn't contain dynamic metadata, stopping."));
            }

            if cur_byte >= 100000000{
                pb.inc(1);
                cur_byte = 0;
            }
        }

        pb.finish();

        writer.flush().ok();
        Ok(String::from("Done."))
    }

    pub fn llc_read_metadata(input: &String) -> Vec<Metadata>{

        //Input
        let f = File::open(input).expect("No file found");
        let reader = BufReader::new(f);

        let maxscl_arr = [1,3,6];

        print!("Generating HDR10+ metadata JSON file... ");
        stdout().flush().ok();

        let mut complete_metadata: Vec<Metadata> = Vec::new();

        //Loop over lines and read metadata, HDR10+ LLC format
        for line in reader.lines().map(|l| l.unwrap()) {

            let line: String = line.chars().skip(1).take(line.len()-2).collect();

            let data: Vec<u8> = line.split(", ").map(|x| x.parse::<u8>().unwrap()).collect();
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
            if targeted_system_display_maximum_luminance == 0{
                reader.read_u8(8).unwrap();
            }

            if targeted_system_display_actual_peak_luminance_flag == 1{
                println!("Targeted peak flag enabled");
            }

            let mut average_maxrgb: u16 = 0;
            let mut maxscl: Vec<u16> = Vec::new();

            let mut num_distribution_maxrgb_percentiles: u8;
            let mut distribution_index: Vec<u8> = Vec::new();
            let mut distribution_values: Vec<u32> = Vec::new();

            for _w in 0..num_windows{
                for _i in 0..3{
                    reader.read_u16(1).unwrap(); //input maxscl >> 16
                    let maxscl_high = reader.read_u16(16).unwrap();

                    /*
                        For LLC, when maxscl == 1,3 or 6, push next byte
                    */
                    if targeted_system_display_maximum_luminance == 0 
                        && maxscl_arr[_i] == maxscl_high{

                        reader.read_u8(1).unwrap();
                        let x = reader.read_u16(7).unwrap();

                        maxscl.push(x);
                    }
                    else if maxscl_high == 0{
                        reader.read_u8(8).unwrap();
                        maxscl.push(maxscl_high);
                    }
                    else{
                        maxscl.push(maxscl_high);
                    }
                }

                reader.read_u8(1).unwrap(); //input maxrgb >> 16
                average_maxrgb = reader.read_u16(16).unwrap();

                /*
                    For LLC, AverageRGB < 16 and MaxScl is all 0, use next byte.
                */
                if average_maxrgb < 16 && maxscl == vec![0,0,0]{
                   average_maxrgb = reader.read_u16(8).unwrap();
                }

                num_distribution_maxrgb_percentiles = reader.read_u8(4).unwrap();

                for _i in 0..num_distribution_maxrgb_percentiles{
                    distribution_index.push(reader.read_u8(7).unwrap());
                    distribution_values.push(reader.read_u32(17).unwrap());
                }

                reader.read_u16(10).unwrap(); //fraction_bright_pixels, unused for now
            }

            let mastering_display_actual_peak_luminance_flag = reader.read_u8(1).unwrap();

            //0 for now
            if mastering_display_actual_peak_luminance_flag == 1{
                println!("Mastering peak flag enabled");
            }

            let mut knee_point_x: u16 = 0;
            let mut knee_point_y: u16 = 0;

            let mut bezier_curve_anchors: Vec<u16> = Vec::new();

            for _w in 0..num_windows{
                let tone_mapping_flag = reader.read_u8(1).unwrap();

                if tone_mapping_flag == 1{

                    knee_point_x = reader.read_u16(12).unwrap();
                    knee_point_y = reader.read_u16(12).unwrap();
                    let num_bezier_curve_anchors = reader.read_u8(4).unwrap();

                    for _i in 0..num_bezier_curve_anchors{
                        bezier_curve_anchors.push(reader.read_u16(10).unwrap());
                    }
                }
            }

            let color_saturation_mapping_flag = reader.read_u8(1).unwrap();

            //0 for now
            if color_saturation_mapping_flag == 1{
                println!("Color saturation mapping flag enabled");
            }

            /* Debug
            println!("NumWindows: {}, Targeted Display Luminance: {}", num_windows, targeted_system_display_maximum_luminance);
            println!("AverageRGB: {}, MaxScl: {:?}", average_maxrgb, maxscl);
            println!("NumPercentiles: {}\nDistributionIndex: {:?}\nDistributionValues: {:?}", num_distribution_maxrgb_percentiles, distribution_index, distribution_values);
            println!("Knee_X: {}, Knee_Y: {}, Anchors: {:?}\n", knee_point_x, knee_point_y, bezier_curve_anchors);
            */

            let meta = Metadata {
                num_windows: num_windows,
                targeted_system_display_maximum_luminance: targeted_system_display_maximum_luminance,
                average_maxrgb: average_maxrgb,
                maxscl: maxscl,
                distribution_index: distribution_index,
                distribution_values: distribution_values,
                knee_x: knee_point_x,
                knee_y: knee_point_y,
                bezier_curve_data: bezier_curve_anchors
            };

            complete_metadata.push(meta);
        }

        println!("Done.");

        complete_metadata
    }

    pub fn write_json(input: String, metadata: Vec<Metadata>) {
        let save_file = File::create(input).expect("Can't create file");
        let mut writer = BufWriter::with_capacity(10000000, save_file);

        if let Err(_) = writeln!(writer, "{{\n\t\"SceneInfo\": ["){
            eprintln!("Couldn't write to file");
        }

        let mut frame = 0;
        let max_frames = metadata.len() - 1;

        for m in metadata{

            //Prepare bezier_curve_data JSON string
            let mut anchors_str = String::new();
            let num_bezier_curve_anchors = m.bezier_curve_data.len();
            for a in 0..num_bezier_curve_anchors{
                let anchor_v = m.bezier_curve_data[a as usize];

                let mut anchor = format!("\t{}, \n", anchor_v);

                if a == num_bezier_curve_anchors - 1 {
                    anchor = format!("\t{}", anchor_v);
                }

                anchors_str.push_str(anchor.as_str());
            }

            //Prepare Distribution JSON string
            let mut index_str = String::new();
            let mut values_str = String::new();
            let num_distribution_maxrgb_percentiles = m.distribution_values.len();

            for a in 0..num_distribution_maxrgb_percentiles{
                let index_v = m.distribution_index[a as usize];
                let values_v = m.distribution_values[a as usize];

                let mut index = format!("\t{},\n", index_v);
                let mut values = format!("\t{},\n", values_v);

                if a == num_distribution_maxrgb_percentiles - 1 {
                    index = format!("\t{}", index_v);
                    values = format!("\t{}", values_v);
                }

                index_str.push_str(index.as_str());
                values_str.push_str(values.as_str());
            }

            //Prepare MaxScl JSON string
            let mut maxscl_str: String = String::new();
            for a in 0..3{
                let value = m.maxscl[a as usize];
                let mut maxscl_l = format!("\t{},\n", value);

                if a == 2{
                    maxscl_l = format!("\t{}", value);
                }
                maxscl_str.push_str(maxscl_l.as_str());
            }

            let bezier_data = format!("\"BezierCurveData\": {{\n\"Anchors\": [\n{}\n],\n\"KneePointX\": {},\n\"KneePointY\": {}\n}},\n", anchors_str, m.knee_x, m.knee_y);
            let luminance_data = format!("\"LuminanceParameters\": {{\n\"AverageRGB\": {},\n\"LuminanceDistributions\": {{\n\"DistributionIndex\": [\n{}\n],\n\"DistributionValues\": [\n{}\n]\n}},\n\"MaxScl\": [\n{}\n]\n}},\n", m.average_maxrgb, index_str, values_str, maxscl_str);
            let windows_data: String = format!("\"NumberOfWindows\": {},\n\"TargetedSystemDisplayMaximumLuminance\": {}", m.num_windows, m.targeted_system_display_maximum_luminance);

            let final_str;

            //Only add bezier_curve_data JSON if it's available, no empty array.
            if num_bezier_curve_anchors != 0{
                final_str = format!("{{\n{}{}{}\n}}", bezier_data, luminance_data, windows_data);
            }
            else{
                final_str = format!("{{\n{}{}\n}}", luminance_data, windows_data);
            }

            let json: Value = serde_json::from_str(&final_str).unwrap();

            let mut json_final = serde_json::to_string_pretty(&json).unwrap();

            if frame < max_frames{
                json_final.push(',');
            }

            frame += 1;

            if let Err(_) = writeln!(writer, "{}", json_final){
                eprintln!("Couldn't write to file");
            }
        }

        if let Err(_) = writeln!(writer, "]\n}}"){
            eprintln!("Couldn't write to file");
        }

        writer.flush().ok();
    }
}