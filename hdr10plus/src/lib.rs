pub mod metadata;

#[cfg(feature = "json")]
pub mod metadata_json;

#[cfg(feature = "hevc")]
pub mod hevc;

/// C API module
#[cfg(any(cargo_c, feature = "capi"))]
pub mod capi;

/// Structs used and exposed in the C API
#[cfg(any(cargo_c, feature = "capi"))]
pub mod c_structs;
