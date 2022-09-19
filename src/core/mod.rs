use std::{fs::File, path::Path};

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use thiserror::Error;

use bitvec_helpers::bitslice_reader::BitSliceReader;
use hevc_parser::hevc::{SeiMessage, USER_DATA_REGISTERED_ITU_T_35};
use hevc_parser::io::IoFormat;
use hevc_parser::utils::clear_start_code_emulation_prevention_3_byte;

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

                let mut reader = BitSliceReader::new(&sei_payload[start..end]);

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
