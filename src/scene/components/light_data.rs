use nalgebra_glm::{ Vec3 };
use specs::{DenseVecStorage, Component};

pub enum LightType {
    Directional(DirectionalLightData),
    Point(PointLightData),
}

pub struct DirectionalLightData {
    pub direction: Vec3,
    pub color: Vec3,
}

impl Default for DirectionalLightData {
    fn default() -> Self {
        Self {
            direction: Vec3::zeros(),
            color: Vec3::zeros(),
        }
    }
}

impl Component for DirectionalLightData {
    type Storage = DenseVecStorage<Self>;
}

pub struct PointLightData {
    pub color: Vec3,
    pub attenuation: f32,
}

impl Default for PointLightData {
    fn default() -> Self {
        Self {
            color: Vec3::zeros(),
            attenuation: 0.0,
        }
    }
}

impl Component for PointLightData {
    type Storage = DenseVecStorage<Self>;
}
