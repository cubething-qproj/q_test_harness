---
name: q-test-harness
description: Deep dive on q_test_harness (lib/q_test_harness), qproj's Bevy test harness — TestRunnerPlugin (headless runner, timeout, single-threaded executor, log capture), Step state and add_step, CommandsExt assertions and entity finders, InputTestPlugin with AppExt key/mouse/motion/step for selected-schedule input tests, and how to pick between full-app and partial-schedule tests. Use when writing or debugging tests for any qproj Bevy crate or game.
---

# q_test_harness

Source: `lib/q_test_harness/src` (~900 lines: `lib.rs`, `app_ext.rs`,
`commands_ext.rs`, `input.rs`, `log.rs`). README has the canonical patterns.
`use q_test_harness::prelude::*;`. Dogfood it: prefer it over ad-hoc
harness code, and push reusable machinery here (not game types).

## Full-app tests: `TestRunnerPlugin`

Adds TaskPool, FrameCount, Time, ScheduleRunner, Log (TRACE + capture),
Asset, States; sets all main/fixed schedules to the single-threaded
executor (`executor_kind: ExecutorKind::MultiThreaded` to opt out). The app
runs normally until something writes `AppExit`.

```rust
let mut app = App::new();
app.add_plugins(TestRunnerPlugin::default());
app.add_step(0, |q: Query<&Name>, mut commands: Commands, mut next: ResMut<NextState<Step>>| {
    if q.is_empty() { return; }                      // wait for readiness
    if commands.assert(q.iter().any(|n| n.as_str() == "subject"), "subject missing") {
        next.set(Step(1));                           // advancing is manual
    }
});
app.add_step(1, |mut commands: Commands| { commands.write_message(AppExit::Success); });
assert!(app.run().is_success());
```

- `add_step(n, systems)`: runs in `PostUpdate` while `State<Step> == Step(n)`
  (starts at 0).
- `commands.assert(cond, msg) -> bool`: logs and writes `AppExit::error()` on
  failure. Success must be written explicitly.
- `TestRunnerTimeout(secs)` resource (default 5.0 of `Time<Real>`) fails
  stalled tests. With `TimeUpdateStrategy::ManualDuration` the timeout
  measures simulated time, so keep an external wall-clock timeout.
- `LogTestSteps(bool)`; captured logs arrive as `LogMessage` messages.
- `CommandsExt`: `log_hierarchy()`, `find_entity(name)`,
  `find_no_entity(name)`, `find_entity_filtered::<F>(name)`,
  `find_no_entity_filtered::<F>(name)`, `find_entity_with(name, value,
  invert)` — each writes an error exit on mismatch.

Real examples: `lib/q_term/tests/term/io_boundary.rs`,
`lib/q_screens/tests/screens/entity_scope.rs`.

## Selected-schedule input tests: `InputTestPlugin` + `AppExt`

```rust
use bevy::ecs::schedule::ScheduleLabel;
let mut app = App::new();
app.add_plugins((MinimalPlugins, InputTestPlugin));
// add plugins/systems under test
app.finish(); app.cleanup();                     // required before step
app.key(KeyCode::KeyW, true);
app.step(Duration::from_millis(16), [PreUpdate.intern(), Update.intern()]);
app.key(KeyCode::KeyW, false);
```

- `InputTestPlugin` initializes input resources and a focused synthetic
  primary window if none exists. **Do not add `InputPlugin`** — it overwrites
  manual input.
- `key`, `mouse` press/release (held until released); `motion(delta)` sets
  accumulated motion for the next step.
- `step(delta, schedules)` advances `Time<Real>` and `Time`, runs only those
  schedules, then clears just-pressed/released and motion/scroll. It is **not
  a full update**: no Startup/First, no message maintenance, no virtual/fixed
  time advance.

## Choosing a test (org policy, see AGENTS.md)

- Selected schedules for precise mechanics; full app for lifecycle and
  schedule integration. A passing partial-schedule test is not evidence that
  omitted phases work.
- Repeated setup → a small test-local `Fixture` owning the `App` and relevant
  `Entity` ids, with scenario methods. Example:
  `bin/quell/src/screen/dev/camera_test/screen.rs`. No generic fixture
  framework.
- Fixture-based tests usually belong in `tests/`, unit tests in `src/`.
- Avoid change-detector tests (constants, incidental ordering).
