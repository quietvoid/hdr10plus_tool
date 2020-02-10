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

    #[test]
    fn sample20() {
        let sample20_file = PathBuf::from("./assets/ToS-s20.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample20_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 0);
        assert_eq!(test[0].average_maxrgb, 0);
        assert_eq!(test[0].maxscl, vec![0,5582,0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }

    #[test]
    fn sample21() {
        let sample21_file = PathBuf::from("./assets/ToS-s21.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample21_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 0);
        assert_eq!(test[0].average_maxrgb, 9);
        assert_eq!(test[0].maxscl, vec![0,0,3584]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }

    #[test]
    fn sample22() {
        let sample22_file = PathBuf::from("./assets/ToS-s22.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample22_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 0);
        assert_eq!(test[0].average_maxrgb, 12);
        assert_eq!(test[0].maxscl, vec![7,0,3584]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }

    #[test]
    fn sample23() {
        let sample23_file = PathBuf::from("./assets/ToS-s23.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample23_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 0);
        assert_eq!(test[0].average_maxrgb, 12);
        assert_eq!(test[0].maxscl, vec![1, 0, 6]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }

    #[test]
    fn sample24() {
        let sample24_file = PathBuf::from("./assets/ToS-s24.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample24_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 1);
        assert_eq!(test[0].average_maxrgb, 1024);
        assert_eq!(test[0].maxscl, vec![0, 5582, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }

    #[test]
    fn sample25() {
        let sample25_file = PathBuf::from("./assets/ToS-s25.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample25_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 0);
        assert_eq!(test[0].average_maxrgb, 1024);
        assert_eq!(test[0].maxscl, vec![3584, 0, 3584]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }

    #[test]
    fn sample26() {
        let sample26_file = PathBuf::from("./assets/ToS-s26.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample26_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 10000);
        assert_eq!(test[0].average_maxrgb, 100000);
        assert_eq!(test[0].maxscl, vec![2048, 2048, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }

    #[test]
    fn sample27() {
        let sample27_file = PathBuf::from("./assets/ToS-s27.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample27_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 10000);
        assert_eq!(test[0].average_maxrgb, 12);
        assert_eq!(test[0].maxscl, vec![2048, 2048, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }
    
    #[test]
    fn sample28() {
        let sample28_file = PathBuf::from("./assets/ToS-s28.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample28_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 10000);
        assert_eq!(test[0].average_maxrgb, 12);
        assert_eq!(test[0].maxscl, vec![2048, 2048, 2048]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }

    #[test]
    fn sample29() {
        let sample29_file = PathBuf::from("./assets/ToS-s29.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample29_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 10000);
        assert_eq!(test[0].average_maxrgb, 1024);
        assert_eq!(test[0].maxscl, vec![2049, 0, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }

    #[test]
    fn sample30() {
        let sample30_file = PathBuf::from("./assets/ToS-s30.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample30_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 0);
        assert_eq!(test[0].average_maxrgb, 1024);
        assert_eq!(test[0].maxscl, vec![12, 3, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }

    #[test]
    fn sample31() {
        let sample31_file = PathBuf::from("./assets/ToS-s31.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample31_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 0);
        assert_eq!(test[0].average_maxrgb, 1024);
        assert_eq!(test[0].maxscl, vec![1, 0, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }

    #[test]
    fn sample32() {
        let sample32_file = PathBuf::from("./assets/ToS-s32.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample32_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 0);
        assert_eq!(test[0].average_maxrgb, 11);
        assert_eq!(test[0].maxscl, vec![1152, 2, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }

    #[test]
    fn sample33() {
        let sample33_file = PathBuf::from("./assets/ToS-s33.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample33_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 0);
        assert_eq!(test[0].average_maxrgb, 1024);
        assert_eq!(test[0].maxscl, vec![32768, 0, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }

    #[test]
    fn sample34() {
        let sample34_file = PathBuf::from("./assets/ToS-s34.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample34_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 0);
        assert_eq!(test[0].average_maxrgb, 1024);
        assert_eq!(test[0].maxscl, vec![1, 2304, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }

    #[test]
    fn sample35() {
        let sample35_file = PathBuf::from("./assets/ToS-s35.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample35_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 8192);
        assert_eq!(test[0].average_maxrgb, 11);
        assert_eq!(test[0].maxscl, vec![158, 1, 1]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }

    #[test]
    fn sample36() {
        let sample36_file = PathBuf::from("./assets/ToS-s36.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample36_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 8192);
        assert_eq!(test[0].average_maxrgb, 11);
        assert_eq!(test[0].maxscl, vec![4096, 0, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }

    #[test]
    fn sample37() {
        let sample37_file = PathBuf::from("./assets/ToS-s37.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample37_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 8192);
        assert_eq!(test[0].average_maxrgb, 1024);
        assert_eq!(test[0].maxscl, vec![0, 0, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }

    #[test]
    fn sample38() {
        let sample38_file = PathBuf::from("./assets/ToS-s38.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample38_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 8192);
        assert_eq!(test[0].average_maxrgb, 1024);
        assert_eq!(test[0].maxscl, vec![0, 2048, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }

    #[test]
    fn sample39() {
        let sample39_file = PathBuf::from("./assets/ToS-s39.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample39_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 8192);
        assert_eq!(test[0].average_maxrgb, 1024);
        assert_eq!(test[0].maxscl, vec![0, 98304, 98304]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }

    #[test]
    fn sample40() {
        let sample40_file = PathBuf::from("./assets/ToS-s40.h265");
        let mut test: Vec<Metadata> = Vec::new();
        match parse_metadata(false, &sample40_file, false) {
            Ok(vec) => test = llc_read_metadata(vec),
            Err(e) => println!("{}", e),
        }

        assert_eq!(test[0].num_windows, 1);
        assert_eq!(test[0].targeted_system_display_maximum_luminance, 8192);
        assert_eq!(test[0].average_maxrgb, 1024);
        assert_eq!(test[0].maxscl, vec![0, 70000, 0]);
        assert_eq!(
            test[0].distribution_index,
            vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
        );
        assert_eq!(
            test[0].distribution_values,
            vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(test[0].knee_x, 0);
        assert_eq!(test[0].knee_y, 0);
        assert_eq!(test[0].bezier_curve_data, Vec::<u16>::new());
    }
}
