use std::ffi::{CStr, CString};

use crate::bindings::*;
use crate::error::*;
use crate::protobuf;

use prost::Message;

/// Scans (lexes) the given SQL statement into tokens.
///
/// # Example
///
/// ```rust
/// let result = pgls_query::scan("SELECT * FROM contacts");
/// assert!(result.is_ok());
/// ```
pub fn scan(sql: &str) -> Result<protobuf::ScanResult> {
    let input = CString::new(sql)?;
    let result = unsafe { pg_query_scan(input.as_ptr()) };
    let scan_result = if !result.error.is_null() {
        let message = unsafe { CStr::from_ptr((*result.error).message) }
            .to_string_lossy()
            .to_string();
        Err(Error::Scan(message))
    } else {
        let data = unsafe {
            std::slice::from_raw_parts(result.pbuf.data as *const u8, result.pbuf.len as usize)
        };
        protobuf::ScanResult::decode(data).map_err(Error::Decode)
    };
    unsafe { pg_query_free_scan_result(result) };
    scan_result
}
