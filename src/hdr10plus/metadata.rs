use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};
use hevc_parser::{hevc, utils::add_start_code_emulation_prevention_3_byte};
use serde_json::{json, Value};

use super::{metadata_json, parser::MetadataFrame};

const DISTRIBUTION_INDEXES_9: &[u8] = &[1, 5, 10, 25, 50, 75, 90, 95, 99];
const DISTRIBUTION_INDEXES_10: &[u8] = &[1, 5, 10, 25, 50, 75, 90, 95, 98, 99];

#[derive(Debug, Clone, Default)]
pub struct Hdr10PlusMetadata {
    pub profile: String,

    pub itu_t_t35_country_code: u8,
    pub itu_t_t35_terminal_provider_code: u16,
    pub itu_t_t35_terminal_provider_oriented_code: u16,

    pub application_identifier: u8,
    pub application_version: u8,
    pub num_windows: u8,

    pub processing_windows: Option<Vec<ProcessingWindow>>,

    pub targeted_system_display_maximum_luminance: u32,
    pub targeted_system_display_actual_peak_luminance_flag: bool,

    pub actual_targeted_system_display: Option<ActualTargetedSystemDisplay>,

    pub maxscl: [u32; 3],
    pub average_maxrgb: u32,
    pub num_distribution_maxrgb_percentiles: u8,
    pub distribution_maxrgb: Vec<DistributionMaxRgb>,
    pub fraction_bright_pixels: u16,

    pub mastering_display_actual_peak_luminance_flag: bool,
    pub actual_mastering_display: Option<ActualMasteringDisplay>,

    pub tone_mapping_flag: bool,
    pub bezier_curve: Option<BezierCurve>,

    pub color_saturation_mapping_flag: bool,
    pub color_saturation_weight: u8,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ProcessingWindow {
    window_upper_left_corner_x: u16,
    window_upper_left_corner_y: u16,
    window_lower_right_corner_x: u16,
    window_lower_right_corner_y: u16,

    center_of_ellipse_x: u16,
    center_of_ellipse_y: u16,
    rotation_angle: u8,

    semimajor_axis_internal_ellipse: u16,
    semimajor_axis_external_ellipse: u16,
    semiminor_axis_external_ellipse: u16,

    overlap_process_option: bool,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ActualTargetedSystemDisplay {
    pub num_rows_targeted_system_display_actual_peak_luminance: u8,
    pub num_cols_targeted_system_display_actual_peak_luminance: u8,
    pub targeted_system_display_actual_peak_luminance: Vec<Vec<u8>>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct DistributionMaxRgb {
    percentage: u8,
    percentile: u32,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ActualMasteringDisplay {
    pub num_rows_mastering_display_actual_peak_luminance: u8,
    pub num_cols_mastering_display_actual_peak_luminanc: u8,
    pub mastering_display_actual_peak_luminance: Vec<Vec<u8>>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct BezierCurve {
    pub knee_point_x: u16,
    pub knee_point_y: u16,
    pub num_bezier_curve_anchors: u8,
    pub bezier_curve_anchors: Vec<u16>,
}

impl Hdr10PlusMetadata {
    pub fn parse(data: Vec<u8>) -> Hdr10PlusMetadata {
        let mut reader = BitVecReader::new(data);

        let mut meta = Hdr10PlusMetadata {
            itu_t_t35_country_code: reader.get_n(8),
            itu_t_t35_terminal_provider_code: reader.get_n(16),
            itu_t_t35_terminal_provider_oriented_code: reader.get_n(16),
            application_identifier: reader.get_n(8),
            application_version: reader.get_n(8),
            num_windows: reader.get_n(2),
            ..Default::default()
        };

        if meta.num_windows > 1 {
            let mut processing_windows = Vec::new();

            for _ in 1..meta.num_windows {
                let pw = ProcessingWindow::parse(&mut reader);
                processing_windows.push(pw);
            }

            meta.processing_windows = Some(processing_windows);
        }

        meta.targeted_system_display_maximum_luminance = reader.get_n(27);

        meta.targeted_system_display_actual_peak_luminance_flag = reader.get();
        if meta.targeted_system_display_actual_peak_luminance_flag {
            let atsd = ActualTargetedSystemDisplay::parse(&mut reader);
            meta.actual_targeted_system_display = Some(atsd);
        }

        for _ in 0..meta.num_windows {
            for i in 0..3 {
                meta.maxscl[i] = reader.get_n(17);
            }

            meta.average_maxrgb = reader.get_n(17);

            meta.num_distribution_maxrgb_percentiles = reader.get_n(4);
            for _ in 0..meta.num_distribution_maxrgb_percentiles {
                let dmrgb = DistributionMaxRgb::parse(&mut reader);
                meta.distribution_maxrgb.push(dmrgb);
            }

            meta.fraction_bright_pixels = reader.get_n(10);
        }

        meta.mastering_display_actual_peak_luminance_flag = reader.get();
        if meta.mastering_display_actual_peak_luminance_flag {
            let amd = ActualMasteringDisplay::parse(&mut reader);
            meta.actual_mastering_display = Some(amd);
        }

        for _ in 0..meta.num_windows {
            meta.tone_mapping_flag = reader.get();

            if meta.tone_mapping_flag {
                let bc = BezierCurve::parse(&mut reader);
                meta.bezier_curve = Some(bc);
            }
        }

        meta.color_saturation_mapping_flag = reader.get();
        if meta.color_saturation_mapping_flag {
            meta.color_saturation_weight = reader.get_n(6);
        }

        meta.set_profile();

        meta
    }

    pub fn validate(&self) {
        // SMPTE ST-2094 Application 4, Version 1
        assert_eq!(self.application_identifier, 4);
        assert_eq!(self.application_version, 1);

        // For version 1
        assert_eq!(self.num_windows, 1);
        assert!(!self.targeted_system_display_actual_peak_luminance_flag);
        assert!(!self.mastering_display_actual_peak_luminance_flag);
        assert!(!self.color_saturation_mapping_flag);

        // The value of targeted_system_display_maximum_luminance shall be in the range of 0 to 10000, inclusive
        assert!(self.targeted_system_display_maximum_luminance <= 10000);

        // Profile B needs Bezier curve information and a non zero target display (for OOTF)
        if self.tone_mapping_flag {
            assert!(self.targeted_system_display_maximum_luminance > 0);
        } else {
            assert_eq!(self.targeted_system_display_maximum_luminance, 0);
        }

        // Shall be under 100000, inclusive
        self.maxscl.iter().for_each(|&v| assert!(v <= 100_000));

        // Shall be under 100000, inclusive
        assert!(self.average_maxrgb <= 100_000);

        // Shall be under 100000, inclusive
        DistributionMaxRgb::validate(
            &self.distribution_maxrgb,
            self.num_distribution_maxrgb_percentiles,
        );

        if let Some(bc) = &self.bezier_curve {
            bc.validate();
        }
    }

    pub fn json_list(list: &[MetadataFrame]) -> (&str, Vec<Value>) {
        let profile = if list.iter().all(|m| m.metadata.profile == "B") {
            "B"
        } else if list.iter().all(|m| m.metadata.profile == "A") {
            "A"
        } else {
            "N/A"
        };

        let mut metadata_json_array = list
            .iter()
            .map(|mf| &mf.metadata)
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

        (profile, metadata_json_array)
    }

    fn set_profile(&mut self) {
        let profile = if self.tone_mapping_flag
            && self.targeted_system_display_maximum_luminance > 0
        {
            if let Some(bc) = &self.bezier_curve {
                if bc.num_bezier_curve_anchors > 0 {
                    "B"
                } else {
                    "N/A"
                }
            } else {
                "N/A"
            }
        } else if !self.tone_mapping_flag && self.targeted_system_display_maximum_luminance == 0 {
            "A"
        } else {
            "N/A"
        };

        self.profile = profile.to_string();
    }

    pub fn encode(&self) -> Vec<u8> {
        // Write NALU SEI_PREFIX header
        let mut header_writer = BitVecWriter::new();

        header_writer.write(false); // forbidden_zero_bit
        header_writer.write_n(&hevc::NAL_SEI_PREFIX.to_be_bytes(), 6); // nal_type
        header_writer.write_n(&(0_u8).to_be_bytes(), 6); // nuh_layer_id
        header_writer.write_n(&(1_u8).to_be_bytes(), 3); // temporal_id

        header_writer.write_n(&hevc::USER_DATA_REGISTERED_ITU_T_35.to_be_bytes(), 8);

        let mut writer = BitVecWriter::new();

        writer.write_n(&self.itu_t_t35_country_code.to_be_bytes(), 8);
        writer.write_n(&self.itu_t_t35_terminal_provider_code.to_be_bytes(), 16);
        writer.write_n(
            &self.itu_t_t35_terminal_provider_oriented_code.to_be_bytes(),
            16,
        );
        writer.write_n(&self.application_identifier.to_be_bytes(), 8);
        writer.write_n(&self.application_version.to_be_bytes(), 8);
        writer.write_n(&self.num_windows.to_be_bytes(), 2);

        if let Some(pws) = &self.processing_windows {
            pws.iter().for_each(|pw| pw.encode(&mut writer));
        }

        writer.write_n(
            &self.targeted_system_display_maximum_luminance.to_be_bytes(),
            27,
        );

        writer.write(self.targeted_system_display_actual_peak_luminance_flag);
        if let Some(atsd) = &self.actual_targeted_system_display {
            atsd.encode(&mut writer);
        }

        for _ in 0..self.num_windows {
            self.maxscl
                .iter()
                .for_each(|e| writer.write_n(&e.to_be_bytes(), 17));

            writer.write_n(&self.average_maxrgb.to_be_bytes(), 17);

            writer.write_n(&self.num_distribution_maxrgb_percentiles.to_be_bytes(), 4);

            self.distribution_maxrgb
                .iter()
                .for_each(|e| e.encode(&mut writer));

            writer.write_n(&self.fraction_bright_pixels.to_be_bytes(), 10);
        }

        writer.write(self.mastering_display_actual_peak_luminance_flag);

        if let Some(amd) = &self.actual_mastering_display {
            amd.encode(&mut writer);
        }

        for _ in 0..self.num_windows {
            writer.write(self.tone_mapping_flag);

            if let Some(bc) = &self.bezier_curve {
                bc.encode(&mut writer);
            }
        }

        writer.write(self.color_saturation_mapping_flag);
        if self.color_saturation_mapping_flag {
            writer.write_n(&self.color_saturation_weight.to_be_bytes(), 6);
        }

        let mut payload = writer.as_slice().to_vec();

        // FIXME: This should probably be 1024 but not sure how to write a longer header
        assert!(payload.len() <= 255);
        header_writer.write_n(&payload.len().to_be_bytes(), 8);

        payload.push(0x80);

        let mut data = header_writer.as_slice().to_vec();
        data.append(&mut payload);

        add_start_code_emulation_prevention_3_byte(&mut data);

        data
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
    pub fn parse(reader: &mut BitVecReader) -> DistributionMaxRgb {
        DistributionMaxRgb {
            percentage: reader.get_n(7),
            percentile: reader.get_n(17),
        }
    }

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

    pub fn encode(&self, writer: &mut BitVecWriter) {
        writer.write_n(&self.percentage.to_be_bytes(), 7);
        writer.write_n(&self.percentile.to_be_bytes(), 17);
    }
}

impl ProcessingWindow {
    pub fn parse(reader: &mut BitVecReader) -> ProcessingWindow {
        ProcessingWindow {
            window_upper_left_corner_x: reader.get_n(16),
            window_upper_left_corner_y: reader.get_n(16),
            window_lower_right_corner_x: reader.get_n(16),
            window_lower_right_corner_y: reader.get_n(16),
            center_of_ellipse_x: reader.get_n(16),
            center_of_ellipse_y: reader.get_n(16),
            rotation_angle: reader.get_n(8),
            semimajor_axis_internal_ellipse: reader.get_n(16),
            semimajor_axis_external_ellipse: reader.get_n(16),
            semiminor_axis_external_ellipse: reader.get_n(16),
            overlap_process_option: reader.get(),
        }
    }

    pub fn encode(&self, writer: &mut BitVecWriter) {
        writer.write_n(&self.window_upper_left_corner_x.to_be_bytes(), 16);
        writer.write_n(&self.window_upper_left_corner_y.to_be_bytes(), 16);
        writer.write_n(&self.window_lower_right_corner_x.to_be_bytes(), 16);
        writer.write_n(&self.window_lower_right_corner_y.to_be_bytes(), 16);
        writer.write_n(&self.center_of_ellipse_x.to_be_bytes(), 16);
        writer.write_n(&self.center_of_ellipse_y.to_be_bytes(), 16);
        writer.write_n(&self.rotation_angle.to_be_bytes(), 8);
        writer.write_n(&self.semimajor_axis_internal_ellipse.to_be_bytes(), 16);
        writer.write_n(&self.semimajor_axis_external_ellipse.to_be_bytes(), 16);
        writer.write_n(&self.semimajor_axis_external_ellipse.to_be_bytes(), 16);
        writer.write(self.overlap_process_option);
    }
}

impl ActualTargetedSystemDisplay {
    pub fn parse(reader: &mut BitVecReader) -> ActualTargetedSystemDisplay {
        let mut atsd = ActualTargetedSystemDisplay {
            num_rows_targeted_system_display_actual_peak_luminance: reader.get_n(5),
            num_cols_targeted_system_display_actual_peak_luminance: reader.get_n(5),
            ..Default::default()
        };

        atsd.targeted_system_display_actual_peak_luminance.resize(
            atsd.num_rows_targeted_system_display_actual_peak_luminance as usize,
            vec![0; atsd.num_cols_targeted_system_display_actual_peak_luminance as usize],
        );

        for i in 0..atsd.num_rows_targeted_system_display_actual_peak_luminance as usize {
            for j in 0..atsd.num_cols_targeted_system_display_actual_peak_luminance as usize {
                atsd.targeted_system_display_actual_peak_luminance[i][j] = reader.get_n(4);
            }
        }

        atsd
    }

    pub fn encode(&self, writer: &mut BitVecWriter) {
        writer.write_n(
            &self
                .num_rows_targeted_system_display_actual_peak_luminance
                .to_be_bytes(),
            5,
        );
        writer.write_n(
            &self
                .num_cols_targeted_system_display_actual_peak_luminance
                .to_be_bytes(),
            5,
        );

        for i in 0..self.num_rows_targeted_system_display_actual_peak_luminance as usize {
            for j in 0..self.num_cols_targeted_system_display_actual_peak_luminance as usize {
                writer.write_n(
                    &self.targeted_system_display_actual_peak_luminance[i][j].to_be_bytes(),
                    4,
                );
            }
        }
    }
}

impl ActualMasteringDisplay {
    pub fn parse(reader: &mut BitVecReader) -> ActualMasteringDisplay {
        let mut amd = ActualMasteringDisplay {
            num_rows_mastering_display_actual_peak_luminance: reader.get_n(5),
            num_cols_mastering_display_actual_peak_luminanc: reader.get_n(5),
            ..Default::default()
        };

        amd.mastering_display_actual_peak_luminance.resize(
            amd.num_rows_mastering_display_actual_peak_luminance as usize,
            vec![0; amd.num_cols_mastering_display_actual_peak_luminanc as usize],
        );

        for i in 0..amd.num_rows_mastering_display_actual_peak_luminance as usize {
            for j in 0..amd.num_cols_mastering_display_actual_peak_luminanc as usize {
                amd.mastering_display_actual_peak_luminance[i][j] = reader.get_n(4);
            }
        }

        amd
    }

    pub fn encode(&self, writer: &mut BitVecWriter) {
        writer.write_n(
            &self
                .num_rows_mastering_display_actual_peak_luminance
                .to_be_bytes(),
            5,
        );
        writer.write_n(
            &self
                .num_cols_mastering_display_actual_peak_luminanc
                .to_be_bytes(),
            5,
        );

        for i in 0..self.num_rows_mastering_display_actual_peak_luminance as usize {
            for j in 0..self.num_cols_mastering_display_actual_peak_luminanc as usize {
                writer.write_n(
                    &self.mastering_display_actual_peak_luminance[i][j].to_be_bytes(),
                    4,
                );
            }
        }
    }
}

impl BezierCurve {
    pub fn parse(reader: &mut BitVecReader) -> BezierCurve {
        let mut bc = BezierCurve {
            knee_point_x: reader.get_n(12),
            knee_point_y: reader.get_n(12),
            num_bezier_curve_anchors: reader.get_n(4),
            ..Default::default()
        };

        bc.bezier_curve_anchors
            .resize(bc.num_bezier_curve_anchors as usize, 0);

        for i in 0..bc.num_bezier_curve_anchors as usize {
            bc.bezier_curve_anchors[i] = reader.get_n(10);
        }

        bc
    }

    fn validate(&self) {
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

    fn to_json(&self) -> Value {
        json!({
            "Anchors": self.bezier_curve_anchors,
            "KneePointX": self.knee_point_x,
            "KneePointY": self.knee_point_y
        })
    }

    pub fn encode(&self, writer: &mut BitVecWriter) {
        writer.write_n(&self.knee_point_x.to_be_bytes(), 12);
        writer.write_n(&self.knee_point_y.to_be_bytes(), 12);
        writer.write_n(&self.num_bezier_curve_anchors.to_be_bytes(), 4);

        self.bezier_curve_anchors
            .iter()
            .for_each(|e| writer.write_n(&e.to_be_bytes(), 10));
    }
}

impl From<&metadata_json::Hdr10PlusJsonMetadata> for Hdr10PlusMetadata {
    fn from(jm: &metadata_json::Hdr10PlusJsonMetadata) -> Self {
        let lp = &jm.luminance_parameters;
        let dists = &lp.luminance_distributions;

        assert!(lp.max_scl.len() == 3);
        let maxscl = [lp.max_scl[0], lp.max_scl[1], lp.max_scl[2]];

        assert!(dists.distribution_index.len() == dists.distribution_values.len());
        assert!(dists.distribution_index.len() <= 10);
        assert!(dists.distribution_values.len() <= 10);

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

        meta
    }
}
