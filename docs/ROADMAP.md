# VRipper Native — Roadmap

**Status:** Phase 0 in progress  
**Reference:** dev-claw/vripper-project  
**Target:** Rust native engine + Flutter UI  
**Compatibility policy:** Preserve reference behavior; this is a technology migration, not a product redesign.

## Non-negotiable rules
- Preserve the reference UI layout, navigation, labels, controls, dialogs, keyboard behavior, drag-and-drop and image-viewer interactions.
- Preserve link recognition, site navigation, thread/post parsing, extraction order, host resolution, retry and concurrency semantics.
- Preserve output directory layout, folder naming, filename normalization, extensions, ordering prefixes, overwrite behavior and temporary-file lifecycle.
- Do not discard user data; migrate SQLite, settings, cookies and file references with backup and rollback.
- Do not bypass site access controls, CAPTCHA, anti-bot challenges or site restrictions.
- Keep the reference repository untouched. Record provenance and comply with GPL-3.0 and third-party license obligations.

## Phase 0 — Reference audit and behavioral specification (current)
- [x] Confirm source repository and inspect its 724-entry Git tree.
- [x] Confirm target repository and write permissions.
- [x] Initialize target README and this roadmap.
- [x] Create migration/phase-0-audit branch.
- [x] Inspect initial desktop shell, main tabs, link scanner, lookup parsers, host base class, download scheduler, image lifecycle, path rules and SQLite connection.
- [x] Record initial shell, tab, menu, action bar, status bar, Add Links and image workspace inventory; exhaustive controls/settings inventory remains pending.
- [x] Verify active host registration and inspect resolver selectors for all 16 registered hosts; full resolver behavior fixtures remain pending.
- [x] Trace SAX response mapping, image filtering, host matching, per-post aggregation and generated post URL.
- [x] Inspect retry defaults, authentication entry points, cookie usage, queue scheduling and image state lifecycle; exhaustive call-site trace remains pending.
- [ ] Complete output-path and filename golden vectors.
- [ ] Capture reference screenshots and authorized local fixtures.
**Exit:** reviewed behavioral specification for every critical workflow.

## Phase 1 — Native architecture foundation
- Define Rust workspace and crate boundaries.
- Define versioned Flutter↔Rust FFI contracts, typed events and error model.
- Define async runtime, cancellation, backpressure and bounded concurrency.
- Define SQLite schema and data import plan.
- Define secure credential/cookie storage adapters.
**Exit:** engine initializes, persists settings, emits events and shuts down cleanly on all targets.

## Phase 2 — Rust engine parity
- Port URL validation and link classification.
- Port thread/post discovery and response parsing.
- Port host registry and every supported host resolver.
- Port HTTP session, cookies, headers, redirects and response validation.
- Port queue, per-host/global concurrency, pause/stop/restart, retry and error semantics.
- Port temporary-file handling, MIME validation, naming and directory layout.
- Port database entities, task/event reporting, logs and cache behavior.
**Exit:** parser, host, queue, lifecycle, persistence and output compatibility tests pass.

## Phase 3 — Flutter UI parity
- Reproduce desktop shell, tabs, toolbar, status and error regions.
- Reproduce Posts, Threads, Tasks and Logs views.
- Reproduce dialogs, settings, filters, tables, grids, context actions and keyboard behavior.
- Reproduce image workspace, overview/browse modes, navigation, zoom, deep zoom, minimap, cache and geometry persistence.
- Adapt only layout constraints required by mobile screens; do not remove workflows or change semantics.
**Exit:** screen-by-screen and interaction-by-interaction comparison accepted.

## Phase 4 — Platform integration
- Windows x86-64: packaging, filesystem, drag/drop, notifications and lifecycle.
- Linux x86-64: DEB/RPM/AppImage, desktop integration and lifecycle.
- macOS x86-64 + ARM64: packaging, file integration, signing/notarization preparation.
- Android ARM64: Rust native library, storage access, foreground/background download lifecycle and secure storage.
- iOS ARM64: Rust framework, background execution constraints, document access and Keychain.
**Exit:** every target builds and passes platform smoke tests.

## Phase 5 — Data migration
- Import SQLite records and relationships without losing IDs or states.
- Preserve settings, cookies and existing download paths.
- Never automatically reorganize or rename existing downloaded files.
- Add transactional backup, validation and rollback.
**Exit:** migration is recoverable and passes old-database fixtures.

## Phase 6 — Quality, security and performance
- Unit, property-based, fixture, integration, UI golden and end-to-end tests.
- Verify shipped applications need no separately installed Java/JVM runtime.
- Audit TLS, authentication, secure storage, cookie protection, filesystem boundaries and log redaction.
- Compare CPU, RAM, throughput, startup and large-gallery rendering against reference.
**Exit:** release gates pass on every target architecture.

## Phase 7 — Release
- Reproducible CI builds and signed artifacts.
- Windows/Linux x86-64; macOS x86-64/ARM64; Android/iOS ARM64.
- Upgrade guide, release notes, privacy/security notes and license notices.
**Exit:** install, upgrade, launch, core workflows and uninstall tested.

## Workstream status
| Workstream | Status |
|---|---|
| GUI inventory | Initial shell identified; full inventory pending |
| Site navigation/parsing | Scanner and lookup parsers inspected |
| Host extraction | Base host inspected; individual resolvers pending |
| Download lifecycle | Scheduler and image runnable inspected |
| Output compatibility | Initial rules traced; golden vectors pending |
| Persistence | SQLite + Liquibase identified |
| Rust engine | Not started |
| Flutter UI | Not started |
| CI and packaging | Target matrix defined |

## Architecture decisions
- ADR-0001: Rust engine + Flutter UI.
- Mobile 64-bit targets are ARM64; desktop x64 targets are x86-64.
- JDK may be required by Android build tooling, but not by the installed application at runtime.
