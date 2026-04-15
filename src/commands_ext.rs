use crate::util::guess_entity_name;
use bevy::ecs::query::QueryFilter;

use crate::prelude::*;

pub trait CommandsExt {
    fn log_hierarchy(&mut self);
    fn find_entity(&mut self, name: impl ToString);
    fn find_no_entity(&mut self, name: impl ToString);
    fn find_entity_filtered<F: QueryFilter + 'static>(&mut self, name: impl ToString);
    fn find_no_entity_filtered<F: QueryFilter + 'static>(&mut self, name: impl ToString);
    fn find_entity_with<C: Component + PartialEq>(
        &mut self,
        name: impl ToString,
        value: C,
        invert: bool,
    );
    fn assert<S: ToString>(&mut self, condition: bool, error_message: S) -> bool;
}
impl<'w, 's> CommandsExt for Commands<'w, 's> {
    fn assert<S: ToString>(&mut self, condition: bool, error_message: S) -> bool {
        if !condition {
            error!("{}", error_message.to_string());
            self.write_message(AppExit::error());
        }
        condition
    }
    fn log_hierarchy(&mut self) {
        self.run_system_cached(|world: &mut World| {
            let mut root_query = world.query_filtered::<Entity, Without<ChildOf>>();
            let entities: Vec<_> = root_query.iter(world).collect();
            let mut output = String::new();
            log_hierarchy_inner(world, &mut output, entities, 0);
            info!("{output}")
        });
    }
    fn find_entity(&mut self, name: impl ToString) {
        self.run_system_cached_with(find_entity, (name.to_string(), false));
    }
    fn find_no_entity(&mut self, name: impl ToString) {
        self.run_system_cached_with(find_entity, (name.to_string(), true));
    }
    fn find_entity_filtered<F: QueryFilter + 'static>(&mut self, name: impl ToString) {
        self.run_system_cached_with(find_entity_filtered::<F>, (name.to_string(), false));
    }
    fn find_no_entity_filtered<F: QueryFilter + 'static>(&mut self, name: impl ToString) {
        self.run_system_cached_with(find_entity_filtered::<F>, (name.to_string(), true));
    }
    fn find_entity_with<C: Component + PartialEq>(
        &mut self,
        name: impl ToString,
        value: C,
        invert: bool,
    ) {
        self.run_system_cached_with(find_entity_with, (name.to_string(), invert, value));
    }
}

// TODO: Allow custom tags.
fn log_hierarchy_inner(world: &mut World, output: &mut String, entities: Vec<Entity>, depth: u32) {
    for &entity in &entities {
        let entity_name = guess_entity_name(world, entity);
        let mut tags = vec![];
        if world.entity(entity).get::<Observer>().is_some() {
            tags.push("Observer");
        }
        let indent = (0..depth).map(|_| "-").collect::<Vec<_>>().join("");
        #[allow(clippy::obfuscated_if_else)]
        let tags = (!tags.is_empty())
            .then(|| format!("<{}>", tags.join(", ")))
            .unwrap_or_default();

        *output = format!("{output}\n{indent}> {entity_name} {tags}");

        if let Some(children) = world.entity(entity).get::<Children>() {
            let children = children.iter().collect::<Vec<Entity>>();
            log_hierarchy_inner(world, output, children, depth + 1);
        }
    }
}

/// Searches for an entity with the given [Name] component.
/// This _will not_ show entities marked with [Internal], including Observers.
fn find_entity(input: In<(String, bool)>, q: Query<&Name>, mut commands: Commands) {
    let (name, invert) = input.0;
    let any = q.iter().any(|ename| (**ename).eq(&name));
    if (invert && any) || (!invert && !any) {
        commands.write_message(AppExit::error());
    }
}

/// Searches for an entity with the given [Name] component
fn find_entity_filtered<F: QueryFilter>(
    input: In<(String, bool)>,
    q: Query<&Name, F>,
    mut commands: Commands,
) {
    let (name, invert) = input.0;
    let any = q.iter().any(|ename| (**ename).eq(&name));
    if (invert && any) || (!invert && !any) {
        commands.write_message(AppExit::error());
    }
}

/// Searches for an entity with the given [Name] and component C.
fn find_entity_with<C: Component + PartialEq>(
    input: In<(String, bool, C)>,
    q: Query<(&Name, &C)>,
    mut commands: Commands,
) {
    let (name, invert, value) = input.0;
    let any = q
        .iter()
        .any(|(ename, c)| (**ename).eq(&name) && *c == value);
    if (invert && any) || (!invert && !any) {
        commands.write_message(AppExit::error());
    }
}

#[test]
fn test_commands() {
    #[derive(Component, PartialEq)]
    struct TestComponent;
    let mut app = App::new();
    app.add_plugins(TestRunnerPlugin {
        log_filter: "bevy=error,q_test_harness=info".into(),
        ..Default::default()
    });
    app.insert_resource(LogTestSteps(false));
    let mut cmds = app.world_mut().commands();
    cmds.spawn((Name::new("1"), children![Name::new("2")]));
    cmds.spawn((Name::new("3"), TestComponent));
    cmds.find_entity("1");
    cmds.find_entity("2");
    cmds.find_entity("3");
    cmds.find_no_entity("4");
    cmds.find_entity_with("3", TestComponent, false);
    cmds.find_entity_filtered::<With<ChildOf>>("2");
    cmds.find_no_entity_filtered::<Without<ChildOf>>("2");
    cmds.log_hierarchy();
    app.add_step(
        0,
        |mut reader: MessageReader<LogMessage>, mut commands: Commands| {
            let msgs = reader.read().collect::<Vec<_>>();
            info!(?msgs);
            if msgs.is_empty() {
                return;
            }
            if msgs[0].message == "Step = 0" {
                return;
            }
            let ok = msgs.iter().any(|msg| {
                let first = msg.message.find("> 1");
                let second = msg.message.find("-> 2");
                let third = msg.message.find("> 3");
                if let Some(idx1) = first
                    && let Some(idx2) = second
                    && let Some(idx3) = third
                {
                    idx1 < idx2 && idx2 < idx3
                } else {
                    false
                }
            });
            if ok {
                commands.write_message(AppExit::Success);
            } else {
                commands.write_message(AppExit::Error(std::num::NonZeroU8::new(1).unwrap()));
            }
        },
    );
    app.update();
}
