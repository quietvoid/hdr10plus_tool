use anyhow::{bail, ensure, Result};
use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

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
    pub percentage: u8,
    pub percentile: u32,
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
    pub fn parse(data: Vec<u8>) -> Result<Hdr10PlusMetadata> {
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
                let pw = ProcessingWindow::parse(&mut reader)?;
                processing_windows.push(pw);
            }

            meta.processing_windows = Some(processing_windows);
        }

        meta.targeted_system_display_maximum_luminance = reader.get_n(27);

        meta.targeted_system_display_actual_peak_luminance_flag = reader.get()?;
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

        meta.mastering_display_actual_peak_luminance_flag = reader.get()?;
        if meta.mastering_display_actual_peak_luminance_flag {
            let amd = ActualMasteringDisplay::parse(&mut reader);
            meta.actual_mastering_display = Some(amd);
        }

        for _ in 0..meta.num_windows {
            meta.tone_mapping_flag = reader.get()?;

            if meta.tone_mapping_flag {
                let bc = BezierCurve::parse(&mut reader);
                meta.bezier_curve = Some(bc);
            }
        }

        meta.color_saturation_mapping_flag = reader.get()?;
        if meta.color_saturation_mapping_flag {
            meta.color_saturation_weight = reader.get_n(6);
        }

        meta.set_profile();

        Ok(meta)
    }

    pub fn validate(&self) -> Result<()> {
        // SMPTE ST-2094 Application 4, Version 1
        ensure!(
            self.application_identifier == 4,
            "Invalid application_identifier: {}",
            self.application_identifier
        );
        ensure!(
            self.application_version == 1,
            "Invalid application_version: {}",
            self.application_version
        );

        // For version 1
        if self.application_version == 1 {
            self.validate_v1()?;
        }

        // The value of targeted_system_display_maximum_luminance shall be in the range of 0 to 10000, inclusive
        ensure!(self.targeted_system_display_maximum_luminance <= 10000, "Invalid targeted_system_display_maximum_luminance, should be at most 10 0000. Actual: {}", self.targeted_system_display_maximum_luminance);

        // Profile B needs Bezier curve information and a non zero target display (for OOTF)
        if self.tone_mapping_flag {
            ensure!(self.targeted_system_display_maximum_luminance != 0, "Invalid targeted_system_display_maximum_luminance for profile B, must not be zero.");
        } else {
            ensure!(
                self.targeted_system_display_maximum_luminance == 0,
                "Invalid targeted_system_display_maximum_luminance for profile A, must be zero."
            );
        }

        // Shall be under 100000, inclusive
        if !self.maxscl.iter().all(|&v| v <= 100_000) {
            bail!("Invalid MaxScl values over 100 000: {:?}", self.maxscl);
        }

        // Shall be under 100000, inclusive
        ensure!(
            self.average_maxrgb <= 100_000,
            "Invalid AverageMaxRGB value over 100 000: {}",
            self.average_maxrgb
        );

        // Shall be under 100000, inclusive
        DistributionMaxRgb::validate(
            &self.distribution_maxrgb,
            self.num_distribution_maxrgb_percentiles,
        )?;

        if let Some(bc) = &self.bezier_curve {
            bc.validate()?;
        }

        Ok(())
    }

    pub(crate) fn set_profile(&mut self) {
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

    pub fn encode(&self, validate: bool) -> Result<Vec<u8>> {
        if validate {
            self.validate()?;
        }

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

        let payload = writer.as_slice().to_vec();

        Ok(payload)
    }

    fn validate_v1(&self) -> Result<()> {
        ensure!(
            self.num_windows == 1,
            "Invalid num_windows: {}",
            self.num_windows
        );
        ensure!(
            !self.targeted_system_display_actual_peak_luminance_flag,
            "Invalid for version 1: targeted_system_display_actual_peak_luminance_flag {}",
            self.targeted_system_display_actual_peak_luminance_flag
        );
        ensure!(
            !self.mastering_display_actual_peak_luminance_flag,
            "Invalid for version 1: mastering_display_actual_peak_luminance_flag {}",
            self.mastering_display_actual_peak_luminance_flag
        );
        ensure!(
            !self.color_saturation_mapping_flag,
            "Invalid for version 1: color_saturation_mapping_flag {}",
            self.color_saturation_mapping_flag
        );

        Ok(())
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

    pub fn validate(list: &[Self], num_distribution_maxrgb_percentiles: u8) -> Result<()> {
        // The value of num_distribution_maxrgb_percentiles shall be 9 or 10 (for all we know)
        let correct_indexes = match num_distribution_maxrgb_percentiles {
            9 => DISTRIBUTION_INDEXES_9,
            10 => DISTRIBUTION_INDEXES_10,
            _ => bail!(
                "Invalid number of percentiles: {}",
                num_distribution_maxrgb_percentiles
            ),
        };

        // Distribution indexes should be equal to:
        // 9 indexes: [1, 5, 10, 25, 50, 75, 90, 95, 99]
        // 10 indexes: [1, 5, 10, 25, 50, 75, 90, 95, 98, 99]
        ensure!(
            Self::distribution_index(list) == correct_indexes,
            "Invalid DistributionIndex values: {:?}",
            Self::distribution_index(list)
        );

        if !Self::distribution_values(list)
            .iter()
            .all(|&v| v <= 100_000)
        {
            bail!(
                "Invalid DistributionValues over 100 000: {:?}",
                Self::distribution_values(list)
            );
        }

        Ok(())
    }

    pub fn encode(&self, writer: &mut BitVecWriter) {
        writer.write_n(&self.percentage.to_be_bytes(), 7);
        writer.write_n(&self.percentile.to_be_bytes(), 17);
    }
}

impl ProcessingWindow {
    pub fn parse(reader: &mut BitVecReader) -> Result<ProcessingWindow> {
        Ok(ProcessingWindow {
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
            overlap_process_option: reader.get()?,
        })
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

    fn validate(&self) -> Result<()> {
        // The value of knee_point_x shall be in the range of 0 to 1, and in multiples of 1/4095
        ensure!(
            self.knee_point_x <= 4095,
            "Invalid knee point x: {}",
            self.knee_point_x
        );
        ensure!(
            self.knee_point_y <= 4095,
            "Invalid knee point y: {}",
            self.knee_point_y
        );

        // The maximum value shall be 9
        ensure!(
            self.num_bezier_curve_anchors <= 9,
            "Invalid number of Bezier curve anchors: {}",
            self.num_bezier_curve_anchors
        );

        // Shall be under 1024
        if !self.bezier_curve_anchors.iter().all(|&v| v < 1024) {
            bail!(
                "Invalid Bezier curve values: {:?}",
                self.bezier_curve_anchors
            );
        }

        Ok(())
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
