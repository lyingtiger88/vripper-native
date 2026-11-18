# Phase 0 — Compatibility Matrix and Evidence Register

**Branch:** `migration/phase-0-audit`  
**Purpose:** Separate source-code inspection, deterministic synthetic tests, and real reference-application acceptance evidence. A checked source-inspection item is not proof of runtime parity.

## Evidence levels

| Level | Meaning | May be reported as |
|---|---|---|
| A — Source inspected | Behavior traced in the Java reference source | Inspected |
| B — Synthetic fixture/test | Deterministic local fixture or unit test exercises the native implementation | Native test coverage |
| C — Reference comparison | Same input is exercised against the reference Java application and native build, with output captured | Compared |
| D — Acceptance | Authorized real account/database/content workflow succeeds on a target platform | Accepted |

Current automated tests are primarily Level B. They do not establish Level C/D parity.

## Link scanning and API parsing

| Area | Native coverage | Remaining evidence |
|---|---|---|
| Thread URL classification | Unit tests for supported bases, trimming, blank lines and invalid links | Compare edge cases with Java scanner |
| Post URL classification | Unit tests for thread/post IDs | Compare query-string variants and malformed IDs |
| XML response mapping | Synthetic XML fixture; post order, image filtering, host counts and generated URL asserted | Capture sanitized reference responses from multiple Java-era site responses |
| Empty/malformed XML and missing attributes | Parser behavior implemented | Add negative fixture matrix and compare Java failure semantics |

## Host resolver matrix

The native resolver registry contains 16 Host IDs. Existing fixture tests cover the 11 simple resolver cases and the five continuation/special cases below.

| Host ID | Host | Existing synthetic fixture/test path | Evidence still required |
|---:|---|---|---|
| 0 | Acidimg | Continue POST + response fixture | Authorized reference capture |
| 1 | DPicMe | Simple HTML fixture | Authorized reference capture |
| 2 | ImageBam | Interstitial cookies + follow-up fixture | Cookie behavior against authorized reference |
| 3 | ImageTwist | Simple fixture + referer assertion | Authorized reference capture |
| 4 | ImageVenue | Continue-page follow-up fixture | Authorized reference capture |
| 5 | ImageZilla | Simple HTML fixture | Authorized reference capture |
| 6 | Imgbox | Simple HTML fixture | Authorized reference capture |
| 7 | ImgSpice | Simple HTML fixture | Authorized reference capture |
| 8 | Imx | Thumbnail URL mapping + optional original-name follow-up | URL variants and filename comparison |
| 9 | Pimpandhost | Original-size URL transformation + fixture | Redirect/query behavior against reference |
| 10 | Pixhost | Simple HTML fixture | Authorized reference capture |
| 11 | PixRoute | Simple HTML fixture | Authorized reference capture |
| 12 | Pixxxels | Simple HTML fixture | Authorized reference capture |
| 13 | PostImg | Simple HTML fixture | Authorized reference capture |
| 14 | TurboImageHost | Simple HTML fixture | Authorized reference capture |
| 15 | ViprIm | Simple HTML fixture + referer assertion | Authorized reference capture |

**Constraint:** No live-site scraping or access-control bypass is part of automated tests. Level C evidence must come from user-authorized test material or a permitted reference test environment.

## Output path and filename behavior

The native engine exposes deterministic implementations for the observed reference rules:

- Folder names replace dot, slash, backslash, pipe, colon, question mark, asterisk, quote, angle brackets and control characters with underscores.
- Optional post ID suffix is appended only when enabled.
- Forum subdirectory and thread sublocation options are independent.
- Detected response MIME determines the final uppercase extension.
- Optional order prefix is one-based and zero-padded to a minimum width of three digits.
- The currently observed source behavior sanitizes forum names but appends thread titles as-is.

New native unit vectors added in this phase cover forbidden characters, Unicode names, optional post ID, MIME-driven extension changes, index boundaries (001, 009, 100, 1000) and independent directory options.

**Still not proven:** Exact parity with the Java build for path separators on each OS, duplicate filename collisions, trailing dots/spaces, reserved Windows names, unusual Unicode normalization, existing-file overwrite policy, temporary-file cleanup after process termination, and all source extension/MIME combinations. These require reference golden vectors and platform acceptance tests.\n\n**Security observation:** the currently documented reference behavior appends the thread title without sanitization. Before release, the implementation must prove that the resolved destination remains inside the configured download root (including `..`, absolute paths, drive prefixes, UNC paths and symlink cases). Compatibility must not permit path traversal; any intentional hardening must be recorded as a deliberate safety deviation from the reference.

## UI and interaction inventory

Previously inspected areas include the primary shell, tabs, menu/action bar/status bar, Add Links, Posts/Threads/Tasks/Logs, Settings sections, selection/removal actions and image workspace.

Still required for Phase 0 sign-off:

- Reference screenshots for each primary screen and important state.
- Interaction recordings or step-by-step golden scripts for keyboard, context menu, list/grid, filtering and dialogs.
- Exact settings defaults, validation messages and persistence behavior confirmed against a runnable Java reference.
- Image workspace navigation, zoom, deep zoom, minimap and cache behavior recorded as observable acceptance criteria.

## Database and migration evidence

The logical entities are mapped in `PERSISTENCE_SCHEMA_AUDIT.md`: POST, IMAGE, THREAD, METADATA, TASK and POST_ACTION, plus Liquibase history tables.

Still required before compatibility sign-off:

- Sanitized real database fixtures from each supported Java schema generation.
- Exact column affinity, nullability, defaults, indexes, foreign keys and enum/state values.
- Before/after row and relationship checks for every entity, including unknown values and raw metadata.
- WAL/SHM snapshot behavior, interrupted import, collision, rollback and source-preservation acceptance.
- Golden comparison of existing image paths, filenames, order, byte counts and download state.

## Phase 0 exit checklist

- [x] Source-level inventory of primary shell and core workflows recorded.
- [x] Source-level inventory of all 16 registered host resolvers recorded.
- [x] Synthetic resolver fixture coverage exists for all 16 host types.
- [x] Synthetic XML parser fixture covers thread/post/image mapping.
- [x] Native output naming/path deterministic unit vectors expanded.
- [ ] Capture reference screenshots and interaction evidence.
- [ ] Capture authorized reference HTML/API fixtures and compare each resolver.
- [ ] Complete Java-vs-native golden vectors for output paths and filenames.
- [ ] Validate real legacy databases across supported schema generations.
- [ ] Complete dependency, asset and license audit.
- [ ] Sign off critical workflow behavioral specification.

## How to record new evidence

For each new comparison, record: evidence level, reference application version, OS, sanitized input fixture identifier, expected output, native output, pass/fail, and a link to the test or capture. Never commit passwords, cookies, tokens, authorization headers, personal account identifiers or private content.
