# Flutter Client

Flutter desktop/mobile client for the VRipper Native migration.

## Implemented vertical slice
- Desktop-style shell with File/View/Help menu strip and Posts, Threads, Tasks and Logs tabs.
- Add Links dialog connected to the native Rust coordinator.
- Native coordinator lifecycle, asynchronous event polling and queue snapshots through ABI v1.
- Queue task list with per-task stop, pending-item reorder, stop-all and clear-finished actions.
- Settings dialog with the reference sections, a functional Network section and Viper account settings.
- Network preferences persisted with SharedPreferences.
- Proxy and Viper credentials stored through the platform secure-storage plugin; secrets are not written to the preferences JSON.
- Native image download pipeline writes temporary files and finalizes output using the Rust engine path/name rules.
- Windows x86-64 and Android ARM64 smoke-build workflow generates Flutter platform runners and bundles the matching Rust library.

## Current limitations
This is still a **pre-alpha test candidate**, not a parity-complete release:
- Live-site authentication and extraction have not yet been validated with a real account and authorized test links.
- The 16 host resolver paths need fixture-by-fixture and runtime compatibility verification.
- Native OS system-proxy discovery is not complete; System mode currently reads proxy environment variables on desktop and fails closed when no endpoint is available.
- Proxy connection testing is not yet connected to the native transport.
- Several legacy settings sections, Posts/Threads/Logs data views, remote session transport and the image workspace remain incomplete.
- SQLite persistence, import/migration, task history and cache parity remain outstanding.
- Reference screenshot and interaction comparison is still required.
- Platform runner directories are generated in CI for the current smoke-build workflow; they are not yet committed as checked-in platform projects.

## Build and checks
From this directory:
```sh
flutter pub get
flutter analyze
flutter test
```

From the repository root:
```sh
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

Windows x86-64 and Android ARM64 smoke builds run through [Native platform builds](../../.github/workflows/native-platform-builds.yml). Successful runs publish downloadable CI artifacts. A successful smoke build verifies packaging and native library loading at build time; it does not replace live-site acceptance testing.
