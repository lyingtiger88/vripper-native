//! Small versioned C ABI used by the Flutter client.
//!
//! Returned strings are allocated by Rust and must be released with
//! `vripper_string_free`. The API never returns borrowed Rust memory.

use crate::api::parse_thread_response;
use crate::hosts::{resolve_host_html, HostId};
use crate::scan_links;
use std::ffi::{CStr, CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};

const ENGINE_VERSION: &str = "0.1.0";
const ABI_VERSION: u32 = 1;

fn into_c_string(value: String) -> *mut c_char {
    CString::new(value)
        .unwrap_or_else(|_| CString::new("{\"error\":\"interior NUL in result\"}").unwrap())
        .into_raw()
}


fn read_utf8(pointer: *const c_char, label: &str) -> Result<String, String> {
    if pointer.is_null() {
        return Err(format!("{label} pointer must not be null"));
    }
    let value = unsafe { CStr::from_ptr(pointer) };
    value
        .to_str()
        .map(str::to_owned)
        .map_err(|_| format!("{label} must be valid UTF-8"))
}

fn read_optional_utf8(pointer: *const c_char, label: &str) -> Result<Option<String>, String> {
    if pointer.is_null() {
        return Ok(None);
    }
    read_utf8(pointer, label).map(Some)
}

fn json_error(message: &str) -> String {
    serde_json::json!({ "error": message }).to_string()
}

/// Returns the engine semantic version as an owned C string.
#[unsafe(no_mangle)]
pub extern "C" fn vripper_engine_version() -> *mut c_char {
    into_c_string(ENGINE_VERSION.to_owned())
}

/// Returns the native FFI ABI version. Flutter must reject unknown major versions.
#[unsafe(no_mangle)]
pub extern "C" fn vripper_abi_version() -> u32 {
    ABI_VERSION
}

/// Scan multiline links using configured base URLs.
///
/// # Safety
/// Both pointers must be null or point to valid NUL-terminated C strings for the duration of the call.
///
/// `input` is UTF-8 text. `configured_bases_json` is a JSON array of strings.
/// The returned JSON contains `threads`, `posts` and `invalid` arrays.
/// The caller owns the returned pointer and must call `vripper_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vripper_scan_links_json(
    input: *const c_char,
    configured_bases_json: *const c_char,
) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| {
        if input.is_null() || configured_bases_json.is_null() {
            return json_error("input pointers must not be null");
        }

        let input = unsafe { CStr::from_ptr(input) };
        let bases = unsafe { CStr::from_ptr(configured_bases_json) };

        let input = match input.to_str() {
            Ok(value) => value,
            Err(_) => return json_error("input must be valid UTF-8"),
        };
        let bases: Vec<String> = match serde_json::from_slice(bases.to_bytes()) {
            Ok(value) => value,
            Err(error) => return json_error(&format!("invalid base URL JSON: {error}")),
        };

        serde_json::to_string(&scan_links(input, &bases))
            .unwrap_or_else(|error| json_error(&format!("serialization failed: {error}")))
    }));

    match result {
        Ok(value) => into_c_string(value),
        Err(_) => into_c_string(json_error("native engine panicked")),
    }
}


/// Parse the reference thread lookup XML response and return a JSON model.
///
/// # Safety
/// Both arguments must point to valid NUL-terminated UTF-8 strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vripper_parse_thread_response_json(
    xml: *const c_char,
    site_proxy: *const c_char,
) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let xml = match read_utf8(xml, "xml") {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };
        let site_proxy = match read_utf8(site_proxy, "site_proxy") {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };
        match parse_thread_response(&xml, &site_proxy) {
            Ok(parsed) => serde_json::to_string(&parsed)
                .unwrap_or_else(|error| json_error(&format!("serialization failed: {error}"))),
            Err(error) => json_error(&error),
        }
    }));
    match result {
        Ok(value) => into_c_string(value),
        Err(_) => into_c_string(json_error("native engine panicked")),
    }
}

/// Resolve a captured host HTML page without performing network requests.
///
/// A null follow_up_html means the caller should execute the returned follow-up
/// request using the central transport, then call this function again with the
/// resulting HTML.
///
/// # Safety
/// Every non-null pointer must reference a valid NUL-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vripper_resolve_host_html_json(
    host_id: u8,
    source_url: *const c_char,
    thumbnail_url: *const c_char,
    html: *const c_char,
    follow_up_html: *const c_char,
    fetch_original_imx_filename: u8,
) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let host = match HostId::try_from(host_id) {
            Ok(host) => host,
            Err(error) => return json_error(&error),
        };
        let source_url = match read_utf8(source_url, "source_url") {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };
        let thumbnail_url = match read_utf8(thumbnail_url, "thumbnail_url") {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };
        let html = match read_utf8(html, "html") {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };
        let follow_up_html = match read_optional_utf8(follow_up_html, "follow_up_html") {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };

        match resolve_host_html(
            host,
            &source_url,
            &thumbnail_url,
            &html,
            follow_up_html.as_deref(),
            fetch_original_imx_filename != 0,
        ) {
            Ok(resolution) => serde_json::to_string(&resolution)
                .unwrap_or_else(|error| json_error(&format!("serialization failed: {error}"))),
            Err(error) => json_error(&error),
        }
    }));
    match result {
        Ok(value) => into_c_string(value),
        Err(_) => into_c_string(json_error("native engine panicked")),
    }
}

/// Release a string returned by this library. Passing null is a no-op.
///
/// # Safety
/// The pointer must be null or a pointer previously returned by this library that has not already been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vripper_string_free(value: *mut c_char) {
    if !value.is_null() {
        unsafe {
            drop(CString::from_raw(value));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_abi_version() {
        assert_eq!(vripper_abi_version(), 1);
    }

    #[test]
    fn ffi_scan_returns_owned_json_string() {
        let input = CString::new("https://vipergirls.to/threads/123").unwrap();
        let bases = CString::new(r#"["https://vipergirls.to"]"#).unwrap();
        let output = unsafe { vripper_scan_links_json(input.as_ptr(), bases.as_ptr()) };
        assert!(!output.is_null());

        let json = unsafe { CStr::from_ptr(output) }.to_str().unwrap();
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(value["threads"][0]["thread_id"], 123);
        unsafe { vripper_string_free(output) };
    }


    #[test]
    fn ffi_parses_thread_xml_to_json() {
        let xml = CString::new(
            r#"<thread id="9" title="T"><post id="10" number="1" title="P"><image type="linked" main_url="https://imgbox.com/a" thumb_url="https://imgbox.com/t"/></post></thread>"#,
        )
        .unwrap();
        let proxy = CString::new("https://vipergirls.to").unwrap();
        let output = unsafe { vripper_parse_thread_response_json(xml.as_ptr(), proxy.as_ptr()) };
        let json = unsafe { CStr::from_ptr(output) }.to_str().unwrap();
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(value["posts"][0]["post_id"], 10);
        unsafe { vripper_string_free(output) };
    }

    #[test]
    fn ffi_resolves_host_html_to_json() {
        let source = CString::new("https://dpic.me/view/a").unwrap();
        let thumb = CString::new("").unwrap();
        let html = CString::new(r#"<html><img id="pic" alt="sample" src="https://dpic.me/images/a.jpg"></html>"#).unwrap();
        let output = unsafe {
            vripper_resolve_host_html_json(
                1,
                source.as_ptr(),
                thumb.as_ptr(),
                html.as_ptr(),
                std::ptr::null(),
                0,
            )
        };
        let json = unsafe { CStr::from_ptr(output) }.to_str().unwrap();
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(value["Resolved"]["name"], "sample");
        unsafe { vripper_string_free(output) };
    }

    #[test]
    fn ffi_rejects_null_input_without_panicking() {
        let bases = CString::new("[]").unwrap();
        let output = unsafe { vripper_scan_links_json(std::ptr::null(), bases.as_ptr()) };
        let json = unsafe { CStr::from_ptr(output) }.to_str().unwrap();
        assert!(json.contains("must not be null"));
        unsafe { vripper_string_free(output) };
    }
}
