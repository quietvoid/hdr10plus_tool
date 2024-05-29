#![deny(missing_docs)]

use anyhow::anyhow;
use libc::{c_char, size_t};
use std::{
    ffi::{CStr, CString},
    path::PathBuf,
    ptr::{null, null_mut},
};

use crate::{
    metadata::{Hdr10PlusMetadata, Hdr10PlusMetadataEncOpts},
    metadata_json::MetadataJsonRoot,
};

use super::c_structs::*;

/// # Safety
/// The pointer to the data must be valid.
///
/// Parse a HDR10+ JSON file from file path.
/// Adds an error if the parsing fails.
#[no_mangle]
pub unsafe extern "C" fn hdr10plus_rs_parse_json(path: *const c_char) -> *mut JsonOpaque {
    if path.is_null() {
        return null_mut();
    }

    let mut opaque = JsonOpaque {
        metadata_root: None,
        error: None,
    };
    let mut error = None;

    if let Ok(str) = CStr::from_ptr(path).to_str() {
        let path = PathBuf::from(str);
        match MetadataJsonRoot::from_file(path) {
            Ok(metadata) => opaque.metadata_root = Some(metadata),
            Err(e) => {
                error = Some(format!(
                    "hdr10plus_rs_parse_json: Errored while parsing: {e}"
                ));
            }
        };
    } else {
        error =
            Some("hdr10plus_rs_parse_json: Failed parsing the input path as a string".to_string());
    }

    if let Some(err) = error {
        opaque.error = CString::new(err).ok();
    }

    Box::into_raw(Box::new(opaque))
}

/// # Safety
/// The pointer to the opaque struct must be valid.
///
/// Get the last logged error for the JsonOpaque operations.
///
/// On invalid parsing, an error is added.
/// The user should manually verify if there is an error, as the parsing does not return an error code.
#[no_mangle]
pub unsafe extern "C" fn hdr10plus_rs_json_get_error(ptr: *const JsonOpaque) -> *const c_char {
    if ptr.is_null() {
        return null();
    }

    let opaque = &*ptr;

    match &opaque.error {
        Some(s) => s.as_ptr(),
        None => null(),
    }
}

/// # Safety
/// The pointer to the opaque struct must be valid.
///
/// Free the Hdr10PlusJsonOpaque
#[no_mangle]
pub unsafe extern "C" fn hdr10plus_rs_json_free(ptr: *const JsonOpaque) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr));
    }
}

/// # Safety
/// The struct pointer must be valid.
///
/// Writes the encoded HDR10+ payload as a byte buffer, including country code
/// If an error occurs in the writing, returns null
#[no_mangle]
pub unsafe extern "C" fn hdr10plus_rs_write_av1_metadata_obu_t35_complete(
    ptr: *mut JsonOpaque,
    frame_number: size_t,
) -> *const Data {
    if ptr.is_null() {
        return null();
    }

    let opaque = &mut *ptr;
    let frame_metadata = opaque
        .metadata_root
        .as_ref()
        .and_then(|root| root.scene_info.get(frame_number))
        .ok_or(anyhow!("No metadata for frame {frame_number}"))
        .and_then(|jm| {
            let enc_opts = Hdr10PlusMetadataEncOpts {
                with_country_code: true,
                ..Default::default()
            };

            Hdr10PlusMetadata::try_from(jm)
                .and_then(|metadata| metadata.encode_with_opts(&enc_opts))
        });

    match frame_metadata {
        Ok(buf) => Box::into_raw(Box::new(Data::from(buf))),
        Err(e) => {
            opaque
                .error
                .replace(CString::new(format!("Failed writing byte buffer: {e}")).unwrap());

            null()
        }
    }
}

/// # Safety
/// The data pointer should exist, and be allocated by Rust.
///
/// Free a Data buffer
#[no_mangle]
pub unsafe extern "C" fn hdr10plus_rs_data_free(data: *const Data) {
    if !data.is_null() {
        let data = Box::from_raw(data as *mut Data);
        data.free();
    }
}
