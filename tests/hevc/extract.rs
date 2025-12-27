use std::path::Path;

use anyhow::Result;
use assert_cmd::cargo;
use assert_fs::prelude::*;
use predicates::prelude::*;

use hdr10plus::{
    metadata::{DistributionMaxRgb, Hdr10PlusMetadata},
    metadata_json::MetadataJsonRoot,
};

const SUBCOMMAND: &str = "extract";

fn assert_cmd_output(input: &Path, output: &Path, validate: bool) -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();

    if !validate {
        cmd.arg("--skip-validation");
    }

    cmd.arg(SUBCOMMAND).arg(input).arg("--output").arg(output);

    let assert = cmd.assert();

    assert.success().stderr(predicate::str::is_empty());

    Ok(())
}

pub fn run_test(input: &Path, validate: bool) -> Result<(Hdr10PlusMetadata, MetadataJsonRoot)> {
    let temp = assert_fs::TempDir::new()?;
    let output_json = temp.child("metadata.json");

    assert_cmd_output(input, output_json.as_ref(), validate)?;
    output_json.assert(predicate::path::is_file());

    let metadata_root = MetadataJsonRoot::from_file(output_json.as_ref())?;
    let metadata_list: Vec<Hdr10PlusMetadata> = metadata_root
        .scene_info
        .iter()
        .map(Hdr10PlusMetadata::try_from)
        .filter_map(Result::ok)
        .collect();

    let first_metadata = metadata_list[0].clone();

    Ok((first_metadata, metadata_root))
}

fn assert_scene_info(
    metadata_root: &MetadataJsonRoot,
    index: usize,
    scene_frame: usize,
    scene_id: usize,
    sequence_frame: usize,
) {
    let metadata = &metadata_root.scene_info[index];

    let scene_frame_index = metadata.scene_frame_index;
    let id = metadata.scene_id;
    let sequence_frame_index = metadata.sequence_frame_index;

    assert_eq!(scene_frame_index, scene_frame);
    assert_eq!(id, scene_id);
    assert_eq!(sequence_frame_index, sequence_frame);
}

#[test]
fn help() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let assert = cmd.arg(SUBCOMMAND).arg("--help").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains(
            "hdr10plus_tool extract [OPTIONS] [input_pos]",
        ));
    Ok(())
}

// x265 Tool_Verification_new_hdr10plus_llc.json 1st frame
#[test]
fn sample01() {
    let input_file = Path::new("assets/metadata/ToS-s01.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "B");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 400);
    assert_eq!(metadata.average_maxrgb, 1037);
    assert_eq!(metadata.maxscl, [17830, 16895, 14252]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![3, 14024, 43, 56, 219, 1036, 2714, 4668, 14445]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 17);
    assert_eq!(bc.knee_point_y, 64);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![265, 666, 741, 800, 848, 887, 920, 945, 957]
    );
}

// All 0 values except arrays
#[test]
fn sample02() {
    let input_file = Path::new("assets/metadata/ToS-s02.h265");
    let (metadata, _metadata_root) = run_test(input_file, false).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 0);
    assert_eq!(metadata.average_maxrgb, 0);
    assert_eq!(metadata.maxscl, [0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![3, 14024, 43, 56, 219, 1036, 2714, 4668, 14445]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![265, 666, 741, 800, 848, 887, 920, 945, 957]
    );
}

// Some small values
#[test]
fn sample03() {
    let input_file = Path::new("assets/metadata/ToS-s03.h265");
    let (metadata, _metadata_root) = run_test(input_file, false).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 0);
    assert_eq!(metadata.average_maxrgb, 12);
    assert_eq!(metadata.maxscl, [0, 1, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![3, 14024, 43, 56, 219, 1036, 2714, 4668, 14445]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![265, 666, 741, 800, 848, 887, 920, 945, 957]
    );
}

// More random values
#[test]
fn sample04() {
    let input_file = Path::new("assets/metadata/ToS-s04.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "B");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 10);
    assert_eq!(metadata.average_maxrgb, 1);
    assert_eq!(metadata.maxscl, [0, 1, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 14024, 43, 56, 219, 0, 2714, 4668, 14445]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 1);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![0, 666, 741, 0, 848, 887, 920, 945, 957]
    );
}

// Some 0 values except targeted display maximum luminance
#[test]
fn sample05() {
    let input_file = Path::new("assets/metadata/ToS-s05.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "B");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 500);
    assert_eq!(metadata.average_maxrgb, 0);
    assert_eq!(metadata.maxscl, [0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 3, 4, 5, 6, 7, 8]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

// More random values
#[test]
fn sample06() {
    let input_file = Path::new("assets/metadata/ToS-s06.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "B");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 500);
    assert_eq!(metadata.average_maxrgb, 1);
    assert_eq!(metadata.maxscl, [1, 3, 6]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 3, 4, 5, 6, 7, 8]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 2048);
    assert_eq!(bc.knee_point_y, 85);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

// Edge case with averageRGB
#[test]
fn sample07() {
    let input_file = Path::new("assets/metadata/ToS-s07.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "B");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 400);
    assert_eq!(metadata.average_maxrgb, 12);
    assert_eq!(metadata.maxscl, [3790, 5508, 3584]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

// Low averageRGB and MaxScl 0s
#[test]
fn sample08() {
    let input_file = Path::new("assets/metadata/ToS-s08.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "B");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 400);
    assert_eq!(metadata.average_maxrgb, 3);
    assert_eq!(metadata.maxscl, [0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

// Low averageRGB, MaxScl 0s and TargetedSystemDisplayMaximumLuminance 0
#[test]
fn sample09() {
    let input_file = Path::new("assets/metadata/ToS-s09.h265");
    let (metadata, _metadata_root) = run_test(input_file, false).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 0);
    assert_eq!(metadata.average_maxrgb, 3);
    assert_eq!(metadata.maxscl, [0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

#[test]
fn sample10() {
    let input_file = Path::new("assets/metadata/ToS-s10.h265");
    let (metadata, _metadata_root) = run_test(input_file, false).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 0);
    assert_eq!(metadata.average_maxrgb, 13);
    assert_eq!(metadata.maxscl, [1, 3, 6]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 1);
    assert_eq!(bc.knee_point_y, 1);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

#[test]
fn sample11() {
    let input_file = Path::new("assets/metadata/ToS-s11.h265");
    let (metadata, _metadata_root) = run_test(input_file, false).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 0);
    assert_eq!(metadata.average_maxrgb, 0);
    assert_eq!(metadata.maxscl, [69700, 67280, 89012]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

#[test]
fn sample12() {
    let input_file = Path::new("assets/metadata/ToS-s12.h265");
    let (metadata, _metadata_root) = run_test(input_file, false).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 0);
    assert_eq!(metadata.average_maxrgb, 0);
    assert_eq!(metadata.maxscl, [0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

#[test]
fn sample13() {
    let input_file = Path::new("assets/metadata/ToS-s13.h265");
    let (metadata, _metadata_root) = run_test(input_file, false).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 0);
    assert_eq!(metadata.average_maxrgb, 78023);
    assert_eq!(metadata.maxscl, [69700, 67280, 89012]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 2305);
    assert_eq!(bc.knee_point_y, 1203);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

#[test]
fn sample14() {
    let input_file = Path::new("assets/metadata/ToS-s14.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "B");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 9998);
    assert_eq!(metadata.average_maxrgb, 78023);
    assert_eq!(metadata.maxscl, [69700, 67280, 89012]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 572, 100, 1, 1, 2, 12, 35, 491]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 2305);
    assert_eq!(bc.knee_point_y, 1203);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

#[test]
fn sample15() {
    let input_file = Path::new("assets/metadata/ToS-s15.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 9998);
    assert_eq!(metadata.average_maxrgb, 0);
    assert_eq!(metadata.maxscl, [0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample16() {
    let input_file = Path::new("assets/metadata/ToS-s16.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "B");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 400);
    assert_eq!(metadata.average_maxrgb, 1);
    assert_eq!(metadata.maxscl, [450, 26, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 9791, 100, 0, 1, 9, 32, 56, 9740]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 35);
    assert_eq!(bc.knee_point_y, 86);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![203, 411, 624, 721, 773, 821, 875, 924, 953]
    );
}

#[test]
fn sample17() {
    let input_file = Path::new("assets/metadata/ToS-s17.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "B");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 400);
    assert_eq!(metadata.average_maxrgb, 11);
    assert_eq!(metadata.maxscl, [0, 0, 3584]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 9791, 100, 0, 1, 9, 32, 56, 9740]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

#[test]
fn sample18() {
    let input_file = Path::new("assets/metadata/ToS-s18.h265");
    let (metadata, _metadata_root) = run_test(input_file, false).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 0);
    assert_eq!(metadata.average_maxrgb, 3584);
    assert_eq!(metadata.maxscl, [0, 0, 8]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 9791, 100, 0, 1, 9, 32, 56, 9740]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

#[test]
fn sample19() {
    let input_file = Path::new("assets/metadata/ToS-s19.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "B");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 4096);
    assert_eq!(metadata.average_maxrgb, 0);
    assert_eq!(metadata.maxscl, [4096, 8192, 16384]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 9791, 100, 0, 1, 9, 32, 56, 9740]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 3823);
    assert_eq!(bc.knee_point_y, 1490);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

#[test]
fn sample20() {
    let input_file = Path::new("assets/metadata/ToS-s20.h265");
    let (metadata, _metadata_root) = run_test(input_file, false).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 0);
    assert_eq!(metadata.average_maxrgb, 0);
    assert_eq!(metadata.maxscl, [0, 5582, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample21() {
    let input_file = Path::new("assets/metadata/ToS-s21.h265");
    let (metadata, _metadata_root) = run_test(input_file, false).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 0);
    assert_eq!(metadata.average_maxrgb, 9);
    assert_eq!(metadata.maxscl, [0, 0, 3584]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample22() {
    let input_file = Path::new("assets/metadata/ToS-s22.h265");
    let (metadata, _metadata_root) = run_test(input_file, false).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 0);
    assert_eq!(metadata.average_maxrgb, 12);
    assert_eq!(metadata.maxscl, [7, 0, 3584]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample23() {
    let input_file = Path::new("assets/metadata/ToS-s23.h265");
    let (metadata, _metadata_root) = run_test(input_file, false).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 0);
    assert_eq!(metadata.average_maxrgb, 12);
    assert_eq!(metadata.maxscl, [1, 0, 6]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample24() {
    let input_file = Path::new("assets/metadata/ToS-s24.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 1);
    assert_eq!(metadata.average_maxrgb, 1024);
    assert_eq!(metadata.maxscl, [0, 5582, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample25() {
    let input_file = Path::new("assets/metadata/ToS-s25.h265");
    let (metadata, _metadata_root) = run_test(input_file, false).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 0);
    assert_eq!(metadata.average_maxrgb, 1024);
    assert_eq!(metadata.maxscl, [3584, 0, 3584]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample26() {
    let input_file = Path::new("assets/metadata/ToS-s26.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 10000);
    assert_eq!(metadata.average_maxrgb, 100000);
    assert_eq!(metadata.maxscl, [2048, 2048, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample27() {
    let input_file = Path::new("assets/metadata/ToS-s27.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 10000);
    assert_eq!(metadata.average_maxrgb, 12);
    assert_eq!(metadata.maxscl, [2048, 2048, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample28() {
    let input_file = Path::new("assets/metadata/ToS-s28.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 10000);
    assert_eq!(metadata.average_maxrgb, 12);
    assert_eq!(metadata.maxscl, [2048, 2048, 2048]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample29() {
    let input_file = Path::new("assets/metadata/ToS-s29.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 10000);
    assert_eq!(metadata.average_maxrgb, 1024);
    assert_eq!(metadata.maxscl, [2049, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample30() {
    let input_file = Path::new("assets/metadata/ToS-s30.h265");
    let (metadata, _metadata_root) = run_test(input_file, false).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 0);
    assert_eq!(metadata.average_maxrgb, 1024);
    assert_eq!(metadata.maxscl, [12, 3, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample31() {
    let input_file = Path::new("assets/metadata/ToS-s31.h265");
    let (metadata, _metadata_root) = run_test(input_file, false).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 0);
    assert_eq!(metadata.average_maxrgb, 1024);
    assert_eq!(metadata.maxscl, [1, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample32() {
    let input_file = Path::new("assets/metadata/ToS-s32.h265");
    let (metadata, _metadata_root) = run_test(input_file, false).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 0);
    assert_eq!(metadata.average_maxrgb, 11);
    assert_eq!(metadata.maxscl, [1152, 2, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample33() {
    let input_file = Path::new("assets/metadata/ToS-s33.h265");
    let (metadata, _metadata_root) = run_test(input_file, false).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 0);
    assert_eq!(metadata.average_maxrgb, 1024);
    assert_eq!(metadata.maxscl, [32768, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample34() {
    let input_file = Path::new("assets/metadata/ToS-s34.h265");
    let (metadata, _metadata_root) = run_test(input_file, false).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 0);
    assert_eq!(metadata.average_maxrgb, 1024);
    assert_eq!(metadata.maxscl, [1, 2304, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample35() {
    let input_file = Path::new("assets/metadata/ToS-s35.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 8192);
    assert_eq!(metadata.average_maxrgb, 11);
    assert_eq!(metadata.maxscl, [158, 1, 1]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample36() {
    let input_file = Path::new("assets/metadata/ToS-s36.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 8192);
    assert_eq!(metadata.average_maxrgb, 11);
    assert_eq!(metadata.maxscl, [4096, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample37() {
    let input_file = Path::new("assets/metadata/ToS-s37.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 8192);
    assert_eq!(metadata.average_maxrgb, 1024);
    assert_eq!(metadata.maxscl, [0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample38() {
    let input_file = Path::new("assets/metadata/ToS-s38.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 8192);
    assert_eq!(metadata.average_maxrgb, 1024);
    assert_eq!(metadata.maxscl, [0, 2048, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample39() {
    let input_file = Path::new("assets/metadata/ToS-s39.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 8192);
    assert_eq!(metadata.average_maxrgb, 1024);
    assert_eq!(metadata.maxscl, [0, 98304, 98304]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample40() {
    let input_file = Path::new("assets/metadata/ToS-s40.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 8192);
    assert_eq!(metadata.average_maxrgb, 1024);
    assert_eq!(metadata.maxscl, [0, 70000, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample41() {
    let input_file = Path::new("assets/metadata/ToS-s41.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 1);
    assert_eq!(metadata.average_maxrgb, 12);
    assert_eq!(metadata.maxscl, [32768, 98304, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample42() {
    let input_file = Path::new("assets/metadata/ToS-s42.h265");
    let (metadata, _metadata_root) = run_test(input_file, false).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 0);
    assert_eq!(metadata.average_maxrgb, 1024);
    assert_eq!(metadata.maxscl, [98304, 98304, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample43() {
    let input_file = Path::new("assets/metadata/ToS-s43.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 1);
    assert_eq!(metadata.average_maxrgb, 1024);
    assert_eq!(metadata.maxscl, [65536, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample44() {
    let input_file = Path::new("assets/metadata/ToS-s44.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 10000);
    assert_eq!(metadata.average_maxrgb, 65535);
    assert_eq!(metadata.maxscl, [0, 4097, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample45() {
    let input_file = Path::new("assets/metadata/ToS-s45.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 4096);
    assert_eq!(metadata.average_maxrgb, 1);
    assert_eq!(metadata.maxscl, [0, 65536, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample46() {
    let input_file = Path::new("assets/metadata/ToS-s46.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 4096);
    assert_eq!(metadata.average_maxrgb, 65536);
    assert_eq!(metadata.maxscl, [0, 65536, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample47() {
    let input_file = Path::new("assets/metadata/ToS-s47.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 8192);
    assert_eq!(metadata.average_maxrgb, 65536);
    assert_eq!(metadata.maxscl, [32768, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample48() {
    let input_file = Path::new("assets/metadata/ToS-s48.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 8192);
    assert_eq!(metadata.average_maxrgb, 65536);
    assert_eq!(metadata.maxscl, [0, 65536, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample49() {
    let input_file = Path::new("assets/metadata/ToS-s49.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 8192);
    assert_eq!(metadata.average_maxrgb, 12);
    assert_eq!(metadata.maxscl, [99000, 98304, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample50() {
    let input_file = Path::new("assets/metadata/ToS-s50.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 8192);
    assert_eq!(metadata.average_maxrgb, 99999);
    assert_eq!(metadata.maxscl, [99000, 98304, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample51() {
    let input_file = Path::new("assets/metadata/ToS-s51.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 400);
    assert_eq!(metadata.average_maxrgb, 32847);
    assert_eq!(metadata.maxscl, [32768, 32768, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample52() {
    let input_file = Path::new("assets/metadata/ToS-s52.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 400);
    assert_eq!(metadata.average_maxrgb, 0);
    assert_eq!(metadata.maxscl, [0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample53() {
    let input_file = Path::new("assets/metadata/ToS-s53.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 400);
    assert_eq!(metadata.average_maxrgb, 100000);
    assert_eq!(metadata.maxscl, [0, 99999, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample54() {
    let input_file = Path::new("assets/metadata/ToS-s54.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "N/A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 10000);
    assert_eq!(metadata.average_maxrgb, 100000);
    assert_eq!(metadata.maxscl, [0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, Vec::<u16>::new());
}

#[test]
fn sample55() {
    let input_file = Path::new("assets/metadata/ToS-s55.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "B");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 350);
    assert_eq!(metadata.average_maxrgb, 1);
    assert_eq!(metadata.maxscl, [4425, 3984, 3292]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 98, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 0, 0, 0, 0, 0, 1, 5, 2756]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, vec![256, 512, 767]);
}

#[test]
fn sample56() {
    let input_file = Path::new("assets/metadata/ToS-s56.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "B");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 350);
    assert_eq!(metadata.average_maxrgb, 1);
    assert_eq!(metadata.maxscl, [0, 65536, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 98, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 0, 0, 0, 0, 0, 1, 5, 2756]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, vec![256, 512, 767]);
}

#[test]
fn sample57() {
    let input_file = Path::new("assets/metadata/ToS-s57.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "B");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 9998);
    assert_eq!(metadata.average_maxrgb, 0);
    assert_eq!(metadata.maxscl, [0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(bc.bezier_curve_anchors, vec![0, 0, 0]);
}

#[test]
fn sample58() {
    let input_file = Path::new("assets/metadata/ToS-s58.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "B");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 10000);
    assert_eq!(metadata.average_maxrgb, 100000);
    assert_eq!(metadata.maxscl, [100000, 100000, 100000]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 98, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![
            100000, 100000, 100000, 100000, 100000, 100000, 100000, 100000, 100000, 100000
        ]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 4095);
    assert_eq!(bc.knee_point_y, 4095);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![1023, 1023, 1023, 1023, 1023, 1023, 1023, 1023, 1023]
    );
}

#[test]
fn sample59() {
    let input_file = Path::new("assets/metadata/ToS-s59.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "B");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 10000);
    assert_eq!(metadata.average_maxrgb, 100000);
    assert_eq!(metadata.maxscl, [0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 98, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![
            100000, 100000, 100000, 100000, 100000, 100000, 100000, 100000, 100000, 100000
        ]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 4095);
    assert_eq!(bc.knee_point_y, 4095);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![1023, 1023, 1023, 1023, 1023, 1023, 1023, 1023, 1023]
    );
}

#[test]
fn sample60() {
    let input_file = Path::new("assets/metadata/ToS-s60.h265");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    assert_eq!(metadata.profile, "B");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 400);
    assert_eq!(metadata.average_maxrgb, 0);
    assert_eq!(metadata.maxscl, [0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );
}

#[test]
fn sample61() {
    let input_file = Path::new("assets/metadata/ToS-s61.h265");
    let (metadata, metadata_root) = run_test(input_file, true).unwrap();

    metadata.validate().unwrap();

    assert_eq!(metadata.profile, "B");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 400);
    assert_eq!(metadata.average_maxrgb, 1037);
    assert_eq!(metadata.maxscl, [17830, 16895, 14252]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![3, 14024, 43, 56, 219, 1036, 2714, 4668, 14445]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 17);
    assert_eq!(bc.knee_point_y, 64);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![265, 666, 741, 800, 848, 887, 920, 945, 957]
    );

    assert_eq!(metadata_root.info.profile, "B");
    assert_scene_info(&metadata_root, 8, 2, 2, 8);

    let info_summary = metadata_root.scene_info_summary;
    let first_frames = info_summary.scene_first_frame_index;
    let scene_lengths = info_summary.scene_frame_numbers;

    assert_eq!(first_frames, vec![0, 3, 6]);
    assert_eq!(scene_lengths, vec![3, 3, 3]);
}

#[test]
fn sample62() {
    let input_file = Path::new("assets/metadata/ToS-s62.h265");
    let (metadata, metadata_root) = run_test(input_file, true).unwrap();

    metadata.validate().unwrap();

    assert_eq!(metadata.profile, "A");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 0);
    assert_eq!(metadata.average_maxrgb, 1037);
    assert_eq!(metadata.maxscl, [17830, 16895, 14252]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![3, 14024, 43, 56, 219, 1036, 2714, 4668, 14445]
    );

    assert!(metadata.bezier_curve.is_none());

    assert_eq!(metadata_root.info.profile, "A");
    assert_scene_info(&metadata_root, 8, 2, 2, 8);

    let info_summary = metadata_root.scene_info_summary;
    let first_frames = info_summary.scene_first_frame_index;
    let scene_lengths = info_summary.scene_frame_numbers;

    assert_eq!(first_frames, vec![0, 3, 6]);
    assert_eq!(scene_lengths, vec![3, 3, 3]);
}

#[test]
fn invalid_null_byte_seq_metadata() {
    let input_file = Path::new("assets/metadata/invalid-null-byte-seq.hevc");
    let (metadata, metadata_root) = run_test(input_file, true).unwrap();

    metadata.validate().unwrap();

    assert_eq!(metadata.profile, "B");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 500);
    assert_eq!(metadata.average_maxrgb, 0);
    assert_eq!(metadata.maxscl, [0, 0, 0]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 0, 100, 0, 0, 0, 0, 0, 0]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 0);
    assert_eq!(bc.knee_point_y, 0);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![102, 205, 307, 410, 512, 614, 717, 819, 922]
    );

    assert_eq!(metadata_root.info.profile, "B");
    assert_scene_info(&metadata_root, 0, 0, 0, 0);

    let info_summary = metadata_root.scene_info_summary;
    let first_frames = info_summary.scene_first_frame_index;
    let scene_lengths = info_summary.scene_frame_numbers;

    assert_eq!(first_frames, vec![0]);
    assert_eq!(scene_lengths, vec![1]);
}

#[test]
fn multimsg_sei_metadata() {
    let input_file = Path::new("assets/hevc_tests/multimsg-sei.hevc");
    let (metadata, _metadata_root) = run_test(input_file, true).unwrap();

    metadata.validate().unwrap();

    assert_eq!(metadata.profile, "B");
    assert_eq!(metadata.num_windows, 1);
    assert_eq!(metadata.targeted_system_display_maximum_luminance, 400);
    assert_eq!(metadata.average_maxrgb, 263);
    assert_eq!(metadata.maxscl, [7768, 6589, 6912]);
    assert_eq!(
        DistributionMaxRgb::distribution_index(&metadata.distribution_maxrgb),
        vec![1, 5, 10, 25, 50, 75, 90, 95, 99]
    );
    assert_eq!(
        DistributionMaxRgb::distribution_values(&metadata.distribution_maxrgb),
        vec![0, 6080, 92, 1, 4, 107, 726, 1784, 5843]
    );

    assert!(metadata.bezier_curve.is_some());
    let bc = metadata.bezier_curve.unwrap();

    assert_eq!(bc.knee_point_x, 164);
    assert_eq!(bc.knee_point_y, 240);
    assert_eq!(
        bc.bezier_curve_anchors,
        vec![143, 298, 447, 592, 731, 864, 891, 917, 938]
    );
}

#[test]
fn dhdr10_opt_gaps() -> Result<()> {
    let input_file = Path::new("assets/hevc_tests/dhdr10-opt.hevc");
    let temp = assert_fs::TempDir::new()?;

    let output_json = temp.child("metadata.json");
    let expected_json = Path::new("assets/hevc_tests/metadata-dhdr10-opt.json");

    assert_cmd_output(input_file, output_json.as_ref(), true)?;
    output_json
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_json));

    Ok(())
}

#[test]
fn pts_injected() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();

    let input_file = Path::new("assets/hevc_tests/pts_injected.hevc");
    let temp = assert_fs::TempDir::new()?;

    let output_json = temp.child("metadata.json");
    let expected_json = Path::new("assets/hevc_tests/metadata-dhdr10-opt.json");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--output")
        .arg(output_json.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());
    output_json
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_json));

    Ok(())
}

#[test]
fn dts_injected() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();

    let input_file = Path::new("assets/hevc_tests/dts_injected.hevc");
    let temp = assert_fs::TempDir::new()?;

    let output_json = temp.child("metadata.json");
    let expected_json = Path::new("assets/hevc_tests/metadata-dhdr10-opt.json");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg("--skip-reorder")
        .arg(input_file)
        .arg("--output")
        .arg(output_json.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());
    output_json
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_json));

    Ok(())
}

#[test]
fn regular() -> Result<()> {
    let input_file = Path::new("assets/hevc_tests/regular.hevc");
    let temp = assert_fs::TempDir::new()?;

    let output_json = temp.child("metadata.json");
    let expected_json = Path::new("assets/hevc_tests/regular_metadata.json");

    assert_cmd_output(input_file, output_json.as_ref(), true)?;
    output_json
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_json));

    Ok(())
}

#[test]
fn regular_mkv() -> Result<()> {
    let input_file = Path::new("assets/hevc_tests/regular.mkv");
    let temp = assert_fs::TempDir::new()?;

    let output_json = temp.child("metadata.json");
    let expected_json = Path::new("assets/hevc_tests/regular_metadata.json");

    assert_cmd_output(input_file, output_json.as_ref(), true)?;
    output_json
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_json));

    Ok(())
}
