# VRipper Native

Native-first, 64-bit cross-platform evolution of VRipper using **Rust** for the application engine and **Flutter** for the user interface.

> **Current status:** Native vertical slice implemented; Rust and Flutter CI checks plus Windows x86-64 and Android ARM64 smoke builds pass. Reference parity and platform acceptance remain in progress.  
> **Implementation status:** Rust engine, proxy-aware transport, Viper lookup/auth path, queue worker, C ABI coordinator and Flutter queue/settings integration are present. Posts and Threads are populated from extraction results and basic list management is available. This is not yet a release-ready or parity-complete build.

## Project roadmap

The complete phase-by-phase roadmap is published directly on this repository front page below. The canonical editable checklist is also maintained in [docs/ROADMAP.md](docs/ROADMAP.md). Status legend: [x] completed/documented; [ ] pending. Source inspection does not imply runtime verification.

Legend: [x] completed/documented; [ ] not yet completed. A source inspection checkbox does not imply that its feature has been ported or runtime-tested.

## Non-negotiable rules

- [x] Preserve the reference repository unchanged.
- [x] Record provenance and GPL-3.0 / third-party licensing obligations for the migration.
- [x] Establish that UI, navigation, extraction, download and output behavior are compatibility requirements.
- [x] Establish that site access controls and restrictions must not be bypassed.
- [ ] Complete and test compatibility for every critical workflow.

## Phase 0 — Reference audit and behavioral specification

- [x] Confirm source repository and inspect its Git tree.
- [x] Confirm target repository and write permissions.
- [x] Initialize target README and roadmap.
- [x] Create `migration/phase-0-audit` branch.
- [x] Inspect initial desktop shell, main tabs, link scanner, lookup parsers, host base class, download scheduler, image lifecycle, path rules and SQLite connection.
- [x] Record initial shell, tab, menu, action bar, status bar, Add Links and image workspace inventory.
- [x] Verify active host registration and inspect resolver selectors for all 16 registered hosts.
- [x] Trace SAX response mapping, image filtering, host matching, per-post aggregation and generated post URL.
- [x] Inspect retry defaults, authentication entry points, cookie usage, queue scheduling and image state lifecycle.
- [x] Document the requested Network settings section and proxy behavior.
- [x] Inventory the nine Settings sections and record their controls, defaults and validation behavior.
- [x] Record Posts table columns, List/Grid selection behavior, Threads/Tasks/Logs interactions, quick filters, grouping and action bar/menu shortcuts.
- [ ] Capture reference screenshots and interaction recordings for pixel/gesture parity.
- [x] Inspect source for every one of the 16 registered host resolvers and record selectors, request mutations and URL transformations.
- [ ] Create authorized HTML fixtures and runtime tests for each host resolver.
- [x] Trace exact folderName generation, including optional post ID suffix.
- [ ] Complete output-path and filename golden vectors.
- [ ] Capture reference screenshots and interaction recordings.
- [x] Map logical POST, IMAGE, THREAD, METADATA, TASK and POST_ACTION entities and observed indexes from Liquibase.
- [ ] Verify schema/entity drift and migration behavior against a real legacy database.
- [ ] Complete third-party dependency, asset and license audit.
- [ ] Capture a runnable reference baseline and sign off the compatibility specification.

**Exit criteria**
- [ ] Critical workflows have a written, reviewed behavioral specification.
- [ ] Deterministic fixtures exist for parsing, host resolution and output naming.
- [ ] Reference UI captures exist for all primary screens and important states.

## Phase 1 — Native architecture foundation

- [x] Initialize Rust workspace and core crate boundaries.
- [x] Define the versioned Flutter↔Rust C ABI and JSON request/response contracts.
- [x] Implement a shared Tokio runtime, coordinator lifecycle and queue event polling.
- [x] Implement bounded queue concurrency, stop/clear/reorder controls and retry baseline.
- [x] Create the Flutter shell and native library loading path.
- [x] Add CI checks for Rust formatting, unit tests and Clippy; current Rust checks pass.
- [ ] Replace JSON-shaped FFI errors with a fully typed, versioned event/error contract.
- [ ] Define SQLite schema and data import plan.
- [x] Add Flutter-side platform secure storage for Viper and proxy credentials; Rust receives credentials only at coordinator initialization.
- [ ] Add secure cookie/session persistence and complete OS keychain/keystore audit.

**Exit criteria**
- [ ] Engine initializes and shuts down cleanly.
- [ ] Settings persist and reload.
- [ ] FFI events and typed errors are tested.
- [ ] Desktop and mobile architecture builds a minimal application.

## Phase 2 — Rust engine parity

- [x] Implement URL validation and link classification baseline.
- [x] Implement thread/post lookup request path and XML response parsing baseline.
- [x] Implement initial resolver paths for the 16 audited host types.
- [x] Implement shared HTTP transport, cookie handling and proxy-aware client construction baseline.
- [x] Wire thread/post lookups and image downloads through the shared transport in the native coordinator.
- [x] Implement bounded queue scheduling, stop, reorder, event reporting and retry baseline.
- [x] Implement streamed temporary-file downloads and initial output naming/path rules.
- [ ] Verify all resolvers against authorized host fixtures and reference behavior.
- [ ] Complete MIME/content validation, overwrite policy, recovery and filename edge cases.
- [x] Persist discovered post and thread metadata locally in app preferences for restart continuity (initial client-side store).
- [ ] Replace the initial preferences-backed store with the reference-compatible SQLite schema, task history, logs and cache behavior.
- [ ] Implement OS-native system proxy discovery and connection testing.
- [ ] Implement secure credential/cookie storage and log redaction.
- [ ] Pass full parser, host, queue, lifecycle and output compatibility suites.

**Exit criteria**
- [ ] Engine parity fixtures pass.
- [ ] All application network requests obey the selected proxy mode.
- [ ] No credentials, cookies or authorization data leak into logs or plaintext persistence.

## Phase 3 — Flutter UI parity

- [x] Establish the Flutter shell and primary Posts/Threads/Tasks/Logs navigation.
- [x] Connect initial link scanning, queue controls, queue status and task list to the native coordinator.
- [x] Add Settings → Network with Disabled / System / Manual modes, protocol, endpoint, credentials and bypass fields.
- [x] Persist non-secret network preferences and store proxy/Viper credentials through the platform secure-storage plugin.
- [ ] Implement native OS proxy discovery and proxy connection testing.
- [x] Populate Posts and Threads views from native extraction results; add post rows, selection, remove-selected, clear-list and thread removal/clear actions.
- [ ] Complete reference-equivalent filters, post lifecycle states, context actions and Logs data view.
- [ ] Reproduce filters, tables, grids, context actions, dialogs and keyboard behavior.
- [ ] Reproduce image workspace, overview/browse modes, navigation, zoom, deep zoom, minimap and cache behavior.
- [ ] Adapt mobile layouts while preserving workflows and semantics.
- [ ] Pass screen-by-screen and interaction-by-interaction comparison.

**Exit criteria**
- [ ] Desktop UI matches the reference behavior and approved captures.
- [ ] Mobile UI preserves all workflows with documented platform adaptations.
- [ ] Network settings validation and feedback are tested.

## Phase 4 — Platform integration

- [ ] Windows x86-64: packaging, filesystem, drag/drop, notifications and lifecycle.
- [ ] Linux x86-64: DEB/RPM/AppImage, desktop integration and lifecycle.
- [ ] macOS x86-64 + ARM64: packaging, file integration, signing/notarization preparation.
- [ ] Android ARM64: Rust native library, storage access, foreground/background download lifecycle and secure storage.
- [ ] iOS ARM64: Rust framework, background execution constraints, document access and Keychain.
- [ ] Complete Windows x86-64 and Android ARM64 smoke builds through CI; resolve any platform runner/toolchain failures.
- [ ] Add Linux x86-64, macOS x86-64/ARM64 and iOS ARM64 packaging/build jobs.

**Exit criteria**
- [ ] All supported targets build reproducibly.
- [ ] Platform-specific file, network, secure-storage and lifecycle tests pass.

## Phase 5 — Data migration

- [ ] Import SQLite records and relationships without losing IDs or states.
- [ ] Preserve settings, cookies and existing download paths.
- [ ] Never automatically reorganize or rename existing downloaded files.
- [ ] Add transactional backup, validation and rollback.
- [ ] Test migration against representative legacy database fixtures.

**Exit criteria**
- [ ] Migration is recoverable and passes old-database fixtures.
- [ ] Existing downloaded files remain untouched.

## Phase 6 — Quality, security and performance

- [ ] Add unit, property-based, fixture, integration, UI golden and end-to-end tests.
- [ ] Verify shipped applications need no separately installed Java/JVM runtime.
- [ ] Audit TLS, authentication, secure storage, cookie protection, filesystem boundaries and log redaction.
- [ ] Compare CPU, RAM, throughput, startup and large-gallery rendering against reference.
- [ ] Pass release gates on every target architecture.

**Exit criteria**
- [ ] No critical compatibility or security failures remain.
- [ ] Performance is measured against the reference and documented.
- [ ] Release test suite passes on all supported architectures.

## Phase 7 — Release

- [ ] Set up reproducible CI builds and signed artifacts.
- [ ] Prepare Windows/Linux x86-64; macOS x86-64/ARM64; Android/iOS ARM64 packages.
- [ ] Publish upgrade guide, release notes, privacy/security notes and license notices.
- [ ] Test install, upgrade, launch, core workflows and uninstall.

**Exit criteria**
- [ ] Release artifacts are signed and reproducible.
- [ ] Upgrade and clean-install scenarios pass.
- [ ] Licensing and release documentation are complete.

## Practical acceptance

- [x] Publish a first-run Windows/Android practical test checklist in `docs/PRACTICAL_TEST_PLAN.md`.
- [ ] Produce and inspect Windows x86-64 and Android ARM64 build artifacts.
- [ ] Complete authorized live-site account, lookup and image-download acceptance tests.

## Workstream status

| Workstream | Current status |
|---|---|
| Reference repository audit | Initial tree and core paths inspected |
| GUI inventory | Initial shell and key screens documented; exhaustive inventory pending |
| Site navigation/parsing | Rust URL scanner and XML parser implemented; Rust CI checks pass |
| Host extraction | 16 initial resolver paths transcribed into Rust; synthetic fixtures added; authorized live-host verification pending |
| Download lifecycle | Initial scheduler and image lifecycle inspected |
| Output compatibility | Rust folder/filename primitives and initial tests authored; CI validation pending |
| Persistence | Posts/Threads metadata persists in local preferences; SQLite/Liquibase migration and task history pending |
| Network/proxy settings | Flutter section prototype and Rust proxy client builder authored; full request integration and secure storage pending |
| Rust engine | Workspace, scanner, output helpers, initial host resolvers, XML parser, FFI and proxy-aware queue pipeline implemented; Rust CI checks pass; SQLite persistence pending |
| Flutter UI | Posts/Threads extraction records and basic list management implemented; Flutter analysis and widget tests pass; full reference parity pending |
| Platform builds | Windows x86-64 and Android ARM64 CI smoke builds pass; artifact inspection and live workflow acceptance pending |
| CI and packaging | Rust, Flutter and Windows/Android smoke-build workflows pass; remaining platform matrix and release packaging pending |

## Architecture decisions

- [x] ADR-0001: Rust engine + Flutter UI selected.
- [x] Mobile 64-bit targets are ARM64; desktop x64 targets are x86-64.
- [x] JDK may be required by Android build tooling, but not by the installed application at runtime.

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
- [Remote client/server protocol audit](docs/REMOTE_API_AUDIT.md)
- [Host resolver audit](docs/HOST_RESOLVER_AUDIT.md)
- [Settings inventory](docs/SETTINGS_INVENTORY.md)
- [Persistence schema audit](docs/PERSISTENCE_SCHEMA_AUDIT.md)
- [Download and output audit](docs/DOWNLOAD_AND_OUTPUT_AUDIT.md)
- [Settings inventory](docs/SETTINGS_INVENTORY.md)
- [Download and output audit](docs/DOWNLOAD_AND_OUTPUT_AUDIT.md)
- [Persistence schema audit](docs/PERSISTENCE_SCHEMA_AUDIT.md)
- [Network and proxy settings](docs/NETWORK_SETTINGS.md)
- [Architecture decision record](docs/ADR-0001-rust-flutter.md)
- [Practical Windows/Android test plan](docs/PRACTICAL_TEST_PLAN.md)

## Reference project and licensing

The compatibility reference is [dev-claw/vripper-project](https://github.com/dev-claw/vripper-project). The reference repository is licensed under GPL-3.0. Before redistributing a derivative, preserve required copyright/license notices and comply with GPL-3.0 source and distribution obligations. Audit third-party assets and dependencies separately.

## Current milestone

**Native vertical slice / pre-alpha integration.** The Rust workspace passes its current automated unit tests and Clippy checks. The Flutter client now loads the native library, creates a coordinator, submits pasted links, displays queue events and exposes queue controls. Flutter analysis and widget tests pass, including compact-width layout coverage. Windows x86-64 and Android ARM64 CI smoke builds pass; artifacts and real-device workflows still require acceptance testing. Real-site authentication, resolver-by-resolver compatibility, persistence, secure credential storage, full UI parity and signed platform packaging remain incomplete. Do not treat the current branch as a production release.
