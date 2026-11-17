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
    if result.to_ascii_lowercase() != "ok" {
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
        compatible: missing_required_tables.is_empty(),
        missing_required_tables,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn legacy_fixture(path: &Path) {
        let connection = Connection::open(path).unwrap();
        connection.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE POST (ID INTEGER PRIMARY KEY, POST_ID INTEGER UNIQUE, TITLE TEXT, TOKEN TEXT);
             CREATE TABLE IMAGE (ID INTEGER PRIMARY KEY, POST_ID_REF INTEGER, IMAGE_INDEX INTEGER, STATUS TEXT, FOREIGN KEY(POST_ID_REF) REFERENCES POST(ID));
             CREATE TABLE THREAD (ID INTEGER PRIMARY KEY, THREAD_ID INTEGER UNIQUE, TITLE TEXT, URL TEXT);
             CREATE TABLE METADATA (POST_ID_REF INTEGER UNIQUE, DATA TEXT);
             CREATE TABLE TASK (ID INTEGER PRIMARY KEY, NAME TEXT, STATUS TEXT);
             CREATE TABLE POST_ACTION (ID INTEGER PRIMARY KEY, POST_ID_REF INTEGER UNIQUE, ACTION TEXT, STATE TEXT);
             CREATE TABLE DATABASECHANGELOG (ID TEXT, AUTHOR TEXT, FILENAME TEXT, ORDEREXECUTED INTEGER);
             INSERT INTO POST VALUES (7, 700, 'fixture post', 'sensitive-token');
             INSERT INTO IMAGE VALUES (11, 7, 0, 'FINISHED');
             INSERT INTO THREAD VALUES (3, 300, 'fixture thread', 'https://example.invalid/threads/300');
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
