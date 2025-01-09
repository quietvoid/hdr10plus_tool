use std::{convert::TryFrom, fs::File, io::Read, path::Path};

use anyhow::{bail, ensure, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::metadata::{
    BezierCurve, DistributionMaxRgb, Hdr10PlusMetadata, PeakBrightnessSource,
    VariablePeakBrightness,
};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct MetadataJsonRoot {
    #[serde(rename = "JSONInfo")]
    pub info: JsonInfo,

    #[serde(rename = "SceneInfo")]
    pub scene_info: Vec<Hdr10PlusJsonMetadata>,

    #[serde(rename = "SceneInfoSummary")]
    pub scene_info_summary: SceneInfoSummary,

    #[serde(rename = "ToolInfo")]
    pub tool_info: ToolInfo,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct JsonInfo {
    #[serde(rename = "HDR10plusProfile")]
    pub profile: String,

    #[serde(rename = "Version")]
    pub version: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Hdr10PlusJsonMetadata {
    pub bezier_curve_data: Option<BezierCurveData>,
    pub luminance_parameters: LuminanceParameters,
    pub number_of_windows: u8,
    pub targeted_system_display_maximum_luminance: u32,
    pub scene_frame_index: usize,
    pub scene_id: usize,
    pub sequence_frame_index: usize,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SceneInfoSummary {
    pub scene_first_frame_index: Vec<usize>,
    pub scene_frame_numbers: Vec<usize>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ToolInfo {
    pub tool: String,
    pub version: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct BezierCurveData {
    pub anchors: Vec<u16>,
    pub knee_point_x: u16,
    pub knee_point_y: u16,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct LuminanceParameters {
    #[serde(rename = "AverageRGB")]
    pub average_rgb: u32,

    pub luminance_distributions: LuminanceDistributions,
    pub max_scl: Vec<u32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct LuminanceDistributions {
    pub distribution_index: Vec<u8>,
    pub distribution_values: Vec<u32>,
}

impl MetadataJsonRoot {
    pub fn parse(str: &str) -> Result<MetadataJsonRoot> {
        let res = serde_json::from_str::<MetadataJsonRoot>(str);

        if let Ok(metadata_root) = res {
            Ok(metadata_root)
        } else {
            bail!("Failed parsing JSON metadata\n{:?}", res);
        }
    }

    pub fn from_file<P: AsRef<Path>>(input: P) -> Result<MetadataJsonRoot> {
        let mut s = String::new();
        File::open(input)?.read_to_string(&mut s)?;

        Self::parse(&s)
    }
}

pub fn generate_json(
    metadata: &[&Hdr10PlusMetadata],
    tool_name: &str,
    tool_version: &str,
) -> Value {
    let (profile, frame_json_list): (String, Vec<Value>) = json_list(metadata);

    let json_info = json!({
        "HDR10plusProfile": profile,
        "Version": format!("{}.0", &metadata[0].application_version),
    });

    let first_frames: Vec<u64> = frame_json_list
        .iter()
        .filter_map(|meta| {
            if meta.get("SceneFrameIndex").unwrap().as_u64().unwrap() == 0 {
                meta.get("SequenceFrameIndex").unwrap().as_u64()
            } else {
                None
            }
        })
        .collect();

    let mut scene_lengths: Vec<u64> = Vec::with_capacity(first_frames.len());

    for i in 0..first_frames.len() {
        if i < first_frames.len() - 1 {
            scene_lengths.push(first_frames[i + 1] - first_frames[i]);
        } else {
            scene_lengths.push(frame_json_list.len() as u64 - first_frames[i]);
        }
    }

    let scene_info_json = json!({
        "SceneFirstFrameIndex": first_frames,
        "SceneFrameNumbers": scene_lengths,
    });

    let final_json = json!({
        "JSONInfo": json_info,
        "SceneInfo": frame_json_list,
        "SceneInfoSummary": scene_info_json,
        "ToolInfo": json!({
            "Tool": tool_name,
            "Version": tool_version,
        })
    });

    final_json
}

pub fn json_list(list: &[&Hdr10PlusMetadata]) -> (String, Vec<Value>) {
    let profile = if list.iter().all(|m| m.profile == "B") {
        "B"
    } else if list.iter().all(|m| m.profile == "A") {
        "A"
    } else {
        "N/A"
    };

    let mut metadata_json_array = list
        .iter()
        .map(|m| {
            // Profile A, no bezier curve data
            if profile == "A" {
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
                let bc = m.bezier_curve.as_ref().expect("Invalid profile B: no Bezier curve data");

                json!({
                    "BezierCurveData": bc.to_json(),
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

    (profile.to_string(), metadata_json_array)
}

pub fn compute_scene_information(profile: &str, metadata_json_array: &mut [Value]) {
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
    pub fn separate_json(list: &[Self]) -> Value {
        json!({
            "DistributionIndex": Self::distribution_index(list),
            "DistributionValues": Self::distribution_values(list),
        })
    }
}

impl BezierCurve {
    pub fn to_json(&self) -> Value {
        json!({
            "Anchors": self.bezier_curve_anchors,
            "KneePointX": self.knee_point_x,
            "KneePointY": self.knee_point_y
        })
    }
}

impl TryFrom<&Hdr10PlusJsonMetadata> for Hdr10PlusMetadata {
    type Error = anyhow::Error;

    fn try_from(jm: &Hdr10PlusJsonMetadata) -> Result<Self> {
        let lp = &jm.luminance_parameters;
        let dists = &lp.luminance_distributions;

        ensure!(
            lp.max_scl.len() == 3,
            "MaxScl must contain exactly 3 elements"
        );

        let maxscl = [lp.max_scl[0], lp.max_scl[1], lp.max_scl[2]];

        ensure!(
            dists.distribution_index.len() == dists.distribution_values.len(),
            "DistributionIndex and DistributionValue sizes don't match"
        );
        ensure!(
            dists.distribution_index.len() <= 10,
            "DistributionIndex size should be at most 10"
        );
        ensure!(
            dists.distribution_values.len() <= 10,
            "DistributionValues size should be at most 10"
        );

        let distribution_parsed = dists
            .distribution_index
            .iter()
            .zip(dists.distribution_values.iter())
            .map(|(percentage, percentile)| DistributionMaxRgb {
                percentage: *percentage,
                percentile: *percentile,
            })
            .collect();

        let tone_mapping_flag = jm.bezier_curve_data.is_some();

        let bezier_curve = if let Some(bcd) = &jm.bezier_curve_data {
            let bc = BezierCurve {
                knee_point_x: bcd.knee_point_x,
                knee_point_y: bcd.knee_point_y,
                num_bezier_curve_anchors: bcd.anchors.len() as u8,
                bezier_curve_anchors: bcd.anchors.clone(),
            };

            Some(bc)
        } else {
            None
        };

        let mut meta = Self {
            itu_t_t35_country_code: 0xB5,
            itu_t_t35_terminal_provider_code: 0x3C,
            itu_t_t35_terminal_provider_oriented_code: 1,
            application_identifier: 4,
            application_version: 1,
            num_windows: jm.number_of_windows,
            processing_windows: None,
            targeted_system_display_maximum_luminance: jm.targeted_system_display_maximum_luminance,
            targeted_system_display_actual_peak_luminance_flag: false,
            actual_targeted_system_display: None,
            maxscl,
            average_maxrgb: lp.average_rgb,
            num_distribution_maxrgb_percentiles: dists.distribution_index.len() as u8,
            distribution_maxrgb: distribution_parsed,
            fraction_bright_pixels: 0,
            mastering_display_actual_peak_luminance_flag: false,
            actual_mastering_display: None,
            tone_mapping_flag,
            bezier_curve,
            color_saturation_mapping_flag: false,
            color_saturation_weight: 0,
            ..Default::default()
        };

        meta.set_profile();

        Ok(meta)
    }
}

impl VariablePeakBrightness for Hdr10PlusJsonMetadata {
    fn peak_brightness_nits(&self, source: PeakBrightnessSource) -> Option<f64> {
        match source {
            PeakBrightnessSource::Histogram => self
                .luminance_parameters
                .luminance_distributions
                .distribution_values
                .iter()
                .max()
                .map(|e| *e as f64 / 10.0),
            PeakBrightnessSource::Histogram99 => self
                .luminance_parameters
                .luminance_distributions
                .distribution_values
                .last()
                .map(|e| *e as f64 / 10.0),
            PeakBrightnessSource::MaxScl => self
                .luminance_parameters
                .max_scl
                .iter()
                .max()
                .map(|max| *max as f64 / 10.0),
            PeakBrightnessSource::MaxSclLuminance => {
                if let [r, g, b] = self.luminance_parameters.max_scl.as_slice() {
                    let r = *r as f64;
                    let g = *g as f64;
                    let b = *b as f64;

                    let luminance = (0.2627 * r) + (0.678 * g) + (0.0593 * b);
                    Some(luminance / 10.0)
                } else {
                    None
                }
            }
        }
    }
}
