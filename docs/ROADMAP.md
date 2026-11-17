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
- [ ] Complete exhaustive GUI inventory: all controls, columns, filters, dialogs, settings, shortcuts and interaction states.
- [ ] Inspect every host resolver in depth and create authorized fixtures.
- [ ] Complete output-path and filename golden vectors.
- [ ] Capture reference screenshots and interaction recordings.
- [ ] Map the full SQLite schema, indexes, settings defaults and migration behavior.
- [ ] Complete third-party dependency, asset and license audit.
- [ ] Review and sign off the compatibility specification.

**Exit criteria**
- [ ] Critical workflows have a written, reviewed behavioral specification.
- [ ] Deterministic fixtures exist for parsing, host resolution and output naming.
- [ ] Reference UI captures exist for all primary screens and important states.

## Phase 1 — Native architecture foundation

- [ ] Define Rust workspace and crate boundaries.
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
- [ ] Implement centralized proxy routing for every application-owned network request.
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
- [ ] Add a distinct Settings → Network section while preserving the existing Connection section.
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
| Site navigation/parsing | Initial scanner and SAX response flow traced |
| Host extraction | 16 registered hosts identified; deep resolver fixtures pending |
| Download lifecycle | Initial scheduler and image lifecycle inspected |
| Output compatibility | Initial rules traced; golden vectors pending |
| Persistence | SQLite/Liquibase identified; full schema mapping pending |
| Network/proxy settings | Requirements documented; implementation pending |
| Rust engine | Not started |
| Flutter UI | Not started |
| Platform builds | Not started |
| CI and packaging | Target matrix defined; implementation pending |

## Architecture decisions

- [x] ADR-0001: Rust engine + Flutter UI selected.
- [x] Mobile 64-bit targets are ARM64; desktop x64 targets are x86-64.
- [x] JDK may be required by Android build tooling, but not by the installed application at runtime.
