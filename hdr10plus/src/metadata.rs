use anyhow::{Result, bail, ensure};
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

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

#[derive(Debug, PartialEq, Clone, Default, Eq)]
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

#[derive(Debug, PartialEq, Clone, Default, Eq)]
pub struct ActualTargetedSystemDisplay {
    pub num_rows_targeted_system_display_actual_peak_luminance: u8,
    pub num_cols_targeted_system_display_actual_peak_luminance: u8,
    pub targeted_system_display_actual_peak_luminance: Vec<Vec<u8>>,
}

#[derive(Debug, PartialEq, Clone, Default, Eq)]
pub struct DistributionMaxRgb {
    pub percentage: u8,
    pub percentile: u32,
}

#[derive(Debug, PartialEq, Clone, Default, Eq)]
pub struct ActualMasteringDisplay {
    pub num_rows_mastering_display_actual_peak_luminance: u8,
    pub num_cols_mastering_display_actual_peak_luminanc: u8,
    pub mastering_display_actual_peak_luminance: Vec<Vec<u8>>,
}

#[derive(Debug, PartialEq, Clone, Default, Eq)]
pub struct BezierCurve {
    pub knee_point_x: u16,
    pub knee_point_y: u16,
    pub num_bezier_curve_anchors: u8,
    pub bezier_curve_anchors: Vec<u16>,
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct Hdr10PlusMetadataEncOpts {
    /// Validate the metadata's conformance
    pub validate: bool,
    /// Useful for AV1 payload, which don't contain `itu_t_t35_country_code`
    pub with_country_code: bool,
}

/// How to extract the peak brightness for the metadata
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeakBrightnessSource {
    /// The max value from the histogram measurements
    Histogram,
    /// The last percentile in the histogram, usually 99.98% brightness percentile
    Histogram99,
    /// The max value in `maxscl`
    MaxScl,
    /// The luminance calculated from the `maxscl` components
    /// Assumed BT.2020 primaries
    MaxSclLuminance,
}
pub trait VariablePeakBrightness {
    fn peak_brightness_nits(&self, source: PeakBrightnessSource) -> Option<f64>;
}

impl Hdr10PlusMetadata {
    pub fn parse(data: &[u8]) -> Result<Hdr10PlusMetadata> {
        let mut reader = BsIoSliceReader::from_slice(data);

        let mut meta = Hdr10PlusMetadata {
            itu_t_t35_country_code: reader.read::<8, u8>()?,
            itu_t_t35_terminal_provider_code: reader.read::<16, u16>()?,
            itu_t_t35_terminal_provider_oriented_code: reader.read::<16, u16>()?,
            application_identifier: reader.read::<8, u8>()?,
            application_version: reader.read::<8, u8>()?,
            num_windows: reader.read::<2, u8>()?,
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

        meta.targeted_system_display_maximum_luminance = reader.read::<27, u32>()?;

        meta.targeted_system_display_actual_peak_luminance_flag = reader.read_bit()?;
        if meta.targeted_system_display_actual_peak_luminance_flag {
            let atsd = ActualTargetedSystemDisplay::parse(&mut reader)?;
            meta.actual_targeted_system_display = Some(atsd);
        }

        for _ in 0..meta.num_windows {
            for i in 0..3 {
                meta.maxscl[i] = reader.read::<17, u32>()?;
            }

            meta.average_maxrgb = reader.read::<17, u32>()?;

            meta.num_distribution_maxrgb_percentiles = reader.read::<4, u8>()?;
            for _ in 0..meta.num_distribution_maxrgb_percentiles {
                let dmrgb = DistributionMaxRgb::parse(&mut reader)?;
                meta.distribution_maxrgb.push(dmrgb);
            }

            meta.fraction_bright_pixels = reader.read::<10, u16>()?;
        }

        meta.mastering_display_actual_peak_luminance_flag = reader.read_bit()?;
        if meta.mastering_display_actual_peak_luminance_flag {
            let amd = ActualMasteringDisplay::parse(&mut reader)?;
            meta.actual_mastering_display = Some(amd);
        }

        for _ in 0..meta.num_windows {
            meta.tone_mapping_flag = reader.read_bit()?;

            if meta.tone_mapping_flag {
                let bc = BezierCurve::parse(&mut reader)?;
                meta.bezier_curve = Some(bc);
            }
        }

        meta.color_saturation_mapping_flag = reader.read_bit()?;
        if meta.color_saturation_mapping_flag {
            meta.color_saturation_weight = reader.read::<6, u8>()?;
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
        ensure!(
            self.targeted_system_display_maximum_luminance <= 10000,
            "Invalid targeted_system_display_maximum_luminance, should be at most 10 0000. Actual: {}",
            self.targeted_system_display_maximum_luminance
        );

        // Profile B needs Bezier curve information and a non zero target display (for OOTF)
        if self.tone_mapping_flag {
            ensure!(
                self.targeted_system_display_maximum_luminance != 0,
                "Invalid targeted_system_display_maximum_luminance for profile B, must not be zero."
            );
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

    pub fn encode_with_opts(&self, opts: &Hdr10PlusMetadataEncOpts) -> Result<Vec<u8>> {
        if opts.validate {
            self.validate()?;
        }

        let mut writer = BitstreamIoWriter::with_capacity(64);

        if opts.with_country_code {
            writer.write::<8, u8>(self.itu_t_t35_country_code)?;
        }

        writer.write::<16, u16>(self.itu_t_t35_terminal_provider_code)?;
        writer.write::<16, u16>(self.itu_t_t35_terminal_provider_oriented_code)?;
        writer.write::<8, u8>(self.application_identifier)?;
        writer.write::<8, u8>(self.application_version)?;
        writer.write::<2, u8>(self.num_windows)?;

        if let Some(pws) = &self.processing_windows {
            for pw in pws {
                pw.encode(&mut writer)?;
            }
        }

        writer.write::<27, u32>(self.targeted_system_display_maximum_luminance)?;

        writer.write_bit(self.targeted_system_display_actual_peak_luminance_flag)?;
        if let Some(atsd) = &self.actual_targeted_system_display {
            atsd.encode(&mut writer)?;
        }

        for _ in 0..self.num_windows {
            for e in self.maxscl {
                writer.write::<17, u32>(e)?;
            }

            writer.write::<17, u32>(self.average_maxrgb)?;

            writer.write::<4, u8>(self.num_distribution_maxrgb_percentiles)?;

            for dm in &self.distribution_maxrgb {
                dm.encode(&mut writer)?;
            }

            writer.write::<10, u16>(self.fraction_bright_pixels)?;
        }

        writer.write_bit(self.mastering_display_actual_peak_luminance_flag)?;

        if let Some(amd) = &self.actual_mastering_display {
            amd.encode(&mut writer)?;
        }

        for _ in 0..self.num_windows {
            writer.write_bit(self.tone_mapping_flag)?;

            if let Some(bc) = &self.bezier_curve {
                bc.encode(&mut writer)?;
            }
        }

        writer.write_bit(self.color_saturation_mapping_flag)?;
        if self.color_saturation_mapping_flag {
            writer.write::<6, u8>(self.color_saturation_weight)?;
        }

        writer.byte_align()?;

        Ok(writer.into_inner())
    }

    #[deprecated(since = "1.2.0", note = "Replaced by encode_with_opts")]
    pub fn encode(&self, validate: bool) -> Result<Vec<u8>> {
        let opts = Hdr10PlusMetadataEncOpts {
            validate,
            ..Default::default()
        };

        self.encode_with_opts(&opts)
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
    fn parse(reader: &mut BsIoSliceReader) -> Result<DistributionMaxRgb> {
        Ok(DistributionMaxRgb {
            percentage: reader.read::<7, u8>()?,
            percentile: reader.read::<17, u32>()?,
        })
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

    fn encode(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        writer.write::<7, u8>(self.percentage)?;
        writer.write::<17, u32>(self.percentile)?;

        Ok(())
    }
}

impl ProcessingWindow {
    fn parse(reader: &mut BsIoSliceReader) -> Result<ProcessingWindow> {
        Ok(ProcessingWindow {
            window_upper_left_corner_x: reader.read::<16, u16>()?,
            window_upper_left_corner_y: reader.read::<16, u16>()?,
            window_lower_right_corner_x: reader.read::<16, u16>()?,
            window_lower_right_corner_y: reader.read::<16, u16>()?,
            center_of_ellipse_x: reader.read::<16, u16>()?,
            center_of_ellipse_y: reader.read::<16, u16>()?,
            rotation_angle: reader.read::<8, u8>()?,
            semimajor_axis_internal_ellipse: reader.read::<16, u16>()?,
            semimajor_axis_external_ellipse: reader.read::<16, u16>()?,
            semiminor_axis_external_ellipse: reader.read::<16, u16>()?,
            overlap_process_option: reader.read_bit()?,
        })
    }

    fn encode(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        writer.write::<16, u16>(self.window_upper_left_corner_x)?;
        writer.write::<16, u16>(self.window_upper_left_corner_y)?;
        writer.write::<16, u16>(self.window_lower_right_corner_x)?;
        writer.write::<16, u16>(self.window_lower_right_corner_y)?;
        writer.write::<16, u16>(self.center_of_ellipse_x)?;
        writer.write::<16, u16>(self.center_of_ellipse_y)?;
        writer.write::<8, u8>(self.rotation_angle)?;
        writer.write::<16, u16>(self.semimajor_axis_internal_ellipse)?;
        writer.write::<16, u16>(self.semimajor_axis_external_ellipse)?;
        writer.write::<16, u16>(self.semimajor_axis_external_ellipse)?;
        writer.write_bit(self.overlap_process_option)?;

        Ok(())
    }
}

impl ActualTargetedSystemDisplay {
    fn parse(reader: &mut BsIoSliceReader) -> Result<ActualTargetedSystemDisplay> {
        let mut atsd = ActualTargetedSystemDisplay {
            num_rows_targeted_system_display_actual_peak_luminance: reader.read::<5, u8>()?,
            num_cols_targeted_system_display_actual_peak_luminance: reader.read::<5, u8>()?,
            ..Default::default()
        };

        atsd.targeted_system_display_actual_peak_luminance.resize(
            atsd.num_rows_targeted_system_display_actual_peak_luminance as usize,
            vec![0; atsd.num_cols_targeted_system_display_actual_peak_luminance as usize],
        );

        for i in 0..atsd.num_rows_targeted_system_display_actual_peak_luminance as usize {
            for j in 0..atsd.num_cols_targeted_system_display_actual_peak_luminance as usize {
                atsd.targeted_system_display_actual_peak_luminance[i][j] =
                    reader.read::<4, u8>()?;
            }
        }

        Ok(atsd)
    }

    fn encode(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        writer.write::<5, u8>(self.num_rows_targeted_system_display_actual_peak_luminance)?;
        writer.write::<5, u8>(self.num_cols_targeted_system_display_actual_peak_luminance)?;

        for i in 0..self.num_rows_targeted_system_display_actual_peak_luminance as usize {
            for j in 0..self.num_cols_targeted_system_display_actual_peak_luminance as usize {
                writer.write::<4, u8>(self.targeted_system_display_actual_peak_luminance[i][j])?;
            }
        }

        Ok(())
    }
}

impl ActualMasteringDisplay {
    fn parse(reader: &mut BsIoSliceReader) -> Result<ActualMasteringDisplay> {
        let mut amd = ActualMasteringDisplay {
            num_rows_mastering_display_actual_peak_luminance: reader.read::<5, u8>()?,
            num_cols_mastering_display_actual_peak_luminanc: reader.read::<5, u8>()?,
            ..Default::default()
        };

        amd.mastering_display_actual_peak_luminance.resize(
            amd.num_rows_mastering_display_actual_peak_luminance as usize,
            vec![0; amd.num_cols_mastering_display_actual_peak_luminanc as usize],
        );

        for i in 0..amd.num_rows_mastering_display_actual_peak_luminance as usize {
            for j in 0..amd.num_cols_mastering_display_actual_peak_luminanc as usize {
                amd.mastering_display_actual_peak_luminance[i][j] = reader.read::<4, u8>()?;
            }
        }

        Ok(amd)
    }

    fn encode(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        writer.write::<5, u8>(self.num_rows_mastering_display_actual_peak_luminance)?;
        writer.write::<5, u8>(self.num_cols_mastering_display_actual_peak_luminanc)?;

        for i in 0..self.num_rows_mastering_display_actual_peak_luminance as usize {
            for j in 0..self.num_cols_mastering_display_actual_peak_luminanc as usize {
                writer.write::<4, u8>(self.mastering_display_actual_peak_luminance[i][j])?;
            }
        }

        Ok(())
    }
}

impl BezierCurve {
    fn parse(reader: &mut BsIoSliceReader) -> Result<BezierCurve> {
        let mut bc = BezierCurve {
            knee_point_x: reader.read::<12, u16>()?,
            knee_point_y: reader.read::<12, u16>()?,
            num_bezier_curve_anchors: reader.read::<4, u8>()?,
            ..Default::default()
        };

        bc.bezier_curve_anchors
            .resize(bc.num_bezier_curve_anchors as usize, 0);

        for i in 0..bc.num_bezier_curve_anchors as usize {
            bc.bezier_curve_anchors[i] = reader.read::<10, u16>()?;
        }

        Ok(bc)
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

    fn encode(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        writer.write::<12, u16>(self.knee_point_x)?;
        writer.write::<12, u16>(self.knee_point_y)?;
        writer.write::<4, u8>(self.num_bezier_curve_anchors)?;

        for e in self.bezier_curve_anchors.iter().copied() {
            writer.write::<10, u16>(e)?;
        }

        Ok(())
    }
}

impl Default for Hdr10PlusMetadataEncOpts {
    fn default() -> Self {
        Self {
            validate: true,
            with_country_code: true,
        }
    }
}

impl VariablePeakBrightness for Hdr10PlusMetadata {
    fn peak_brightness_nits(&self, source: PeakBrightnessSource) -> Option<f64> {
        match source {
            PeakBrightnessSource::Histogram => self
                .distribution_maxrgb
                .iter()
                .max_by_key(|x| x.percentile)
                .map(|e| e.percentile as f64 / 10.0),
            PeakBrightnessSource::Histogram99 => self
                .distribution_maxrgb
                .iter()
                .last()
                .map(|e| e.percentile as f64 / 10.0),
            PeakBrightnessSource::MaxScl => self.maxscl.iter().max().map(|max| *max as f64 / 10.0),
            PeakBrightnessSource::MaxSclLuminance => {
                let [r, g, b] = self.maxscl.map(|e| e as f64);
                let luminance = (0.2627 * r) + (0.678 * g) + (0.0593 * b);
                Some(luminance / 10.0)
            }
        }
    }
}

impl std::fmt::Display for PeakBrightnessSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Histogram => f.write_str("Histogram maximum value"),
            Self::Histogram99 => f.write_str("Histogram 99.98% percentile from metadata"),
            Self::MaxScl => f.write_str("MaxSCL maximum value"),
            Self::MaxSclLuminance => {
                f.write_str("MaxSCL Luminance, calculated from the components")
            }
        }
    }
}
