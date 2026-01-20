//! C ABI wrappers for Emscripten/WASM.
//!
//! This module provides C-compatible functions that can be called from JavaScript
//! via Emscripten's runtime.
//!
//! # Memory Management
//!
//! Strings returned by these functions are allocated on the Rust heap and must be
//! freed by calling `pgls_free_string`. The caller is responsible for freeing
//! all returned strings.
//!
//! # Thread Safety
//!
//! The workspace is stored in a global static and is NOT thread-safe.
//! This is acceptable for WASM which is single-threaded.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;

use crate::{WasmError, Workspace};

/// Global workspace instance.
/// WASM is single-threaded, but we use Mutex for safe initialization.
static WORKSPACE: Mutex<Option<Workspace>> = Mutex::new(None);

/// Helper to get or create the workspace.
fn with_workspace<F, T>(f: F) -> T
where
    F: FnOnce(&mut Workspace) -> T,
{
    let mut guard = WORKSPACE.lock().unwrap();
    if guard.is_none() {
        *guard = Some(Workspace::new());
    }
    f(guard.as_mut().unwrap())
}

/// Convert a C string to a Rust string slice.
/// Returns None if the pointer is null or the string is not valid UTF-8.
///
/// # Safety
/// The pointer must be valid and point to a null-terminated string.
unsafe fn c_str_to_str<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: Caller guarantees ptr is valid and null-terminated
    unsafe { CStr::from_ptr(ptr).to_str().ok() }
}

/// Convert a Rust string to a C string.
/// Returns a pointer to a null-terminated string allocated on the heap.
/// The caller must free this with `pgls_free_string`.
fn str_to_c_string(s: &str) -> *mut c_char {
    match CString::new(s) {
        Ok(cstr) => cstr.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Convert a Result<String, WasmError> to a C string.
/// On error, returns the error message.
fn result_to_c_string(result: Result<String, WasmError>) -> *mut c_char {
    match result {
        Ok(s) => str_to_c_string(&s),
        Err(e) => str_to_c_string(&format!("ERROR: {e}")),
    }
}

// ============================================================================
// Exported C Functions
// ============================================================================

/// Initialize the workspace. Call this before using other functions.
/// Returns 0 on success, non-zero on failure.
#[unsafe(no_mangle)]
pub extern "C" fn pgls_init() -> i32 {
    with_workspace(|_| 0)
}

/// Free a string that was returned by any pgls_* function.
///
/// # Safety
/// The pointer must have been returned by a pgls_* function and not already freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pgls_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        // SAFETY: Caller guarantees ptr was allocated by this library
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}

/// Set the database schema from a JSON string.
/// Returns NULL on success, or an error message on failure.
/// The returned string (if not NULL) must be freed with `pgls_free_string`.
///
/// # Safety
/// The json pointer must be valid and point to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pgls_set_schema(json: *const c_char) -> *mut c_char {
    // SAFETY: Caller guarantees json is valid
    let json_str = match unsafe { c_str_to_str(json) } {
        Some(s) => s,
        None => return str_to_c_string("ERROR: Invalid JSON string pointer"),
    };

    with_workspace(|ws| match ws.set_schema(json_str) {
        Ok(()) => std::ptr::null_mut(),
        Err(e) => str_to_c_string(&format!("ERROR: {e}")),
    })
}

/// Clear the current schema.
#[unsafe(no_mangle)]
pub extern "C" fn pgls_clear_schema() {
    with_workspace(|ws| ws.clear_schema());
}

/// Insert or update a file in the workspace.
/// Returns NULL on success, or an error message on failure.
///
/// # Safety
/// Both path and content pointers must be valid and point to null-terminated UTF-8 strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pgls_insert_file(
    path: *const c_char,
    content: *const c_char,
) -> *mut c_char {
    // SAFETY: Caller guarantees path is valid
    let path_str = match unsafe { c_str_to_str(path) } {
        Some(s) => s,
        None => return str_to_c_string("ERROR: Invalid path pointer"),
    };
    // SAFETY: Caller guarantees content is valid
    let content_str = match unsafe { c_str_to_str(content) } {
        Some(s) => s,
        None => return str_to_c_string("ERROR: Invalid content pointer"),
    };

    with_workspace(|ws| {
        ws.insert_file(path_str, content_str);
        std::ptr::null_mut()
    })
}

/// Remove a file from the workspace.
///
/// # Safety
/// The path pointer must be valid and point to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pgls_remove_file(path: *const c_char) {
    // SAFETY: Caller guarantees path is valid
    if let Some(path_str) = unsafe { c_str_to_str(path) } {
        with_workspace(|ws| ws.remove_file(path_str));
    }
}

/// Lint a file and return diagnostics as JSON.
/// Returns a JSON array of diagnostics, or an error message starting with "ERROR:".
/// The returned string must be freed with `pgls_free_string`.
///
/// # Safety
/// The path pointer must be valid and point to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pgls_lint(path: *const c_char) -> *mut c_char {
    // SAFETY: Caller guarantees path is valid
    let path_str = match unsafe { c_str_to_str(path) } {
        Some(s) => s,
        None => return str_to_c_string("ERROR: Invalid path pointer"),
    };

    with_workspace(|ws| {
        let result = ws.lint(path_str).map(|diagnostics| {
            serde_json::to_string(&diagnostics).unwrap_or_else(|e| format!("ERROR: {e}"))
        });
        result_to_c_string(result)
    })
}

/// Get completions at a position in a file.
/// Returns a JSON array of completion items, or an error message starting with "ERROR:".
/// The returned string must be freed with `pgls_free_string`.
///
/// # Safety
/// The path pointer must be valid and point to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pgls_complete(path: *const c_char, offset: u32) -> *mut c_char {
    // SAFETY: Caller guarantees path is valid
    let path_str = match unsafe { c_str_to_str(path) } {
        Some(s) => s,
        None => return str_to_c_string("ERROR: Invalid path pointer"),
    };

    with_workspace(|ws| {
        let result = ws.complete(path_str, offset).map(|completions| {
            serde_json::to_string(&completions).unwrap_or_else(|e| format!("ERROR: {e}"))
        });
        result_to_c_string(result)
    })
}

/// Get hover information at a position in a file.
/// Returns the hover text (markdown), or NULL if no hover info is available,
/// or an error message starting with "ERROR:".
/// The returned string must be freed with `pgls_free_string`.
///
/// # Safety
/// The path pointer must be valid and point to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pgls_hover(path: *const c_char, offset: u32) -> *mut c_char {
    // SAFETY: Caller guarantees path is valid
    let path_str = match unsafe { c_str_to_str(path) } {
        Some(s) => s,
        None => return str_to_c_string("ERROR: Invalid path pointer"),
    };

    with_workspace(|ws| match ws.hover(path_str, offset) {
        Ok(Some(text)) => str_to_c_string(&text),
        Ok(None) => std::ptr::null_mut(),
        Err(e) => str_to_c_string(&format!("ERROR: {e}")),
    })
}

/// Parse SQL and return any parse errors.
/// Returns a JSON array of error messages, or an empty array if parsing succeeded.
/// The returned string must be freed with `pgls_free_string`.
///
/// # Safety
/// The sql pointer must be valid and point to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pgls_parse(sql: *const c_char) -> *mut c_char {
    // SAFETY: Caller guarantees sql is valid
    let sql_str = match unsafe { c_str_to_str(sql) } {
        Some(s) => s,
        None => return str_to_c_string("ERROR: Invalid SQL pointer"),
    };

    with_workspace(|ws| {
        let errors = ws.parse(sql_str);
        let json = serde_json::to_string(&errors).unwrap_or_else(|e| format!("ERROR: {e}"));
        str_to_c_string(&json)
    })
}

/// Get the version of the library.
/// The returned string must be freed with `pgls_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn pgls_version() -> *mut c_char {
    str_to_c_string(env!("CARGO_PKG_VERSION"))
}

// ============================================================================
// LSP Message Handler
// ============================================================================

use crate::lsp::LspHandler;

/// Global LSP handler instance.
/// Separate from the workspace to allow independent state management.
static LSP_HANDLER: Mutex<Option<LspHandler>> = Mutex::new(None);

/// Handle an LSP JSON-RPC message.
///
/// This function processes an LSP message and returns a JSON array of outgoing
/// messages (response + notifications like publishDiagnostics).
///
/// The returned string must be freed with `pgls_free_string`.
///
/// # Usage with Web Workers
///
/// The web worker should iterate over the returned array and send each message
/// separately via `postMessage`, as `BrowserMessageReader` expects individual messages.
///
/// # Safety
/// The message pointer must be valid and point to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pgls_handle_message(message: *const c_char) -> *mut c_char {
    // SAFETY: Caller guarantees message is valid
    let msg = match unsafe { c_str_to_str(message) } {
        Some(s) => s,
        None => return str_to_c_string("[]"),
    };

    let mut guard = LSP_HANDLER.lock().unwrap();
    if guard.is_none() {
        *guard = Some(LspHandler::new());
    }

    let response = guard.as_mut().unwrap().handle_message(msg);
    str_to_c_string(&response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_init() {
        assert_eq!(pgls_init(), 0);
    }

    #[test]
    fn test_ffi_version() {
        let version = pgls_version();
        assert!(!version.is_null());
        unsafe {
            let version_str = CStr::from_ptr(version).to_str().unwrap();
            assert_eq!(version_str, "0.0.0");
            pgls_free_string(version);
        }
    }

    #[test]
    fn test_ffi_insert_and_lint() {
        pgls_init();

        let path = CString::new("/test.sql").unwrap();
        let content = CString::new("SELECT 1;").unwrap();

        unsafe {
            let result = pgls_insert_file(path.as_ptr(), content.as_ptr());
            assert!(result.is_null()); // Success

            let diagnostics = pgls_lint(path.as_ptr());
            assert!(!diagnostics.is_null());
            let diagnostics_str = CStr::from_ptr(diagnostics).to_str().unwrap();
            assert!(diagnostics_str.starts_with('['));
            pgls_free_string(diagnostics);
        }
    }

    #[test]
    fn test_ffi_parse() {
        pgls_init();

        unsafe {
            // Valid SQL
            let sql = CString::new("SELECT 1;").unwrap();
            let result = pgls_parse(sql.as_ptr());
            assert!(!result.is_null());
            let result_str = CStr::from_ptr(result).to_str().unwrap();
            assert_eq!(result_str, "[]"); // No errors
            pgls_free_string(result);

            // Invalid SQL
            let sql = CString::new("SELEC 1;").unwrap();
            let result = pgls_parse(sql.as_ptr());
            assert!(!result.is_null());
            let result_str = CStr::from_ptr(result).to_str().unwrap();
            assert!(result_str.contains("error") || result_str.len() > 2); // Has errors
            pgls_free_string(result);
        }
    }
}
