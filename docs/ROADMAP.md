# VRipper Native — Roadmap

> **Default branch status:** The repository's `main` branch currently contains the migration requirements and compatibility baseline. Implementation work is being developed in draft PR [#1](https://github.com/lyingtiger88/vripper-native/pull/1) and is not yet part of `main`.
>
> **Status rule:** A task is marked complete on this page only after its implementation is present in `main` and its relevant checks pass. Work present only on a feature branch is marked **in review / not merged**.

## At a glance

- [x] Establish the Rust + Flutter migration direction and non-negotiable compatibility requirements.
- [x] Identify the reference project and define target platforms.
- [ ] Review and merge the native migration implementation from PR #1.
- [ ] Connect the Java-compatible SQLite database to the live native application.
- [ ] Verify feature parity against the Java reference with real user data.
- [ ] Complete platform acceptance and prepare a distributable release.

## Phase 0 — Reference audit and compatibility contract

- [x] Record the Rust engine + Flutter UI architecture.
- [x] Define supported target platforms and 64-bit requirement.
- [x] Establish that this is a technology migration, not a product redesign.
- [ ] Complete source-to-native behavior traceability for every major workflow.
- [ ] Confirm licensing, notices and third-party dependency obligations before distribution.

## Phase 1 — Native application foundation

**Current state: implementation exists on PR #1; not merged to main.**

- [ ] Review the Rust workspace, engine boundaries and FFI contract.
- [ ] Review the Flutter navigation, settings and user workflows.
- [ ] Verify URL classification, thread/post discovery and image extraction.
- [ ] Verify host resolution and download behavior under authorized access.
- [ ] Verify queue controls, retries, cancellation and bounded concurrency.
- [ ] Pass CI and code review; merge the reviewed implementation to main.

## Phase 2 — Java database compatibility

**Current state: read-only inspection and copy-first importer are implemented on PR #1; live-store integration remains pending.**

- [ ] Inspect Java SQLite databases from every supported schema generation.
- [ ] Preserve the legacy logical schema, IDs, relationships, statuses and Liquibase history.
- [ ] Import by SQLite online backup into a new file; never modify the user's source database.
- [ ] Validate integrity, schema/columns, row counts, foreign keys and sensitive-field handling.
- [ ] Implement the native `DatabaseStore` and repositories for `POST`, `IMAGE`, `THREAD`, `METADATA`, `TASK` and `POST_ACTION`.
- [ ] Connect the imported database to the live download coordinator and UI.
- [ ] Restore previous posts, threads, image states and task history.
- [ ] Resume eligible unfinished downloads without duplicating or overwriting existing files.
- [ ] Test backup, restore, migration rollback and interrupted-import recovery.

Detailed design: [Database compatibility and Java import](DATABASE_COMPATIBILITY_AND_IMPORT.md).  
Schema inventory: [Persistence schema audit](PERSISTENCE_SCHEMA_AUDIT.md).

## Phase 3 — Reference parity and reliability

- [ ] Match the reference UI and interaction flows.
- [ ] Match output folders, filenames, ordering, extensions and overwrite rules.
- [ ] Verify supported hosts and error/retry behavior.
- [ ] Verify proxy modes, credentials handling and network failure recovery.
- [ ] Verify logs, task history and user-visible diagnostics.
- [ ] Run regression tests using representative Java database fixtures and existing download folders.

## Phase 4 — Platform acceptance

- [ ] Windows x86-64 build and install/upgrade test.
- [ ] Linux x86-64 build and desktop integration test.
- [ ] macOS x86-64/ARM64 build and signing test.
- [ ] Android ARM64 build, storage permission and document-provider test.
- [ ] iOS ARM64 build, file access and lifecycle test.
- [ ] Verify the delivered applications do not require a separately installed Java runtime.

## Phase 5 — Release readiness

- [ ] Complete security, privacy and licensing review.
- [ ] Verify clean install, upgrade, backup and rollback paths.
- [ ] Produce signed release artifacts where supported.
- [ ] Publish user-facing migration instructions from Java to Native.
- [ ] Tag the first release only after all critical acceptance tests pass.

## Current implementation branch

The active implementation and CI evidence are tracked in [PR #1](https://github.com/lyingtiger88/vripper-native/pull/1). Because that pull request is still a draft, its code and CI results must not be interpreted as already released or merged into `main`.
