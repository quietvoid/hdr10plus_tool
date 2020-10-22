use std::path::PathBuf;

use crate::hdr10plus::metadata::DistributionMaxRgb;
use crate::hdr10plus::parser::{Format, Parser};

// x265 Tool_Verification_new_hdr10plus_llc.json 1st frame
#[test]
fn sample1() {
    let input_file = PathBuf::from("./assets/matroska/sample01.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 400);
    assert_eq!(result.average_maxrgb, 1037);
    assert_eq!(result.maxscl, vec![17830, 16895, 14252]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![3, 14024, 43, 56, 219, 1036, 2714, 4668, 14445]
    );
    assert_eq!(result.knee_point_x, 17);
    assert_eq!(result.knee_point_y, 64);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![265, 666, 741, 800, 848, 887, 920, 945, 957]
    );
}

// All 0 values except arrays
#[test]
fn sample2() {
    let input_file = PathBuf::from("./assets/matroska/sample02.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 0);
    assert_eq!(result.average_maxrgb, 0);
    assert_eq!(result.maxscl, vec![0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![3, 14024, 43, 56, 219, 1036, 2714, 4668, 14445]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![265, 666, 741, 800, 848, 887, 920, 945, 957]
    );
}

// Some small values
#[test]
fn sample3() {
    let input_file = PathBuf::from("./assets/matroska/sample03.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 0);
    assert_eq!(result.average_maxrgb, 12);
    assert_eq!(result.maxscl, vec![0, 1, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![3, 14024, 43, 56, 219, 1036, 2714, 4668, 14445]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![265, 666, 741, 800, 848, 887, 920, 945, 957]
    );
}

// More random values
#[test]
fn sample4() {
    let input_file = PathBuf::from("./assets/matroska/sample04.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 10);
    assert_eq!(result.average_maxrgb, 1);
    assert_eq!(result.maxscl, vec![0, 1, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 14024, 43, 56, 219, 0, 2714, 4668, 14445]
    );
    assert_eq!(result.knee_point_x, 1);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![0, 666, 741, 0, 848, 887, 920, 945, 957]
    );
}

// Some 0 values except targeted display maximum luminance
#[test]
fn sample5() {
    let input_file = PathBuf::from("./assets/matroska/sample05.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 500);
    assert_eq!(result.average_maxrgb, 0);
    assert_eq!(result.maxscl, vec![0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 3, 4, 5, 6, 7, 8]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

// More random values
#[test]
fn sample6() {
    let input_file = PathBuf::from("./assets/matroska/sample06.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 500);
    assert_eq!(result.average_maxrgb, 1);
    assert_eq!(result.maxscl, vec![1, 3, 6]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 3, 4, 5, 6, 7, 8]
    );
    assert_eq!(result.knee_point_x, 2048);
    assert_eq!(result.knee_point_y, 85);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

// Edge case with averageRGB
#[test]
fn sample7() {
    let input_file = PathBuf::from("./assets/matroska/sample07.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 9);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 400);
    assert_eq!(result.average_maxrgb, 12);
    assert_eq!(result.maxscl, vec![3790, 5508, 3584]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

// Low averageRGB and MaxScl 0s
#[test]
fn sample8() {
    let input_file = PathBuf::from("./assets/matroska/sample08.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 9);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 400);
    assert_eq!(result.average_maxrgb, 3);
    assert_eq!(result.maxscl, vec![0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

// Low averageRGB, MaxScl 0s and TargetedSystemDisplayMaximumLuminance 0
#[test]
fn sample9() {
    let input_file = PathBuf::from("./assets/matroska/sample09.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 9);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 0);
    assert_eq!(result.average_maxrgb, 3);
    assert_eq!(result.maxscl, vec![0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

#[test]
fn sample10() {
    let input_file = PathBuf::from("./assets/matroska/sample10.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 3);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 0);
    assert_eq!(result.average_maxrgb, 13);
    assert_eq!(result.maxscl, vec![1, 3, 6]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
    );
    assert_eq!(result.knee_point_x, 1);
    assert_eq!(result.knee_point_y, 1);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

#[test]
fn sample11() {
    let input_file = PathBuf::from("./assets/matroska/sample11.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 3);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 0);
    assert_eq!(result.average_maxrgb, 0);
    assert_eq!(result.maxscl, vec![69700, 67280, 89012]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

#[test]
fn sample12() {
    let input_file = PathBuf::from("./assets/matroska/sample12.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 3);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 0);
    assert_eq!(result.average_maxrgb, 0);
    assert_eq!(result.maxscl, vec![0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

#[test]
fn sample13() {
    let input_file = PathBuf::from("./assets/matroska/sample13.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 3);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 0);
    assert_eq!(result.average_maxrgb, 78023);
    assert_eq!(result.maxscl, vec![69700, 67280, 89012]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
    );
    assert_eq!(result.knee_point_x, 2305);
    assert_eq!(result.knee_point_y, 1203);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

#[test]
fn sample14() {
    let input_file = PathBuf::from("./assets/matroska/sample14.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 3);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 9998);
    assert_eq!(result.average_maxrgb, 78023);
    assert_eq!(result.maxscl, vec![69700, 67280, 89012]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
    );
    assert_eq!(result.knee_point_x, 2305);
    assert_eq!(result.knee_point_y, 1203);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

#[test]
fn sample15() {
    let input_file = PathBuf::from("./assets/matroska/sample15.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 60);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 9998);
    assert_eq!(result.average_maxrgb, 0);
    assert_eq!(result.maxscl, vec![0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample16() {
    let input_file = PathBuf::from("./assets/matroska/sample16.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 400);
    assert_eq!(result.average_maxrgb, 1);
    assert_eq!(result.maxscl, vec![450, 26, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 9791, 100, 0, 1, 9, 32, 56, 9740]
    );
    assert_eq!(result.knee_point_x, 35);
    assert_eq!(result.knee_point_y, 86);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![203, 411, 624, 721, 773, 821, 875, 924, 953]
    );
}

#[test]
fn sample17() {
    let input_file = PathBuf::from("./assets/matroska/sample17.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 400);
    assert_eq!(result.average_maxrgb, 11);
    assert_eq!(result.maxscl, vec![0, 0, 3584]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 9791, 100, 0, 1, 9, 32, 56, 9740]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

#[test]
fn sample18() {
    let input_file = PathBuf::from("./assets/matroska/sample18.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 0);
    assert_eq!(result.average_maxrgb, 3584);
    assert_eq!(result.maxscl, vec![0, 0, 8]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 9791, 100, 0, 1, 9, 32, 56, 9740]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

#[test]
fn sample19() {
    let input_file = PathBuf::from("./assets/matroska/sample19.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 4096);
    assert_eq!(result.average_maxrgb, 0);
    assert_eq!(result.maxscl, vec![4096, 8192, 16384]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 9791, 100, 0, 1, 9, 32, 56, 9740]
    );
    assert_eq!(result.knee_point_x, 3823);
    assert_eq!(result.knee_point_y, 1490);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

#[test]
fn sample20() {
    let input_file = PathBuf::from("./assets/matroska/sample20.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 0);
    assert_eq!(result.average_maxrgb, 0);
    assert_eq!(result.maxscl, vec![0, 5582, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample21() {
    let input_file = PathBuf::from("./assets/matroska/sample21.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 0);
    assert_eq!(result.average_maxrgb, 9);
    assert_eq!(result.maxscl, vec![0, 0, 3584]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample22() {
    let input_file = PathBuf::from("./assets/matroska/sample22.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 0);
    assert_eq!(result.average_maxrgb, 12);
    assert_eq!(result.maxscl, vec![7, 0, 3584]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample23() {
    let input_file = PathBuf::from("./assets/matroska/sample23.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 0);
    assert_eq!(result.average_maxrgb, 12);
    assert_eq!(result.maxscl, vec![1, 0, 6]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample24() {
    let input_file = PathBuf::from("./assets/matroska/sample24.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 1);
    assert_eq!(result.average_maxrgb, 1024);
    assert_eq!(result.maxscl, vec![0, 5582, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample25() {
    let input_file = PathBuf::from("./assets/matroska/sample25.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 0);
    assert_eq!(result.average_maxrgb, 1024);
    assert_eq!(result.maxscl, vec![3584, 0, 3584]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample26() {
    let input_file = PathBuf::from("./assets/matroska/sample26.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 10000);
    assert_eq!(result.average_maxrgb, 100000);
    assert_eq!(result.maxscl, vec![2048, 2048, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample27() {
    let input_file = PathBuf::from("./assets/matroska/sample27.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 10000);
    assert_eq!(result.average_maxrgb, 12);
    assert_eq!(result.maxscl, vec![2048, 2048, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample28() {
    let input_file = PathBuf::from("./assets/matroska/sample28.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 10000);
    assert_eq!(result.average_maxrgb, 12);
    assert_eq!(result.maxscl, vec![2048, 2048, 2048]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample29() {
    let input_file = PathBuf::from("./assets/matroska/sample29.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 10000);
    assert_eq!(result.average_maxrgb, 1024);
    assert_eq!(result.maxscl, vec![2049, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample30() {
    let input_file = PathBuf::from("./assets/matroska/sample30.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 0);
    assert_eq!(result.average_maxrgb, 1024);
    assert_eq!(result.maxscl, vec![12, 3, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample31() {
    let input_file = PathBuf::from("./assets/matroska/sample31.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 0);
    assert_eq!(result.average_maxrgb, 1024);
    assert_eq!(result.maxscl, vec![1, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample32() {
    let input_file = PathBuf::from("./assets/matroska/sample32.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 0);
    assert_eq!(result.average_maxrgb, 11);
    assert_eq!(result.maxscl, vec![1152, 2, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample33() {
    let input_file = PathBuf::from("./assets/matroska/sample33.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 0);
    assert_eq!(result.average_maxrgb, 1024);
    assert_eq!(result.maxscl, vec![32768, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample34() {
    let input_file = PathBuf::from("./assets/matroska/sample34.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 0);
    assert_eq!(result.average_maxrgb, 1024);
    assert_eq!(result.maxscl, vec![1, 2304, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample35() {
    let input_file = PathBuf::from("./assets/matroska/sample35.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 8192);
    assert_eq!(result.average_maxrgb, 11);
    assert_eq!(result.maxscl, vec![158, 1, 1]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample36() {
    let input_file = PathBuf::from("./assets/matroska/sample36.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 8192);
    assert_eq!(result.average_maxrgb, 11);
    assert_eq!(result.maxscl, vec![4096, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample37() {
    let input_file = PathBuf::from("./assets/matroska/sample37.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 8192);
    assert_eq!(result.average_maxrgb, 1024);
    assert_eq!(result.maxscl, vec![0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample38() {
    let input_file = PathBuf::from("./assets/matroska/sample38.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 8192);
    assert_eq!(result.average_maxrgb, 1024);
    assert_eq!(result.maxscl, vec![0, 2048, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample39() {
    let input_file = PathBuf::from("./assets/matroska/sample39.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 8192);
    assert_eq!(result.average_maxrgb, 1024);
    assert_eq!(result.maxscl, vec![0, 98304, 98304]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample40() {
    let input_file = PathBuf::from("./assets/matroska/sample40.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 8192);
    assert_eq!(result.average_maxrgb, 1024);
    assert_eq!(result.maxscl, vec![0, 70000, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample41() {
    let input_file = PathBuf::from("./assets/matroska/sample41.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 1);
    assert_eq!(result.average_maxrgb, 12);
    assert_eq!(result.maxscl, vec![32768, 98304, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample42() {
    let input_file = PathBuf::from("./assets/matroska/sample42.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 0);
    assert_eq!(result.average_maxrgb, 1024);
    assert_eq!(result.maxscl, vec![98304, 98304, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample43() {
    let input_file = PathBuf::from("./assets/matroska/sample43.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 1);
    assert_eq!(result.average_maxrgb, 1024);
    assert_eq!(result.maxscl, vec![65536, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample44() {
    let input_file = PathBuf::from("./assets/matroska/sample44.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 10000);
    assert_eq!(result.average_maxrgb, 65535);
    assert_eq!(result.maxscl, vec![0, 4097, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample45() {
    let input_file = PathBuf::from("./assets/matroska/sample45.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 4096);
    assert_eq!(result.average_maxrgb, 1);
    assert_eq!(result.maxscl, vec![0, 65536, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample46() {
    let input_file = PathBuf::from("./assets/matroska/sample46.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 4096);
    assert_eq!(result.average_maxrgb, 65536);
    assert_eq!(result.maxscl, vec![0, 65536, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample47() {
    let input_file = PathBuf::from("./assets/matroska/sample47.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 8192);
    assert_eq!(result.average_maxrgb, 65536);
    assert_eq!(result.maxscl, vec![32768, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample48() {
    let input_file = PathBuf::from("./assets/matroska/sample48.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 8192);
    assert_eq!(result.average_maxrgb, 65536);
    assert_eq!(result.maxscl, vec![0, 65536, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample49() {
    let input_file = PathBuf::from("./assets/matroska/sample49.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 8192);
    assert_eq!(result.average_maxrgb, 12);
    assert_eq!(result.maxscl, vec![99000, 98304, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample50() {
    let input_file = PathBuf::from("./assets/matroska/sample50.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 8192);
    assert_eq!(result.average_maxrgb, 99999);
    assert_eq!(result.maxscl, vec![99000, 98304, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample51() {
    let input_file = PathBuf::from("./assets/matroska/sample51.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 400);
    assert_eq!(result.average_maxrgb, 32847);
    assert_eq!(result.maxscl, vec![32768, 32768, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample52() {
    let input_file = PathBuf::from("./assets/matroska/sample52.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 400);
    assert_eq!(result.average_maxrgb, 0);
    assert_eq!(result.maxscl, vec![0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample53() {
    let input_file = PathBuf::from("./assets/matroska/sample53.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 400);
    assert_eq!(result.average_maxrgb, 100000);
    assert_eq!(result.maxscl, vec![0, 99999, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample54() {
    let input_file = PathBuf::from("./assets/matroska/sample54.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 10000);
    assert_eq!(result.average_maxrgb, 100000);
    assert_eq!(result.maxscl, vec![0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample55() {
    let input_file = PathBuf::from("./assets/matroska/sample55.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 350);
    assert_eq!(result.average_maxrgb, 1);
    assert_eq!(result.maxscl, vec![4425, 3984, 3292]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 98, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 0, 0, 0, 0, 0, 1, 5, 2756]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, vec![256, 512, 767]);
}

#[test]
fn sample56() {
    let input_file = PathBuf::from("./assets/matroska/sample56.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 1);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 350);
    assert_eq!(result.average_maxrgb, 1);
    assert_eq!(result.maxscl, vec![0, 65536, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 98, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 0, 0, 0, 0, 0, 1, 5, 2756]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, vec![256, 512, 767]);
}

#[test]
fn sample57() {
    let input_file = PathBuf::from("./assets/matroska/sample57.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 60);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 9998);
    assert_eq!(result.average_maxrgb, 0);
    assert_eq!(result.maxscl, vec![0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(result.bezier_curve_anchors, vec![0, 0, 0]);
}

#[test]
fn sample58() {
    let input_file = PathBuf::from("./assets/matroska/sample58.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 60);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 10000);
    assert_eq!(result.average_maxrgb, 100000);
    assert_eq!(result.maxscl, vec![100000, 100000, 100000]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 98, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![100000, 100000, 100000, 100000, 100000, 100000, 100000, 100000, 100000, 100000]
    );
    assert_eq!(result.knee_point_x, 4095);
    assert_eq!(result.knee_point_y, 4095);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![1023, 1023, 1023, 1023, 1023, 1023, 1023, 1023, 1023]
    );
}

#[test]
fn sample59() {
    let input_file = PathBuf::from("./assets/matroska/sample59.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 60);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 10000);
    assert_eq!(result.average_maxrgb, 100000);
    assert_eq!(result.maxscl, vec![0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 98, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![100000, 100000, 100000, 100000, 100000, 100000, 100000, 100000, 100000, 100000]
    );
    assert_eq!(result.knee_point_x, 4095);
    assert_eq!(result.knee_point_y, 4095);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![1023, 1023, 1023, 1023, 1023, 1023, 1023, 1023, 1023]
    );
}

#[test]
fn sample60() {
    let input_file = PathBuf::from("./assets/matroska/sample60.mkv");
    let parser = Parser::new(Format::Matroska, input_file, None, false, false);
    let (count, result) = parser._test().unwrap();

    assert_eq!(count, 6);

    assert_eq!(result.num_windows, 1);
    assert_eq!(result.targeted_system_display_maximum_luminance, 400);
    assert_eq!(result.average_maxrgb, 0);
    assert_eq!(result.maxscl, vec![0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&result.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&result.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(result.knee_point_x, 0);
    assert_eq!(result.knee_point_y, 0);
    assert_eq!(
        result.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}
