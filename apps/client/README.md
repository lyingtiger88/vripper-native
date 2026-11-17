# Flutter Client

This is the first Flutter UI shell for the VRipper Native migration.

## Current contents
- Desktop-style application shell with File/View/Help menu strip.
- Posts, Threads, Tasks and Logs navigation tabs.
- Add Links dialog connected to the Rust URL scanner when the native library is loaded.
- Settings dialog with the original nine sections plus the requested Network section.
- Local/Remote session selection prototype.
- Dart FFI bindings for ABI v1 URL scanning, XML parsing and host HTML resolution.

## Important limitations

This is an **engineering prototype**, not a user-test release:
- Download queue and live site requests are not wired.
- Posts/Threads/Tasks/Logs currently show empty-state placeholders.
- Most legacy settings are represented by disabled placeholder controls; they are not persisted.
- Network proxy controls are UI-only. The Rust proxy client builder exists, but it is not yet used by all request paths and OS secure credential storage is not connected.
- Local/Remote session controls do not yet establish a session.
- Rust native libraries are not yet bundled into Flutter platform projects.
- Pixel-perfect UI comparison still needs screenshots and interaction recordings from a running reference application.

## CI checks

From this directory:
```sh
flutter pub get
flutter analyze
flutter test
```

The Rust engine is in `../../engine/vripper-core`; run `cargo test --workspace --all-targets` from the repository root.
