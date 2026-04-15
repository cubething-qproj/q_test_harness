use crate::prelude::*;

#[derive(States, Debug, Default, Deref, DerefMut, Clone, Copy, Hash, Eq, PartialEq)]
pub struct Step(pub u32);

pub trait TestFn: Fn(&mut App) {}
impl<T> TestFn for T where T: Fn(&mut App) {}

#[derive(SystemSet, Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct TestRunnerSystems;
