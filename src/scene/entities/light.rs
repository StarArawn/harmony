use crate::scene::components::{LightType, Transform};
use specs::{Builder, Entity, World, WorldExt};

// pub fn create(world: &mut World, light_type: LightType, transform: Transform) -> Entity {
//     match light_type {
//         LightType::Directional(data) => world.create_entity().with(data).with(transform).build(),
//         LightType::Point(data) => world.create_entity().with(data).with(transform).build(),
//     }
// }
