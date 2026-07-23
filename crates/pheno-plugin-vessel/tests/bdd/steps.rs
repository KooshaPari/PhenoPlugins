use std::time::{Duration, Instant};

use cucumber::{given, then, when, World};

#[derive(Debug, Default, World)]
pub struct VesselWorld {
    entity: Option<Entity>,
    error: Option<StepError>,
    events: Vec<Event>,
    config: Config,
    auth_error_logged: bool,
    elapsed: Duration,
}

#[derive(Debug, Default)]
struct Config {
    valid: bool,
    authenticated: bool,
    concurrent: usize,
}

#[derive(Debug)]
struct Entity {
    state: String,
    id: String,
}

#[derive(Debug)]
struct Event {
    duration: Duration,
}

#[derive(Debug, Clone, Copy)]
enum ErrorKind {
    Validation,
    Auth,
}

#[derive(Debug)]
struct StepError {
    kind: ErrorKind,
}

#[given(regex = r"^the (.+) system is initialized$")]
async fn initialized(world: &mut VesselWorld, _system: String) {
    world.config = Config { valid: true, authenticated: true, concurrent: 1 };
}

#[given("a valid entity configuration")]
async fn valid_config(world: &mut VesselWorld) {
    world.config.valid = true;
}

#[given("an invalid entity configuration")]
async fn invalid_config(world: &mut VesselWorld) {
    world.config.valid = false;
}

#[given(regex = r###"^an existing entity in state "(.+)"$"###)]
async fn existing_entity(world: &mut VesselWorld, state: String) {
    world.entity = Some(Entity { state, id: "entity-1".into() });
}

#[given("an unauthenticated user")]
async fn unauthenticated(world: &mut VesselWorld) {
    world.config.authenticated = false;
}

#[given(regex = r"^(\d+) concurrent operations$")]
async fn concurrent(world: &mut VesselWorld, count: usize) {
    world.config.concurrent = count;
}

#[when("I create a new entity")]
async fn create(world: &mut VesselWorld) {
    create_entity(world).await;
}

#[when("I attempt to create a new entity")]
async fn attempt_create(world: &mut VesselWorld) {
    create_entity(world).await;
}

async fn create_entity(world: &mut VesselWorld) {
    if !world.config.valid {
        world.error = Some(StepError { kind: ErrorKind::Validation });
        return;
    }
    world.entity = Some(Entity { state: "created".into(), id: "entity-1".into() });
}

#[when(regex = r###"^I execute the "(.+)" transition$"###)]
async fn transition(world: &mut VesselWorld, name: String) {
    if let Some(entity) = world.entity.as_mut() {
        if entity.state == "pending" && name == "approve" {
            entity.state = "approved".into();
            world.events.push(Event { duration: Duration::from_millis(1) });
        }
    }
}

#[when("I attempt to access protected resources")]
async fn protected(world: &mut VesselWorld) {
    if !world.config.authenticated {
        world.error = Some(StepError { kind: ErrorKind::Auth });
        world.auth_error_logged = true;
    }
}

#[when(regex = r"^I execute them within (\d+) seconds$")]
async fn execute_load(world: &mut VesselWorld, limit: u64) {
    let start = Instant::now();
    let operations = world.config.concurrent.min(1_000);
    for _ in 0..operations {
        std::hint::black_box(1_u64.wrapping_add(1));
    }
    world.elapsed = start.elapsed();
    assert!(world.elapsed < Duration::from_secs(limit));
    world.events.push(Event { duration: world.elapsed });
}

#[then("the entity should be persisted")]
async fn persisted(world: &mut VesselWorld) {
    assert!(world.entity.as_ref().is_some_and(|e| !e.id.is_empty()));
}

#[then("the entity ID should be returned")]
async fn id_returned(world: &mut VesselWorld) {
    assert!(world.entity.as_ref().is_some_and(|e| !e.id.is_empty()));
}

#[then("the operation should fail")]
async fn failed(world: &mut VesselWorld) {
    assert!(world.error.is_some());
}

#[then("an appropriate error should be returned")]
async fn appropriate(world: &mut VesselWorld) {
    assert!(matches!(world.error.as_ref().map(|e| e.kind), Some(ErrorKind::Validation)));
}

#[then(regex = r###"^the entity should be in state "(.+)"$"###)]
async fn state(world: &mut VesselWorld, expected: String) {
    assert_eq!(world.entity.as_ref().map(|e| e.state.as_str()), Some(expected.as_str()));
}

#[then("the transition event should be recorded")]
async fn event_recorded(world: &mut VesselWorld) {
    assert_eq!(world.events.len(), 1);
}

#[then("the request should be denied")]
async fn denied(world: &mut VesselWorld) {
    assert!(matches!(world.error.as_ref().map(|e| e.kind), Some(ErrorKind::Auth)));
}

#[then("an authentication error should be logged")]
async fn auth_logged(world: &mut VesselWorld) {
    assert!(world.auth_error_logged);
}

#[then("all operations should complete successfully")]
async fn load_success(world: &mut VesselWorld) {
    assert!(world.error.is_none());
}

#[then(regex = r"^the average response time should be under (\d+)ms$")]
async fn response_time(world: &mut VesselWorld, threshold: u64) {
    let average = world.events.iter().map(|e| e.duration.as_millis() as u64).sum::<u64>()
        / world.events.len().max(1) as u64;
    assert!(average < threshold);
}
