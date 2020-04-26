use crate::scene::components::CameraData;
use legion::prelude::*;

/// A function to help create a camera entity.
pub fn create(world: &mut World, camera_data: CameraData) -> &[Entity] {
    world.insert((), vec![(camera_data,)])
}
