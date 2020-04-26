use crate::scene::components::{LightType, Transform};
use legion::prelude::*;

pub fn create(world: &mut World, light_type: LightType, transform: Transform) -> &[Entity] {
    match light_type {
        LightType::Directional(data) => world.insert((), vec![(data, transform)]),
        LightType::Point(data) => world.insert((), vec![(data, transform)]),
    }
}
