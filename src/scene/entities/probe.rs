use legion::prelude::*;
use nalgebra_glm::Vec3;

use crate::{
    graphics::resources::{ProbeFormat, ProbeQuality},
    Application,
    scene::components,
};

pub fn create(app: &mut Application, position: Vec3, quality: ProbeQuality, format: ProbeFormat) -> Entity {
    let probe_id = {
        let device = app.resources.get::<wgpu::Device>().unwrap();
        app.probe_manager.create(Vec3::zeros(), &device, quality, format)
    };
    let probe_component = components::Probe {
        id: probe_id,
    };

    let mut transform = components::Transform::new(app);
    transform.position = position;
    
    app.current_scene.world.insert((), vec![(probe_component, transform)])[0]
}