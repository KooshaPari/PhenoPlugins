//! L11 chaos game-day test for `pheno-plugin-core` (v25 cycle-15 T5).
//!
//! Exercises the public [`PluginRegistry`] surface under two injected
//! failure modes:
//!
//! 1. **Latency injection** — adds a deterministic 100-500 ms sleep before
//!    each `register_vcs` call (drawn from a xorshift64 PRNG seeded with
//!    `42`). The sync path uses `std::thread::sleep`.
//! 2. **Error injection** — a flaky mock plugin returns
//!    `Err(PluginError::NotInitialized)` on every Nth `health_check` call
//!    (N = 3) so 1/3 of calls exercise the error-propagation path through
//!    `PluginRegistry::health_check`.
//!
//! Determinism note: every test seeds the PRNG with the same seed (`42`) so
//! the fault sequence is reproducible across runs and CI machines.
//!
//! Run with:
//!   cargo test -p pheno-plugin-core --test chaos_l11_2026_06_22
//!
//! Invariant: every test must complete in under 10 s wall time. The
//! latency-injection test bounds the upper register-count at 4 (so worst
//! case 4 * 500 ms = 2 s), keeping wall time well under the 10 s CI gate.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use pheno_plugin_core::{
    AdapterPlugin, ConflictInfo, FeatureArtifacts, MergeResult, PluginError, PluginRegistry,
    PluginResult, StoragePlugin, VcsPlugin, WorktreeInfo,
};
use std::path::{Path, PathBuf};

/// Deterministic xorshift64 PRNG. Seed = 42 produces the same fault sequence
/// across runs and platforms.
struct Xorshift64(u64);

impl Xorshift64 {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn range(&mut self, lo: u64, hi: u64) -> u64 {
        assert!(hi >= lo, "range invalid: lo={lo} hi={hi}");
        let span = hi - lo + 1;
        lo + (self.next_u64() % span)
    }
}

/// Flaky mock VCS plugin that fails every Nth `health_check` call. The
/// counter is shared so multiple instances cooperate (mirrors a real
/// distributed-plugin scenario where the registry aggregates health across
/// instances).
struct FlakyMockPlugin {
    name: String,
    fail_every: usize,
    call_count: Arc<AtomicUsize>,
}

impl FlakyMockPlugin {
    fn new(name: &str, fail_every: usize, call_count: Arc<AtomicUsize>) -> Self {
        Self {
            name: name.to_string(),
            fail_every,
            call_count,
        }
    }
}

impl AdapterPlugin for FlakyMockPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    fn version(&self) -> &str {
        "0.1.0"
    }
    fn initialize(&self, _config: pheno_plugin_core::PluginConfig) -> PluginResult<()> {
        Ok(())
    }
}

#[async_trait]
impl VcsPlugin for FlakyMockPlugin {
    async fn create_worktree(&self, _: &str, _: &str) -> PluginResult<PathBuf> {
        Ok(PathBuf::from("/tmp/chaos"))
    }
    async fn list_worktrees(&self) -> PluginResult<Vec<WorktreeInfo>> {
        Ok(vec![])
    }
    async fn cleanup_worktree(&self, _: &Path) -> PluginResult<()> {
        Ok(())
    }
    async fn create_branch(&self, _: &str, _: &str) -> PluginResult<()> {
        Ok(())
    }
    async fn checkout_branch(&self, _: &str) -> PluginResult<()> {
        Ok(())
    }
    async fn merge_to_target(&self, _: &str, _: &str) -> PluginResult<MergeResult> {
        Ok(MergeResult {
            success: true,
            conflicts: vec![],
            merged_commit: None,
        })
    }
    async fn detect_conflicts(&self, _: &str, _: &str) -> PluginResult<Vec<ConflictInfo>> {
        Ok(vec![])
    }
    async fn read_artifact(&self, _: &str, _: &str) -> PluginResult<String> {
        Ok(String::new())
    }
    async fn write_artifact(&self, _: &str, _: &str, _: &str) -> PluginResult<()> {
        Ok(())
    }
    async fn artifact_exists(&self, _: &str, _: &str) -> PluginResult<bool> {
        Ok(false)
    }
    async fn scan_feature_artifacts(&self, _: &str) -> PluginResult<FeatureArtifacts> {
        Ok(FeatureArtifacts {
            meta_json: None,
            audit_chain: None,
            evidence_paths: vec![],
        })
    }
    async fn health_check(&self) -> PluginResult<()> {
        let n = self.call_count.fetch_add(1, Ordering::SeqCst) + 1;
        if self.fail_every > 0 && n % self.fail_every == 0 {
            Err(PluginError::NotInitialized(format!(
                "chaos: forced failure on call {n}"
            )))
        } else {
            Ok(())
        }
    }
}

/// Wrap `register_vcs` with a chaos gate. Sleeps 100-500 ms before each
/// registration, then delegates to the underlying registry.
fn chaos_register_vcs(
    injector: &mut Xorshift64,
    registry: &PluginRegistry,
    plugin: Box<dyn VcsPlugin>,
) -> PluginResult<()> {
    let delay_ms = injector.range(100, 500);
    std::thread::sleep(Duration::from_millis(delay_ms));
    registry.register_vcs(plugin)
}

/// **Chaos enabled (latency + error)**: 4 chaotic registrations + 3 chaotic
/// health checks. With seed=42 + 1/3 error rate, at least one health check
/// must error and at least one must succeed.
#[test]
fn chaos_latency_and_error_injection_handled_cleanly() {
    let registry = PluginRegistry::new();
    let call_count = Arc::new(AtomicUsize::new(0));
    let mut injector = Xorshift64::new(42);

    let start = Instant::now();

    // Latency-injected registrations (4 plugins).
    for i in 0..4 {
        let plugin = FlakyMockPlugin::new(
            &format!("plugin-{i}"),
            3, // 1/3 error rate on health_check
            call_count.clone(),
        );
        chaos_register_vcs(&mut injector, &registry, Box::new(plugin))
            .expect("chaos gate must not break registration");
    }
    let stats = registry.stats();
    assert_eq!(stats.vcs_count, 4, "all 4 registrations must land");

    // Drive the async runtime to run the chaos health-check sweep.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let (ok_runs, err_runs) = rt.block_on(async {
        let mut ok_runs = 0usize;
        let mut err_runs = 0usize;
        for _ in 0..3 {
            match registry.health_check().await {
                Ok(()) => ok_runs += 1,
                Err(PluginError::NotInitialized(_)) => err_runs += 1,
                Err(other) => panic!("unexpected error variant: {other:?}"),
            }
        }
        (ok_runs, err_runs)
    });

    let elapsed = start.elapsed();
    assert_eq!(ok_runs + err_runs, 3);
    assert!(ok_runs > 0, "at least one health check must succeed; got 0");
    assert!(err_runs > 0, "at least one health check must error; got 0");
    assert!(
        elapsed < Duration::from_secs(10),
        "wall time {elapsed:?} exceeded 10 s gate"
    );
}

/// **Chaos disabled (control)**: a baseline `health_check` loop without any
/// injected latency. With fail_every = 0 the mock never errors.
#[tokio::test(flavor = "current_thread")]
async fn chaos_disabled_baseline_health_check_succeeds() {
    let registry = PluginRegistry::new();
    let call_count = Arc::new(AtomicUsize::new(0));
    registry
        .register_vcs(Box::new(FlakyMockPlugin::new("baseline", 0, call_count)))
        .unwrap();

    let start = Instant::now();
    for _ in 0..3 {
        registry.health_check().await.unwrap();
    }
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(1),
        "baseline should be sub-second; took {elapsed:?}"
    );
}

/// **Determinism**: two PRNG instances with seed=42 produce the same latency
/// sequence. CI depends on this for reproducible triage.
#[test]
fn chaos_deterministic_seed_produces_identical_sequence() {
    let mut a = Xorshift64::new(42);
    let mut b = Xorshift64::new(42);

    let latencies: Vec<u64> = (0..50).map(|_| a.range(100, 500)).collect();
    let latencies_b: Vec<u64> = (0..50).map(|_| b.range(100, 500)).collect();
    assert_eq!(latencies, latencies_b);
    // Latencies must stay within the 100-500 ms band.
    assert!(latencies.iter().all(|&d| (100..=500).contains(&d)));
}

/// **Forced error propagation**: pin the error-injection contract
/// independent of the random draw — the failure message must include the
/// call count for log triage.
#[tokio::test(flavor = "current_thread")]
async fn chaos_forced_error_message_contains_call_count() {
    let registry = PluginRegistry::new();
    let call_count = Arc::new(AtomicUsize::new(0));
    registry
        .register_vcs(Box::new(FlakyMockPlugin::new("triage", 3, call_count)))
        .unwrap();

    // Three calls; the third must error with a message that mentions the
    // call number.
    registry.health_check().await.unwrap();
    registry.health_check().await.unwrap();
    let err = registry.health_check().await.unwrap_err();
    match err {
        PluginError::NotInitialized(msg) => {
            assert!(
                msg.contains("3"),
                "forced-error message must mention the failing call number; got {msg:?}"
            );
        }
        other => panic!("expected NotInitialized, got {other:?}"),
    }
}
