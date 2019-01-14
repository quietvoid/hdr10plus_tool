use std::io::{stdout, stdin, Seek, Write, BufRead, Read, BufReader, BufWriter, SeekFrom};
use std::path::Path;
use serde_json::Value;
use std::fs::File;
use bitreader::BitReader;
use std::process;
use std::env;
use indicatif::{ProgressBar, ProgressStyle};
use read_byte_slice::{ByteSliceIter, FallibleStreamingIterator};

fn main() {

    let mut input = String::new();

    let mut args: Vec<String> = env::args().collect();

    if args.is_empty(){
        print!("Enter path to HEVC file: ");
        stdout().flush().ok();

        match stdin().read_line(&mut input){
            Ok(_) =>{
                input = input.trim().to_string();
                process_input(input);
            }
            Err(error) => println!("Error: {}", error),
        }
    }
    else if args.len() == 2{
        input = args.pop().unwrap();
        input = input.trim().to_string();

        process_input(input);
    }
}

fn process_input(input: String){
    let path = Path::new(&input);
    let parent_dir = path.parent().unwrap();
    let save_str = parent_dir.join(path.file_name().unwrap()).to_str().unwrap().to_string();


    if !path.is_file(){
        println!("Invalid file path.");
        process::exit(1);
    }

    let path_str = path.to_str().unwrap().to_string();
    if path_str.contains(".h265") || path_str.contains(".hevc"){

        let log_file = format!("{}-sei.log", save_str);
        let metadata_file = format!("{}-meta.json", save_str);

        match parse_metadata(path_str, &log_file){
            Ok(_) => llc_metadata_to_json(&log_file, metadata_file),
            Err(e) => println!("{}", e)
        }
    }
    else{
        println!("Invalid file type.");
    }
}

fn parse_metadata(input: String, log: &String) -> Result<String, std::io::Error>{

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

struct Metadata{
    BezierCurveData: Vec<u16>,
    Knee_X: u16,
    Knee_Y: u16,
    Average_MaxRGB: u16,
    MaxScl: Vec<u16>,
    DistributionIndex: Vec<u8>,
    DistributionValues: Vec<u32>,
    TargetedSystemDisplayMaximumLuminance: u32,
    NumWindows: u8
}

fn llc_metadata_to_json(input: &String, metadata: String) {

    //Input
    let f = File::open(input).expect("No file found");
    let mut reader = BufReader::new(f);

    let maxscl_arr = [1,3,6];

    print!("Generating HDR10+ metadata JSON file... ");
    stdout().flush().ok();

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
        if _targeted_system_display_maximum_luminance == 0{
            reader.read_u8(8).unwrap();
        }

        if _targeted_system_display_actual_peak_luminance_flag == 1{
            println!("Targeted peak flag enabled");
        }

        //println!("NumWindows: {}, Targeted Display Luminance: {}", num_windows, targeted_system_display_maximum_luminance);

        let mut average_maxrgb: u16 = 0;
        let mut maxscl: Vec<u16> = Vec::new();

        let mut num_distribution_maxrgb_percentiles= 0;
        let mut distribution_index: Vec<u8> = Vec::new();
        let mut distribution_values: Vec<u32> = Vec::new();

        let mut fraction_bright_pixels = 0;

        for _w in 0..num_windows{
            for _i in 0..3{
                reader.read_u16(1).unwrap(); //maxscl >> 16
                let maxscl_high = reader.read_u16(16).unwrap();

                /*
                    For LLC, when maxscl == 1,3 or 6, push next byte
                */
                if targeted_system_display_maximum_luminance == 0 
                    && maxscl_arr[_i] == maxscl_high{

                    reader.read_u8(1).unwrap();
                    let x = reader.read_u16(7).unwrap();

                    _maxscl.push(x);
                }
                else if maxscl_high == 0{
                    reader.read_u8(8).unwrap();
                    maxscl.push(maxscl_high);
                }
                else{
                    maxscl.push(maxscl_high);
                }
            }

            reader.read_u8(1).unwrap();
            average_maxrgb = reader.read_u16(16).unwrap();

            /*
                For LLC, AverageRGB < 16 so next byte is the actual value,
                otherwise it's the 16 bits taken before.
            */
            if average_maxrgb == 12{
                average_maxrgb = reader.read_u16(8).unwrap();
            }

            num_distribution_maxrgb_percentiles = reader.read_u8(4).unwrap();

            for _i in 0..num_distribution_maxrgb_percentiles{
                distribution_index.push(reader.read_u8(7).unwrap());
                distribution_values.push(reader.read_u32(17).unwrap());
            }

            fraction_bright_pixels = reader.read_u16(10).unwrap();
        }

        //println!("AverageRGB: {}, MaxScl: {:?}", _average_maxrgb, _maxscl);
        //println!("NumPercentiles: {}\nDistributionIndex: {:?}\nDistributionValues: {:?}", num_distribution_maxrgb_percentiles, distribution_index, distribution_values);

        let mastering_display_actual_peak_luminance_flag = reader.read_u8(1).unwrap();

        //0 for now
        if _mastering_display_actual_peak_luminance_flag == 1{
            println!("Mastering peak flag enabled");
        }

        let mut knee_point_x: u16 = 0;
        let mut knee_point_y: u16 = 0;
        let mut num_bezier_curve_anchors: u8 = 0;

        let mut bezier_curve_anchors: Vec<u16> = Vec::new();

        let mut color_saturation_mapping_flag: u8 = 0;

        for _w in 0..num_windows{
            let tone_mapping_flag = reader.read_u8(1).unwrap();

            if tone_mapping_flag == 1{

                knee_point_x = reader.read_u16(12).unwrap();
                knee_point_y = reader.read_u16(12).unwrap();
                num_bezier_curve_anchors = reader.read_u8(4).unwrap();

                for _i in 0..num_bezier_curve_anchors{
                    bezier_curve_anchors.push(reader.read_u16(10).unwrap());
                }

                //println!("Knee_X: {}, Knee_Y: {}, Anchors: {:?}\n", knee_point_x, knee_point_y, bezier_curve_anchors);
            }
        }

        color_saturation_mapping_flag = reader.read_u8(1).unwrap();

        //0 for now
        if color_saturation_mapping_flag == 1{
            println!("Color saturation mapping flag enabled");
        }
    }

    println!("Done.");
}

fn write_json(input: String, meta: Vec<Metadata>){
    let save_file = File::create(metadata).expect("Can't create file");
    let mut writer = BufWriter::with_capacity(10000000, save_file);

    if let Err(_) = writeln!(writer, "{{\n\t\"SceneInfo\": ["){
        eprintln!("Couldn't write to file");
    }

    //Prepare BezierCurveData JSON string
    let mut anchors_str = String::new();
    for a in 0..num_bezier_curve_anchors{
        let anchor_v = bezier_curve_anchors[a as usize];

        let mut anchor = format!("\t{}, \n", anchor_v);

        if a == num_bezier_curve_anchors - 1 {
            anchor = format!("\t{}", anchor_v);
        }

        anchors_str.push_str(anchor.as_str());
    }

    //Prepare Distribution JSON string
    let mut index_str = String::new();
    let mut values_str = String::new();
    for a in 0..num_distribution_maxrgb_percentiles{
        let index_v = _distribution_index[a as usize];
        let values_v = _distribution_values[a as usize];

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
        let value = _maxscl[a as usize];
        let mut maxscl_l = format!("\t{},\n", value);

        if a == 2{
            maxscl_l = format!("\t{}", value);
        }
        maxscl_str.push_str(maxscl_l.as_str());
    }

    let bezier_data = format!("\"BezierCurveData\": {{\n\"Anchors\": [\n{}\n],\n\"KneePointX\": {},\n\"KneePointY\": {}\n}},\n", anchors_str, _knee_point_x, _knee_point_y);
    let luminance_data = format!("\"LuminanceParameters\": {{\n\"AverageRGB\": {},\n\"LuminanceDistributions\": {{\n\"DistributionIndex\": [\n{}\n],\n\"DistributionValues\": [\n{}\n]\n}},\n\"MaxScl\": [\n{}\n]\n}},\n", _average_maxrgb, index_str, values_str, maxscl_str);
    let windows_data: String = format!("\"NumberOfWindows\": {},\n\"TargetedSystemDisplayMaximumLuminance\": {}", _num_windows, _targeted_system_display_maximum_luminance);

    let final_str;

    //Only add BezierCurveData JSON if it's available, no empty array.
    if num_bezier_curve_anchors != 0{
        final_str = format!("{{\n{}{}{}\n}}", bezier_data, luminance_data, windows_data);
    }
    else{
        final_str = format!("{{\n{}{}\n}}", luminance_data, windows_data);
    }

    //println!("{}", final_str);

    let json: Value = serde_json::from_str(&final_str).unwrap();

    let mut json_final = serde_json::to_string_pretty(&json).unwrap();

    if frame != number_frames{
        json_final.push(',');
    }

    frame += 1;

    if let Err(_) = writeln!(writer, "{}", json_final){
        eprintln!("Couldn't write to file");
    }

    if let Err(_) = writeln!(writer, "]\n}}"){
        eprintln!("Couldn't write to file");
    }

    writer.flush().ok();
}