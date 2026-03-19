#![allow(dead_code, unused_imports)]

use anyhow::Result;
use bitvec_helpers::bitstream_io_reader::BsIoSliceReader;

// Re-export all generic types/functions from the crate
pub use av1_parser::{
    IvfFrameHeader, IvfWriter, Obu, ObuReader, ObuWriter,
    OBU_TEMPORAL_DELIMITER, OBU_METADATA,
    OBU_SEQUENCE_HEADER, OBU_FRAME_HEADER, OBU_FRAME, OBU_REDUNDANT_FRAME_HEADER,
    decode_leb128, encode_leb128,
    try_read_ivf_file_header, read_ivf_frame_header, write_ivf_frame_header,
    read_obus_from_ivf_frame,
};

// ---------------------------------------------------------------------------
// HDR10+-specific constants
// ---------------------------------------------------------------------------

// Metadata type for ITU-T T.35 (HDR10+)
pub const METADATA_TYPE_ITUT_T35: u64 = 4;

// HDR10+ T.35 header identifiers
pub const HDR10PLUS_COUNTRY_CODE: u8 = 0xB5;
pub const HDR10PLUS_PROVIDER_CODE: u16 = 0x003C;
pub const HDR10PLUS_ORIENTED_CODE: u16 = 0x0001;
pub const HDR10PLUS_APP_ID: u8 = 4;

// ---------------------------------------------------------------------------
// HDR10+ detection helper
// ---------------------------------------------------------------------------

/// Returns the T.35 payload bytes (starting at country_code = 0xB5) if this
/// OBU_METADATA payload contains HDR10+ data.  The returned slice is the
/// portion that `Hdr10PlusMetadata::parse()` expects.
///
/// Layout of an OBU_METADATA payload for HDR10+:
/// ```text
/// metadata_type  (LEB128)  = 4
/// country_code   (u8)      = 0xB5
/// provider_code  (u16 BE)  = 0x003C
/// oriented_code  (u16 BE)  = 0x0001
/// app_id         (u8)      = 4
/// app_version    (u8)      = 1
/// <HDR10+ payload bits>
/// ```
pub fn extract_hdr10plus_t35_bytes(
    obu_payload: &[u8],
    validate: bool,
) -> Option<Vec<u8>> {
    if obu_payload.is_empty() {
        return None;
    }

    // metadata_type
    let (metadata_type, mt_len) = decode_leb128(obu_payload);
    if metadata_type != METADATA_TYPE_ITUT_T35 {
        return None;
    }

    let t35 = &obu_payload[mt_len..];
    if t35.len() < 7 {
        return None;
    }

    let country_code = t35[0];
    if country_code != HDR10PLUS_COUNTRY_CODE {
        return None;
    }

    let provider_code = u16::from_be_bytes([t35[1], t35[2]]);
    let oriented_code = u16::from_be_bytes([t35[3], t35[4]]);
    let app_id = t35[5];
    let app_version = t35[6];

    if provider_code != HDR10PLUS_PROVIDER_CODE
        || oriented_code != HDR10PLUS_ORIENTED_CODE
        || app_id != HDR10PLUS_APP_ID
    {
        return None;
    }

    let valid_version = if validate {
        app_version == 1
    } else {
        app_version <= 1
    };

    if !valid_version {
        return None;
    }

    // Return T.35 bytes starting at country_code (what parse() expects)
    Some(t35.to_vec())
}

/// Returns `true` if this OBU is an OBU_METADATA carrying HDR10+ T.35 data.
pub fn is_hdr10plus_obu(obu: &Obu, validate: bool) -> bool {
    obu.obu_type == OBU_METADATA
        && extract_hdr10plus_t35_bytes(&obu.payload, validate).is_some()
}

// ---------------------------------------------------------------------------
// Stateful AV1 parser
// ---------------------------------------------------------------------------

/// Information about a single temporal unit derived from stream parsing.
#[derive(Debug, Clone)]
pub struct TemporalUnitInfo {
    /// Zero-based index of this temporal unit in the stream.
    pub index: usize,
    /// `true` when the temporal unit produces a visible output frame.
    /// This is `false` only for frames with `show_frame = 0` and
    /// `showable_frame = 0`.
    pub is_displayed: bool,
    /// `true` when the temporal unit reuses a previously decoded frame
    /// via `show_existing_frame`.
    pub is_show_existing: bool,
}

/// Stateful AV1 bitstream parser.
///
/// Feed it each `Obu` in stream order via [`process_obu`].  After parsing the
/// entire stream, [`temporal_units`] provides per-TU display metadata needed
/// for frame-accurate HDR10+ injection.
pub struct Av1NaluParser {
    /// Derived from the sequence header; affects frame header interpretation.
    pub reduced_still_picture_header: bool,

    /// Running count of temporal units seen so far (incremented on each TD).
    pub temporal_unit_count: usize,

    /// Per-temporal-unit display info (populated as frame headers are parsed).
    pub temporal_units: Vec<TemporalUnitInfo>,

    /// A frame header was already parsed in the current TU (to skip redundant ones).
    frame_header_parsed_in_tu: bool,
}

impl Av1NaluParser {
    pub fn new() -> Self {
        Self {
            reduced_still_picture_header: false,
            temporal_unit_count: 0,
            temporal_units: Vec::new(),
            frame_header_parsed_in_tu: false,
        }
    }

    /// Process one OBU and update parser state.
    pub fn process_obu(&mut self, obu: &Obu) -> Result<()> {
        match obu.obu_type {
            OBU_TEMPORAL_DELIMITER => {
                // Marks the beginning of a new temporal unit.
                self.temporal_unit_count += 1;
                self.frame_header_parsed_in_tu = false;
            }
            OBU_SEQUENCE_HEADER => {
                self.parse_sequence_header(&obu.payload)?;
            }
            OBU_FRAME_HEADER | OBU_FRAME => {
                if !self.frame_header_parsed_in_tu {
                    let info = self.parse_frame_display_info(&obu.payload)?;
                    let tu_idx = self.temporal_unit_count.saturating_sub(1);
                    self.temporal_units.push(TemporalUnitInfo {
                        index: tu_idx,
                        is_displayed: info.0,
                        is_show_existing: info.1,
                    });
                    self.frame_header_parsed_in_tu = true;
                }
            }
            OBU_REDUNDANT_FRAME_HEADER => {
                // Redundant copies carry the same info — skip to avoid duplicates.
            }
            _ => {}
        }
        Ok(())
    }

    /// Return all collected temporal unit infos.
    pub fn temporal_units(&self) -> &[TemporalUnitInfo] {
        &self.temporal_units
    }

    /// Returns the number of temporal units that produce a displayed frame.
    pub fn display_frame_count(&self) -> usize {
        self.temporal_units.iter().filter(|t| t.is_displayed).count()
    }

    // -----------------------------------------------------------------------
    // Sequence header parsing
    // -----------------------------------------------------------------------

    fn parse_sequence_header(&mut self, payload: &[u8]) -> Result<()> {
        if payload.len() < 1 {
            return Ok(());
        }
        let mut r = BsIoSliceReader::from_slice(payload);

        // seq_profile (3 bits)
        let _seq_profile = r.read::<3, u8>()?;
        // still_picture (1 bit)
        let _still_picture = r.read::<1, u8>()?;
        // reduced_still_picture_header (1 bit)
        self.reduced_still_picture_header = r.read::<1, u8>()? != 0;

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Frame header parsing (minimal — only what is needed for display flags)
    // -----------------------------------------------------------------------

    /// Returns `(is_displayed, is_show_existing)`.
    fn parse_frame_display_info(&self, payload: &[u8]) -> Result<(bool, bool)> {
        if self.reduced_still_picture_header {
            // Section 5.9.2: reduced_still_picture_header implies
            // show_existing_frame = 0, show_frame = 1.
            return Ok((true, false));
        }

        if payload.is_empty() {
            return Ok((true, false));
        }

        let mut r = BsIoSliceReader::from_slice(payload);

        // show_existing_frame (f(1))
        let show_existing_frame = r.read::<1, u8>()? != 0;
        if show_existing_frame {
            return Ok((true, true));
        }

        // frame_type (f(2))
        let _frame_type = r.read::<2, u8>()?;

        // show_frame (f(1))
        let show_frame = r.read::<1, u8>()? != 0;

        if show_frame {
            Ok((true, false))
        } else {
            // showable_frame (f(1))
            let showable_frame = r.read::<1, u8>()? != 0;
            Ok((showable_frame, false))
        }
    }
}
