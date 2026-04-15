<div align="center">
<img src="https://raw.githubusercontent.com/ada-x64/qproj/refs/heads/main/.doc/test_harness.png" height=300 alt="Illustration of a common robin with worms in its mouth. Text, 'bevy test harness'" title="test harness logo" />
</div>

[ ![Coveralls](https://img.shields.io/coverallsCoverage/github/ada-x64/qproj?branch=q_test_harness) ]( https://coveralls.io/github/ada-x64/qproj?branch=q_test_harness )

This is a simple test harness for bevy projects.

## Features

- [x] Utility functions for easy, step-based testing.
- [x] Timeout functionality
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
| main              | 0.18 |


## Testing patterns

There are to main ways to run tests. The first is with a manual-update style.
This is typically what you see in something like the official bevy tests, where
we want to guarantee a specific number of updates has occured before the next
assertion. As a nice side-effect, this guarantees that the test takes as little
time as possible.

```rust
#[test]
fn pattern_1() {
    let mut app = App::new();
    // add plugins etc.
    app.update();
    assert!(something);
    // repeat as needed
}
```

The second style of test requires a nondeterministic number of updates to pass
before the next assertion can be called, i.e. when the test requires I/O (such
as asset loading). These tests require some more setup.

```rust
#[test]
fn pattern_2() {
    let mut app = App::new();
    // plugins etc.
    // then set up systems
    let a = |mut commmands: Commands| {
        // do this to end the test successfully
        commands.write_message(AppExit::Success);
    }
    let b = |mut commmands: Commands| {
        // do this to end the test in failure
        error!("Something went wrong!")
        commands.write_message(AppExit::error());
    }
    app.add_systems(PostUpdate, (a,b).chain());
    assert!(app.run().is_success());
}
```

Note that in pattern 2 you _cannot_ use assertions as bevy systems run in
separate threads. Panicking will not kill the process, but only the thread.

## About the bird

"The American robin (Turdus migratorius) is a migratory bird of the true thrush genus and Turdidae, the wider thrush family. It is named after the European robin because of its reddish-orange breast, though the two species are not closely related, with the European robin belonging to the Old World flycatcher family. According to the Partners in Flight database (2019), the American robin is the most abundant landbird in North America (with 370 million individuals), ahead of red-winged blackbirds, introduced European starlings, mourning doves and house finches. It has seven subspecies. " ([wikipedia](https://en.wikipedia.org/wiki/American_robin))
