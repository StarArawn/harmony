use nalgebra_glm::Vec3;
use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    pub fn new(x: f32, y: f32, z: f32, distance: f32) -> Self {
        Self {
            normal: Vec3::new(x, y, z),
            distance,
        }
    }

    pub fn normalize(mut self) -> Self {
        let mag = self.normal.magnitude();

        self.normal /= mag;
        self.distance /= mag;

        self
    }

    pub fn distance(&self, point: Vec3) -> f32 {
        self.normal.dot(&point) + self.distance
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GpuPlane {
    pub data: [f32; 4],
}

unsafe impl Zeroable for GpuPlane { }
unsafe impl Pod for GpuPlane { }
