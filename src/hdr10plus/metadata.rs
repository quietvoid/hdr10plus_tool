use ansi_term::Colour::Yellow;
use deku::prelude::*;
use serde_json::{json, Value};

use super::parser::MetadataFrame;

const DISTRIBUTION_INDEXES_9: &[u8] = &[1, 5, 10, 25, 50, 75, 90, 95, 99];
const DISTRIBUTION_INDEXES_10: &[u8] = &[1, 5, 10, 25, 50, 75, 90, 95, 98, 99];

#[derive(Debug, DekuRead, Clone)]
#[deku(endian = "big")]
pub struct Metadata {
    #[deku(bits = "8")]
    pub country_code: u8,

    #[deku(bits = "16")]
    pub terminal_provider_code: u16,

    #[deku(bits = "16")]
    pub terminal_provider_oriented_code: u16,

    #[deku(bits = "8")]
    pub application_identifier: u8,

    #[deku(bits = "8")]
    pub application_version: u8,

    #[deku(bits = "2")]
    pub num_windows: u8,

    #[deku(bits = "27")]
    pub targeted_system_display_maximum_luminance: u32,

    #[deku(bits = "1")]
    pub targeted_system_display_actual_peak_luminance_flag: u8,

    #[deku(count = "3", bits = "17")]
    pub maxscl: Vec<u32>,

    #[deku(bits = "17")]
    pub average_maxrgb: u32,

    #[deku(bits = "4")]
    pub num_distribution_maxrgb_percentiles: u8,

    #[deku(count = "num_distribution_maxrgb_percentiles")]
    pub distribution_maxrgb: Vec<DistributionMaxRgb>,

    #[deku(bits = "10")]
    pub fraction_bright_pixels: u16,

    #[deku(bits = "1")]
    pub mastering_display_actual_peak_luminance_flag: u8,

    #[deku(bits = "1")]
    pub tone_mapping_flag: u8,

    #[deku(bits = "12", cond = "*tone_mapping_flag == 1")]
    pub knee_point_x: u16,

    #[deku(bits = "12", cond = "*tone_mapping_flag == 1")]
    pub knee_point_y: u16,

    #[deku(bits = "4", cond = "*tone_mapping_flag == 1")]
    pub num_bezier_curve_anchors: u8,

    #[deku(
        count = "num_bezier_curve_anchors",
        bits = "10",
        cond = "*tone_mapping_flag == 1"
    )]
    pub bezier_curve_anchors: Vec<u16>,

    #[deku(bits = "1")]
    pub color_saturation_mapping_flag: u8,
}

#[derive(Debug, PartialEq, DekuRead, Clone)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub struct DistributionMaxRgb {
    #[deku(bits = "7")]
    percentage: u8,
    #[deku(bits = "17")]
    percentile: u32,
}

impl Metadata {
    pub fn validate(&self) {
        // SMPTE ST-2094 Application 4, Version 1
        assert_eq!(self.application_identifier, 4);
        assert_eq!(self.application_version, 1);

        // The value of targeted_system_display_maximum_luminance shall be in the range of 0 to 10000, inclusive
        assert!(self.targeted_system_display_maximum_luminance <= 10000);

        // Shall be under 100000, inclusive
        self.maxscl.iter().for_each(|&v| assert!(v <= 100_000));

        // Shall be under 100000, inclusive
        assert!(self.average_maxrgb <= 100_000);

        // Shall be under 100000, inclusive
        DistributionMaxRgb::validate(
            &self.distribution_maxrgb,
            self.num_distribution_maxrgb_percentiles,
        );

        // The value of knee_point_x shall be in the range of 0 to 1, and in multiples of 1/4095
        assert!(self.knee_point_x < 4096);
        assert!(self.knee_point_y < 4096);

        // THe maximum value shall be 9
        assert!(self.num_bezier_curve_anchors <= 9);

        // Shall be under 1024
        self.bezier_curve_anchors
            .iter()
            .for_each(|&v| assert!(v < 1024));
    }

    pub fn json_list(
        list: &[MetadataFrame],
        force_single_profile: bool,
    ) -> (&str, Vec<Value>, Option<String>) {
        // Get highest number of anchors (should be constant across frames other than empty)
        let num_bezier_curve_anchors = if let Some(max) = list
            .iter()
            .map(|m| m.metadata.bezier_curve_anchors.len())
            .max()
        {
            max
        } else {
            0
        };

        // Use max with 0s instead of empty
        let replacement_curve_data = vec![0; num_bezier_curve_anchors];
        let mut warning = None;

        let mut profile = "A";

        let mut metadata_json_array = list
            .iter()
            .map(|mf| &mf.metadata)
            .map(|m| {
                // Profile A, no bezier curve data
                if m.targeted_system_display_maximum_luminance == 0 && m.bezier_curve_anchors.is_empty() && num_bezier_curve_anchors == 0 {
                    json!({
                        "LuminanceParameters": {
                            "AverageRGB": m.average_maxrgb,
                            "LuminanceDistributions": DistributionMaxRgb::separate_json(&m.distribution_maxrgb),
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
                    let bezier_curve_anchors = if force_single_profile && m.bezier_curve_anchors.is_empty() && num_bezier_curve_anchors != 0 {
                        if warning.is_none() {
                            warning = Some(format!("{}", Yellow.paint("Forced profile B.")));
                        }

                        &replacement_curve_data
                    } else {
                        if warning.is_none() && m.bezier_curve_anchors.is_empty() && num_bezier_curve_anchors != 0 {
                            warning = Some(format!("{} Different profiles appear to be present in the metadata, this can cause errors when used with x265.\nUse {} to \"fix\".", Yellow.paint("Warning:"), Yellow.paint("--force-single-profile")));
                        }

                        &m.bezier_curve_anchors
                    };

                    json!({
                        "BezierCurveData": {
                            "Anchors": bezier_curve_anchors,
                            "KneePointX": m.knee_point_x,
                            "KneePointY": m.knee_point_y
                        },
                        "LuminanceParameters": {
                            "AverageRGB": m.average_maxrgb,
                            "LuminanceDistributions": DistributionMaxRgb::separate_json(&m.distribution_maxrgb),
                            "MaxScl": m.maxscl
                        },
                        "NumberOfWindows": m.num_windows,
                        "TargetedSystemDisplayMaximumLuminance": m.targeted_system_display_maximum_luminance
                    })
                }
            })
            .collect::<Vec<Value>>();

        compute_scene_information(profile, &mut metadata_json_array);

        (profile, metadata_json_array, warning)
    }
}

fn compute_scene_information(profile: &str, metadata_json_array: &mut Vec<Value>) {
    let mut scene_frame_index: u64 = 0;
    let mut scene_id: u64 = 0;

    for (sequence_frame_index, index) in (0..metadata_json_array.len()).enumerate() {
        if index > 0 {
            if let Some(metadata) = metadata_json_array[index].as_object() {
                if let Some(prev_metadata) = metadata_json_array[index - 1].as_object() {
                    // Can only be different if profile B
                    let different_bezier = if profile == "B" {
                        metadata.get("BezierCurveData") != prev_metadata.get("BezierCurveData")
                    } else {
                        false
                    };

                    let different_luminance = metadata.get("LuminanceParameters")
                        != prev_metadata.get("LuminanceParameters");
                    let different_windows =
                        metadata.get("NumberOfWindows") != prev_metadata.get("NumberOfWindows");
                    let different_target = metadata.get("TargetedSystemDisplayMaximumLuminance")
                        != prev_metadata.get("TargetedSystemDisplayMaximumLuminance");

                    if different_bezier
                        || different_luminance
                        || different_windows
                        || different_target
                    {
                        scene_id += 1;
                        scene_frame_index = 0;
                    }
                }
            }
        }

        if let Some(map) = metadata_json_array[index].as_object_mut() {
            map.insert("SceneFrameIndex".to_string(), json!(scene_frame_index));
            map.insert("SceneId".to_string(), json!(scene_id));
            map.insert(
                "SequenceFrameIndex".to_string(),
                json!(sequence_frame_index),
            );
        }

        scene_frame_index += 1;
    }
}

impl DistributionMaxRgb {
    pub fn distribution_index(list: &[Self]) -> Vec<u8> {
        list.iter().map(|v| v.percentage).collect::<Vec<u8>>()
    }

    pub fn distribution_values(list: &[Self]) -> Vec<u32> {
        list.iter().map(|v| v.percentile).collect::<Vec<u32>>()
    }

    fn separate_json(list: &[Self]) -> Value {
        json!({
            "DistributionIndex": Self::distribution_index(list),
            "DistributionValues": Self::distribution_values(list),
        })
    }

    pub fn validate(list: &[Self], num_distribution_maxrgb_percentiles: u8) {
        // The value of num_distribution_maxrgb_percentiles shall be 9 or 10 (for all we know)
        let correct_indexes = match num_distribution_maxrgb_percentiles {
            9 => DISTRIBUTION_INDEXES_9,
            10 => DISTRIBUTION_INDEXES_10,
            _ => panic!(
                "Invalid number of percentiles: {}",
                num_distribution_maxrgb_percentiles
            ),
        };

        // Distribution indexes should be equal to:
        // 9 indexes: [1, 5, 10, 25, 50, 75, 90, 95, 99]
        // 10 indexes: [1, 5, 10, 25, 50, 75, 90, 95, 98, 99]
        assert_eq!(Self::distribution_index(list), correct_indexes);

        Self::distribution_values(list)
            .iter()
            .for_each(|&v| assert!(v <= 100_000));
    }
}
