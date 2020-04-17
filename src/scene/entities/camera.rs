use crate::scene::components::CameraData;
use specs::{Builder, Entity, World, WorldExt};

/// A function to help create a camera entity.
pub fn create(world: &mut World, camera_data: CameraData) -> Entity {
    world.create_entity().with(camera_data).build()
}
