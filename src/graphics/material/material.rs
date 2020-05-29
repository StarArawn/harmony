use nalgebra_glm::Vec4;
use std::{sync::Arc, collections::HashMap};

use crate::graphics::resources::GPUResourceManager;
use crate::graphics::resources::GPUImageHandle;
use super::Image;

struct PBRMaterial {
    pub index: u32,
    pub main_texture: Arc<Image>,
    pub roughness_texture: Arc<Image>,
    pub normal_texture: Arc<Image>,
    pub roughness: f32,
    pub metallic: f32,
    pub color: Vec4,
    pub uniform_buf: Option<wgpu::Buffer>,
}

pub enum Material {
    PBR(PBRMaterial),   
}

impl Material {
    pub fn ready(&self, gpu_image_manager: GPUImageManager, gpu_resource_manager: GPUResourceManager) -> bool {
        match self {
            Material::PBR(data) => {
                let color_image: Option<GPUImageHandle> = None; //gpu_image_manager.get(data.main_texture);
                let normal_image: Option<GPUImageHandle> = None; //gpu_image_manager.get(data.normal_texture);
                let roughness_texture: Option<GPUImageHandle> = None; //gpu_image_manager.get(data.roughness_texture);
                
                let images_ready = color_image.is_some() && normal_image.is_some() && roughness_texture.is_some();

                // Create bind group here if we are ready?

                return images_ready;
            },
        }
    }
}

pub struct MaterialNode {
    pub material: Material,
    pub can_render: bool,
}

struct MaterialManager {
    internal_materials: HashMap<usize, MaterialNode>,
}

impl MaterialManager {
    pub fn update(&mut self, gpu_image_manager: GPUImageManager, gpu_resource_manager: GPUResourceManager) {
        for material in self.internal_materials.values_mut() {
            // Maybe this should be async?
            let is_ready = material.material.ready(gpu_image_manager, gpu_resource_manager);
            material.can_render = is_ready;
        }
    }
}