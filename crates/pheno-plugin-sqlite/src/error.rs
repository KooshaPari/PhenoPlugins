//! SQLite plugin error types.

use thiserror::Error;

/// SQLite-specific error type.
#[derive(Debug, Error)]
pub enum SqliteError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Connection error: {0}")]
    Connection(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_error_display() {
        // Migration variant
        let migration = SqliteError::Migration("schema_v2".to_string());
        let s = migration.to_string();
        assert!(
            s.contains("Migration"),
            "Migration display should contain 'Migration': {s}"
        );
        assert!(
            s.contains("schema_v2"),
            "Migration display should contain 'schema_v2': {s}"
        );

        // Connection variant
        let connection = SqliteError::Connection("db locked".to_string());
        let s = connection.to_string();
        assert!(
            s.contains("Connection"),
            "Connection display should contain 'Connection': {s}"
        );
        assert!(
            s.contains("db locked"),
            "Connection display should contain 'db locked': {s}"
        );

        // Sqlite variant (converted from rusqlite::Error)
        let sqlite_err: SqliteError = rusqlite::Error::InvalidQuery.into();
        let s = sqlite_err.to_string();
        assert!(!s.is_empty(), "Sqlite display should be non-empty");
        assert!(
            s.contains("SQLite"),
            "Sqlite display should contain 'SQLite': {s}"
        );

        // Json variant (converted from serde_json::Error)
        let json_err: SqliteError = serde_json::from_str::<i32>("bad").unwrap_err().into();
        let s = json_err.to_string();
        assert!(!s.is_empty(), "Json display should be non-empty");
        assert!(
            s.contains("JSON"),
            "Json display should contain 'JSON': {s}"
        );
    }

    #[test]
    fn test_sqlite_error_from_sqlite() {
        let rusqlite_err = rusqlite::Error::InvalidQuery;
        let err: SqliteError = rusqlite_err.into();
        assert!(matches!(err, SqliteError::Sqlite(_)));
    }

    #[test]
    fn test_sqlite_error_from_json() {
        let serde_err = serde_json::from_str::<i32>("bad").unwrap_err();
        let err: SqliteError = serde_err.into();
        assert!(matches!(err, SqliteError::Json(_)));
    }

    #[test]
    fn test_sqlite_error_from_real_rusqlite_errors() {
        // NoRows variant
        let err: SqliteError = rusqlite::Error::QueryReturnedNoRows.into();
        assert!(matches!(err, SqliteError::Sqlite(_)));

        // InvalidQuery variant
        let err: SqliteError = rusqlite::Error::InvalidQuery.into();
        assert!(matches!(err, SqliteError::Sqlite(_)));

        // InvalidColumnIndex variant (rusqlite 0.40: InvalidParameterName has a
        // different signature than in older versions, so we use a stable variant
        // with a clear single-arg constructor).
        let err: SqliteError = rusqlite::Error::InvalidColumnIndex(0).into();
        assert!(matches!(err, SqliteError::Sqlite(_)));
    }

    #[test]
    fn test_sqlite_error_from_real_serde_errors() {
        // Parse error
        let parse_err = serde_json::from_str::<i32>("not a number").unwrap_err();
        let err: SqliteError = parse_err.into();
        assert!(matches!(err, SqliteError::Json(_)));

        // Type error
        let type_err = serde_json::from_str::<Vec<i64>>("\"string not array\"").unwrap_err();
        let err: SqliteError = type_err.into();
        assert!(matches!(err, SqliteError::Json(_)));
    }

    #[test]
    fn test_sqlite_error_source_chain() {
        use std::error::Error;

        // Sqlite variant: source() should be reachable and downcast to rusqlite::Error.
        let rusqlite_err = rusqlite::Error::InvalidQuery;
        let err = SqliteError::Sqlite(rusqlite_err);
        let source = err
            .source()
            .expect("SqliteError::Sqlite should expose an inner source");
        assert!(
            source.downcast_ref::<rusqlite::Error>().is_some(),
            "source() of SqliteError::Sqlite should downcast to rusqlite::Error"
        );

        // Json variant: source() should be reachable and downcast to serde_json::Error.
        let serde_err = serde_json::from_str::<i32>("bad").unwrap_err();
        let err = SqliteError::Json(serde_err);
        let source = err
            .source()
            .expect("SqliteError::Json should expose an inner source");
        assert!(
            source.downcast_ref::<serde_json::Error>().is_some(),
            "source() of SqliteError::Json should downcast to serde_json::Error"
        );

        // Migration and Connection have no #[from], so they have no source.
        assert!(SqliteError::Migration("x".to_string()).source().is_none());
        assert!(SqliteError::Connection("y".to_string()).source().is_none());
    }

    #[test]
    fn test_sqlite_error_migration_empty_string() {
        let err = SqliteError::Migration("".to_string());
        let s = err.to_string();
        assert!(
            !s.is_empty(),
            "Migration display with empty message should still be non-empty"
        );
        assert!(
            s.contains("Migration"),
            "Migration display should contain 'Migration': {s}"
        );
    }

    #[test]
    fn test_sqlite_error_debug_includes_variant_name() {
        let sqlite = SqliteError::Sqlite(rusqlite::Error::InvalidQuery);
        assert!(
            format!("{sqlite:?}").contains("Sqlite"),
            "Debug of Sqlite variant should contain 'Sqlite': {sqlite:?}"
        );

        let json = SqliteError::Json(serde_json::from_str::<i32>("bad").unwrap_err());
        assert!(
            format!("{json:?}").contains("Json"),
            "Debug of Json variant should contain 'Json': {json:?}"
        );

        let migration = SqliteError::Migration("x".to_string());
        assert!(
            format!("{migration:?}").contains("Migration"),
            "Debug of Migration variant should contain 'Migration': {migration:?}"
        );

        let connection = SqliteError::Connection("y".to_string());
        assert!(
            format!("{connection:?}").contains("Connection"),
            "Debug of Connection variant should contain 'Connection': {connection:?}"
        );
    }
}
