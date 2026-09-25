use std::time::Duration;

use bevy::{
    ecs::{schedule::InternedScheduleLabel, system::ScheduleSystem},
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
};

use crate::{log_step, prelude::*};

pub trait AppExt {
    fn add_step<M>(
        &mut self,
        step: u32,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self;

    /// Presses/releases a key. Requires `InputTestPlugin`.
    /// Held input persists across `step` calls until explicitly released.
    fn key(&mut self, key: KeyCode, pressed: bool);

    /// Presses/releases a mouse button. Requires initialized input resources.
    fn mouse(&mut self, button: MouseButton, pressed: bool);

    /// Sets (rather than adds to) the accumulated motion for the next step.
    fn motion(&mut self, delta: Vec2);

    /// Advances real/generic time and runs only the supplied schedules, in order.
    /// Clears transient input flags and motion/scroll afterward; held buttons remain.
    ///
    /// Requires `InputTestPlugin` and time resources from `TestRunnerPlugin` or
    /// `MinimalPlugins`. This is not a full app update: it does not run
    /// startup/First, maintain messages, or advance virtual
    /// time/fixed simulation automatically. Do not use `InputPlugin` with manually
    /// supplied input, as its event processing would overwrite that input.
    /// Selected schedules can themselves run other schedules or modify time.
    /// Import `bevy::ecs::schedule::ScheduleLabel` to call `.intern()` on labels.
    fn step(&mut self, delta: Duration, schedules: impl IntoIterator<Item = InternedScheduleLabel>);
}
impl AppExt for App {
    /// Registers a system which runs in PostUpdate (after all screen events have occured).
    /// Will only run if the state is set to the specified value.
    fn add_step<M>(
        &mut self,
        step: u32,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.add_systems(
            PostUpdate,
            system.run_if(in_state(Step(step))).after(log_step),
        )
    }

    fn key(&mut self, key: KeyCode, pressed: bool) {
        let mut input = self.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        if pressed {
            input.press(key);
        } else {
            input.release(key);
        }
    }

    fn mouse(&mut self, button: MouseButton, pressed: bool) {
        let mut input = self.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        if pressed {
            input.press(button);
        } else {
            input.release(button);
        }
    }

    fn motion(&mut self, delta: Vec2) {
        self.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = delta;
    }

    fn step(
        &mut self,
        delta: Duration,
        schedules: impl IntoIterator<Item = InternedScheduleLabel>,
    ) {
        let world = self.world_mut();
        world.resource_mut::<Time<Real>>().advance_by(delta);
        world.resource_mut::<Time>().advance_by(delta);
        for schedule in schedules {
            world.run_schedule(schedule);
        }
        world.resource_mut::<ButtonInput<KeyCode>>().clear();
        world.resource_mut::<ButtonInput<MouseButton>>().clear();
        world.resource_mut::<AccumulatedMouseMotion>().delta = Vec2::ZERO;
        world.resource_mut::<AccumulatedMouseScroll>().delta = Vec2::ZERO;
    }
}
