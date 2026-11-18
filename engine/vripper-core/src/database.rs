//! Safe inspection and import of the legacy Java application's SQLite database.
//!
//! The source is always opened read-only. Import uses SQLite's online backup API
//! into a new staging file, validates the result, then atomically publishes it.
//! The source database and its WAL state are never modified by this module.

use rusqlite::{backup::Backup, Connection, OpenFlags};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

const REQUIRED_TABLES: &[&str] = &["POST", "IMAGE", "THREAD"];
const REQUIRED_COLUMNS: &[(&str, &[&str])] = &[( "POST", &["ID", "POST_ID", "THREAD_ID", "POST_TITLE", "THREAD_TITLE", "FORUM", "URL", "TOTAL", "DONE", "STATUS", "OUTPUT_PATH", "FOLDER_NAME", "SIZE", "DOWNLOADED", "RANK"]), ("IMAGE", &["ID", "POST_ID_REF", "HOST", "URL", "THUMB_URL", "INDEX", "STATUS", "FILENAME", "DOWNLOADED", "SIZE"]), ("THREAD", &["ID", "THREAD_ID", "TITLE", "URL", "TOTAL"])];
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("database path does not exist: {0}")]
    SourceMissing(String),
    #[error("destination already exists: {0}")]
    DestinationExists(String),
    #[error("source and destination refer to the same file")]
    SameFile,
    #[error("not a compatible VRipper SQLite database; missing required tables: {0}")]
    Incompatible(String),
    #[error("SQLite operation failed: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("filesystem operation failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("database integrity check failed: {0}")]
    Integrity(String),
    #[error("database path has no parent directory")]
    MissingParent,
}

#[derive(Debug, Clone, Serialize)]
pub struct TableReport {
    pub name: String,
    pub row_count: u64,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatabaseReport {
    pub file_name: String,
    pub schema_version: Option<String>,
    pub compatible: bool,
    pub missing_required_tables: Vec<String>,
    pub missing_required_columns: Vec<String>,
    pub tables: Vec<TableReport>,
    pub total_rows: u64,
    pub integrity_check: String,
    pub foreign_key_violations: usize,
    pub has_liquibase_history: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportReport {
    pub source: DatabaseReport,
    pub imported: DatabaseReport,
    pub destination: String,
    pub source_preserved: bool,
    pub verified: bool,
}

fn open_read_only(path: &Path) -> Result<Connection, DatabaseError> {
    if !path.is_file() {
        return Err(DatabaseError::SourceMissing(path.display().to_string()));
    }
    Ok(Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?)
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn integrity_check(connection: &Connection) -> Result<String, DatabaseError> {
    let result: String = connection.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    if !result.eq_ignore_ascii_case("ok") {
        return Err(DatabaseError::Integrity(result));
    }
    Ok(result)
}

fn inspect_connection(path: &Path, connection: &Connection) -> Result<DatabaseReport, DatabaseError> {
    let mut statement = connection.prepare(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name COLLATE NOCASE"
    )?;
    let names = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(statement);

    let normalized: Vec<String> = names.iter().map(|name| name.to_ascii_uppercase()).collect();
    let missing_required_tables: Vec<String> = REQUIRED_TABLES
        .iter()
        .filter(|required| !normalized.iter().any(|name| name == **required))
        .map(|name| (*name).to_owned())
        .collect();

    let mut tables = Vec::with_capacity(names.len());
    let mut missing_required_columns = Vec::new();
    let mut total_rows = 0_u64;
    let mut schema_version = None;
    for name in &names {
        let quoted = quote_identifier(name);
        let count_sql = format!("SELECT COUNT(*) FROM {quoted}");
        let row_count: u64 = connection.query_row(&count_sql, [], |row| row.get(0))?;
        total_rows = total_rows.saturating_add(row_count);

        let pragma = format!("PRAGMA table_info({quoted})");
        let mut columns_statement = connection.prepare(&pragma)?;
        let columns = columns_statement
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?;

        if let Some((_, required_columns)) = REQUIRED_COLUMNS.iter().find(|(table, _)| name.eq_ignore_ascii_case(table)) {
            for required in *required_columns {
                if !columns.iter().any(|column| column.eq_ignore_ascii_case(required)) {
                    missing_required_columns.push(format!("{name}.{required}"));
                }
            }
        }

        if name.eq_ignore_ascii_case("DATABASECHANGELOG") {
            let version_sql = format!(
                "SELECT MAX(ORDEREXECUTED) FROM {quoted}"
            );
            schema_version = connection
                .query_row(&version_sql, [], |row| row.get::<_, Option<i64>>(0))
                .ok()
                .flatten()
                .map(|value| value.to_string());
        }

        tables.push(TableReport {
            name: name.clone(),
            row_count,
            columns,
        });
    }

    let has_liquibase_history = normalized
        .iter()
        .any(|name| name == "DATABASECHANGELOG");
    let mut foreign_key_statement = connection.prepare("PRAGMA foreign_key_check")?;
    let mut foreign_key_rows = foreign_key_statement.query([])?;
    let mut foreign_key_violations = 0_usize;
    while foreign_key_rows.next()?.is_some() {
        foreign_key_violations += 1;
    }

    Ok(DatabaseReport {
        file_name: path.file_name().and_then(|name| name.to_str()).unwrap_or_default().to_owned(),
        schema_version,
        compatible: missing_required_tables.is_empty() && missing_required_columns.is_empty(),
        missing_required_tables,
        missing_required_columns,
        tables,
        total_rows,
        integrity_check: integrity_check(connection)?,
        foreign_key_violations,
        has_liquibase_history,
    })
}

/// Read-only inspection. No schema migrations or writes are performed.
pub fn inspect_legacy_database(path: &Path) -> Result<DatabaseReport, DatabaseError> {
    let connection = open_read_only(path)?;
    inspect_connection(path, &connection)
}

/// Import a Java VRipper database into a new native database file.
///
/// The original file is never modified. The import is rejected if the target
/// exists or the source lacks the core POST/IMAGE/THREAD tables. Every table,
/// index, trigger and Liquibase history row is copied using SQLite backup.
pub fn import_legacy_database(source: &Path, destination: &Path) -> Result<ImportReport, DatabaseError> {
    if !source.is_file() {
        return Err(DatabaseError::SourceMissing(source.display().to_string()));
    }
    if destination.exists() {
        return Err(DatabaseError::DestinationExists(destination.display().to_string()));
    }
    if source.canonicalize().ok() == destination.canonicalize().ok() {
        return Err(DatabaseError::SameFile);
    }

    let source_connection = open_read_only(source)?;
    let source_report = inspect_connection(source, &source_connection)?;
    if !source_report.compatible {
        return Err(DatabaseError::Incompatible(
            source_report.missing_required_tables.join(", "),
        ));
    }

    let parent = destination.parent().ok_or(DatabaseError::MissingParent)?;
    fs::create_dir_all(parent)?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let staging = staging_path(destination, nonce);
    if staging.exists() {
        return Err(DatabaseError::DestinationExists(staging.display().to_string()));
    }

    let copy_result = (|| -> Result<DatabaseReport, DatabaseError> {
        let mut target_connection = Connection::open(&staging)?;
        {
            let backup = Backup::new(&source_connection, &mut target_connection)?;
            backup.run_to_completion(64, Duration::from_millis(10), None)?;
        }
        let imported_report = inspect_connection(&staging, &target_connection)?;
        if !imported_report.compatible {
            return Err(DatabaseError::Incompatible(
                imported_report.missing_required_tables.join(", "),
            ));
        }
        let same_schema = source_report.tables.iter().zip(imported_report.tables.iter()).all(|(source_table, imported_table)| {
            source_table.name.eq_ignore_ascii_case(&imported_table.name)
                && source_table.columns == imported_table.columns
                && source_table.row_count == imported_table.row_count
        });
        if source_report.total_rows != imported_report.total_rows
            || source_report.tables.len() != imported_report.tables.len()
            || !same_schema
            || imported_report.foreign_key_violations != 0
        {
            return Err(DatabaseError::Integrity(
                "source and imported table/row totals differ".to_owned(),
            ));
        }
        target_connection.close().map_err(|(_, error)| DatabaseError::Sqlite(error))?;
        Ok(imported_report)
    })();

    let imported_report = match copy_result {
        Ok(report) => report,
        Err(error) => {
            let _ = fs::remove_file(&staging);
            return Err(error);
        }
    };

    if destination.exists() {
        let _ = fs::remove_file(&staging);
        return Err(DatabaseError::DestinationExists(destination.display().to_string()));
    }
    fs::rename(&staging, destination)?;
    let source_preserved = source.is_file();
    Ok(ImportReport {
        source: source_report,
        imported: imported_report,
        destination: destination.display().to_string(),
        source_preserved,
        verified: source_preserved,
    })
}

fn staging_path(destination: &Path, nonce: u128) -> PathBuf {
    let file_name = destination.file_name().and_then(|name| name.to_str()).unwrap_or("vripper.db");
    destination.with_file_name(format!(".{file_name}.import-{nonce}.tmp"))
}


/// Read the catalog of an imported database without exposing tokens or changing records.
/// The projection uses the original Liquibase column names, not a guessed generic schema.
pub fn read_legacy_catalog(path: &Path) -> Result<serde_json::Value, DatabaseError> {
    let connection = open_read_only(path)?;
    let mut statement = connection.prepare(
        "SELECT ID, POST_ID, THREAD_ID, POST_TITLE, THREAD_TITLE, FORUM, URL, TOTAL, DONE, STATUS, OUTPUT_PATH, FOLDER_NAME, SIZE, DOWNLOADED FROM POST ORDER BY RANK, ID"
    )?;
    let posts = statement.query_map([], |row| {
        Ok(serde_json::json!({
            "legacy_id": row.get::<_, i64>(0)?,
            "post_id": row.get::<_, i64>(1)?,
            "thread_id": row.get::<_, i64>(2)?,
            "title": row.get::<_, String>(3)?,
            "thread_title": row.get::<_, String>(4)?,
            "forum": row.get::<_, String>(5)?,
            "url": row.get::<_, String>(6)?,
            "image_count": row.get::<_, i64>(7)?,
            "done": row.get::<_, i64>(8)?,
            "status": row.get::<_, String>(9)?,
            "output_path": row.get::<_, Option<String>>(10)?,
            "folder_name": row.get::<_, String>(11)?,
            "size": row.get::<_, i64>(12)?,
            "downloaded": row.get::<_, i64>(13)?,
            "legacy": true
        }))
    })?.collect::<Result<Vec<_>, _>>()?;
    let mut statement = connection.prepare(
        "SELECT ID, THREAD_ID, TITLE, URL, TOTAL FROM THREAD ORDER BY ID"
    )?;
    let threads = statement.query_map([], |row| {
        Ok(serde_json::json!({
            "legacy_id": row.get::<_, i64>(0)?,
            "thread_id": row.get::<_, i64>(1)?,
            "title": row.get::<_, String>(2)?,
            "url": row.get::<_, String>(3)?,
            "post_count": row.get::<_, i64>(4)?,
            "legacy": true
        }))
    })?.collect::<Result<Vec<_>, _>>()?;
    let mut statement = connection.prepare("SELECT ID, POST_ID_REF, HOST, \"INDEX\", URL, THUMB_URL, STATUS, SIZE, DOWNLOADED, FILENAME FROM IMAGE ORDER BY POST_ID_REF, \"INDEX\", ID")?;
    let images = statement.query_map([], |row| {
        Ok(serde_json::json!({
            "legacy_id": row.get::<_, i64>(0)?,
            "post_legacy_id": row.get::<_, i64>(1)?,
            "host": row.get::<_, i64>(2)?,
            "index": row.get::<_, i64>(3)?,
            "url": row.get::<_, String>(4)?,
            "thumbnail_url": row.get::<_, String>(5)?,
            "status": row.get::<_, String>(6)?,
            "size": row.get::<_, i64>(7)?,
            "downloaded": row.get::<_, i64>(8)?,
            "filename": row.get::<_, String>(9)?,
            "legacy": true
        }))
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(serde_json::json!({"posts": posts, "threads": threads, "images": images}))
}

    
/// Persist one extracted thread/post/image set into the writable native copy of
/// the legacy Java schema. Existing download state is preserved; only metadata
/// discovered by a fresh lookup is refreshed.
pub fn persist_extracted_post(
    path: &Path,
    thread: &crate::api::ParsedThread,
    post: &crate::api::ParsedPost,
    output_path: &Path,
    folder_name: &str,
) -> Result<(), DatabaseError> {
    let mut connection = Connection::open(path)?;
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
    let transaction = connection.transaction()?;

    let thread_url = format!("{}/threads/{}", "https://vipergirls.to", thread.thread_id);
    transaction.execute(
        r#"INSERT INTO THREAD (THREAD_ID, TITLE, URL, TOTAL)
           VALUES (?1, ?2, ?3, ?4)
           ON CONFLICT(THREAD_ID) DO UPDATE SET TITLE=excluded.TITLE, URL=excluded.URL, TOTAL=excluded.TOTAL"#,
        rusqlite::params![thread.thread_id, thread.title, thread_url, post_count_for_thread(thread)],
    )?;

    let thread_row_id: i64 = transaction.query_row(
        "SELECT ID FROM THREAD WHERE THREAD_ID=?1",
        [thread.thread_id],
        |row| row.get(0),
    )?;

    let existing_post_id: Option<i64> = transaction
        .query_row(
            "SELECT ID FROM POST WHERE POST_ID=?1",
            [post.post_id],
            |row| row.get(0),
        )
        .ok();

    let post_row_id = if let Some(id) = existing_post_id {
        transaction.execute(
            r#"UPDATE POST SET THREAD_ID=?1, POST_TITLE=?2, THREAD_TITLE=?3, FORUM=?4,
               URL=?5, TOTAL=?6, TOKEN=?7, RANK=?8 WHERE ID=?9"#,
            rusqlite::params![
                thread_row_id,
                post.title,
                post.thread_title,
                post.forum,
                post.url,
                post.image_count as i64,
                post.security_token,
                post.post_number,
                id
            ],
        )?;
        id
    } else {
        transaction.execute(
            r#"INSERT INTO POST
               (POST_ID, THREAD_ID, POST_TITLE, THREAD_TITLE, FORUM, URL, TOTAL, DONE,
                STATUS, OUTPUT_PATH, FOLDER_NAME, SIZE, DOWNLOADED, TOKEN, RANK)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, 'PENDING', ?8, ?9, 0, 0, ?10, ?11)"#,
            rusqlite::params![
                post.post_id,
                thread_row_id,
                post.title,
                post.thread_title,
                post.forum,
                post.url,
                post.image_count as i64,
                output_path.to_string_lossy().as_ref(),
                folder_name,
                post.security_token,
                post.post_number
            ],
        )?;
        transaction.last_insert_rowid()
    };

    for (index, image) in post.images.iter().enumerate() {
        let existing_image_id: Option<i64> = transaction
            .query_row(
                r#"SELECT ID FROM IMAGE WHERE POST_ID_REF=?1 AND "INDEX"=?2 ORDER BY ID LIMIT 1"#,
                rusqlite::params![post_row_id, index as i64],
                |row| row.get(0),
            )
            .ok();

        if let Some(image_id) = existing_image_id {
            transaction.execute(
                r#"UPDATE IMAGE SET HOST=?1, URL=?2, THUMB_URL=?3, "INDEX"=?4 WHERE ID=?5"#,
                rusqlite::params![
                    image.host as u8 as i64,
                    image.main_url,
                    image.thumbnail_url,
                    index as i64,
                    image_id
                ],
            )?;
        } else {
            transaction.execute(
                r#"INSERT INTO IMAGE
                   (POST_ID_REF, HOST, URL, THUMB_URL, "INDEX", STATUS, FILENAME, DOWNLOADED, SIZE)
                   VALUES (?1, ?2, ?3, ?4, ?5, 'PENDING', '', 0, 0)"#,
                rusqlite::params![
                    post_row_id,
                    image.host as u8 as i64,
                    image.main_url,
                    image.thumbnail_url,
                    index as i64
                ],
            )?;
        }
    }

    transaction.commit()?;
    Ok(())
}

fn post_count_for_thread(thread: &crate::api::ParsedThread) -> i64 {
    thread.posts.len() as i64
}


/// Native catalog overlay, independent of the preserved Java tables.
pub fn native_catalog(path: &Path, replacement: Option<&str>) -> Result<serde_json::Value, DatabaseError> {
    let mut connection = Connection::open(path)?;
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; CREATE TABLE IF NOT EXISTS NATIVE_CATALOG (ID INTEGER PRIMARY KEY CHECK(ID=1), DATA TEXT NOT NULL)")?;
    if let Some(json) = replacement {
        let value: serde_json::Value = serde_json::from_str(json)
            .map_err(|_| DatabaseError::Integrity("invalid catalog JSON".into()))?;
        if !value["posts"].is_array() || !value["threads"].is_array() {
            return Err(DatabaseError::Integrity("invalid catalog shape".into()));
        }
        let transaction = connection.transaction()?;
        transaction.execute("INSERT INTO NATIVE_CATALOG(ID,DATA) VALUES(1,?1) ON CONFLICT(ID) DO UPDATE SET DATA=excluded.DATA", [json])?;
        transaction.commit()?;
        return Ok(value);
    }
    use rusqlite::OptionalExtension;
    let saved: Option<String> = connection.query_row("SELECT DATA FROM NATIVE_CATALOG WHERE ID=1", [], |row| row.get(0)).optional()?;
    match saved {
        Some(json) => serde_json::from_str(&json).map_err(|_| DatabaseError::Integrity("invalid saved catalog".into())),
        None => Ok(serde_json::Value::Null),
    }
}


#[derive(Debug, Clone, Serialize)]
pub struct LegacyQueueEntry {
    pub image_id: i64,
    pub post_id: i64,
    pub job: crate::queue::QueueJob,
    pub status: crate::queue::QueueStatus,
    pub filename: Option<String>,
    pub bytes_written: u64,
    pub expected_bytes: Option<u64>,
    pub destination_path: Option<String>,
}

/// Import only records from a verified backup; never execute a download on import.
/// Do not guess a destination when Java has not assigned an output directory.
pub fn read_legacy_queue(path: &Path) -> Result<Vec<LegacyQueueEntry>, DatabaseError> {
    use crate::queue::{QueueJob, QueueStatus};
    let connection = open_read_only(path)?;
    let mut statement = connection.prepare(
        r#"SELECT I.ID, P.POST_ID, I.HOST, I.URL, I.THUMB_URL, I."INDEX", I.STATUS, I.FILENAME, I.DOWNLOADED, I.SIZE, P.OUTPUT_PATH, P.FOLDER_NAME FROM IMAGE I JOIN POST P ON P.ID = I.POST_ID_REF ORDER BY P.RANK, P.ID, I."INDEX", I.ID"#
    )?;
    let mut rows = statement.query([])?;
    let mut entries = Vec::new();
    while let Some(row) = rows.next()? {
        let image_id: i64 = row.get(0)?;
        let post_id: i64 = row.get(1)?;
        let host_number: u8 = row.get(2)?;
        let host = crate::hosts::HostId::try_from(host_number).map_err(DatabaseError::Integrity)?;
        let index: usize = row.get(5)?;
        let original_status: String = row.get(6)?;
        let filename: String = row.get(7)?;
        let output: Option<String> = row.get(10)?;
        let folder: String = row.get(11)?;
        let Some(output) = output.filter(|value| !value.trim().is_empty()) else { continue; };
        let folder_path = Path::new(&folder);
        if folder_path.components().count() != 1 || !matches!(folder_path.components().next(), Some(std::path::Component::Normal(_))) {
            return Err(DatabaseError::Integrity("unsafe legacy folder name".into()));
        }
        let directory = PathBuf::from(output).join(folder_path);
        if !directory.is_absolute() { continue; }
        let source_url: String = row.get(3)?;
        let thumbnail_url: String = row.get(4)?;
        if !source_url.starts_with("https://") && !source_url.starts_with("http://") { continue; }
        let status = match original_status.as_str() {
            "FINISHED" => QueueStatus::Finished,
            "ERROR" => QueueStatus::Error,
            "STOPPED" | "PENDING" | "DOWNLOADING" => QueueStatus::Stopped,
            _ => continue,
        };
        let bytes_written: u64 = row.get(8)?;
        let size: u64 = row.get(9)?;
        let valid_filename = !filename.is_empty() && Path::new(&filename).components().count() == 1
            && matches!(Path::new(&filename).components().next(), Some(std::path::Component::Normal(_)));
        let force_order = valid_filename && filename.starts_with(&format!("{:03}_", index.saturating_add(1)));
        let destination_path = valid_filename.then(|| directory.join(&filename).to_string_lossy().into_owned());
        entries.push(LegacyQueueEntry {
            image_id, post_id,
            job: QueueJob { id: 0, host, source_url, thumbnail_url, image_index_zero_based: index, force_order, destination_directory: directory },
            status,
            filename: valid_filename.then_some(filename),
            bytes_written,
            expected_bytes: (size > 0).then_some(size),
            destination_path,
        });
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;




    #[test]
    fn inspection_rejects_core_tables_with_incomplete_columns() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("partial.db");
        let db = Connection::open(&path).unwrap();
        db.execute_batch("CREATE TABLE POST (ID INTEGER, POST_ID INTEGER); CREATE TABLE IMAGE (ID INTEGER, POST_ID_REF INTEGER); CREATE TABLE THREAD (ID INTEGER, THREAD_ID INTEGER);").unwrap();
        drop(db);
        let report = inspect_legacy_database(&path).unwrap();
        assert!(!report.compatible);
        assert!(report.missing_required_columns.contains(&"POST.POST_TITLE".to_string()));
        assert!(report.missing_required_columns.contains(&"IMAGE.HOST".to_string()));
        assert!(report.missing_required_columns.contains(&"THREAD.TITLE".to_string()));
    }

    #[test]
    fn legacy_image_rows_map_to_native_jobs_without_changing_files_or_starting_downloads() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("legacy-images.db");
        let output = temp.path().join("existing-downloads");
        fs::create_dir_all(output.join("post-folder")).unwrap();
        fs::write(output.join("post-folder").join("already.jpg"), b"keep-me").unwrap();
        let db = Connection::open(&path).unwrap();
        db.execute_batch(&format!(
            "CREATE TABLE POST (ID INTEGER PRIMARY KEY, POST_ID INTEGER, OUTPUT_PATH TEXT, FOLDER_NAME TEXT, RANK INTEGER);
             CREATE TABLE IMAGE (ID INTEGER PRIMARY KEY, POST_ID_REF INTEGER, HOST INTEGER, URL TEXT, THUMB_URL TEXT, \"INDEX\" INTEGER, STATUS TEXT, FILENAME TEXT, DOWNLOADED INTEGER, SIZE INTEGER);
             INSERT INTO POST VALUES (7,700,'{}','post-folder',0);
             INSERT INTO IMAGE VALUES
               (11,7,1,'https://dpic.me/a.jpg','https://dpic.me/t.jpg',0,'FINISHED','already.jpg',4,4),
               (12,7,1,'https://dpic.me/b.jpg','https://dpic.me/t2.jpg',1,'DOWNLOADING','',2,10),
               (13,7,1,'https://dpic.me/c.jpg','https://dpic.me/t3.jpg',2,'PENDING','',0,0);",
            output.to_string_lossy().replace('\'', "''")
        )).unwrap();
        drop(db);

        let entries = read_legacy_queue(&path).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].image_id, 11);
        assert_eq!(entries[0].status, crate::queue::QueueStatus::Finished);
        assert_eq!(entries[0].destination_path.as_deref(), Some(output.join("post-folder").join("already.jpg").to_str().unwrap()));
        assert_eq!(entries[1].status, crate::queue::QueueStatus::Stopped);
        assert_eq!(entries[1].bytes_written, 2);
        assert_eq!(entries[2].status, crate::queue::QueueStatus::Stopped);
        assert_eq!(fs::read(output.join("post-folder").join("already.jpg")).unwrap(), b"keep-me");
    }

    #[test]
    fn legacy_queue_rejects_path_traversal_in_folder_names() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("unsafe.db");
        let db = Connection::open(&path).unwrap();
        db.execute_batch("CREATE TABLE POST (ID INTEGER, POST_ID INTEGER, OUTPUT_PATH TEXT, FOLDER_NAME TEXT, RANK INTEGER);
            CREATE TABLE IMAGE (ID INTEGER, POST_ID_REF INTEGER, HOST INTEGER, URL TEXT, THUMB_URL TEXT, \"INDEX\" INTEGER, STATUS TEXT, FILENAME TEXT, DOWNLOADED INTEGER, SIZE INTEGER);
            INSERT INTO POST VALUES (1,1,'/tmp','../escape',0);
            INSERT INTO IMAGE VALUES (1,1,1,'https://dpic.me/a.jpg','',0,'PENDING','',0,0);").unwrap();
        drop(db);
        assert!(read_legacy_queue(&path).is_err());
    }

    #[test]
    fn catalog_projects_reference_columns_without_secrets() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("catalog.db");
        let db = Connection::open(&path).unwrap();
        db.execute_batch("CREATE TABLE POST (ID INTEGER, POST_ID INTEGER, THREAD_ID INTEGER, POST_TITLE TEXT, THREAD_TITLE TEXT, FORUM TEXT, URL TEXT, TOTAL INTEGER, DONE INTEGER, STATUS TEXT, OUTPUT_PATH TEXT, FOLDER_NAME TEXT, SIZE INTEGER, DOWNLOADED INTEGER, RANK INTEGER, TOKEN TEXT);
            CREATE TABLE THREAD (ID INTEGER, THREAD_ID INTEGER, TITLE TEXT, URL TEXT, TOTAL INTEGER);
            CREATE TABLE IMAGE (ID INTEGER, POST_ID_REF INTEGER, HOST INTEGER, \"INDEX\" INTEGER, URL TEXT, THUMB_URL TEXT, STATUS TEXT, SIZE INTEGER, DOWNLOADED INTEGER, FILENAME TEXT);
            INSERT INTO IMAGE VALUES (11,7,1,0,'https://dpic.me/a.jpg','https://dpic.me/t.jpg','FINISHED',10,10,'a.jpg');
            INSERT INTO POST VALUES (7,700,300,'post','thread','forum','https://example.invalid',2,1,'STOPPED',NULL,'folder',100,50,0,'secret-fixture');
            INSERT INTO THREAD VALUES (3,300,'thread','https://example.invalid',1);").unwrap();
        drop(db);
        let catalog = read_legacy_catalog(&path).unwrap();
        assert_eq!(catalog["posts"][0]["post_id"], 700);
        assert_eq!(catalog["posts"][0]["status"], "STOPPED");
        assert_eq!(catalog["threads"][0]["thread_id"], 300);
        assert_eq!(catalog["images"][0]["legacy_id"], 11);
        assert_eq!(catalog["images"][0]["status"], "FINISHED");
        assert!(!catalog.to_string().contains("secret-fixture"));
    }


    #[test]
    fn native_catalog_survives_reopen_and_rejects_invalid_replacement() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("native.db");
        assert!(native_catalog(&path, None).unwrap().is_null());
        let data = r#"{"posts":[{"post_id":12}],"threads":[]}"#;
        native_catalog(&path, Some(data)).unwrap();
        assert!(native_catalog(&path, Some("{}")).is_err());
        assert_eq!(native_catalog(&path, None).unwrap()["posts"][0]["post_id"], 12);
        native_catalog(&path, Some(r#"{"posts":[],"threads":[]}"#)).unwrap();
        assert_eq!(native_catalog(&path, None).unwrap()["posts"].as_array().unwrap().len(), 0);
    }

    fn legacy_fixture(path: &Path) {
        let connection = Connection::open(path).unwrap();
        connection.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE POST (ID INTEGER PRIMARY KEY, POST_ID INTEGER UNIQUE, THREAD_ID INTEGER, POST_TITLE TEXT, THREAD_TITLE TEXT, FORUM TEXT, URL TEXT, TOTAL INTEGER, DONE INTEGER, STATUS TEXT, OUTPUT_PATH TEXT, FOLDER_NAME TEXT, SIZE INTEGER, DOWNLOADED INTEGER, RANK INTEGER, TOKEN TEXT);
             CREATE TABLE IMAGE (ID INTEGER PRIMARY KEY, POST_ID_REF INTEGER, HOST INTEGER, URL TEXT, THUMB_URL TEXT, \"INDEX\" INTEGER, STATUS TEXT, FILENAME TEXT, DOWNLOADED INTEGER, SIZE INTEGER, FOREIGN KEY(POST_ID_REF) REFERENCES POST(ID));
             CREATE TABLE THREAD (ID INTEGER PRIMARY KEY, THREAD_ID INTEGER UNIQUE, TITLE TEXT, URL TEXT, TOTAL INTEGER);
             CREATE TABLE METADATA (POST_ID_REF INTEGER UNIQUE, DATA TEXT);
             CREATE TABLE TASK (ID INTEGER PRIMARY KEY, NAME TEXT, STATUS TEXT);
             CREATE TABLE POST_ACTION (ID INTEGER PRIMARY KEY, POST_ID_REF INTEGER UNIQUE, ACTION TEXT, STATE TEXT);
             CREATE TABLE DATABASECHANGELOG (ID TEXT, AUTHOR TEXT, FILENAME TEXT, ORDEREXECUTED INTEGER);
             INSERT INTO POST VALUES (7, 700, 300, 'fixture post', 'fixture thread', 'forum', 'https://example.invalid/posts/700', 1, 0, 'STOPPED', '/tmp', 'fixture-folder', 100, 0, 0, 'sensitive-token');
             INSERT INTO IMAGE VALUES (11, 7, 1, 'https://dpic.me/fixture.jpg', 'https://dpic.me/thumb.jpg', 0, 'FINISHED', 'fixture.jpg', 100, 100);
             INSERT INTO THREAD VALUES (3, 300, 'fixture thread', 'https://example.invalid/threads/300', 1);
             INSERT INTO METADATA VALUES (7, '{\"poster\":\"fixture\"}');
             INSERT INTO TASK VALUES (1, 'fixture task', 'DONE');
             INSERT INTO POST_ACTION VALUES (1, 7, 'REVIEW', 'DONE');
             INSERT INTO DATABASECHANGELOG VALUES ('001', 'author', 'changelog.xml', 1);"
        ).unwrap();
    }

    #[test]
    fn inspect_reports_legacy_tables_and_counts_without_mutating_source() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("vripper.db");
        legacy_fixture(&source);

        let report = inspect_legacy_database(&source).unwrap();
        assert!(report.compatible);
        assert!(report.has_liquibase_history);
        assert_eq!(report.total_rows, 7);
        assert_eq!(report.foreign_key_violations, 0);
        assert!(report.tables.iter().any(|table| table.name == "POST" && table.row_count == 1));
        assert!(source.exists());
    }

    #[test]
    fn import_copies_all_tables_and_preserves_original_database() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("legacy.db");
        let destination = temp.path().join("native").join("vripper.db");
        legacy_fixture(&source);

        let report = import_legacy_database(&source, &destination).unwrap();
        assert!(report.verified);
        assert!(report.source_preserved);
        assert!(destination.exists());
        assert_eq!(report.source.total_rows, report.imported.total_rows);
        assert_eq!(report.source.tables.len(), report.imported.tables.len());

        let imported = Connection::open(&destination).unwrap();
        let token: String = imported.query_row("SELECT TOKEN FROM POST WHERE ID=7", [], |row| row.get(0)).unwrap();
        assert_eq!(token, "sensitive-token");
    }

    #[test]
    fn import_refuses_to_overwrite_an_existing_destination() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("legacy.db");
        let destination = temp.path().join("native.db");
        legacy_fixture(&source);
        fs::write(&destination, b"keep existing file").unwrap();

        let error = import_legacy_database(&source, &destination).unwrap_err();
        assert!(matches!(error, DatabaseError::DestinationExists(_)));
        assert_eq!(fs::read(&destination).unwrap(), b"keep existing file");
    }

    #[test]
    fn import_rejects_unrelated_sqlite_database() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("other.db");
        let connection = Connection::open(&source).unwrap();
        connection.execute_batch("CREATE TABLE unrelated (id INTEGER PRIMARY KEY);").unwrap();
        drop(connection);

        let error = import_legacy_database(&source, &temp.path().join("out.db")).unwrap_err();
        assert!(matches!(error, DatabaseError::Incompatible(_)));
    }
}
