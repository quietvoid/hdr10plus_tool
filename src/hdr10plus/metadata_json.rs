use std::{fs::File, io::Read, path::Path};

use serde::{Deserialize, Serialize};

use super::metadata::Hdr10PlusMetadata;

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

#[derive(Debug, Default, Deserialize, Serialize)]
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

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct BezierCurveData {
    pub anchors: Vec<u16>,
    pub knee_point_x: u16,
    pub knee_point_y: u16,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct LuminanceParameters {
    #[serde(rename = "AverageRGB")]
    pub average_rgb: u32,

    pub luminance_distributions: LuminanceDistributions,
    pub max_scl: Vec<u32>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct LuminanceDistributions {
    pub distribution_index: Vec<u8>,
    pub distribution_values: Vec<u32>,
}

impl MetadataJsonRoot {
    pub fn parse(input: &Path) -> MetadataJsonRoot {
        let mut s = String::new();
        File::open(input).unwrap().read_to_string(&mut s).unwrap();

        let res = serde_json::from_str::<MetadataJsonRoot>(&s);

        if let Ok(metadata_root) = res {
            metadata_root
        } else {
            panic!("Failed parsing JSON metadata\n{:?}", res);
        }
    }
}

impl Hdr10PlusJsonMetadata {
    pub fn encode_binary(&self, validate: bool) -> Vec<u8> {
        let meta = Hdr10PlusMetadata::from(self);

        if validate {
            meta.validate()
        }

        meta.encode()
    }
}
