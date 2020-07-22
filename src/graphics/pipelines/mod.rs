use bytemuck::{Pod, Zeroable};
use nalgebra_glm::{Mat4, Vec4};

// mod unlit;
// pub(crate) use unlit::UnlitPipelineDesc;

pub mod pbr;

// mod line;
// pub(crate) use line::LinePipelineDesc;

pub mod mipmap;

pub(crate) mod brdf;

pub(crate) mod irradiance;
pub(crate) mod specular2;

pub(crate) mod realtime_sky;
pub(crate) mod skybox;

pub(crate) mod equirectangular;

// TODO: Move all global uniforms out of here.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GlobalUniform {
    pub view_projection: Mat4,
    pub camera_pos: Vec4,
    pub view: Mat4,
    pub projection: Mat4,
}

impl Default for GlobalUniform {
    fn default() -> Self {
        Self {
            view_projection: Mat4::identity(),
            camera_pos: Vec4::zeros(),
            view: Mat4::identity(),
            projection: Mat4::identity(),
        }
    }
}

unsafe impl Zeroable for GlobalUniform {}
unsafe impl Pod for GlobalUniform {}


// TODO: We can support more lights, but a uniform buffer probably isn't the best.
// We likely want to use wgpu's belt buffer.
pub const MAX_LIGHTS: usize = 16;

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
    pub view_position: Vec4,
    pub color: Vec4,
    pub attenuation: Vec4,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            attenuation: Vec4::zeros(),
            position: Vec4::zeros(),
            color: Vec4::zeros(),
            view_position: Vec4::zeros(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LightingUniform {
    pub cluster_count: [u32; 4],
    pub light_num: Vec4,
    pub directional_lights: [DirectionalLight; 4],
    pub point_lights: [PointLight; MAX_LIGHTS],
}

impl Default for LightingUniform {
    fn default() -> Self {
        Self {
            cluster_count: [0; 4],
            light_num: Vec4::zeros(),
            directional_lights: [
                DirectionalLight::default(); 4
            ],
            point_lights: [
                PointLight::default(); MAX_LIGHTS
            ],
        }
    }
}

unsafe impl Zeroable for LightingUniform {}
unsafe impl Pod for LightingUniform {}
