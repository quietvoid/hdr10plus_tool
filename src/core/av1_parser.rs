// Re-export generic types/functions from the crate
pub use av1_parser::{
    IvfFrameHeader, Obu,
    OBU_TEMPORAL_DELIMITER, OBU_METADATA,
    decode_leb128,
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
