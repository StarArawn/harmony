pub mod globals;
pub mod line;
pub mod mesh;
pub mod render;
pub mod skybox;

use legion::prelude::*;
use legion::systems::schedule::Builder;
pub fn create_render_schedule_builder() -> Builder {
    Schedule::builder().add_system(skybox::create())
    // .add_system(line::create())
    // .add_system(mesh::create())
}
