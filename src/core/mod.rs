use std::{fs::File, path::Path};

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use thiserror::Error;

use bitvec_helpers::bitstream_io_reader::BsIoSliceReader;
use hevc_parser::hevc::{NALUnit, SeiMessage, NAL_SEI_PREFIX, USER_DATA_REGISTERED_ITU_T_35};
use hevc_parser::io::IoFormat;
use hevc_parser::utils::{
    add_start_code_emulation_prevention_3_byte, clear_start_code_emulation_prevention_3_byte,
};

pub mod parser;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("File doesn't contain dynamic metadata")]
    NoMetadataFound,
    #[error("Dynamic HDR10+ metadata detected.")]
    MetadataDetected,
}

pub fn initialize_progress_bar(format: &IoFormat, input: &Path) -> Result<ProgressBar> {
    let pb: ProgressBar;
    let bytes_count;

    if let IoFormat::RawStdin = format {
        pb = ProgressBar::hidden();
    } else {
        let file = File::open(input).expect("No file found");

        //Info for indicatif ProgressBar
        let file_meta = file.metadata()?;
        bytes_count = file_meta.len() / 100_000_000;

        pb = ProgressBar::new(bytes_count);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:60.cyan} {percent}%")?,
        );
    }

    Ok(pb)
}

pub fn st2094_40_sei_msg(sei_payload: &[u8], validate: bool) -> Result<Option<SeiMessage>> {
    let sei_payload = clear_start_code_emulation_prevention_3_byte(sei_payload);

    let res = if sei_payload.len() >= 4 {
        let sei = SeiMessage::parse_sei_rbsp(&sei_payload)?;

        sei.into_iter().find(|msg| {
            if msg.payload_type == USER_DATA_REGISTERED_ITU_T_35 && msg.payload_size >= 7 {
                let start = msg.payload_offset;
                let end = start + msg.payload_size;

                let bytes = &sei_payload[start..end];
                let mut reader = BsIoSliceReader::from_slice(bytes);

                let itu_t_t35_country_code = reader.get_n::<u8>(8).unwrap();
                let itu_t_t35_terminal_provider_code = reader.get_n::<u16>(16).unwrap();
                let itu_t_t35_terminal_provider_oriented_code = reader.get_n::<u16>(16).unwrap();

                if itu_t_t35_country_code == 0xB5
                    && itu_t_t35_terminal_provider_code == 0x003C
                    && itu_t_t35_terminal_provider_oriented_code == 0x0001
                {
                    let application_identifier = reader.get_n::<u8>(8).unwrap();
                    let application_version = reader.get_n::<u8>(8).unwrap();

                    let valid_version = if validate {
                        application_version == 1
                    } else {
                        application_version <= 1
                    };

                    if application_identifier == 4 && valid_version {
                        return true;
                    }
                }
            }

            false
        })
    } else {
        None
    };

    Ok(res)
}

// Returns Some when the SEI needs to be written
// Otherwise, the NALU only contains one SEI message, and can be dropped
pub fn prefix_sei_removed_hdr10plus_nalu(
    chunk: &[u8],
    nal: &NALUnit,
) -> Result<(bool, Option<Vec<u8>>)> {
    let (st2094_40_msg, payload) = if nal.nal_type == NAL_SEI_PREFIX {
        let sei_payload = clear_start_code_emulation_prevention_3_byte(&chunk[nal.start..nal.end]);
        let msg = st2094_40_sei_msg(&sei_payload, false)?;

        (msg, Some(sei_payload))
    } else {
        (None, None)
    };

    let has_st2094_40 = st2094_40_msg.is_some();

    if let (Some(msg), Some(mut payload)) = (st2094_40_msg, payload) {
        let messages = SeiMessage::parse_sei_rbsp(&payload)?;

        // Only remove ST2094-40 message if there are others
        if messages.len() > 1 {
            let start = msg.msg_offset;
            let end = msg.payload_offset + msg.payload_size;

            payload.drain(start..end);
            add_start_code_emulation_prevention_3_byte(&mut payload);

            return Ok((true, Some(payload)));
        }
    }

    Ok((has_st2094_40, None))
}
