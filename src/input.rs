use bevy::{
    input::{
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
        touch::Touches,
    },
    prelude::*,
    window::{CursorOptions, PrimaryWindow},
};

/// Manual input support, complementary to `TestRunnerPlugin` or `MinimalPlugins`.
///
/// Initializes input resources and, during plugin finalization, supplies a focused
/// synthetic primary window if none exists. Adds no clock, runner, renderer, or
/// `InputPlugin`; the latter would overwrite manually supplied input.
/// Add the plugins under test, then call `finish()` and `cleanup()` before using
/// [`crate::prelude::AppExt::step`]. This is not a full-frame/fixed-step runner.
pub struct InputTestPlugin;

impl Plugin for InputTestPlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(PreUpdate)
            .init_schedule(RunFixedMainLoop)
            .init_schedule(Update)
            .init_schedule(PostUpdate)
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<AccumulatedMouseMotion>()
            .init_resource::<AccumulatedMouseScroll>()
            .init_resource::<Touches>()
            .add_message::<AppExit>();
    }

    fn finish(&self, app: &mut App) {
        let world = app.world_mut();
        if world
            .query_filtered::<Entity, With<PrimaryWindow>>()
            .iter(world)
            .next()
            .is_none()
        {
            world.spawn((
                Window {
                    focused: true,
                    ..default()
                },
                PrimaryWindow,
                CursorOptions::default(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{AppExt, TestRunnerPlugin};
    use bevy::ecs::schedule::{InternedScheduleLabel, ScheduleLabel};
    use std::time::Duration;

    #[test]
    fn complements_runner_without_replacing_an_existing_window() {
        let mut app = App::new();
        app.add_plugins((TestRunnerPlugin::default(), InputTestPlugin));
        let window = app
            .world_mut()
            .spawn((
                Window {
                    focused: false,
                    ..default()
                },
                PrimaryWindow,
            ))
            .id();
        app.finish();
        app.cleanup();
        app.key(KeyCode::KeyW, true);
        app.mouse(MouseButton::Left, true);
        app.motion(Vec2::ONE);
        app.add_systems(
            Update,
            |keys: Res<ButtonInput<KeyCode>>, motion: Res<AccumulatedMouseMotion>| {
                assert!(keys.just_pressed(KeyCode::KeyW));
                assert_eq!(motion.delta, Vec2::ONE);
            },
        );
        app.step(Duration::from_millis(10), [Update.intern()]);
        assert!(
            app.world()
                .resource::<ButtonInput<KeyCode>>()
                .pressed(KeyCode::KeyW)
        );
        assert!(
            !app.world()
                .resource::<ButtonInput<KeyCode>>()
                .just_pressed(KeyCode::KeyW)
        );
        assert_eq!(
            app.world().resource::<AccumulatedMouseMotion>().delta,
            Vec2::ZERO
        );
        let world = app.world_mut();
        assert_eq!(
            world
                .query_filtered::<Entity, With<PrimaryWindow>>()
                .single(world)
                .unwrap(),
            window,
        );
        assert!(!world.get::<Window>(window).unwrap().focused);
    }

    #[test]
    fn held_buttons_persist_across_steps_until_released() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, InputTestPlugin));
        app.finish();
        app.cleanup();
        let world = app.world_mut();
        let (window, _) = world
            .query_filtered::<(&Window, &CursorOptions), With<PrimaryWindow>>()
            .single(world)
            .unwrap();
        assert!(window.focused);
        app.key(KeyCode::KeyW, true);
        app.mouse(MouseButton::Left, true);

        for _ in 0..2 {
            app.step(Duration::from_millis(10), [Update.intern()]);
            let keys = app.world().resource::<ButtonInput<KeyCode>>();
            let mouse = app.world().resource::<ButtonInput<MouseButton>>();
            assert!(keys.pressed(KeyCode::KeyW));
            assert!(mouse.pressed(MouseButton::Left));
            assert!(!keys.just_pressed(KeyCode::KeyW));
            assert!(!mouse.just_pressed(MouseButton::Left));
        }

        app.key(KeyCode::KeyW, false);
        app.mouse(MouseButton::Left, false);
        app.step(Duration::from_millis(10), [Update.intern()]);
        let keys = app.world().resource::<ButtonInput<KeyCode>>();
        let mouse = app.world().resource::<ButtonInput<MouseButton>>();
        assert!(!keys.pressed(KeyCode::KeyW));
        assert!(!mouse.pressed(MouseButton::Left));
        assert!(!keys.just_released(KeyCode::KeyW));
        assert!(!mouse.just_released(MouseButton::Left));
    }

    #[test]
    fn transients_are_available_to_all_schedules_then_cleared() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, InputTestPlugin));
        app.finish();
        app.cleanup();
        app.mouse(MouseButton::Left, true);
        app.step(Duration::ZERO, []);
        app.mouse(MouseButton::Left, false);
        app.key(KeyCode::KeyW, true);
        let motion_delta = Vec2::new(4.0, -2.0);
        let scroll_delta = Vec2::new(0.0, 3.0);
        app.motion(motion_delta);
        app.world_mut()
            .resource_mut::<AccumulatedMouseScroll>()
            .delta = scroll_delta;

        let schedules = [PreUpdate.intern(), Update.intern()];
        for schedule in schedules {
            app.add_systems(
                schedule,
                move |keys: Res<ButtonInput<KeyCode>>,
                      mouse: Res<ButtonInput<MouseButton>>,
                      motion: Res<AccumulatedMouseMotion>,
                      scroll: Res<AccumulatedMouseScroll>| {
                    assert!(keys.just_pressed(KeyCode::KeyW));
                    assert!(mouse.just_released(MouseButton::Left));
                    assert_eq!(motion.delta, motion_delta);
                    assert_eq!(scroll.delta, scroll_delta);
                },
            );
        }
        app.step(Duration::from_millis(10), schedules);

        assert!(
            !app.world()
                .resource::<ButtonInput<KeyCode>>()
                .just_pressed(KeyCode::KeyW)
        );
        assert!(
            !app.world()
                .resource::<ButtonInput<MouseButton>>()
                .just_released(MouseButton::Left)
        );
        assert_eq!(
            app.world().resource::<AccumulatedMouseMotion>().delta,
            Vec2::ZERO
        );
        assert_eq!(
            app.world().resource::<AccumulatedMouseScroll>().delta,
            Vec2::ZERO
        );
    }

    #[test]
    fn step_uses_only_requested_schedules_in_order_and_explicit_time() {
        #[derive(Resource, Default)]
        struct Trace(Vec<InternedScheduleLabel>);

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, InputTestPlugin));
        app.init_resource::<Trace>();
        let delta = Duration::from_millis(7);
        for schedule in [
            First.intern(),
            Startup.intern(),
            PreUpdate.intern(),
            Update.intern(),
            PostUpdate.intern(),
        ] {
            app.add_systems(
                schedule,
                move |mut trace: ResMut<Trace>, time: Res<Time>, real: Res<Time<Real>>| {
                    trace.0.push(schedule);
                    assert_eq!(time.delta(), delta);
                    assert_eq!(real.delta(), delta);
                    assert_eq!(time.elapsed(), delta);
                    assert_eq!(real.elapsed(), delta);
                },
            );
        }
        app.finish();
        app.cleanup();

        let requested = [Update.intern(), PreUpdate.intern(), Update.intern()];
        app.step(delta, requested);
        assert_eq!(app.world().resource::<Trace>().0, requested);

        let next_delta = Duration::from_millis(23);
        app.step(next_delta, []);
        assert_eq!(app.world().resource::<Trace>().0, requested);
        let time = app.world().resource::<Time>();
        let real = app.world().resource::<Time<Real>>();
        assert_eq!(time.delta(), next_delta);
        assert_eq!(real.delta(), next_delta);
        assert_eq!(time.elapsed(), delta + next_delta);
        assert_eq!(real.elapsed(), delta + next_delta);
    }
}
