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
        // String-bearing variants: each row is (variant, unique payload, recognizable keyword).
        let cases: Vec<(PluginError, &str, &str)> = vec![
            (
                PluginError::Initialization("init_payload".to_string()),
                "init_payload",
                "initialization",
            ),
            (
                PluginError::NotFound("notfound_payload".to_string()),
                "notfound_payload",
                "not found",
            ),
            (
                PluginError::AlreadyRegistered("reg_payload".to_string()),
                "reg_payload",
                "already registered",
            ),
            (
                PluginError::AlreadyExists("exists_payload".to_string()),
                "exists_payload",
                "already exists",
            ),
            (
                PluginError::Operation("op_payload".to_string()),
                "op_payload",
                "operation",
            ),
            (
                PluginError::Config("cfg_payload".to_string()),
                "cfg_payload",
                "configuration",
            ),
            (
                PluginError::Execution("exec_payload".to_string()),
                "exec_payload",
                "execution",
            ),
            (
                PluginError::Validation("val_payload".to_string()),
                "val_payload",
                "validation",
            ),
        ];

        for (err, payload, keyword) in cases {
            let displayed = format!("{}", err);
            let lower = displayed.to_lowercase();
            assert!(
                displayed.contains(payload),
                "Display for {:?} missing payload `{}`: `{}`",
                err,
                payload,
                displayed
            );
            assert!(
                lower.contains(keyword),
                "Display for {:?} missing keyword `{}`: `{}`",
                err,
                keyword,
                displayed
            );
        }

        // `#[from]` variants: build via the From conversion and verify Display is
        // non-empty and preserves the inner error's text.
        let io_err: PluginError =
            std::io::Error::new(std::io::ErrorKind::NotFound, "io_inner_text").into();
        let io_displayed = format!("{}", io_err);
        assert!(!io_displayed.is_empty(), "Io Display should not be empty");
        assert!(
            io_displayed.contains("io_inner_text"),
            "Io Display should contain inner text: `{}`",
            io_displayed
        );

        let bad_json: serde_json::Error =
            serde_json::from_str::<i32>("{ not valid json").unwrap_err();
        let inner_text = bad_json.to_string();
        let ser_err: PluginError = bad_json.into();
        let ser_displayed = format!("{}", ser_err);
        assert!(
            !ser_displayed.is_empty(),
            "Serialization Display should not be empty"
        );
        assert!(
            ser_displayed.contains(&inner_text),
            "Serialization Display should contain inner serde error text: `{}`",
            ser_displayed
        );
    }

    #[test]
    fn test_plugin_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "x");
        let err: PluginError = io_err.into();
        assert!(matches!(err, PluginError::Io(_)));
        assert!(std::error::Error::source(&err).is_some());
    }

    #[test]
    fn test_plugin_error_from_serde() {
        let bad: serde_json::Error = serde_json::from_str::<i32>("{ not valid json").unwrap_err();
        let err: PluginError = bad.into();
        assert!(matches!(err, PluginError::Serialization(_)));
        assert!(std::error::Error::source(&err).is_some());
    }

    #[test]
    fn test_plugin_result_alias() {
        let r: PluginResult<i64> = Ok(42);
        assert_eq!(r.unwrap(), 42);

        let r: PluginResult<()> = Err(PluginError::Validation("bad".into()));
        assert!(r.is_err());
    }
}
