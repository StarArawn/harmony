use bytemuck::{Pod, Zeroable};
use nalgebra_glm::{Mat4, Vec4};

mod unlit;
pub(crate) use unlit::{UnlitPipelineDesc};

mod pbr;
pub(crate) use pbr::{PBRPipelineDesc};


mod skybox;
pub(crate) use skybox::{SkyboxPipelineDesc};

pub(crate) mod equirectangular;
pub(crate) mod irradiance;

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
    pub direction: Vec4,
    pub color: Vec4,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: Vec4::zeros(),
            color: Vec4::zeros(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub position: Vec4,
    pub color: Vec4,
    pub attenuation: Vec4,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            attenuation: Vec4::zeros(),
            position: Vec4::zeros(),
            color: Vec4::zeros(),
        }
    }
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LightingUniform {
    pub light_num: Vec4,
    pub directional_lights: [DirectionalLight; MAX_LIGHTS / 2],
    pub point_lights: [PointLight; MAX_LIGHTS / 2],
}

impl Default for LightingUniform {
    fn default() -> Self {
        Self {
            light_num: Vec4::zeros(),
            directional_lights: [DirectionalLight::default(), DirectionalLight::default(), DirectionalLight::default(), DirectionalLight::default(), DirectionalLight::default()],
            point_lights: [PointLight::default(), PointLight::default(), PointLight::default(), PointLight::default(), PointLight::default()],
        }
    }
}

unsafe impl Zeroable for LightingUniform {}
unsafe impl Pod for LightingUniform {}
