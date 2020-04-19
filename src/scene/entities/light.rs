use specs::{Builder, Entity, World, WorldExt};
use crate::scene::components::LightType;

pub fn create(world: &mut World, light_type: LightType) -> Entity {
    match light_type {
        LightType::Directional(data) => {
            world.create_entity().with(data).build()
        },
        LightType::Point(data) => {
            world.create_entity().with(data).build()
        },
    }
}