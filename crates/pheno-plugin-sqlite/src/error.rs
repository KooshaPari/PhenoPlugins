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
}
