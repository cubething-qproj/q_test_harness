<div align="center">
<img src="https://raw.githubusercontent.com/cubething-qproj/q_test_harness/refs/heads/main/.doc/q_test_harness.png" height=300 alt="Illustration of a common robin with worms in its mouth. Text, 'bevy test harness'" title="test harness logo" />
</div>

[![Coverage Status](https://coveralls.io/repos/github/cubething-qproj/q_test_harness/badge.svg)](https://coveralls.io/github/cubething-qproj/q_test_harness)

This is a simple test harness for bevy projects.

## Features

- [x] Utility functions for easy, step-based testing.
- [x] Timeout functionality
- [x] Input helpers
- [x] Logging utilities
  - [x] Log the world hierarchy in a simple and readable format
  - [ ] Add names for common types
  - [x] Log capturing
- [x] Utilities for finding specific entities (by name) and testing their properties.
    - `find_entity` 
    - `find_no_entity` 
    - `find_entity_filtered<QueryFilter>`
    - `find_no_entity_filtered<QueryFilter>`
    - `find_entity_with<Component>`
  
## Stretch goals

- [ ] Headless rendering support
- [ ] Screenshots
- [ ] Scene snapshots
- [ ] Replay
- [ ] Reporting

## Non-goals

- Advanced trace viewer (a la playwright)
- Benchmark functionality

## Compatibility table

| q_test_harness | bevy |
| ----------------- | ---- |
| main              | 0.19 |


## Testing patterns

Downstream tests use an ordinary `App` with `TestRunnerPlugin` and run the normal
app loop until a system writes `AppExit`. For example, `q_term` uses `add_step`
for staged checks; `q_screens` also checks results inside screen lifecycle callbacks.

```rust
use bevy::prelude::*;
use q_test_harness::prelude::*;

#[test]
fn spawns_subject() {
    let mut app = App::new();
    app.add_plugins(TestRunnerPlugin::default());
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn(Name::new("subject"));
    });
    app.add_step(0, |names: Query<&Name>, mut commands: Commands| {
        if commands.assert(names.iter().any(|name| name.as_str() == "subject"), "subject missing") {
            commands.write_message(AppExit::Success);
        }
    });
    assert!(app.run().is_success());
}
```

`add_step(n, ...)` runs in `PostUpdate` while `State<Step>` is `Step(n)` (initially
zero). A step can wait for readiness, then advance with `NextState<Step>`;
advancement is not automatic. `commands.assert` logs failure and writes an error
exit; successful completion is explicit. The runner times out stalled tests.

See `q_term/tests/term/io_boundary.rs` and
`q_screens/tests/screens/entity_scope.rs` for real multi-step/lifecycle examples.

### What sort of test should I use?

Choose the smallest test that covers the contract: selected schedules for precise
mechanics, full app updates for lifecycle/scheduling integration, and bounded
readiness waits for asynchronous I/O. Full updates can also use a controlled clock
with Bevy's `TimeUpdateStrategy::ManualDuration`; waiting until success is not a
substitute for asserting a frame deadline. With manual time, the harness timeout
measures simulated elapsed time, so retain an external wall-clock test timeout.

### Testing Input

Add `InputTestPlugin` alongside `TestRunnerPlugin` or `MinimalPlugins`.
It initializes manual input and supplies a synthetic primary window if needed.
`AppExt` provides `key`, `mouse`, `motion`, and `step` on the ordinary `App`.

```rust
use std::time::Duration;
use bevy::{ecs::schedule::ScheduleLabel, prelude::*};
use q_test_harness::prelude::*;

let mut app = App::new();
app.add_plugins((MinimalPlugins, InputTestPlugin));
// Add the plugins/systems under test before finishing plugins.
app.finish();
app.cleanup();
app.key(KeyCode::KeyW, true);
app.step(Duration::from_millis(16), [PreUpdate.intern(), Update.intern()]);
app.key(KeyCode::KeyW, false);
```

`step` advances real/generic time and runs only the selected schedules, then clears
transient input while preserving held buttons. It is **not a full app update**:
startup, message maintenance, and virtual/fixed time are not advanced automatically.
Do not add `InputPlugin` when supplying input manually; it overwrites that input.

## About the bird

"The American robin (Turdus migratorius) is a migratory bird of the true thrush genus and Turdidae, the wider thrush family. It is named after the European robin because of its reddish-orange breast, though the two species are not closely related, with the European robin belonging to the Old World flycatcher family. According to the Partners in Flight database (2019), the American robin is the most abundant landbird in North America (with 370 million individuals), ahead of red-winged blackbirds, introduced European starlings, mourning doves and house finches. It has seven subspecies. " ([wikipedia](https://en.wikipedia.org/wiki/American_robin))
