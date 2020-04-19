use bytemuck::{Pod, Zeroable};
use nalgebra_glm::{Vec3, Mat4};

mod unlit;
pub(crate) use unlit::{UnlitPipelineDesc};

mod pbr;
pub(crate) use pbr::{PBRPipelineDesc};


mod skybox;
pub(crate) use skybox::{SkyboxPipelineDesc, SkyboxUniforms};
use std::convert::TryInto;

pub(crate) mod equirectangular;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GlobalUniforms {
    pub view_projection: Mat4,
}

impl Default for GlobalUniforms {
    fn default() -> Self {
        Self {
            view_projection: Mat4::identity(),
        }
    }
}

unsafe impl Zeroable for GlobalUniforms {}
unsafe impl Pod for GlobalUniforms {}

pub const MAX_LIGHTS: usize = 10;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: Vec3::zeros(),
            color: Vec3::zeros(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub position: Vec3,
    pub attenuation: f32,
    pub color: Vec3,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            attenuation: 0.0,
            position: Vec3::zeros(),
            color: Vec3::zeros(),
        }
    }
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LightingUniform {
    pub TOTAL_DIRECTIONAL_LIGHTS: u32,
    pub TOTAL_POINT_LIGHTS: u32,
    pub directional_lights: [DirectionalLight; MAX_LIGHTS / 2],
    pub point_lights: [PointLight; MAX_LIGHTS / 2],
}

impl Default for LightingUniform {
    fn default() -> Self {
        Self {
            TOTAL_DIRECTIONAL_LIGHTS: 0,
            TOTAL_POINT_LIGHTS: 0,
            directional_lights: [DirectionalLight::default(), DirectionalLight::default(), DirectionalLight::default(), DirectionalLight::default(), DirectionalLight::default()],
            point_lights: [PointLight::default(), PointLight::default(), PointLight::default(), PointLight::default(), PointLight::default()],
        }
    }
}

unsafe impl Zeroable for LightingUniform {}
unsafe impl Pod for LightingUniform {}
