use nalgebra_glm::Vec4;
use std::{sync::Arc, path::PathBuf};

use super::Image;
use assetmanage_rs::{Loader, MemoryLoader, Source};

pub(crate) struct PBRMaterial {
    pub index: u32,
    pub main_texture: Arc<Image>,
    pub roughness_texture: Arc<Image>,
    pub normal_texture: Arc<Image>,
    pub roughness: f32,
    pub metallic: f32,
    pub color: Vec4,
    pub uniform_buf: Option<wgpu::Buffer>,
}

pub(crate) enum NewMaterial {
    PBRMaterial {
        index: u32,
        main_texture: Arc<Image>,
        roughness_texture: Arc<Image>,
        normal_texture: Arc<Image>,
        roughness: f32,
        metallic: f32,
        color: Vec4,
        uniform_buf: Option<wgpu::Buffer>,
    },
    Test{
        id: u32
    },
}

//impl Material {
//    pub fn ready(&self, gpu_image_manager: GPUImageManager, gpu_resource_manager: GPUResourceManager) -> bool {
//        match self {
//            Material::PBR(data) => {
//                let color_image: Option<GPUImageHandle> = None; //gpu_image_manager.get(data.main_texture);
//                let normal_image: Option<GPUImageHandle> = None; //gpu_image_manager.get(data.normal_texture);
//                let roughness_texture: Option<GPUImageHandle> = None; //gpu_image_manager.get(data.roughness_texture);
//                
//                let images_ready = color_image.is_some() && normal_image.is_some() && roughness_texture.is_some();
//
//                // Create bind group here if we are ready?
//
//                return images_ready;
//            },
//        }
//    }
//}

pub(crate) struct MaterialNode {
    pub material: NewMaterial,
    pub can_render: bool,
}

//impl MaterialManager {
//    pub fn update(&mut self, gpu_image_manager: GPUImageManager, gpu_resource_manager: GPUResourceManager) {
//        for material in self.internal_materials.values_mut() {
//            // Maybe this should be async?
//            let is_ready = material.material.ready(gpu_image_manager, gpu_resource_manager);
//            material.can_render = is_ready;
//        }
//    }
//}

#[derive(serde::Deserialize)]
pub(crate) enum MaterialRon{
    PBRMaterial{
        main_texture: PathBuf,
        roughness_texture: PathBuf,
        normal_texture: PathBuf,
        roughness: f32,
        metallic: f32,
        color: [f32;4],
    },
}

impl assetmanage_rs::Asset<MemoryLoader> for MaterialRon{
    type ManagerSupplement = ();
    type AssetSupplement = ();
    type Structure = MaterialRon;
    fn construct(
        data_load: Vec<u8>,
        data_ass: &Self::AssetSupplement,
        data_mgr: &Self::ManagerSupplement,
    ) -> Result<Self::Structure, std::io::Error> {
        todo!()
    }
}
