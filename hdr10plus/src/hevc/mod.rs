use anyhow::{Result, ensure};
use bitvec_helpers::bitstream_io_writer::BitstreamIoWriter;
use hevc::{NAL_SEI_PREFIX, USER_DATA_REGISTERED_ITU_T_35};
use hevc_parser::{hevc, utils::add_start_code_emulation_prevention_3_byte};

use super::metadata::{Hdr10PlusMetadata, Hdr10PlusMetadataEncOpts};
use super::metadata_json::Hdr10PlusJsonMetadata;

pub fn encode_hdr10plus_nal(metadata: &Hdr10PlusMetadata, validate: bool) -> Result<Vec<u8>> {
    let opts = Hdr10PlusMetadataEncOpts {
        validate,
        ..Default::default()
    };

    // Write NALU SEI_PREFIX header
    let mut header_writer = BitstreamIoWriter::with_capacity(64);

    header_writer.write_bit(false)?; // forbidden_zero_bit

    header_writer.write::<6, u8>(NAL_SEI_PREFIX)?; // nal_type
    header_writer.write_const::<6, 0>()?; // nuh_layer_id
    header_writer.write_const::<3, 1>()?; // nuh_temporal_id_plus1

    header_writer.write::<8, u8>(USER_DATA_REGISTERED_ITU_T_35)?;

    let mut payload = metadata.encode_with_opts(&opts)?;

    // FIXME: This should probably be 1024 but not sure how to write a longer header
    ensure!(
        payload.len() <= 255,
        "Payload too large: {} bytes",
        payload.len()
    );

    header_writer.write::<8, u8>(payload.len() as u8)?;

    payload.push(0x80);

    let mut data = header_writer.into_inner();
    data.append(&mut payload);

    add_start_code_emulation_prevention_3_byte(&mut data);

    Ok(data)
}

pub fn encode_hevc_from_json(metadata: &Hdr10PlusJsonMetadata, validate: bool) -> Result<Vec<u8>> {
    let meta = Hdr10PlusMetadata::try_from(metadata)?;
    encode_hdr10plus_nal(&meta, validate)
}
