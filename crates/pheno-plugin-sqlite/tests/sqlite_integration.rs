//! End-to-end integration tests for `SqliteStoragePlugin`.
//!
//! These tests live in a separate `tests/` directory (rather than
//! `mod tests` inside the crate) so they exercise the *public* API of
//! the plugin and validate cross-method interactions, persistence, and
//! the full lifecycle (create, migrate, read, write, audit, dispose) the
//! way a downstream consumer would.
//!
//! No new dependencies are introduced — only items already declared in
//! `Cargo.toml` (`rusqlite`, `serde_json`, `tokio`, `tokio-test`) plus
//! the standard library.

use std::path::PathBuf;

use pheno_plugin_core::error::PluginError;
use pheno_plugin_core::traits::{AdapterPlugin, StoragePlugin};
use pheno_plugin_sqlite::SqliteStoragePlugin;

/// Build a unique filesystem path under the system temp directory for
/// tests that need a real on-disk SQLite database. The `label` makes the
/// file easy to identify in `/tmp` if a test ever panics before cleaning
/// up.
fn unique_db_path(label: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("sqlite-int-{}-{}-{}.db", label, pid, nanos))
}

/// Build a default `PluginConfig` for `initialize()` calls.
fn default_config() -> pheno_plugin_core::traits::PluginConfig {
    pheno_plugin_core::traits::PluginConfig {
        name: "sqlite-storage".to_string(),
        version: "0.1.0".to_string(),
        adapter_config: serde_json::json!({}),
    }
}

// =============================================================================
// 1. Full plugin lifecycle
// =============================================================================

#[test]
fn test_full_plugin_lifecycle() {
    // Create plugin from a temp file path, initialize, create a feature,
    // update state, append an audit entry, get the audit trail, list all
    // features, dispose. Verify all data is consistent end-to-end.
    let path = unique_db_path("lifecycle");

    let plugin = SqliteStoragePlugin::new(&path).expect("SqliteStoragePlugin::new should succeed");
    plugin
        .initialize(default_config())
        .expect("initialize should succeed against a fresh file-backed plugin");

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        // -- Create a feature --
        let feature = serde_json::json!({
            "slug": "lifecycle-feature",
            "name": "Lifecycle Feature",
            "description": "end-to-end lifecycle check"
        });
        let id = plugin
            .create_feature(&feature)
            .await
            .expect("create_feature should succeed");
        assert!(id > 0, "created feature id should be positive");

        // -- Update state --
        plugin
            .update_feature_state(id, "active")
            .await
            .expect("update_feature_state should succeed");

        // -- Verify state via get_feature_by_id --
        let by_id = plugin
            .get_feature_by_id(id)
            .await
            .expect("get_feature_by_id should succeed")
            .expect("feature should exist after create");
        assert_eq!(by_id["state"], "active");
        assert_eq!(by_id["slug"], "lifecycle-feature");
        assert_eq!(by_id["name"], "Lifecycle Feature");

        // -- Append an audit entry --
        let entry = serde_json::json!({
            "feature_id": id,
            "entry_type": "state_changed",
            "actor": "lifecycle-test",
            "details": "draft -> active"
        });
        let audit_id = plugin
            .append_audit_entry(&entry)
            .await
            .expect("append_audit_entry should succeed");
        assert!(audit_id > 0);

        // -- Get the audit trail --
        let trail = plugin
            .get_audit_trail(id)
            .await
            .expect("get_audit_trail should succeed");
        assert_eq!(trail.len(), 1, "should have exactly one audit entry");
        assert_eq!(trail[0]["entry_type"], "state_changed");
        assert_eq!(trail[0]["actor"], "lifecycle-test");
        assert_eq!(trail[0]["feature_id"], id);

        // -- List all features --
        let all = plugin
            .list_all_features()
            .await
            .expect("list_all_features should succeed");
        assert_eq!(all.len(), 1, "exactly one feature should be listed");
        assert_eq!(all[0]["id"], id);
        assert_eq!(all[0]["state"], "active");
    });

    // -- Dispose (drop the plugin, releasing the file handle) --
    drop(plugin);
    let _ = std::fs::remove_file(&path);
}

// =============================================================================
// 2. Persistence across plugin instances
// =============================================================================

#[test]
fn test_persistence_across_plugin_instances() {
    // Create plugin A at a temp file path, create a feature, drop A,
    // create plugin B at the same path. Verify the feature still exists
    // in plugin B (validates that migrations, WAL, and FKs all persist).
    let path = unique_db_path("persist");

    // Phase 1: plugin A creates a feature and an audit entry, then is
    // dropped (which flushes the WAL and releases the file handle).
    {
        let plugin_a = SqliteStoragePlugin::new(&path).expect("plugin A: new should succeed");
        plugin_a.initialize(default_config()).expect("plugin A: initialize should succeed");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let feature = serde_json::json!({
                "slug": "persist-feature",
                "name": "Persist Feature",
                "description": "must survive a reopen"
            });
            let id = plugin_a
                .create_feature(&feature)
                .await
                .expect("plugin A: create_feature should succeed");
            assert!(id > 0);

            // Attach an audit entry as well so we exercise multiple tables
            // across the reopen.
            let entry = serde_json::json!({
                "feature_id": id,
                "entry_type": "created",
                "actor": "plugin-a",
                "details": "phase 1"
            });
            plugin_a
                .append_audit_entry(&entry)
                .await
                .expect("plugin A: append_audit_entry should succeed");
        });
        // Drop plugin A here so the WAL is checkpointed and the file is
        // released before plugin B opens it.
    }

    // Phase 2: plugin B reopens the same file. Migrations should be a
    // no-op (tables already exist), and the feature + audit entry should
    // still be present.
    {
        let plugin_b = SqliteStoragePlugin::new(&path).expect("plugin B: new should succeed");
        plugin_b.initialize(default_config()).expect("plugin B: initialize should succeed");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let retrieved = plugin_b
                .get_feature_by_slug("persist-feature")
                .await
                .expect("plugin B: get_feature_by_slug should succeed")
                .expect("the persisted feature should still be present after reopen");
            assert_eq!(retrieved["name"], "Persist Feature");
            assert_eq!(retrieved["state"], "draft");
            assert_eq!(retrieved["description"], "must survive a reopen");

            // Verify the audit entry persisted too.
            let id = retrieved["id"].as_i64().expect("id should be i64");
            let trail = plugin_b
                .get_audit_trail(id)
                .await
                .expect("plugin B: get_audit_trail should succeed");
            assert_eq!(trail.len(), 1);
            assert_eq!(trail[0]["actor"], "plugin-a");

            // list_all_features should also return it.
            let all = plugin_b
                .list_all_features()
                .await
                .expect("plugin B: list_all_features should succeed");
            assert_eq!(all.len(), 1);
            assert_eq!(all[0]["slug"], "persist-feature");
        });
        drop(plugin_b);
    }

    let _ = std::fs::remove_file(&path);
}

// =============================================================================
// 3. Audit trail isolation between features
// =============================================================================

#[test]
fn test_audit_trail_isolation_between_features() {
    // Create 2 features F1 and F2. Add 2 audit entries to F1 and 3 to F2.
    // Verify get_audit_trail(F1).len() == 2 and get_audit_trail(F2).len() == 3.
    let plugin = SqliteStoragePlugin::in_memory().expect("in_memory should succeed");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let f1 = serde_json::json!({
            "slug": "f1",
            "name": "Feature One"
        });
        let f2 = serde_json::json!({
            "slug": "f2",
            "name": "Feature Two"
        });
        let f1_id = plugin
            .create_feature(&f1)
            .await
            .expect("create f1 should succeed");
        let f2_id = plugin
            .create_feature(&f2)
            .await
            .expect("create f2 should succeed");
        assert_ne!(f1_id, f2_id);

        // 2 entries for F1
        for actor in ["alice", "bob"].iter() {
            let entry = serde_json::json!({
                "feature_id": f1_id,
                "entry_type": "f1-event",
                "actor": actor,
                "details": format!("f1-by-{}", actor)
            });
            plugin
                .append_audit_entry(&entry)
                .await
                .expect("append f1 entry should succeed");
        }
        // 3 entries for F2
        for actor in ["carol", "dave", "eve"].iter() {
            let entry = serde_json::json!({
                "feature_id": f2_id,
                "entry_type": "f2-event",
                "actor": actor,
                "details": format!("f2-by-{}", actor)
            });
            plugin
                .append_audit_entry(&entry)
                .await
                .expect("append f2 entry should succeed");
        }

        let f1_trail = plugin
            .get_audit_trail(f1_id)
            .await
            .expect("get_audit_trail(f1) should succeed");
        let f2_trail = plugin
            .get_audit_trail(f2_id)
            .await
            .expect("get_audit_trail(f2) should succeed");

        assert_eq!(
            f1_trail.len(),
            2,
            "F1 should have exactly 2 audit entries, got {}",
            f1_trail.len()
        );
        assert_eq!(
            f2_trail.len(),
            3,
            "F2 should have exactly 3 audit entries, got {}",
            f2_trail.len()
        );

        // Cross-check that no F1 entries leaked into F2's trail and vice
        // versa, by inspecting the entry_type field on every returned row.
        for e in f1_trail.iter() {
            assert_eq!(
                e["entry_type"], "f1-event",
                "F1 trail should only contain f1-event entries"
            );
            assert_eq!(
                e["feature_id"].as_i64(),
                Some(f1_id),
                "F1 trail rows should reference F1"
            );
        }
        for e in f2_trail.iter() {
            assert_eq!(
                e["entry_type"], "f2-event",
                "F2 trail should only contain f2-event entries"
            );
            assert_eq!(
                e["feature_id"].as_i64(),
                Some(f2_id),
                "F2 trail rows should reference F2"
            );
        }
    });
}

// =============================================================================
// 4. Work package lifecycle
// =============================================================================

#[test]
fn test_work_package_lifecycle() {
    // Create a feature, create 3 work packages for it (mix of priorities:
    // high/medium/low), update one's state, verify all 3 are returned by
    // get_work_package and that the state update was persisted.
    let plugin = SqliteStoragePlugin::in_memory().expect("in_memory should succeed");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let feature = serde_json::json!({
            "slug": "wp-lifecycle",
            "name": "WP Lifecycle"
        });
        let feature_id = plugin
            .create_feature(&feature)
            .await
            .expect("create_feature should succeed");

        let priorities = ["high", "medium", "low"];
        let mut wp_ids: Vec<i64> = Vec::new();
        for (i, priority) in priorities.iter().enumerate() {
            let wp = serde_json::json!({
                "feature_id": feature_id,
                "title": format!("WP-{}", i),
                "description": format!("description for work package {}", i),
                "priority": priority
            });
            let id = plugin
                .create_work_package(&wp)
                .await
                .expect("create_work_package should succeed");
            assert!(id > 0, "work package id should be positive");
            wp_ids.push(id);
        }

        // Update the state of the *second* work package (the medium one).
        plugin
            .update_wp_state(wp_ids[1], "in_progress")
            .await
            .expect("update_wp_state should succeed");

        // Verify all three work packages come back via get_work_package
        // and that the state update was persisted.
        for (i, &id) in wp_ids.iter().enumerate() {
            let wp = plugin
                .get_work_package(id)
                .await
                .expect("get_work_package should succeed")
                .expect("work package should exist after create");
            assert_eq!(wp["id"].as_i64(), Some(id));
            assert_eq!(wp["feature_id"].as_i64(), Some(feature_id));
            assert_eq!(wp["title"], format!("WP-{}", i));
            assert_eq!(wp["priority"], priorities[i]);
            if i == 1 {
                assert_eq!(
                    wp["state"], "in_progress",
                    "the second work package should have its state updated to in_progress"
                );
            } else {
                assert_eq!(
                    wp["state"], "backlog",
                    "untouched work packages should retain the default state 'backlog'"
                );
            }
        }
    });
}

// =============================================================================
// 5. Concurrent plugin instances share data
// =============================================================================

#[test]
fn test_concurrent_plugin_instances_share_data() {
    // SQLite in WAL mode allows multiple connections to read the same
    // database file simultaneously, with writes serialized. The plugin
    // here uses a per-instance Mutex around its single Connection. The
    // simplest contention-free pattern is: open instance 1, write,
    // drop, open instance 2, read.
    //
    // The test name uses "concurrent" loosely: the two `SqliteStoragePlugin`
    // instances are not alive simultaneously. What it actually proves is
    // that a second open against the same file path sees the writes the
    // first open committed, which is the property the test is named
    // after. This documents the real-world lifecycle for downstream
    // consumers that share a database file across processes / reopens.
    let path = unique_db_path("concurrent");

    // Open instance 1, insert a feature, then drop.
    {
        let plugin_1 = SqliteStoragePlugin::new(&path).expect("instance 1: new should succeed");
        plugin_1
            .initialize(default_config())
            .expect("instance 1: initialize should succeed");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let feature = serde_json::json!({
                "slug": "shared-feature",
                "name": "Shared Feature",
                "description": "written by instance 1"
            });
            let id = plugin_1
                .create_feature(&feature)
                .await
                .expect("instance 1: create_feature should succeed");
            assert!(id > 0);
        });
        // Drop releases the file handle and flushes the WAL.
    }

    // Open instance 2 against the same file and read.
    {
        let plugin_2 = SqliteStoragePlugin::new(&path).expect("instance 2: new should succeed");
        plugin_2
            .initialize(default_config())
            .expect("instance 2: initialize should succeed");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let by_slug = plugin_2
                .get_feature_by_slug("shared-feature")
                .await
                .expect("instance 2: get_feature_by_slug should succeed")
                .expect("instance 2 should see the feature written by instance 1");
            assert_eq!(by_slug["name"], "Shared Feature");
            assert_eq!(by_slug["description"], "written by instance 1");

            // list_all_features() should also reflect the write.
            let all = plugin_2
                .list_all_features()
                .await
                .expect("instance 2: list_all_features should succeed");
            assert_eq!(all.len(), 1);
            assert_eq!(all[0]["slug"], "shared-feature");
        });
        drop(plugin_2);
    }

    let _ = std::fs::remove_file(&path);
}

// =============================================================================
// 6. Feature id auto-increments
// =============================================================================

#[test]
fn test_feature_id_auto_increments() {
    // Create 5 features in sequence. Assert their id values are
    // monotonically increasing (1, 2, 3, 4, 5). Verifies AUTOINCREMENT
    // works as declared on the features table (lib.rs:78).
    let plugin = SqliteStoragePlugin::in_memory().expect("in_memory should succeed");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut ids: Vec<i64> = Vec::new();
        for i in 0..5 {
            let feature = serde_json::json!({
                "slug": format!("auto-{}", i),
                "name": format!("Auto {}", i)
            });
            let id = plugin
                .create_feature(&feature)
                .await
                .expect("create_feature should succeed");
            ids.push(id);
        }
        // The features table is created with INTEGER PRIMARY KEY
        // AUTOINCREMENT (lib.rs:78). For a fresh in-memory database the
        // first inserted row's id must be 1, then 2, 3, 4, 5.
        assert_eq!(ids, vec![1, 2, 3, 4, 5], "ids should be strictly 1..=5");
    });
}

// =============================================================================
// 7. list_all_features includes all states
// =============================================================================

#[test]
fn test_list_all_features_includes_all_states() {
    // Create 4 features with explicit states: draft, active, complete,
    // archived. Verify all 4 are returned by list_all_features().
    let plugin = SqliteStoragePlugin::in_memory().expect("in_memory should succeed");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let states = ["draft", "active", "complete", "archived"];
        for (i, state) in states.iter().enumerate() {
            let feature = serde_json::json!({
                "slug": format!("state-{}", i),
                "name": format!("State {}", i),
                "state": state
            });
            let id = plugin
                .create_feature(&feature)
                .await
                .expect("create_feature should succeed");
            // Sanity: read-back should reflect the requested state.
            let read = plugin
                .get_feature_by_id(id)
                .await
                .expect("get_feature_by_id should succeed")
                .expect("feature should exist");
            assert_eq!(read["state"], *state);
        }

        // All four should be present in list_all_features(), regardless
        // of state. list_all_features() orders by created_at DESC, but
        // we sort by slug for a stable, order-independent comparison.
        let mut all = plugin
            .list_all_features()
            .await
            .expect("list_all_features should succeed");
        assert_eq!(all.len(), 4, "all 4 features should be listed");

        all.sort_by(|a, b| a["slug"].as_str().cmp(&b["slug"].as_str()));
        let returned_states: Vec<&str> = all
            .iter()
            .map(|f| f["state"].as_str().expect("state should be a string"))
            .collect();
        // `all` is sorted by slug ("state-0", "state-1", "state-2",
        // "state-3"), and we created feature i with `states[i]`. So the
        // returned states in slug-sorted order should match `states`
        // verbatim (no need to re-sort `states`).
        assert_eq!(
            returned_states, states,
            "list_all_features should include draft/active/complete/archived"
        );
    });
}

// =============================================================================
// 8. Audit trail returns descending by created_at
// =============================================================================

#[test]
fn test_audit_trail_returns_descending_by_created_at() {
    // Create a feature, append 3 audit entries with a small sleep between
    // them. Verify the returned trail has the LAST-inserted entry first
    // (DESC order). get_audit_trail() is implemented with
    // `ORDER BY created_at DESC` (lib.rs:374), and SQLite's
    // CURRENT_TIMESTAMP has 1-second resolution, so the sleeps must
    // exceed 1 second to guarantee distinct timestamps and a stable
    // ordering.
    let plugin = SqliteStoragePlugin::in_memory().expect("in_memory should succeed");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let feature = serde_json::json!({
            "slug": "audit-desc",
            "name": "Audit Desc"
        });
        let feature_id = plugin
            .create_feature(&feature)
            .await
            .expect("create_feature should succeed");

        let actors = ["first", "second", "third"];
        for actor in actors.iter() {
            let entry = serde_json::json!({
                "feature_id": feature_id,
                "entry_type": "ordered",
                "actor": actor,
                "details": format!("payload-{}", actor)
            });
            plugin
                .append_audit_entry(&entry)
                .await
                .expect("append_audit_entry should succeed");
            // Sleep > 1s so the next CURRENT_TIMESTAMP differs from the
            // previous one. Without this, the three entries could share
            // the same created_at value and the DESC ordering would be
            // undefined.
            std::thread::sleep(std::time::Duration::from_millis(1100));
        }

        let trail = plugin
            .get_audit_trail(feature_id)
            .await
            .expect("get_audit_trail should succeed");
        assert_eq!(trail.len(), 3, "all three entries should be returned");

        // The trail is ordered DESC by created_at, so the last-inserted
        // entry ("third") should be first, then "second", then "first".
        assert_eq!(
            trail[0]["actor"], "third",
            "first entry in DESC-ordered trail should be the last one inserted"
        );
        assert_eq!(trail[1]["actor"], "second");
        assert_eq!(trail[2]["actor"], "first");

        // All three entries should reference the same feature_id.
        for e in trail.iter() {
            assert_eq!(e["feature_id"].as_i64(), Some(feature_id));
        }
    });
}

// =============================================================================
// 9. State transitions round-trip through DB
// =============================================================================

#[test]
fn test_state_transitions_round_trip_through_db() {
    // Create a feature, transition state draft -> active -> complete, drop
    // the plugin, recreate the plugin from the same file path, verify the
    // final state is "complete" (persistence verified across instances).
    let path = unique_db_path("state-roundtrip");

    // Phase 1: create + transition state.
    {
        let plugin = SqliteStoragePlugin::new(&path).expect("phase 1: new should succeed");
        plugin
            .initialize(default_config())
            .expect("phase 1: initialize should succeed");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let feature = serde_json::json!({
                "slug": "state-roundtrip",
                "name": "State Roundtrip"
            });
            let id = plugin
                .create_feature(&feature)
                .await
                .expect("phase 1: create_feature should succeed");

            // No state supplied -> defaults to "draft" (lib.rs:172).
            let initial = plugin
                .get_feature_by_id(id)
                .await
                .expect("phase 1: get should succeed")
                .expect("phase 1: feature should exist");
            assert_eq!(initial["state"], "draft");

            plugin
                .update_feature_state(id, "active")
                .await
                .expect("phase 1: draft -> active");
            plugin
                .update_feature_state(id, "complete")
                .await
                .expect("phase 1: active -> complete");

            let mid = plugin
                .get_feature_by_id(id)
                .await
                .expect("phase 1: get should succeed")
                .expect("phase 1: feature should exist");
            assert_eq!(mid["state"], "complete");
        });
        // Drop the plugin so the file is fully flushed and unlocked.
    }

    // Phase 2: re-open and confirm the final state survived.
    {
        let plugin = SqliteStoragePlugin::new(&path).expect("phase 2: new should succeed");
        plugin
            .initialize(default_config())
            .expect("phase 2: initialize should succeed");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let read = plugin
                .get_feature_by_slug("state-roundtrip")
                .await
                .expect("phase 2: get_feature_by_slug should succeed")
                .expect("phase 2: feature should survive a reopen");
            assert_eq!(
                read["state"], "complete",
                "state should round-trip across instances"
            );
            assert_eq!(read["name"], "State Roundtrip");
        });
        drop(plugin);
    }

    let _ = std::fs::remove_file(&path);
}

// =============================================================================
// 10. Duplicate slug returns Operation error
// =============================================================================

#[test]
fn test_duplicate_slug_returns_operation_error() {
    // Create 2 features with the same slug, verify the second call
    // returns an error of type PluginError::Operation (since the
    // production code wraps conn.execute failures as PluginError::Operation
    // at lib.rs:178).
    let plugin = SqliteStoragePlugin::in_memory().expect("in_memory should succeed");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let first = serde_json::json!({
            "slug": "duplicate",
            "name": "First Dup"
        });
        let first_id = plugin
            .create_feature(&first)
            .await
            .expect("first insert should succeed");
        assert!(first_id > 0);

        let second = serde_json::json!({
            "slug": "duplicate",
            "name": "Second Dup"
        });
        let err = plugin
            .create_feature(&second)
            .await
            .expect_err("second insert with the same slug should fail");

        // The production code at lib.rs:178 wraps the `conn.execute`
        // failure as `PluginError::Operation(format!("failed to create
        // feature: {}", e))`. This pins that error variant down so
        // downstream callers can match on it.
        match err {
            PluginError::Operation(msg) => {
                assert!(
                    msg.contains("failed to create feature"),
                    "Operation error message should mention the failed operation, got: {}",
                    msg
                );
            }
            other => panic!(
                "expected PluginError::Operation for duplicate slug, got: {:?}",
                other
            ),
        }

        // Sanity: the first insert is still there, untouched.
        let still_there = plugin
            .get_feature_by_slug("duplicate")
            .await
            .expect("get should not error")
            .expect("first insert should still be present");
        assert_eq!(still_there["id"].as_i64(), Some(first_id));
        assert_eq!(still_there["name"], "First Dup");
    });
}
