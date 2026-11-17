# Native Engine ABI — Version 1

Header: `engine/vripper-core/include/vripper_native.h`  
Library crate: `engine/vripper-core`  
FFI implementation: `engine/vripper-core/src/ffi.rs`

## ABI contract

- ABI version: `1`.
- UTF-8 strings are NUL-terminated C strings.
- Every returned string is Rust-allocated and must be released exactly once with `vripper_string_free`.
- Never free returned pointers with Dart's allocator.
- Pointer-taking functions require valid NUL-terminated pointers; null is accepted only for documented optional values.
- Functions catch Rust panics and return a JSON error object where applicable.
- No borrowed Rust memory crosses the ABI.

## Exported functions

| Function | Purpose |
|---|---|
| `vripper_abi_version` | Return ABI version |
| `vripper_engine_version` | Return semantic engine version |
| `vripper_scan_links_json` | Classify multiline thread/post URLs |
| `vripper_parse_thread_response_json` | Parse reference XML response into a thread/post/image JSON model |
| `vripper_resolve_host_html_json` | Resolve a captured HTML page for a registered host; may return a follow-up request |
| `vripper_string_free` | Release Rust-owned output string |

## Host IDs

Stable reference numeric IDs:
0 Acidimg, 1 DPicMe, 2 ImageBam, 3 ImageTwist, 4 ImageVenue, 5 ImageZilla, 6 Imgbox, 7 ImgSpice, 8 Imx, 9 Pimpandhost, 10 Pixhost, 11 PixRoute, 12 Pixxxels, 13 PostImg, 14 TurboImageHost, 15 ViprIm.

Do not reorder or reuse IDs. Add new IDs only with a deliberate ABI/versioning decision.

## Host resolution flow

1. Flutter or a future engine transport obtains the initial HTML through the centralized HTTP client.
2. Call `vripper_resolve_host_html_json` with `follow_up_html = NULL`.
3. If the result is a `FollowUp` variant, execute that request through the central transport, applying its method, URL, form fields, Referer and cookie mutations.
4. Call the same function again with the captured follow-up HTML.
5. A `Resolved` result contains the final image name and URL plus any resolver-specific Referer/cookie state.
6. The function itself performs no network access.

This separation ensures host resolvers cannot silently bypass proxy configuration or construct independent HTTP clients.

## Flutter binding

`apps/client/lib/native_engine.dart` loads:
- Windows: `vripper_core.dll`
- Linux/Android: `libvripper_core.so`
- macOS: `libvripper_core.dylib`
- iOS: symbols from `DynamicLibrary.process()`

The current Dart wrapper binds all ABI v1 parsing/resolver functions. Packaging the Rust library into each Flutter platform target remains pending.

## Test status

- Rust tests cover URL scan, XML parsing, host resolution and C ABI ownership.
- The ABI is not yet integrated into a packaged desktop/mobile application.
- Any change to exported signatures requires updating the C header, Rust exports, Dart bindings and ABI tests together.
