# Dependency, Asset and License Audit — Initial Inventory

**Audited ref:** `migration/phase-0-audit` at `c3bd866827c462f4dfb699228d359dad65d85ee4`  
**Audit level:** manifest/tree inventory only — not a completed legal or transitive dependency audit.

## Project-level provenance

- The repository contains a root `LICENSE` file identifying GNU GPL version 3.
- The Cargo workspace declares `GPL-3.0-only` as its package license.
- The reference project provenance and migration obligations are recorded in the roadmap and compatibility documentation.
- Before distribution, confirm the exact source attribution/copyright notices and ensure every copied or adapted source file retains required notices. This document is not legal advice.

## Direct Rust dependencies

Manifest: `Cargo.toml` and `engine/vripper-core/Cargo.toml`.

| Crate | Declared version requirement | Use in native engine | License evidence status |
|---|---:|---|---|
| regex | 1.11 | URL matching and output-name sanitization | Verify package metadata and exact resolved version |
| thiserror | 2.0 | Typed error definitions | Verify package metadata and exact resolved version |
| scraper | 0.22 | HTML parsing for host resolvers | Verify package metadata and exact resolved version |
| quick-xml | 0.37 | Thread/post XML parsing | Verify package metadata and exact resolved version |
| serde | 1.0 | Serialization/deserialization | Verify package metadata and exact resolved version |
| serde_json | 1.0 | FFI and persistence payloads | Verify package metadata and exact resolved version |
| reqwest | 0.12 | HTTP transport | Verify package metadata and exact resolved version |
| url | 2.5 | URL parsing | Verify package metadata and exact resolved version |
| tokio | 1.43 | Async runtime and queue coordination | Verify package metadata and exact resolved version |
| futures-util | 0.3 | Async stream utilities | Verify package metadata and exact resolved version |
| tempfile | 3.15 | Temporary files and test fixtures | Verify package metadata and exact resolved version |
| md-5 | 0.10 | Reference-compatible digest functionality | Verify package metadata and exact resolved version |
| rusqlite | 0.32 | SQLite persistence and online backup | Verify crate and bundled SQLite notices |
| wiremock | 0.6 | Rust test-only mock HTTP server | Verify package metadata and exact resolved version |

## Direct Flutter dependencies

Manifest: `apps/client/pubspec.yaml`.

| Package | Declared version requirement | Use | License evidence status |
|---|---:|---|---|
| flutter (SDK) | SDK | UI framework | Record Flutter SDK version and bundled notices for release |
| ffi | ^2.1.4 | Rust C ABI integration | Verify resolved package metadata |
| path_provider | ^2.1.5 | Platform storage directories | Verify resolved package metadata and platform implementation notices |
| file_picker | ^8.1.7 | File selection | Verify resolved package metadata and platform implementation notices |
| shared_preferences | ^2.5.3 | Non-secret preference persistence | Verify resolved package metadata and platform implementation notices |
| flutter_secure_storage | ^9.2.4 | Platform secure credential storage | Verify resolved package metadata and platform implementation notices |
| flutter_test (SDK) | SDK | Widget tests | Development/test dependency |
| flutter_lints | ^5.0.0 | Static analysis rules | Development-only dependency |

## Lockfiles and transitive dependencies

At the audited Git tree:
- No `Cargo.lock` is committed.
- No `apps/client/pubspec.lock` is committed.
- Exact resolved dependency versions and transitive dependency graph therefore cannot be established from the repository snapshot alone.
- No complete SPDX/CycloneDX SBOM or machine-generated license report is present.

**Required before release:**
1. Generate and commit lockfiles for the supported application targets (with documented platform-specific exceptions, if any).
2. Produce an SBOM from the locked dependency graph.
3. Review every transitive package license, notice and bundled/native component.
4. Specifically inspect bundled SQLite, TLS backend, Android/iOS plugin implementations and platform-native code.
5. Add third-party notices and a reproducible license-check step to CI.
6. Re-run the audit whenever dependency versions or platform plugins change.

## Assets and fonts

The recursive repository tree at the audited ref contains no dedicated assets, images, fonts, or bundled media directory. Flutter currently enables Material icons through `uses-material-design: true` and declares no custom asset bundle in `pubspec.yaml`.

This tree observation does not prove that all runtime dependencies are asset-free: plugin packages may bundle native resources, fonts, icons or platform code. Include those resources in the transitive package and platform notice review.

## Phase 0 conclusion

**Initial inventory complete.**

The project-level GPL-3.0 provenance, direct manifest dependencies and visible asset tree have been inventoried. The full third-party license audit remains **open** until lockfiles, transitive package metadata, native plugin notices, attribution review and generated notices are available. Do not describe the distribution as license-cleared yet.
