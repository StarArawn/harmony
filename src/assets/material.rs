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

trait IntoBindGroup {
    fn create_bindgroup(&mut self, device: Arc<wgpu::Device>, gpu_resource_manager: &mut GPUResourceManager, asset_manager: Arc<AssetManager>) -> BindGroup;
}

impl IntoBindGroup for PBRMaterial {
    fn create_bindgroup(&mut self, device: Arc<wgpu::Device>, gpu_resource_manager: &mut GPUResourceManager, asset_manager: Arc<AssetManager>) -> BindGroup {
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
        let main_image = texture_manager.get(&self.main_texture);

        let normal_image = texture_manager.get(&self.normal_texture);

        let roughness_image = texture_manager.get(&self.roughness_texture);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&main_image.sampler),
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

        BindGroup::new(2, bind_group)
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
}