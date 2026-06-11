//! Plugin error types.

use thiserror::Error;

/// Errors that can occur in plugin operations.
#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Plugin initialization failed: {0}")]
    Initialization(String),

    #[error("Plugin `{0}` not found in registry")]
    NotFound(String),

    #[error("Plugin `{0}` already registered")]
    AlreadyRegistered(String),

    #[error("Entity already exists: {0}")]
    AlreadyExists(String),

    #[error("Operation failed: {0}")]
    Operation(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Plugin execution error: {0}")]
    Execution(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

/// Result type alias for plugin operations.
pub type PluginResult<T> = Result<T, PluginError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_error_display() {
        assert_eq!(
            PluginError::Initialization("boot".to_string()).to_string(),
            "Plugin initialization failed: boot"
        );
        assert_eq!(
            PluginError::NotFound("git".to_string()).to_string(),
            "Plugin `git` not found in registry"
        );
        assert_eq!(
            PluginError::AlreadyRegistered("git".to_string()).to_string(),
            "Plugin `git` already registered"
        );
        assert_eq!(
            PluginError::AlreadyExists("foo".to_string()).to_string(),
            "Entity already exists: foo"
        );
        assert_eq!(
            PluginError::Operation("merge".to_string()).to_string(),
            "Operation failed: merge"
        );
        assert_eq!(
            PluginError::Config("bad".to_string()).to_string(),
            "Configuration error: bad"
        );
        assert_eq!(
            PluginError::Execution("panic".to_string()).to_string(),
            "Plugin execution error: panic"
        );
        assert_eq!(
            PluginError::Validation("missing slug".to_string()).to_string(),
            "Validation error: missing slug"
        );

        let io_err: PluginError =
            std::io::Error::new(std::io::ErrorKind::NotFound, "file").into();
        assert!(
            io_err.to_string().starts_with("IO error:"),
            "expected display to start with 'IO error:', got: {}",
            io_err
        );

        let ser_err: PluginError = serde_json::from_str::<i32>("nope").unwrap_err().into();
        assert!(
            ser_err.to_string().starts_with("Serialization error:"),
            "expected display to start with 'Serialization error:', got: {}",
            ser_err
        );
    }

    #[test]
    fn test_plugin_result_alias() {
        let r: PluginResult<i32> = Ok(42);
        assert_eq!(r.unwrap(), 42);

        let r: PluginResult<i32> = Err(PluginError::Validation("bad".to_string()));
        assert!(r.is_err());
    }
}
