use std::ffi::CString;

use libc::size_t;

use crate::metadata_json::MetadataJsonRoot;

/// Opaque HDR10+ JSON file handle
///
/// Use `hdr10plus_rs_json_free` to free.
/// It should be freed regardless of whether or not an error occurred.
pub struct JsonOpaque {
    /// Optional parsed JSON, present when parsing is successful.
    pub metadata_root: Option<MetadataJsonRoot>,

    pub error: Option<CString>,
}

/// Struct representing a data buffer
#[repr(C)]
pub struct Data {
    /// Pointer to the data buffer
    pub data: *const u8,
    /// Data buffer size
    pub len: size_t,
}

impl Data {
    /// # Safety
    /// The pointers should all be valid.
    pub unsafe fn free(&self) {
        unsafe {
            Vec::from_raw_parts(self.data as *mut u8, self.len, self.len);
        }
    }
}

impl From<Vec<u8>> for Data {
    fn from(buf: Vec<u8>) -> Self {
        Data {
            len: buf.len(),
            data: Box::into_raw(buf.into_boxed_slice()) as *const u8,
        }
    }
}
