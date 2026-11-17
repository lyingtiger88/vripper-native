# Persistence Schema Audit

Reference source: `vripper-core/src/main/resources/db.changelog-master.xml`, entities and repositories. This is a logical schema inventory; exact SQL type affinity, timestamp conversion, repository query behavior and migration edge cases still need validation against a real copy of the legacy database.

## Database location and migration technology

- Database filename: `vripper.db`.
- Location: VRipper application data directory.
- Schema updates: Liquibase master changelog.
- Native migration must copy the original database before opening it, inspect Liquibase history, validate row counts/relationships, and preserve the original file unchanged until acceptance.

## Logical tables

### POST
- Primary key: ID (auto-increment integer).
- Unique external post ID: POST_ID.
- Thread ID, post title, thread title, forum, URL, token.
- Status, done count, total image count, total size and downloaded bytes.
- Hosts collection, output path, folder name, added timestamp, rank.
- Important: TOKEN is persisted and must be treated as sensitive session/security material.

### IMAGE
- Primary key: ID.
- Parent reference: POST_ID_REF → POST(ID).
- Source URL, thumbnail URL, host numeric ID, per-post image index.
- Status, expected size, downloaded bytes and final filename.
- Changelog initially created IMAGE.POST_ID, then later dropped it; the current logical relation is POST_ID_REF.

### THREAD
- Primary key: ID.
- Unique external THREAD_ID.
- Title, URL and total count.

### METADATA
- POST_ID_REF (unique) and DATA text.
- DATA serializes metadata including poster, resolved names and custom fields.

### TASK
- Primary key: ID.
- Name, category, description, status, result, created/updated timestamps.
- Optional post association exists in TaskEntity; verify whether current changelog adds that column in a later migration or whether the entity/schema have drifted.

### POST_ACTION
- Primary key: ID.
- Unique POST_ID_REF → POST(ID).
- Action, state, attempts, last error, missing fields, target path, created/updated timestamps.

## Indexes observed

- IMAGE.STATUS
- POST.POST_ID
- POST.STATUS
- THREAD.THREAD_ID
- TASK.STATUS
- TASK.CREATED_AT
- POST_ACTION.STATE

The changelog creates and later drops IMAGE.POST_ID index when the redundant IMAGE.POST_ID column is dropped.

## Entity status values

Image/Post status:
- PENDING
- DOWNLOADING
- FINISHED
- ERROR
- STOPPED

PostAction state:
- NONE
- AWAITING_REVIEW
- READY
- RUNNING
- DONE
- FAILED
- SKIPPED

Task states/categories require their enum definitions to be mapped before migration.

## Data migration invariants

- Preserve primary IDs and unique external IDs.
- Preserve every image's original order index, host ID, source URL, thumbnail URL, status, byte counts and filename.
- Preserve post folder/path fields exactly; do not automatically rename or reorganize existing files.
- Preserve thread IDs and source URLs.
- Preserve metadata JSON as raw data before any decoding/re-encoding.
- Preserve task/action state and timestamps.
- Preserve Liquibase changelog history or translate it to a versioned native migration ledger.
- Treat TOKEN, cookies, Viper auth credentials and proxy credentials as secrets; do not expose them in logs or diagnostics.
- Run migration on a backup copy and produce a verifiable before/after report.
