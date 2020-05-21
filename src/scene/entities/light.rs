use crate::scene::components::{LightType, Transform};
use legion::prelude::*;

/// Creates a new light depending on LightType, A transform must be passed in as well.
pub fn create(world: &mut World, light_type: LightType, transform: Transform) -> &[Entity] {
    match light_type {
        LightType::Directional(data) => world.insert((), vec![(data, transform)]),
        LightType::Point(data) => world.insert((), vec![(data, transform)]),
    }
}
