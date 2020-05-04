use legion::prelude::*;
use nalgebra_glm::Vec3;

use crate::scene::components;
use super::{ProbeQuality, Probe, ProbeFormat};

/// Keeps track of probes matches them up with entities for updates.
/// TODO: Calculate probes based off of distance to camera. Prioritized baised off of distance.
/// TODO: Some how stream probes in an out depending on distance. We likely shouldn't keep them in memory.
pub struct ProbeManager {
    probes: Vec<Probe>,    
}

impl ProbeManager {
    pub fn new() -> Self {
        Self {
            probes: Vec::new(),
        }
    } 

    pub fn create(&mut self, position: Vec3, resources: &Resources, quality: ProbeQuality, format: ProbeFormat) -> u32 {
        let id = self.probes.len() as u32;
        self.probes.push(Probe::new(id, position, resources, quality, format));
        id
    }

    pub(crate) fn render(&mut self, resources: &mut Resources, scene: &mut crate::scene::Scene) {
        //TODO: Fix this as it's not very well optimized. Perhaps a oct tree would work better?

        let query = <(Read<components::Probe>, Read<components::Transform>)>::query();
        let probe_ids: Vec<(u32, Vec3)> = query.iter(&scene.world).map(|(probe_comp, transform)|  (probe_comp.id, transform.position)).collect();

        for (probe_id, position) in probe_ids {
            let probe = &mut self.probes[probe_id as usize];
            probe.position = position;
            probe.render_scene(resources, scene);
            probe.render_brdf(resources, scene);
            probe.has_rendered = true;
        }
    }
}