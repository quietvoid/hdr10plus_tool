use anyhow::Result;

use crate::metadata::{Hdr10PlusMetadata, Hdr10PlusMetadataEncOpts};

#[cfg(feature = "json")]
use crate::metadata_json::Hdr10PlusJsonMetadata;

/// OBU type for metadata
const OBU_METADATA: u8 = 5;

/// T.35 metadata type value for ITU-T T.35
const METADATA_TYPE_ITUT_T35: u64 = 4;

/// Encodes HDR10+ metadata as a complete AV1 OBU_METADATA unit.
///
/// The OBU contains:
/// - OBU header (1 byte, type=5, has_size_field=1)
/// - OBU size (LEB128)
/// - metadata_type = 4 (METADATA_TYPE_ITUT_T35, LEB128)
/// - ITU-T T.35 payload (country code + HDR10+ bitstream)
pub fn encode_hdr10plus_obu(metadata: &Hdr10PlusMetadata, validate: bool) -> Result<Vec<u8>> {
    let opts = Hdr10PlusMetadataEncOpts {
        validate,
        with_country_code: true,
        ..Default::default()
    };

    // T.35 payload including country code (0xB5), terminal_provider_code, etc.
    let t35_payload = metadata.encode_with_opts(&opts)?;

    // OBU_METADATA payload: metadata_type (LEB128) + T.35 payload
    let mut obu_payload = encode_leb128(METADATA_TYPE_ITUT_T35);
    obu_payload.extend_from_slice(&t35_payload);

    // OBU header byte:
    //   bit 7:   forbidden = 0
    //   bits 6-3: obu_type = 5 (OBU_METADATA)
    //   bit 2:   obu_extension_flag = 0
    //   bit 1:   obu_has_size_field = 1
    //   bit 0:   reserved = 0
    // => (5 << 3) | 0x02 = 0x2A
    let header_byte = (OBU_METADATA << 3) | 0x02u8;
    let size_bytes = encode_leb128(obu_payload.len() as u64);

    let mut result = Vec::with_capacity(1 + size_bytes.len() + obu_payload.len());
    result.push(header_byte);
    result.extend_from_slice(&size_bytes);
    result.extend_from_slice(&obu_payload);

    Ok(result)
}

#[cfg(feature = "json")]
pub fn encode_av1_from_json(metadata: &Hdr10PlusJsonMetadata, validate: bool) -> Result<Vec<u8>> {
    let meta = Hdr10PlusMetadata::try_from(metadata)?;
    encode_hdr10plus_obu(&meta, validate)
}

/// Encode a `u64` value as LEB128 (unsigned).
pub fn encode_leb128(mut value: u64) -> Vec<u8> {
    let mut result = Vec::new();
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        result.push(byte);
        if value == 0 {
            break;
        }
    }
    result
}

/// Decode a LEB128-encoded value from `data`.
/// Returns `(value, bytes_consumed)`.
pub fn decode_leb128(data: &[u8]) -> (u64, usize) {
    let mut value = 0u64;
    let mut bytes_read = 0usize;
    for (i, &byte) in data.iter().enumerate() {
        if i >= 8 {
            break;
        }
        value |= ((byte & 0x7F) as u64) << (7 * i);
        bytes_read += 1;
        if byte & 0x80 == 0 {
            break;
        }
    }
    (value, bytes_read)
}
