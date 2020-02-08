mod hdr10plus;

//Regression tests
#[cfg(test)]
mod tests {
    use crate::hdr10plus::{llc_read_metadata, parse_metadata, Metadata};
    use std::path::PathBuf;

    // x265 Tool_Verification_new_hdr10plus_llc.json 1st frame
    #[test]
    fn sample1() {
        let sample1_file = PathBuf::from("./assets/ToS-s1.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample1_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 400);
        assert_eq!(test[0].average_maxrgb, 1037);
        assert_eq!(test[0].maxscl, vec![17830, 16895, 14252]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![3, 14024, 43, 56, 219, 1036, 2714, 4668, 14445]
        );
        assert_eq!(test[0].knee_x, 17);
        assert_eq!(test[0].knee_y, 64);
        assert_eq!(
            test[0].bezier_curve_data,
            vec![265, 666, 741, 800, 848, 887, 920, 945, 957]
        );
    }

    // All 0 values except arrays
    #[test]
    fn sample2() {
        let sample2_file = PathBuf::from("./assets/ToS-s2.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample2_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 0);
        assert_eq!(test[0].average_maxrgb, 0);
        assert_eq!(test[0].maxscl, vec![0, 0, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![3, 14024, 43, 56, 219, 1036, 2714, 4668, 14445]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(
            test[0].bezier_curve_data,
            vec![265, 666, 741, 800, 848, 887, 920, 945, 957]
        );
    }

    // Some small values
    #[test]
    fn sample3() {
        let sample3_file = PathBuf::from("./assets/ToS-s3.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample3_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 0);
        assert_eq!(test[0].average_maxrgb, 12);
        assert_eq!(test[0].maxscl, vec![0, 1, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![3, 14024, 43, 56, 219, 1036, 2714, 4668, 14445]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(
            test[0].bezier_curve_data,
            vec![265, 666, 741, 800, 848, 887, 920, 945, 957]
        );
    }

    // More random values
    #[test]
    fn sample4() {
        let sample4_file = PathBuf::from("./assets/ToS-s4.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample4_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 10);
        assert_eq!(test[0].average_maxrgb, 1);
        assert_eq!(test[0].maxscl, vec![0, 1, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 14024, 43, 56, 219, 0, 2714, 4668, 14445]
        );
        assert_eq!(test[0].knee_x, 1);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(
            test[0].bezier_curve_data,
            vec![0, 666, 741, 0, 848, 887, 920, 945, 957]
        );
    }

    // Some 0 values except targeted display maximum luminance
    #[test]
    fn sample5() {
        let sample5_file = PathBuf::from("./assets/ToS-s5.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample5_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 500);
        assert_eq!(test[0].average_maxrgb, 0);
        assert_eq!(test[0].maxscl, vec![0, 0, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 3, 4, 5, 6, 7, 8]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(
            test[0].bezier_curve_data,
            vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
        );
    }

    // More random values
    #[test]
    fn sample6() {
        let sample6_file = PathBuf::from("./assets/ToS-s6.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample6_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 500);
        assert_eq!(test[0].average_maxrgb, 1);
        assert_eq!(test[0].maxscl, vec![1, 3, 6]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 3, 4, 5, 6, 7, 8]
        );
        assert_eq!(test[0].knee_x, 2048);
        assert_eq!(test[0].knee_y, 85);
        assert_eq!(
            test[0].bezier_curve_data,
            vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
        );
    }

    // Edge case with averageRGB
    #[test]
    fn sample7() {
        let sample7_file = PathBuf::from("./assets/ToS-s7.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample7_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 400);
        assert_eq!(test[0].average_maxrgb, 12);
        assert_eq!(test[0].maxscl, vec![3790, 5508, 3584]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(
            test[0].bezier_curve_data,
            vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
        );
    }

    // Low averageRGB and MaxScl 0s
    #[test]
    fn sample8() {
        let sample8_file = PathBuf::from("./assets/ToS-s8.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample8_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 400);
        assert_eq!(test[0].average_maxrgb, 3);
        assert_eq!(test[0].maxscl, vec![0, 0, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(
            test[0].bezier_curve_data,
            vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
        );
    }

    // Low averageRGB, MaxScl 0s and TargetedSystemDisplayMaximumLuminance 0
    #[test]
    fn sample9() {
        let sample9_file = PathBuf::from("./assets/ToS-s9.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample9_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 0);
        assert_eq!(test[0].average_maxrgb, 3);
        assert_eq!(test[0].maxscl, vec![0, 0, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(
            test[0].bezier_curve_data,
            vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
        );
    }

    #[test]
    fn sample10() {
        let sample10_file = PathBuf::from("./assets/ToS-s10.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample10_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 0);
        assert_eq!(test[0].average_maxrgb, 13);
        assert_eq!(test[0].maxscl, vec![1, 3, 6]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
        );
        assert_eq!(test[0].knee_x, 1);
        assert_eq!(test[0].knee_y, 1);
        assert_eq!(
            test[0].bezier_curve_data,
            vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
        );
    }

    #[test]
    fn sample11() {
        let sample11_file = PathBuf::from("./assets/ToS-s11.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample11_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 0);
        assert_eq!(test[0].average_maxrgb, 0);
        assert_eq!(test[0].maxscl, vec![69700, 67280, 89012]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(
            test[0].bezier_curve_data,
            vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
        );
    }

    #[test]
    fn sample12() {
        let sample12_file = PathBuf::from("./assets/ToS-s12.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample12_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 0);
        assert_eq!(test[0].average_maxrgb, 0);
        assert_eq!(test[0].maxscl, vec![0, 0, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(
            test[0].bezier_curve_data,
            vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
        );
    }

    #[test]
    fn sample13() {
        let sample13_file = PathBuf::from("./assets/ToS-s13.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample13_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 0);
        assert_eq!(test[0].average_maxrgb, 78023);
        assert_eq!(test[0].maxscl, vec![69700, 67280, 89012]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
        );
        assert_eq!(test[0].knee_x, 2305);
        assert_eq!(test[0].knee_y, 1203);
        assert_eq!(
            test[0].bezier_curve_data,
            vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
        );
    }

    #[test]
    fn sample14() {
        let sample14_file = PathBuf::from("./assets/ToS-s14.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample14_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 9998);
        assert_eq!(test[0].average_maxrgb, 78023);
        assert_eq!(test[0].maxscl, vec![69700, 67280, 89012]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
        );
        assert_eq!(test[0].knee_x, 2305);
        assert_eq!(test[0].knee_y, 1203);
        assert_eq!(
            test[0].bezier_curve_data,
            vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
        );
    }

    #[test]
    fn sample15() {
        let sample15_file = PathBuf::from("./assets/ToS-s15.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample15_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 9998);
        assert_eq!(test[0].average_maxrgb, 0);
        assert_eq!(test[0].maxscl, vec![0, 0, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(test[0].distribution_values, vec![0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }

    #[test]
    fn sample16() {
        let sample16_file = PathBuf::from("./assets/ToS-s16.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample16_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 400);
        assert_eq!(test[0].average_maxrgb, 1);
        assert_eq!(test[0].maxscl, vec![450, 26, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 9791, 100, 0, 1, 9, 32, 56, 9740]
        );
        assert_eq!(test[0].knee_x, 35);
        assert_eq!(test[0].knee_y, 86);
        assert_eq!(
            test[0].bezier_curve_data,
            vec![203, 411, 624, 721, 773, 821, 875, 924, 953]
        );
    }

    #[test]
    fn sample17() {
        let sample17_file = PathBuf::from("./assets/ToS-s17.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample17_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 400);
        assert_eq!(test[0].average_maxrgb, 11);
        assert_eq!(test[0].maxscl, vec![0, 0, 3584]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 9791, 100, 0, 1, 9, 32, 56, 9740]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(
            test[0].bezier_curve_data,
            vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
        );
    }

    #[test]
    fn sample18() {
        let sample18_file = PathBuf::from("./assets/ToS-s18.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample18_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 0);
        assert_eq!(test[0].average_maxrgb, 3584);
        assert_eq!(test[0].maxscl, vec![0, 0, 8]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 9791, 100, 0, 1, 9, 32, 56, 9740]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(
            test[0].bezier_curve_data,
            vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
        );
    }

    #[test]
    fn sample19() {
        let sample19_file = PathBuf::from("./assets/ToS-s19.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample19_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 4096);
        assert_eq!(test[0].average_maxrgb, 0);
        assert_eq!(test[0].maxscl, vec![4096, 8192, 16384]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 9791, 100, 0, 1, 9, 32, 56, 9740]
        );
        assert_eq!(test[0].knee_x, 3823);
        assert_eq!(test[0].knee_y, 1490);
        assert_eq!(
            test[0].bezier_curve_data,
            vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
        );
    }
}
