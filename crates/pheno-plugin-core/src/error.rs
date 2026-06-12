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
    use std::error::Error;

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

    #[test]
    fn test_plugin_error_debug_for_all_variants() {
        // Each row is (variant-constructor-call, expected-variant-tag-substring-in-Debug).
        // Debug output for an enum variant starts with the variant name, so each tag
        // must appear in the formatted `{:?}` output.
        let bad: serde_json::Error = serde_json::from_str::<i32>("bad").unwrap_err();
        let io = std::io::Error::new(std::io::ErrorKind::Other, "x");

        let cases: Vec<(PluginError, &str)> = vec![
            (PluginError::Initialization("x".into()), "Initialization"),
            (PluginError::NotFound("x".into()), "NotFound"),
            (
                PluginError::AlreadyRegistered("x".into()),
                "AlreadyRegistered",
            ),
            (PluginError::AlreadyExists("x".into()), "AlreadyExists"),
            (PluginError::Operation("x".into()), "Operation"),
            (PluginError::Config("x".into()), "Config"),
            (PluginError::Io(io), "Io"),
            (PluginError::Serialization(bad), "Serialization"),
            (PluginError::Execution("x".into()), "Execution"),
            (PluginError::Validation("x".into()), "Validation"),
        ];

        for (err, tag) in cases {
            let dbg = format!("{:?}", err);
            assert!(
                dbg.contains(tag),
                "Debug output `{}` should contain variant name `{}`",
                dbg,
                tag
            );
        }
    }

    #[test]
    fn test_plugin_error_source_chain_io() {
        // `#[from] std::io::Error` should yield a non-None source chain.
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "x");
        let e: PluginError = io.into();
        assert!(e.source().is_some());
    }

    #[test]
    fn test_plugin_error_source_chain_serde() {
        // `#[from] serde_json::Error` should yield a non-None source chain.
        let s: serde_json::Error = serde_json::from_str::<i32>("bad").unwrap_err();
        let e: PluginError = s.into();
        assert!(e.source().is_some());
    }

    #[test]
    fn test_plugin_error_source_for_custom_variant() {
        // Custom (non-`#[from]`) variants have no underlying source.
        let e = PluginError::Validation("x".into());
        assert!(e.source().is_none());
    }

    #[test]
    fn test_plugin_error_send_sync() {
        // Compile-time assertion: PluginError must be Send + Sync.
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PluginError>();
    }

    #[test]
    fn test_plugin_error_display_all_variants_unique() {
        // Every variant should produce a distinct Display string. Each `#[error(...)]`
        // template starts with a unique prefix, so the rendered output must be unique.
        let bad: serde_json::Error = serde_json::from_str::<i32>("bad").unwrap_err();
        let io = std::io::Error::new(std::io::ErrorKind::Other, "io_unique_payload");

        let variants: Vec<PluginError> = vec![
            PluginError::Initialization("init_unique_payload".into()),
            PluginError::NotFound("notfound_unique_payload".into()),
            PluginError::AlreadyRegistered("reg_unique_payload".into()),
            PluginError::AlreadyExists("exists_unique_payload".into()),
            PluginError::Operation("op_unique_payload".into()),
            PluginError::Config("cfg_unique_payload".into()),
            PluginError::Io(io),
            PluginError::Serialization(bad),
            PluginError::Execution("exec_unique_payload".into()),
            PluginError::Validation("val_unique_payload".into()),
        ];

        let original: Vec<String> = variants.iter().map(|e| format!("{}", e)).collect();
        let mut sorted = original.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            original.len(),
            "Expected all {} variants to have distinct Display output, but found duplicates among: {:?}",
            original.len(),
            original
        );
    }

    #[test]
    fn test_plugin_result_alias_with_string() {
        // `PluginResult<T>` should work for any T, including owned `String`.
        let r: PluginResult<String> = Ok("hello".into());
        assert_eq!(r.unwrap(), "hello");
    }

    #[test]
    fn test_plugin_result_alias_with_vec() {
        // `PluginResult<T>` should work for any T, including heap-allocated `Vec<u8>`.
        let r: PluginResult<Vec<u8>> = Ok(vec![1, 2, 3]);
        assert_eq!(r.unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn test_plugin_error_io_preserves_error_kind() {
        // When converting via `#[from]`, the inner `std::io::Error` (and thus its
        // `ErrorKind`) must be preserved on the wrapped variant.
        let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "x");
        let e: PluginError = io.into();
        if let PluginError::Io(inner) = e {
            assert_eq!(inner.kind(), std::io::ErrorKind::PermissionDenied);
        } else {
            panic!("expected Io variant");
        }
    }

    #[test]
    fn test_plugin_error_via_map_err() {
        // `.map_err(|e| e.into())` should cleanly convert any `std::io::Error`
        // source into a `PluginError`.
        let r: Result<i32, std::io::Error> =
            Err(std::io::Error::new(std::io::ErrorKind::Other, "x"));
        let mapped: PluginResult<i32> = r.map_err(|e| e.into());
        assert!(mapped.is_err());
    }
}
