use super::Image;
use crate::{AssetManager, graphics::resources::BindGroup};
use bytemuck::{Pod, Zeroable};
use nalgebra_glm::Vec4;
use std::{collections::HashMap, mem};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PBRMaterialUniform {
    pub color: Vec4,
    pub info: Vec4,
}

unsafe impl Zeroable for PBRMaterialUniform {}
unsafe impl Pod for PBRMaterialUniform {}

pub struct PBRMaterial {
    pub index: u32,
    pub main_texture: String,
    pub roughness_texture: String,
    pub normal_texture: String,
    pub roughness: f32,
    pub metallic: f32,
    pub color: Vec4,
    pub uniform_buf: Option<wgpu::Buffer>,
}

impl PBRMaterial {
    pub fn new<T>(main_texture: T, normal_texture: T, roughness_texture: T, color: Vec4, material_index: u32) -> Self
    where
        T: Into<String>,
    {
        Self {
            index: material_index,
            main_texture: main_texture.into(),
            roughness_texture: roughness_texture.into(),
            normal_texture: normal_texture.into(),
            color,
            roughness: 0.0,
            metallic: 0.0,
            uniform_buf: None,
        }
    }

    pub(crate) fn create_bind_group<'a>(
        &mut self,
        asset_manager: &AssetManager, //should be material
        device: &wgpu::Device,
        pipeline_layout: &'a wgpu::BindGroupLayout,
    ) -> BindGroup {

        let uniform = PBRMaterialUniform {
            color: self.color,
            info: Vec4::new(self.metallic, self.roughness, 0.0, 0.0),
        };

        let material_uniform_size = mem::size_of::<PBRMaterialUniform>() as wgpu::BufferAddress;
        let uniform_buf = device.create_buffer_with_data(bytemuck::bytes_of(&uniform), wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST);
        self.uniform_buf = Some(uniform_buf);

        let main_image = asset_manager.get_image_or_white(&self.main_texture);
        
        let normal_image = asset_manager.get_image_or_white(&self.normal_texture);
        
        let roughness_image = asset_manager.get_image_or_white(&&self.roughness_texture);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: self.uniform_buf.as_ref().unwrap(),
                        range: 0..material_uniform_size,
                    },
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
