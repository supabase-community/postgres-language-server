use std::ffi::{CStr, CString};

use crate::bindings::*;
use crate::error::*;
use crate::protobuf;

use pgls_text_size::{TextRange, TextSize};
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
        let data =
            unsafe { std::slice::from_raw_parts(result.pbuf.data as *const u8, result.pbuf.len) };
        protobuf::ScanResult::decode(data).map_err(Error::Decode)
    };
    unsafe { pg_query_free_scan_result(result) };
    scan_result
}

impl protobuf::ScanToken {
    /// The range of the token in the scanned text. `None` if the scanner reported a negative or
    /// reversed range, which it doesn't for text it accepted.
    pub fn range(&self) -> Option<TextRange> {
        let start = TextSize::from(u32::try_from(self.start).ok()?);
        let end = TextSize::from(u32::try_from(self.end).ok()?);
        (start <= end).then(|| TextRange::new(start, end))
    }

    /// The text of the token in `sql`, the text that was scanned. `None` if the range of the
    /// token isn't in it.
    pub fn text<'a>(&self, sql: &'a str) -> Option<&'a str> {
        sql.get(std::ops::Range::<usize>::from(self.range()?))
    }
}

#[cfg(test)]
mod tests {
    use pgls_text_size::TextRange;

    use crate::protobuf::ScanToken;

    #[test]
    fn token_ranges_are_checked() {
        let sql = "select abc";
        let scanned = crate::scan(sql).unwrap();
        let abc = &scanned.tokens[1];
        assert_eq!(abc.range(), Some(TextRange::new(7.into(), 10.into())));
        assert_eq!(abc.text(sql), Some("abc"));
        assert_eq!(abc.text("select"), None);

        let token = |start, end| ScanToken {
            start,
            end,
            ..Default::default()
        };
        assert_eq!(token(-1, 3).range(), None);
        assert_eq!(token(3, 2).range(), None);
    }
}
