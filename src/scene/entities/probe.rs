use legion::prelude::*;
use nalgebra_glm::Vec3;

use crate::{
    graphics::resources::{ProbeFormat, ProbeQuality},
    scene::components,
    Application,
};

/// Creates a new probe at a specific position.
/// quality - Quality of the probe.
/// format - Format of the probe: HDR16 or HDR32.
pub fn create(
    app: &mut Application,
    position: Vec3,
    quality: ProbeQuality,
    format: ProbeFormat,
) -> Entity {
    let probe_id = {
        app.probe_manager
            .create(Vec3::zeros(), &app.resources, quality, format)
    };
    let probe_component = components::Probe { id: probe_id };

    let mut transform = components::Transform::new(app);
    transform.position = position;

    app.current_scene
        .world
        .insert((), vec![(probe_component, transform)])[0]
}
