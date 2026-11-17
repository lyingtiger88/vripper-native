# VRipper Native

Native-first, 64-bit cross-platform evolution of VRipper using **Rust** for the application engine and **Flutter** for the user interface.

> **Current status:** Phase 0 — reference audit and behavioral specification.  
> **Implementation status:** No production Rust engine or Flutter UI has been migrated yet.

## Live roadmap

The checklist below is the project status at a glance. Completed items are checked; unchecked items are still pending. The detailed plan and acceptance criteria live in the [full roadmap](docs/ROADMAP.md).

### Phase 0 — Reference audit

- [x] Confirm the reference repository and inspect its Git tree.
- [x] Confirm access to the target repository and create the migration branch.
- [x] Establish the README, roadmap and reverse-engineering documentation.
- [x] Inspect the desktop shell, primary tabs, link scanner and lookup parsers.
- [x] Trace the initial download scheduler, image lifecycle, path rules and SQLite location.
- [x] Inventory the initial UI shell, Posts/Threads/Tasks/Logs tabs, Add Links dialog and image workspace.
- [x] Verify the 16 registered host implementations and record initial resolver selectors.
- [x] Trace SAX response mapping, linked-image filtering, host matching and post URL construction.
- [x] Inspect retry defaults, authentication entry points, cookie usage and queue lifecycle.
- [x] Document the new Network settings and proxy requirements (Disabled / System / Manual).
- [ ] Complete the exhaustive screen, menu, settings, keyboard-shortcut and interaction inventory.
- [ ] Complete all host resolver traces and authorized test fixtures.
- [ ] Build golden test vectors for output folders, filenames, extensions, ordering and overwrite behavior.
- [ ] Capture reference screenshots and interaction recordings.
- [ ] Map the complete SQLite schema, settings defaults and migration/rollback behavior.
- [ ] Complete the GPL-3.0 and third-party dependency/asset license audit.

### Phase 1 — Native architecture foundation

- [ ] Define the Rust workspace and crate boundaries.
- [ ] Define the versioned Flutter ↔ Rust FFI contracts, typed events and errors.
- [ ] Define async runtime, cancellation, backpressure and bounded concurrency.
- [ ] Define the SQLite schema and data import plan.
- [ ] Define secure credential and cookie storage adapters.
- [ ] Create the initial native app and engine skeleton.

### Phase 2 — Rust engine parity

- [ ] Port URL validation and link classification.
- [ ] Port thread/post discovery and response parsing.
- [ ] Port all supported host resolvers.
- [ ] Port HTTP sessions, cookies, headers, redirects and response validation.
- [ ] Port queue ordering, concurrency, pause/stop/restart and retry behavior.
- [ ] Port temporary-file handling, MIME validation, naming and directory layout.
- [ ] Port persistence, tasks, events, logs and cache behavior.
- [ ] Implement centralized proxy routing for all application network requests.
- [ ] Implement proxy connection testing and secure credential handling.
- [ ] Pass parser, host, queue, lifecycle, persistence and output compatibility tests.

### Phase 3 — Flutter UI parity

- [ ] Reproduce the desktop shell, tabs, toolbar, status and error regions.
- [ ] Reproduce Posts, Threads, Tasks and Logs views.
- [ ] Reproduce dialogs, settings, filters, tables, grids, context actions and keyboard behavior.
- [ ] Add the Settings → Network section without changing the existing Connection section.
- [ ] Implement Disabled / System / Manual proxy modes and HTTP / HTTPS / SOCKS5 controls.
- [ ] Reproduce the image workspace, overview/browse modes, zoom, deep zoom, minimap and cache behavior.
- [ ] Adapt only screen constraints on mobile; preserve workflows and semantics.
- [ ] Pass screen-by-screen and interaction-by-interaction comparison.

### Phase 4 — Platform integration

- [ ] Windows x86-64 packaging, filesystem, drag/drop, notifications and lifecycle.
- [ ] Linux x86-64 DEB/RPM/AppImage packaging and desktop integration.
- [ ] macOS x86-64 and ARM64 packaging and signing preparation.
- [ ] Android ARM64 native library, storage, background downloads and secure storage.
- [ ] iOS ARM64 framework, document access, background constraints and Keychain.
- [ ] Build and smoke-test every supported target.

### Phase 5 — Data migration

- [ ] Import SQLite records and relationships without losing IDs or states.
- [ ] Preserve settings, cookies and existing download paths.
- [ ] Ensure existing downloaded files are never automatically renamed or reorganized.
- [ ] Implement transactional backup, validation and rollback.
- [ ] Pass migration fixtures and recovery tests.

### Phase 6 — Quality, security and performance

- [ ] Add unit, property-based, fixture, integration, UI golden and end-to-end tests.
- [ ] Verify installed apps require no separately installed Java/JVM runtime.
- [ ] Audit TLS, authentication, secure storage, cookie protection, filesystem boundaries and log redaction.
- [ ] Compare CPU, RAM, throughput, startup and large-gallery rendering against the reference.
- [ ] Pass release gates on every target architecture.

### Phase 7 — Release

- [ ] Set up reproducible CI builds and signed artifacts.
- [ ] Prepare Windows/Linux x86-64, macOS x86-64/ARM64 and Android/iOS ARM64 releases.
- [ ] Publish upgrade guide, release notes, privacy/security notes and license notices.
- [ ] Test install, upgrade, launch, core workflows and uninstall.

## Non-negotiable compatibility requirement

This is a **technology migration, not a product redesign**. Preserve the reference application's:

- UI layout, navigation, controls, dialogs and interaction flows.
- Site navigation, link parsing, thread/post discovery and image extraction.
- Supported host resolution and download behavior, subject to site access rules and applicable terms.
- Output directory layout, folder naming, filename normalization, ordering prefixes, extensions, overwrite behavior and temporary-file handling.

No behavior is to be intentionally changed without an explicit compatibility decision documented in this repository.

## Target platforms

- Windows x86-64
- Linux x86-64
- macOS x86-64 and ARM64
- Android ARM64
- iOS ARM64

Shipped applications must not require a separately installed Java Runtime or JVM. Java/JDK may still be needed by parts of the Android build toolchain; that is a build-time dependency, not a runtime dependency of the delivered app.

## Technology direction

- **Flutter:** cross-platform UI, preserving the reference UI structure and interaction behavior.
- **Rust:** native engine for parsing, host resolution, scheduling, downloads, file output, persistence and platform services.
- **FFI boundary:** versioned, typed interface between Flutter and Rust; platform-specific code stays behind adapters.

## Documentation

- [Full roadmap](docs/ROADMAP.md)
- [Reverse-engineering baseline](docs/REVERSE_ENGINEERING.md)
- [Reference UI inventory](docs/REFERENCE_UI_INVENTORY.md)
- [Detailed UI behavior audit](docs/UI_BEHAVIOR_AUDIT.md)
- [Engine behavior trace](docs/ENGINE_TRACE.md)
- [Host compatibility matrix](docs/HOST_COMPATIBILITY_MATRIX.md)
- [Compatibility specification](docs/COMPATIBILITY_SPEC.md)
- [Host resolver audit](docs/HOST_RESOLVER_AUDIT.md)
- [Settings inventory](docs/SETTINGS_INVENTORY.md)
- [Persistence schema audit](docs/PERSISTENCE_SCHEMA_AUDIT.md)
- [Download and output audit](docs/DOWNLOAD_AND_OUTPUT_AUDIT.md)
- [Settings inventory](docs/SETTINGS_INVENTORY.md)
- [Download and output audit](docs/DOWNLOAD_AND_OUTPUT_AUDIT.md)
- [Persistence schema audit](docs/PERSISTENCE_SCHEMA_AUDIT.md)
- [Network and proxy settings](docs/NETWORK_SETTINGS.md)
- [Architecture decision record](docs/ADR-0001-rust-flutter.md)

## Reference project and licensing

The compatibility reference is [dev-claw/vripper-project](https://github.com/dev-claw/vripper-project). The reference repository is licensed under GPL-3.0. Before redistributing a derivative, preserve required copyright/license notices and comply with GPL-3.0 source and distribution obligations. Audit third-party assets and dependencies separately.

## Current milestone

**Phase 0 is underway.** Source-level audits cover all 16 registered host resolvers, settings, UI interactions, download lifecycle, output naming and the logical SQLite schema. The Rust engine now contains an initial link/output compatibility slice, a host resolver layer and a thread XML parser with synthetic fixtures. CI verification is pending; runtime reference captures and end-to-end parity are still outstanding.
