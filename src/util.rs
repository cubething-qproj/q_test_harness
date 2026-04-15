// from bevy_inspector_egui
// https://github.com/jakobhellermann/bevy-inspector-egui/blob/9106021e8b80e441689ebbcb2c8f9da14e782426/crates/bevy-inspector-egui/src/utils.rs#L22
use bevy::ecs::prelude::Name;
use bevy::ecs::{archetype::Archetype, prelude::*, world::unsafe_world_cell::UnsafeWorldCell};

/// Guesses an appropriate entity name like `Light (6)` or falls back to `Entity (8)`
pub fn guess_entity_name(world: &World, entity: Entity) -> String {
    match world.get_entity(entity) {
        Ok(entity_ref) => {
            if let Some(name) = entity_ref.get::<Name>() {
                return format!("{} ({})", name.as_str(), entity);
            }

            guess_entity_name_inner(
                world.as_unsafe_world_cell_readonly(),
                entity,
                entity_ref.archetype(),
            )
        }
        Err(_) => format!("Entity {} (inexistent)", entity.index()),
    }
}

fn guess_entity_name_inner(
    world: UnsafeWorldCell<'_>,
    entity: Entity,
    archetype: &Archetype,
) -> String {
    #[rustfmt::skip]
        let associations = &[
            ("bevy_window::window::PrimaryWindow", "Primary Window"),
            ("bevy_camera::components::Camera3d", "Camera3d"),
            ("bevy_camera::components::Camera2d", "Camera2d"),
            ("bevy_light::point_light::PointLight", "PointLight"),
            ("bevy_light::directional_light::DirectionalLight", "DirectionalLight"),
            ("bevy_text::text::Text", "Text"),
            ("bevy_ui::ui_node::Node", "Node"),
            ("bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::pbr_material::StandardMaterial>", "Pbr Mesh"),
            ("bevy_window::window::Window", "Window"),
            ("bevy_ecs::observer::distributed_storage::Observer", "Observer"),
            ("bevy_window::monitor::Monitor", "Monitor"),
            ("bevy_picking::pointer::PointerId", "Pointer"),
        ];

    let type_names = archetype.components().iter().filter_map(|id| {
        let name = world.components().get_info(*id)?.name();
        Some(name)
    });

    for component_type in type_names {
        if let Some(name) = associations
            .iter()
            .find_map(|&(name, matches)| (component_type.to_string() == name).then_some(matches))
        {
            return format!("{name} ({entity})");
        }
    }

    format!("Entity ({entity})")
}
