use bevy::ecs::system::ScheduleSystem;

use crate::{log_step, prelude::*};

pub trait AppExt {
    fn add_step<M>(
        &mut self,
        step: u32,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self;
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
}
