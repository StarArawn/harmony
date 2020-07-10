/// WIP
use nalgebra_glm::Vec4;
use std::{path::PathBuf, convert::TryFrom, sync::Arc};
use super::{texture::Texture, new_asset_manager::AssetManager};
use crate::graphics::{material::PBRMaterialUniform, resources::{BindGroup, GPUResourceManager}};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PBRMaterial {
    pub main_texture: String,
    pub roughness_texture: String,
    pub normal_texture: String,
    pub roughness: f32,
    pub metallic: f32,
    pub color: Vec4,
}

impl TryFrom<(PathBuf, Vec<u8>)> for PBRMaterial {
    type Error = ron::de::Error;
    fn try_from((_p, v): (PathBuf, Vec<u8>)) -> Result<Self, Self::Error> {
        ron::de::from_bytes(&v)
    }
}

pub trait IntoBindGroup {
    fn create_bindgroup(&self, device: Arc<wgpu::Device>, gpu_resource_manager: &mut GPUResourceManager, asset_manager: &mut AssetManager) -> Option<BindGroup>;
}

impl IntoBindGroup for PBRMaterial {
    fn create_bindgroup(&self, device: Arc<wgpu::Device>, gpu_resource_manager: &mut GPUResourceManager, asset_manager: &mut AssetManager) -> Option<BindGroup> {
        let layout = gpu_resource_manager.get_bind_group_layout("pbr_material_layout").unwrap();

        let uniform = PBRMaterialUniform {
            color: self.color,
            info: Vec4::new(self.metallic, self.roughness, 0.0, 0.0),
        };

        let material_uniform_size = std::mem::size_of::<PBRMaterialUniform>() as wgpu::BufferAddress;
        let uniform_buf = device.create_buffer_with_data(
            bytemuck::bytes_of(&uniform),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        // Asset manager will panic if image doesn't exist, but we don't want that.
        // So use get_image_option instead.

        let main_image = match asset_manager.get_texture(&self.main_texture) {
            async_filemanager::LoadStatus::Loaded(texture) => Some(texture),
            _ => None,
        };
        let normal_image = match asset_manager.get_texture(&self.normal_texture) {
            async_filemanager::LoadStatus::Loaded(texture) => Some(texture),
            _ => None,
        };
        let roughness_image = match asset_manager.get_texture(&self.roughness_texture) {
            async_filemanager::LoadStatus::Loaded(texture) => Some(texture),
            _ => None,
        };

        if main_image.is_none() || normal_image.is_none() || roughness_image.is_none() {
            return None;
        }

        let main_image = main_image.unwrap();
        let normal_image = normal_image.unwrap();
        let roughness_image = roughness_image.unwrap();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("PBRMaterialSampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v:  wgpu::AddressMode::Repeat,
            address_mode_w:  wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&main_image.view),
                },
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&normal_image.view),
                },
                wgpu::Binding {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&roughness_image.view),
                },
            ],
            label: None,
        });

        Some(BindGroup::new(2, bind_group))
    }
}

// Handles transferring materials from CPU to GPU memory.
pub struct MaterialManager {
}

impl MaterialManager {
    pub fn new(asset_manager: &mut AssetManager) -> Self {
        asset_manager.register::<PBRMaterial>();

        Self {

        }
    }

    pub fn upload<T: Into<String>, T2: IntoBindGroup + 'static>(device: Arc<wgpu::Device>, gpu_resource_manager: &mut GPUResourceManager, asset_manager: &mut AssetManager, name: T, material: T2) {
        let bind_group = material.create_bindgroup(device, gpu_resource_manager, asset_manager);

        
    }
}