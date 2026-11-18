# VRipper Native — Roadmap

**Status:** Native vertical slice implemented; parity and platform integration in progress  
**Reference:** dev-claw/vripper-project  
**Target:** Rust native engine + Flutter UI  
**Compatibility policy:** Preserve reference behavior; this is a technology migration, not a product redesign.

Legend: [x] completed/documented; [ ] not yet completed. A source inspection checkbox does not imply that its feature has been ported or runtime-tested.

## Non-negotiable rules

- [x] Preserve the reference repository unchanged.
- [x] Record provenance and GPL-3.0 / third-party licensing obligations for the migration.
- [x] Establish that UI, navigation, extraction, download and output behavior are compatibility requirements.
- [x] Establish that site access controls and restrictions must not be bypassed.
- [ ] Complete and test compatibility for every critical workflow.

## Phase 0 — Reference audit and behavioral specification
\n**Compatibility evidence register:** [Phase 0 compatibility matrix](PHASE0_COMPATIBILITY_MATRIX.md). It separates source inspection and synthetic tests from reference-application comparison and real acceptance evidence.\n
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
- [x] Add synthetic HTML fixtures and resolver unit coverage for all 16 host types.\n- [ ] Capture authorized reference HTML and compare resolver behavior for each host.
- [x] Trace exact folderName generation, including optional post ID suffix.
- [x] Expand deterministic native output-path and filename unit vectors (synthetic; Java comparison remains pending).\n- [x] Reject unsafe remote thread-title path components before native database registration and download queue insertion; Java parity intentionally yields to path safety.\n- [ ] Validate local symlink/race containment and Windows reserved-name behavior on real target filesystems.
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
- [x] Define SQLite schema compatibility and copy-first legacy import plan.
- [x] Add bundled SQLite runtime and online-backup API foundation.
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
- [ ] Connect the imported legacy SQLite database to a native DatabaseStore and replace preferences-backed post/thread storage.
- [ ] Implement SQLite repositories for POST, IMAGE, THREAD, METADATA, TASK and POST_ACTION, including task history, logs and cache behavior.
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
- [ ] Complete Windows x86-64 and Android ARM64 smoke builds through CI on the latest branch head; inspect and retain artifacts.
- [ ] Add Linux x86-64, macOS x86-64/ARM64 and iOS ARM64 packaging/build jobs.

**Exit criteria**
- [ ] All supported targets build reproducibly.
- [ ] Platform-specific file, network, secure-storage and lifecycle tests pass.

## Phase 5 — Data migration

- [x] Implement read-only legacy database inspection and SQLite online-backup import into a new staging file.
- [x] Validate core tables, table/column layout, row counts, quick_check and foreign-key integrity before publishing.
- [x] Add Settings → Database UI to select, inspect and import a copy of the Java database.
- [ ] Connect imported records to native repositories and UI without losing IDs or states.
- [ ] Preserve settings, cookies and existing download paths.
- [ ] Never automatically reorganize or rename existing downloaded files.
- [ ] Add transactional backup, validation and rollback.
- [x] Add synthetic legacy SQLite fixtures for inspection, successful import, source preservation, destination collision and incompatible database rejection.
- [ ] Test migration against representative real Java database fixtures from each supported schema generation.

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
| Site navigation/parsing | Rust URL scanner and XML parser implemented with synthetic tests; Rust CI checks pass |
| Host extraction | 16 initial resolver paths transcribed into Rust; synthetic fixtures added; authorized live-host verification pending |
| Download lifecycle | Initial scheduler and image lifecycle inspected |
| Output compatibility | Folder/filename primitives, cross-platform unsafe-path rejection and synthetic tests authored; Java golden comparison and filesystem acceptance pending |
| Persistence | Verified Java-copy import is now the active writable catalog; extracted POST/IMAGE/THREAD metadata is persisted into the legacy-compatible SQLite tables while download state is preserved; full repositories/TASK/POST_ACTION and real legacy fixtures remain pending |
| Network/proxy settings | Flutter section prototype and Rust proxy client builder authored; full request integration and secure storage pending |
| Rust engine | Workspace, scanner, output helpers, initial host resolvers, XML parser, FFI and proxy-aware queue pipeline implemented; Rust CI checks pass; SQLite persistence pending |
| Flutter UI | Posts/Threads extraction records and basic list management implemented; Flutter analysis and widget tests pass; full reference parity pending |
| Platform builds | Rust native engine compilation passed for Windows x86-64 and Android ARM64 on an earlier head; complete app packaging for the latest head is pending |
| CI and packaging | Rust and Flutter checks passed on the previous verified head; latest-head CI reruns and final Windows/Android packaging remain pending |

## Current persistence milestone

The verified imported Java SQLite copy is now selected as the active native catalog after import and remains the active catalog after restart. New extracted thread/post/image records are written transactionally to the legacy-compatible `THREAD`, `POST` and `IMAGE` tables. Existing download state (`STATUS`, `DONE`, `DOWNLOADED`, `SIZE`, `FILENAME`) is preserved when metadata is refreshed. The separate `NATIVE_CATALOG` overlay remains for UI-specific state until the full repository layer is completed.

## Architecture decisions

- [x] ADR-0001: Rust engine + Flutter UI selected.
- [x] Mobile 64-bit targets are ARM64; desktop x64 targets are x86-64.
- [x] JDK may be required by Android build tooling, but not by the installed application at runtime.
