//! Small versioned C ABI used by the Flutter client.
//!
//! Returned strings are allocated by Rust and must be released with
//! `vripper_string_free`. The API never returns borrowed Rust memory.

use crate::api::parse_thread_response;
use crate::coordinator::{CoordinatorConfig, DownloadCoordinator};
use crate::queue::QueueMove;
use crate::hosts::{resolve_host_html, HostId};
use crate::scan_links;
use std::ffi::{CStr, CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{Mutex as AsyncMutex, broadcast};

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


type CoordinatorRegistry = Mutex<std::collections::HashMap<u64, Arc<DownloadCoordinator>>>;
type EventReceiver = Arc<AsyncMutex<broadcast::Receiver<crate::queue::QueueEvent>>>;
type EventRegistry = Mutex<std::collections::HashMap<u64, EventReceiver>>;

static NEXT_COORDINATOR_ID: AtomicU64 = AtomicU64::new(1);
static COORDINATORS: OnceLock<CoordinatorRegistry> = OnceLock::new();
static COORDINATOR_EVENTS: OnceLock<EventRegistry> = OnceLock::new();
static FFI_RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn ffi_runtime() -> &'static Runtime {
    FFI_RUNTIME.get_or_init(|| {
        Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("failed to initialize VRipper native runtime")
    })
}

fn coordinators() -> &'static Mutex<std::collections::HashMap<u64, Arc<DownloadCoordinator>>> {
    COORDINATORS.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

fn coordinator_events() -> &'static Mutex<std::collections::HashMap<u64, Arc<AsyncMutex<broadcast::Receiver<crate::queue::QueueEvent>>>>> {
    COORDINATOR_EVENTS.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

fn coordinator_for(handle: u64) -> Result<Arc<DownloadCoordinator>, String> {
    coordinators()
        .lock()
        .map_err(|_| "coordinator registry lock poisoned".to_owned())?
        .get(&handle)
        .cloned()
        .ok_or_else(|| format!("unknown coordinator handle: {handle}"))
}

/// Create a local/remote download coordinator from JSON configuration.
///
/// Secrets in the input are used only to initialize in-memory clients/session
/// cookies. The coordinator deliberately does not retain the config or passwords.
///
/// # Safety
/// config_json must point to a valid NUL-terminated UTF-8 JSON string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vripper_coordinator_create_json(
    config_json: *const c_char,
) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let config_json = match read_utf8(config_json, "config_json") {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };
        let config: CoordinatorConfig = match serde_json::from_str(&config_json) {
            Ok(value) => value,
            Err(error) => return json_error(&format!("invalid coordinator configuration: {error}")),
        };
        let coordinator = match ffi_runtime().block_on(DownloadCoordinator::new(config)) {
            Ok(value) => Arc::new(value),
            Err(error) => return json_error(&error.to_string()),
        };
        let handle = NEXT_COORDINATOR_ID.fetch_add(1, Ordering::Relaxed);
        let receiver = coordinator.subscribe();
        if let Ok(mut registry) = coordinators().lock() {
            registry.insert(handle, coordinator);
        } else {
            return json_error("coordinator registry lock poisoned");
        }
        if let Ok(mut registry) = coordinator_events().lock() {
            registry.insert(handle, Arc::new(AsyncMutex::new(receiver)));
        } else {
            return json_error("coordinator event registry lock poisoned");
        }
        serde_json::json!({ "handle": handle }).to_string()
    }));
    match result {
        Ok(value) => into_c_string(value),
        Err(_) => into_c_string(json_error("native coordinator panicked")),
    }
}

/// Scan links, perform site lookups and enqueue extracted images.
///
/// # Safety
/// input and configured_bases_json must be valid NUL-terminated UTF-8 strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vripper_coordinator_scan_and_queue_json(
    handle: u64,
    input: *const c_char,
    configured_bases_json: *const c_char,
) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let coordinator = match coordinator_for(handle) {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };
        let input = match read_utf8(input, "input") {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };
        let bases_pointer = match read_utf8(configured_bases_json, "configured_bases_json") {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };
        let bases: Vec<String> = match serde_json::from_str(&bases_pointer) {
            Ok(value) => value,
            Err(error) => return json_error(&format!("invalid configured base list: {error}")),
        };
        let summary = ffi_runtime().block_on(coordinator.scan_and_queue(&input, &bases));
        serde_json::to_string(&summary)
            .unwrap_or_else(|error| json_error(&format!("serialization failed: {error}")))
    }));
    match result {
        Ok(value) => into_c_string(value),
        Err(_) => into_c_string(json_error("native coordinator panicked")),
    }
}

/// Return the current queue snapshot as JSON.
#[unsafe(no_mangle)]
pub extern "C" fn vripper_coordinator_snapshot_json(handle: u64) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let coordinator = match coordinator_for(handle) {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };
        match ffi_runtime().block_on(coordinator.snapshot()) {
            Ok(snapshot) => serde_json::to_string(&snapshot)
                .unwrap_or_else(|error| json_error(&format!("serialization failed: {error}"))),
            Err(error) => json_error(&error.to_string()),
        }
    }));
    match result {
        Ok(value) => into_c_string(value),
        Err(_) => into_c_string(json_error("native coordinator panicked")),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vripper_coordinator_stop_json(
    handle: u64,
    job_id: u64,
) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let coordinator = match coordinator_for(handle) {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };
        match ffi_runtime().block_on(coordinator.stop(job_id)) {
            Ok(stopped) => serde_json::json!({ "stopped": stopped }).to_string(),
            Err(error) => json_error(&error.to_string()),
        }
    }));
    match result {
        Ok(value) => into_c_string(value),
        Err(_) => into_c_string(json_error("native coordinator panicked")),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vripper_coordinator_stop_all_json(handle: u64) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let coordinator = match coordinator_for(handle) {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };
        match ffi_runtime().block_on(coordinator.stop_all()) {
            Ok(()) => serde_json::json!({ "ok": true }).to_string(),
            Err(error) => json_error(&error.to_string()),
        }
    }));
    match result {
        Ok(value) => into_c_string(value),
        Err(_) => into_c_string(json_error("native coordinator panicked")),
    }
}

/// Move a pending queue job.
///
/// # Safety
/// movement must point to a valid NUL-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vripper_coordinator_move_json(
    handle: u64,
    job_id: u64,
    movement: *const c_char,
) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let coordinator = match coordinator_for(handle) {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };
        let movement = match read_utf8(movement, "movement") {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };
        let movement = match movement.as_str() {
            "top" => QueueMove::Top,
            "up" => QueueMove::Up,
            "down" => QueueMove::Down,
            "bottom" => QueueMove::Bottom,
            _ => return json_error("movement must be top, up, down or bottom"),
        };
        match ffi_runtime().block_on(coordinator.move_job(job_id, movement)) {
            Ok(moved) => serde_json::json!({ "moved": moved }).to_string(),
            Err(error) => json_error(&error.to_string()),
        }
    }));
    match result {
        Ok(value) => into_c_string(value),
        Err(_) => into_c_string(json_error("native coordinator panicked")),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vripper_coordinator_clear_finished_json(handle: u64) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let coordinator = match coordinator_for(handle) {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };
        match ffi_runtime().block_on(coordinator.clear_finished()) {
            Ok(()) => serde_json::json!({ "ok": true }).to_string(),
            Err(error) => json_error(&error.to_string()),
        }
    }));
    match result {
        Ok(value) => into_c_string(value),
        Err(_) => into_c_string(json_error("native coordinator panicked")),
    }
}

/// Poll one queue event. Returns {"timeout":true} if no event arrives in time.
///
/// # Safety
/// No pointers are accepted by this function.
#[unsafe(no_mangle)]
pub extern "C" fn vripper_coordinator_poll_event_json(
    handle: u64,
    timeout_ms: u64,
) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let receiver = match coordinator_events().lock() {
            Ok(registry) => registry.get(&handle).cloned(),
            Err(_) => return json_error("coordinator event registry lock poisoned"),
        };
        let Some(receiver) = receiver else {
            return json_error(&format!("unknown coordinator handle: {handle}"));
        };
        let event = ffi_runtime().block_on(async {
            let mut receiver = receiver.lock().await;
            tokio::time::timeout(Duration::from_millis(timeout_ms.min(30_000)), receiver.recv()).await
        });
        match event {
            Ok(Ok(event)) => serde_json::to_string(&event)
                .unwrap_or_else(|error| json_error(&format!("serialization failed: {error}"))),
            Ok(Err(broadcast::error::RecvError::Lagged(skipped))) => {
                serde_json::json!({ "error": "event receiver lagged", "skipped": skipped }).to_string()
            }
            Ok(Err(broadcast::error::RecvError::Closed)) => json_error("event stream closed"),
            Err(_) => serde_json::json!({ "timeout": true }).to_string(),
        }
    }));
    match result {
        Ok(value) => into_c_string(value),
        Err(_) => into_c_string(json_error("native coordinator panicked")),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vripper_coordinator_destroy_json(handle: u64) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let coordinator = coordinators()
            .lock()
            .ok()
            .and_then(|mut registry| registry.remove(&handle));
        if let Ok(mut registry) = coordinator_events().lock() {
            registry.remove(&handle);
        }
        let Some(coordinator) = coordinator else {
            return json_error(&format!("unknown coordinator handle: {handle}"));
        };
        match ffi_runtime().block_on(coordinator.shutdown()) {
            Ok(()) => serde_json::json!({ "ok": true }).to_string(),
            Err(error) => json_error(&error.to_string()),
        }
    }));
    match result {
        Ok(value) => into_c_string(value),
        Err(_) => into_c_string(json_error("native coordinator panicked")),
    }
}

/// Inspect a legacy VRipper SQLite database without modifying it.
///
/// # Safety
/// path_json must point to a valid NUL-terminated UTF-8 filesystem path.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vripper_database_inspect_json(
    path_json: *const c_char,
) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let path = match read_utf8(path_json, "database_path") {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };
        match crate::database::inspect_legacy_database(std::path::Path::new(&path)) {
            Ok(report) => serde_json::to_string(&report)
                .unwrap_or_else(|error| json_error(&format!("serialization failed: {error}"))),
            Err(error) => json_error(&error.to_string()),
        }
    }));
    match result {
        Ok(value) => into_c_string(value),
        Err(_) => into_c_string(json_error("database inspection panicked")),
    }
}

/// Import a legacy VRipper SQLite database into a new destination file.
///
/// The source is opened read-only. The destination must not already exist.
///
/// # Safety
/// Both pointers must reference valid NUL-terminated UTF-8 filesystem paths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vripper_database_import_json(
    source_json: *const c_char,
    destination_json: *const c_char,
) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let source = match read_utf8(source_json, "source_database_path") {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };
        let destination = match read_utf8(destination_json, "destination_database_path") {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };
        match crate::database::import_legacy_database(
            std::path::Path::new(&source),
            std::path::Path::new(&destination),
        ) {
            Ok(report) => serde_json::to_string(&report)
                .unwrap_or_else(|error| json_error(&format!("serialization failed: {error}"))),
            Err(error) => json_error(&error.to_string()),
        }
    }));
    match result {
        Ok(value) => into_c_string(value),
        Err(_) => into_c_string(json_error("database import panicked")),
    }
}

/// Read the imported Java catalog as JSON.
///
/// # Safety
/// path must be a valid NUL-terminated UTF-8 path.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vripper_database_catalog_json(path: *const c_char) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let path = match read_utf8(path, "database_path") {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };
        match crate::database::read_legacy_catalog(std::path::Path::new(&path)) {
            Ok(value) => value.to_string(),
            Err(error) => json_error(&error.to_string()),
        }
    }));
    match result {
        Ok(value) => into_c_string(value),
        Err(_) => into_c_string(json_error("database catalog panicked")),
    }
}

/// Load or transactionally replace a native catalog. Null data means read.
///
/// # Safety
/// path and non-null data must be valid NUL-terminated UTF-8 strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vripper_native_catalog_json(path: *const c_char, data: *const c_char) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let path = match read_utf8(path, "catalog_path") {
            Ok(value) => value,
            Err(error) => return json_error(&error),
        };
        let data = if data.is_null() { None } else {
            match read_utf8(data, "catalog_data") {
                Ok(value) => Some(value),
                Err(error) => return json_error(&error),
            }
        };
        match crate::database::native_catalog(std::path::Path::new(&path), data.as_deref()) {
            Ok(value) => serde_json::json!({"catalog": value}).to_string(),
            Err(error) => json_error(&error.to_string()),
        }
    }));
    match result {
        Ok(value) => into_c_string(value),
        Err(_) => into_c_string(json_error("native catalog panicked")),
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
    fn ffi_coordinator_lifecycle_scans_and_returns_queue_snapshot() {
        let temp = tempfile::tempdir().unwrap();
        let config = serde_json::json!({
            "network": {
                "mode": "disabled",
                "manual_proxy": null,
                "bypass_hosts": []
            },
            "proxy_credentials": null,
            "system_proxy": null,
            "viper_host": "https://vipergirls.to",
            "site_proxy": "https://vipergirls.to",
            "viper_credentials": null,
            "request_limit_per_second": 2,
            "max_attempts": 3,
            "connection_timeout_secs": 30,
            "queue_settings": {
                "max_global_concurrent": 0,
                "max_per_host_concurrent": 2
            },
            "download_path": temp.path().join("downloads").to_string_lossy(),
            "temp_path": temp.path().join("temp").to_string_lossy(),
            "forum_subdirectory": false,
            "thread_sublocation": false,
            "append_post_id": false,
            "force_order": false,
            "fetch_original_imx_filename": false
        });
        let config = CString::new(config.to_string()).unwrap();
        let created = unsafe { vripper_coordinator_create_json(config.as_ptr()) };
        let created_json = unsafe { CStr::from_ptr(created) }.to_str().unwrap();
        let created_value: serde_json::Value = serde_json::from_str(created_json).unwrap();
        let handle = created_value["handle"].as_u64().unwrap();
        unsafe { vripper_string_free(created) };

        let input = CString::new("not a valid link").unwrap();
        let bases = CString::new("[]").unwrap();
        let scanned = unsafe {
            vripper_coordinator_scan_and_queue_json(handle, input.as_ptr(), bases.as_ptr())
        };
        let scanned_json = unsafe { CStr::from_ptr(scanned) }.to_str().unwrap();
        let scanned_value: serde_json::Value = serde_json::from_str(scanned_json).unwrap();
        assert_eq!(scanned_value["invalid_links"].as_array().unwrap().len(), 1);
        unsafe { vripper_string_free(scanned) };

        let snapshot = vripper_coordinator_snapshot_json(handle);
        let snapshot_json = unsafe { CStr::from_ptr(snapshot) }.to_str().unwrap();
        let snapshot_value: serde_json::Value = serde_json::from_str(snapshot_json).unwrap();
        assert_eq!(snapshot_value["remaining"], 0);
        unsafe { vripper_string_free(snapshot) };

        let destroyed = vripper_coordinator_destroy_json(handle);
        let destroyed_json = unsafe { CStr::from_ptr(destroyed) }.to_str().unwrap();
        assert!(destroyed_json.contains("true"));
        unsafe { vripper_string_free(destroyed) };
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
