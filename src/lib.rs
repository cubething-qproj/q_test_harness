mod app_ext;
mod commands_ext;
mod data;
mod log;
mod util;

pub mod prelude {
    pub use super::app_ext::*;
    pub use super::commands_ext::*;
    pub use super::data::*;
    pub use super::log::*;
    pub use super::*;
    pub(crate) use bevy::prelude::*;
}

use bevy::{
    app::{FixedMain, ScheduleRunnerPlugin},
    diagnostic::FrameCountPlugin,
    ecs::schedule::ExecutorKind,
    log::{DEFAULT_FILTER, LogPlugin},
    state::app::StatesPlugin,
    time::TimePlugin,
};

use crate::prelude::*;

macro_rules! configure_sets {
    ($app:expr, $kind:expr, $($set:ident),*) => {
        $(
            if let Some(set) = $app.get_schedule_mut($set) {
                set.set_executor_kind($kind);
            }
        )*
    };
}

/// Test runner timeout in seconds
#[derive(Resource, Deref, PartialEq)]
pub struct TestRunnerTimeout(pub f32);
impl Default for TestRunnerTimeout {
    fn default() -> Self {
        Self(5.)
    }
}

/// Should the test runner log step counts?
#[derive(Resource, Deref, PartialEq, Eq)]
pub struct LogTestSteps(pub bool);
impl Default for LogTestSteps {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Debug)]
pub struct TestRunnerPlugin {
    pub log_level: bevy::log::Level,
    pub log_filter: String,
    /// Whether to run single-threaded or multi-threaded.
    /// Defaults to single-threaded to avoid out-of-order errors.
    pub executor_kind: ExecutorKind,
}
impl Default for TestRunnerPlugin {
    fn default() -> Self {
        Self {
            log_level: bevy::log::Level::TRACE,
            log_filter: DEFAULT_FILTER.to_string(),
            executor_kind: ExecutorKind::SingleThreaded,
        }
    }
}
impl Plugin for TestRunnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TaskPoolPlugin::default(),
            FrameCountPlugin,
            TimePlugin,
            ScheduleRunnerPlugin::default(),
            LogPlugin {
                level: self.log_level,
                filter: self.log_filter.clone(),
                custom_layer: crate::log::custom_layer,
                ..Default::default()
            },
            AssetPlugin::default(),
            StatesPlugin,
        ));
        app.init_resource::<LogTestSteps>();
        app.init_resource::<TestRunnerTimeout>();

        app.add_systems(
            Update,
            (move |time: Res<Time<Real>>,
                   timeout: Res<TestRunnerTimeout>,
                   mut events: MessageWriter<AppExit>| {
                let elapsed = time.elapsed_secs();
                if elapsed > **timeout {
                    error!("Timeout after {elapsed}s");
                    events.write(AppExit::error());
                }
            })
            .in_set(TestRunnerSystems),
        );
        app.add_systems(
            PostUpdate,
            (
                log_step.run_if(resource_exists_and_equals(LogTestSteps(true))),
                check_exit,
            )
                .chain()
                .in_set(TestRunnerSystems),
        );

        app.init_state::<Step>();

        configure_sets!(
            app,
            self.executor_kind,
            StateTransition,
            PreStartup,
            Startup,
            PostStartup,
            First,
            PreUpdate,
            RunFixedMainLoop,
            Update,
            SpawnScene,
            PostUpdate,
            Last,
            Main,
            FixedMain,
            FixedFirst,
            FixedUpdate,
            FixedPreUpdate,
            FixedPostUpdate,
            FixedLast
        );
    }
}

fn log_step(step: Res<State<Step>>, mut local_step: Local<u32>) {
    info_once!("Step = {}", ***step); // when step = 0
    if ***step != *local_step {
        *local_step = ***step;
        info!("Step = {}", ***step);
    }
}

fn check_exit(mut reader: MessageReader<AppExit>) {
    for msg in reader.read() {
        match msg {
            AppExit::Success => info!("Successful exit!"),
            AppExit::Error(non_zero) => {
                error!("Got bad exit code {non_zero}");
            }
        }
    }
}

#[test]
fn timeout() {
    let mut app = App::new();
    app.add_plugins(TestRunnerPlugin::default());
    app.insert_resource(TestRunnerTimeout(0.5));
    assert!(app.run().is_error());
}

#[test]
fn explicit_failure() {
    let mut app = App::new();
    app.add_plugins(TestRunnerPlugin::default());
    app.add_systems(First, |mut commands: Commands| {
        commands.write_message(AppExit::error());
    });
    assert!(app.run().is_error());
}

#[test]
fn explicit_success() {
    let mut app = App::new();
    app.add_plugins(TestRunnerPlugin::default());
    app.add_systems(First, |mut commands: Commands| {
        commands.write_message(AppExit::Success);
    });
    assert!(app.run().is_success());
}
