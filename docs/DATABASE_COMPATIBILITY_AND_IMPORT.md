# Database compatibility and Java import design

## Decision

VRipper Native will use SQLite as its persistent store and preserve the Java edition's logical schema as the compatibility baseline. The first native database release should open a verified copy of the legacy schema and continue using the original entity/table model rather than prematurely translating records into a new normalized schema.

The application must support two entry paths:

1. **Compatible database mode:** open a database that has been verified as a supported VRipper schema version. The Java database is never opened for writing in place; the user chooses whether to work from a verified native copy or a separately backed-up file.
2. **Import mode:** select the Java vripper.db, inspect it read-only, then create a separate native copy. The original is retained unchanged.

Direct write access to the Java installation's live database is intentionally not the default. A running Java process may have active transactions, WAL state, or Liquibase migrations. Copy-first avoids cross-process locking and accidental corruption.

## Legacy schema contract

The current logical schema audit identifies these tables:

| Table | Purpose | Key compatibility requirements |
|---|---|---|
| POST | Post metadata and download lifecycle | Preserve ID, unique POST_ID, TOKEN, status, counts, byte totals, hosts, output path, folder name, timestamp and rank |
| IMAGE | Per-image download records | Preserve ID, POST_ID_REF, source/thumbnail URLs, host ID, image order, status, sizes, bytes and filename |
| THREAD | Thread catalog | Preserve ID, unique THREAD_ID, title, URL and total count |
| METADATA | Additional post metadata | Preserve POST_ID_REF and raw DATA text without re-serialization |
| TASK | User task/history records | Preserve IDs, category, description, status/result and timestamps, including optional POST_ENTITY_ID |
| POST_ACTION | Post action/review state | Preserve ID, unique post reference, action, state, attempts, errors, missing fields, target path and timestamps |
| DATABASECHANGELOG / DATABASECHANGELOGLOCK | Liquibase migration history/lock | Preserve the complete migration ledger and lock table |

The schema audit is based on source inspection. Exact column affinities, nullability, defaults, indexes, constraints, enum mappings and historical schema variants still require validation against real database copies from supported Java releases. docs/PERSISTENCE_SCHEMA_AUDIT.md is the current logical inventory.

## Import workflow

1. User selects vripper.db (or a supported SQLite file).
2. Open source read-only and run SQLite quick_check.
3. Inspect table names, columns, row counts, Liquibase history and foreign-key violations.
4. Require the core POST, IMAGE and THREAD tables. Report missing tables and do not guess a schema version.
5. Use SQLite's online backup API to create a new staging database. This is preferred over a raw file copy because it can produce a consistent snapshot when WAL is in use.
6. Validate the staging database: integrity, table/column equality, per-table row counts and foreign-key check.
7. Publish the staged file under a new destination name only after all checks pass. Never overwrite an existing destination.
8. Show a report with source/destination paths, table counts, row counts and warnings. Do not include token, cookie, password or other secret values.
9. Retain the original source file. Do not move, delete, rename or reorganize downloaded image files.

If any stage fails, delete only the incomplete staging file and leave both the source and any pre-existing destination untouched.

## Runtime compatibility plan

- Implement a Rust DatabaseStore backed by SQLite, with repositories for posts, images, threads, metadata, tasks and post actions.
- Keep the legacy table and column names at the persistence boundary. Map them to Rust domain structs internally.
- Preserve primary keys and relationships. Do not regenerate IDs during import.
- Preserve unknown metadata fields and unrecognized enum/status values as raw strings until their semantics are explicitly mapped.
- Use transactions for multi-record state transitions and queue updates.
- Enable foreign keys on native connections and use WAL for the native working copy where appropriate.
- Keep Liquibase history intact in imported databases. Native schema changes must use a separate versioned migration ledger or a tested Liquibase-compatible strategy; never mark an unexecuted migration as applied.
- Add migrations as additive changes first. Any destructive migration requires backup, validation, rollback and compatibility tests.
- Keep the importer independent from the downloader so users can inspect/restore a database even if site access is unavailable.

## Security and privacy

- POST.TOKEN and session-related data are sensitive. Preserve them for compatibility, but never print their values in logs, import reports, exceptions or diagnostics.
- Keep database files in the platform application-support directory and apply restrictive file permissions where the OS supports them.
- Do not put database contents in crash telemetry.
- The import UI must clearly identify the source and destination and require explicit user action before copying.
- A database backup contains private history and session material; treat it as sensitive user data.

## Current implementation status

Implemented on the migration branch:
- Rust SQLite dependency with bundled SQLite and online backup support.
- Read-only database inspection API.
- Core-table compatibility check (POST, IMAGE, THREAD).
- Online backup into a new staging file.
- Integrity, table/column and row-count validation before publishing.
- Destination collision protection and source preservation.
- Versioned FFI entry points and Flutter background wrappers.
- Settings → Database UI for selecting, inspecting and importing a Java database copy.

Not yet complete:
- DatabaseStore repositories are not yet connected to the download coordinator.
- The imported database path is saved as a selected database path, but the current posts/threads UI still uses its initial preferences-backed store.
- Runtime reads/writes for all six legacy entities, task history and image lifecycle are pending.
- Real-world fixtures from multiple Java releases and schema migration/rollback tests are pending.
- Android/iOS document-provider edge cases and desktop file-permission tests are pending.

Therefore, the current importer is a safe inspection/copy foundation, not yet a claim of full database runtime compatibility.

## Acceptance tests

- Import a valid Java database and compare every table's columns and row counts.
- Verify every imported primary/unique ID, foreign-key relation, image order, status, byte count, path and filename.
- Verify POST.TOKEN and raw METADATA.DATA survive exactly without appearing in logs.
- Import a database in WAL mode while the Java source is open read-only where supported; verify a consistent snapshot.
- Reject corrupt, unrelated, unsupported and partially migrated databases with actionable diagnostics.
- Refuse to overwrite an existing destination.
- Simulate interruption before publish and confirm the source remains unchanged and no partial destination is presented as complete.
- Open the imported database in the native application and verify Posts, Threads, Tasks, Logs and download continuation against the Java reference.
- Verify that existing downloaded files are not moved, renamed, overwritten or deleted during import.
