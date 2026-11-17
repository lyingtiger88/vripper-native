# VRipper Native

Native-first, 64-bit cross-platform evolution of VRipper using **Rust** for the application engine and **Flutter** for the user interface.

## Non-negotiable compatibility requirement

This project is a technology migration, not a product redesign. The reference application's user-visible behavior must be preserved:

- UI layout, navigation, controls, dialogs, and interaction flows.
- Site navigation and link parsing behavior.
- Thread/post discovery and image extraction behavior.
- Supported host resolution and download behavior, subject to the source site's access rules and applicable terms.
- Output directory layout, folder naming, filename normalization, ordering prefixes, extensions, overwrite behavior, and temporary-file handling.

No behavior is to be intentionally changed without an explicit compatibility decision documented in this repository.

## Target platforms

- Windows x86-64
- Linux x86-64
- macOS x86-64 and ARM64
- Android ARM64
- iOS ARM64

The shipped applications must not require a separately installed Java Runtime or JVM. Java/JDK may still be needed by parts of the Android/iOS build toolchain; that is a build-time dependency, not a runtime dependency of the delivered app.

## Technology direction

- **Flutter**: cross-platform UI, preserving the reference UI structure and interaction behavior.
- **Rust**: native engine for parsing, host resolution, scheduling, downloads, file output, persistence, and platform services.
- **FFI boundary**: a versioned, typed interface between Flutter and Rust. Keep platform-specific code behind adapters.

## Repository documents

- [Roadmap](docs/ROADMAP.md)
- [Reverse-engineering baseline](docs/REVERSE_ENGINEERING.md)
- [Compatibility specification](docs/COMPATIBILITY_SPEC.md)
- [Architecture decision record](docs/ADR-0001-rust-flutter.md)

## Reference project

The compatibility reference is [dev-claw/vripper-project](https://github.com/dev-claw/vripper-project). The reference repository is licensed under GPL-3.0. Before redistributing a derivative, preserve required copyright/license notices and comply with GPL-3.0 source and distribution obligations. Audit third-party assets and dependencies separately.

## Project status

**Phase 0 — reference audit and behavioral specification.** No production engine or UI has been migrated yet. The first milestone is a traceable, testable compatibility specification before implementation.
