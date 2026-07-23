#[path = "bdd/steps.rs"]
mod steps;

use cucumber::World;

#[tokio::test]
async fn bdd_feature_suite() {
    steps::VesselWorld::run(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/bdd/features/test.feature"
    ))
    .await;
}
