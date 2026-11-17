# VRipper Native — Roadmap

**Status:** Phase 0 in progress  
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

- [x] Initialize Rust workspace and core crate; deterministic link/path primitives, host resolver layer and XML response parser authored (CI verification pending).
- [ ] Define versioned Flutter↔Rust FFI contracts, typed events and error model.
- [ ] Define async runtime, cancellation, backpressure and bounded concurrency.
- [ ] Define SQLite schema and data import plan.
- [ ] Define secure credential/cookie storage adapters.
- [ ] Create minimal native app and engine skeleton.
- [ ] Add CI checks for formatting, linting and unit tests.

**Exit criteria**
- [ ] Engine initializes and shuts down cleanly.
- [ ] Settings persist and reload.
- [ ] FFI events and typed errors are tested.
- [ ] Desktop and mobile architecture builds a minimal application.

## Phase 2 — Rust engine parity

- [ ] Port URL validation and link classification.
- [ ] Port thread/post discovery and response parsing.
- [ ] Port host registry and every supported host resolver.
- [ ] Port HTTP sessions, cookies, headers, redirects and response validation.
- [ ] Port queue, per-host/global concurrency, pause/stop/restart, retry and error semantics.
- [ ] Port temporary-file handling, MIME validation, naming and directory layout.
- [ ] Port database entities, task/event reporting, logs and cache behavior.
- [x] Implement a centralized Rust proxy-aware HTTP client builder with Disabled/System/Manual modes and HTTP/HTTPS/SOCKS5h support.
- [ ] Wire the shared client into every login, lookup, host-resolution and image-download request.
- [ ] Implement OS system-proxy discovery and secure credential storage adapters.
- [ ] Implement proxy connection testing and secure credential handling.
- [ ] Pass parser, host, queue, lifecycle, persistence and output compatibility tests.

**Exit criteria**
- [ ] Engine parity fixtures pass.
- [ ] All application network requests obey the selected proxy mode.
- [ ] No credentials, cookies or authorization data leak into logs or plaintext persistence.

## Phase 3 — Flutter UI parity

- [ ] Reproduce desktop shell, tabs, toolbar, status and error regions.
- [ ] Reproduce Posts, Threads, Tasks and Logs views.
- [ ] Reproduce dialogs, settings, filters, tables, grids, context actions and keyboard behavior.
- [x] Add a distinct Settings → Network section prototype while preserving the existing Connection section.
- [ ] Persist and validate Network settings through the native configuration/secure-storage bridge.
- [ ] Add Disabled / System / Manual proxy modes and HTTP / HTTPS / SOCKS5 settings.
- [ ] Add manual endpoint, optional authentication, bypass list and Test connection feedback.
- [ ] Reproduce image workspace, overview/browse modes, navigation, zoom, deep zoom, minimap, cache and geometry persistence.
- [ ] Adapt only layout constraints required by mobile screens; do not remove workflows or change semantics.
- [ ] Complete screen-by-screen and interaction-by-interaction comparison.

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
- [ ] Build and smoke-test every supported target.

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

## Workstream status

| Workstream | Current status |
|---|---|
| Reference repository audit | Initial tree and core paths inspected |
| GUI inventory | Initial shell and key screens documented; exhaustive inventory pending |
| Site navigation/parsing | Rust URL scanner and XML parser implemented with synthetic tests; CI validation pending |
| Host extraction | 16 resolver implementations transcribed into Rust; synthetic fixtures added; CI validation pending |
| Download lifecycle | Initial scheduler and image lifecycle inspected |
| Output compatibility | Rust folder/filename primitives and initial tests authored; CI validation pending |
| Persistence | SQLite/Liquibase identified; full schema mapping pending |
| Network/proxy settings | Flutter section prototype and Rust proxy client builder authored; full request integration and secure storage pending |
| Rust engine | Workspace, scanner, output helpers, host resolvers, XML parser, FFI and proxy client builder authored; CI verification and queue/persistence pending |
| Flutter UI | Not started |
| Platform builds | Not started |
| CI and packaging | Target matrix defined; implementation pending |

## Architecture decisions

- [x] ADR-0001: Rust engine + Flutter UI selected.
- [x] Mobile 64-bit targets are ARM64; desktop x64 targets are x86-64.
- [x] JDK may be required by Android build tooling, but not by the installed application at runtime.
