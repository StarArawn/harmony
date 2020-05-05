use super::Image;
use crate::{AssetManager, graphics::pipeline::BindGroupWithData};
use bytemuck::{Pod, Zeroable};
use nalgebra_glm::Vec4;
use std::{collections::HashMap, mem, sync::Arc};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UnlitUniform {
    pub color: Vec4,
}

unsafe impl Zeroable for UnlitUniform {}
unsafe impl Pod for UnlitUniform {}

pub struct UnlitMaterial {
    pub index: u32,
    pub main_texture: String,
    pub color: Vec4,
    pub(crate) bind_group_data: Option<BindGroupWithData>,
}

impl UnlitMaterial {
    pub fn new<T>(main_texture: T, color: Vec4, material_index: u32) -> Self
    where
        T: Into<String>,
    {
        let main_texture = main_texture.into();
        Self {
            index: material_index,
            main_texture: main_texture.clone(),
            color,
            bind_group_data: None,
        }
    }

    // Note: local_bind_group_layout needs to be passed in from the pipeline.
    // Be careful here to make sure the layout of the pipeline matches our layout here.
    pub(crate) fn create_bind_group(
        &mut self,
        asset_manager: &AssetManager, //should be material
        device: &wgpu::Device,
        local_bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        let material_uniform_size = mem::size_of::<UnlitUniform>() as wgpu::BufferAddress;
        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            size: material_uniform_size,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            label: None,
        });

        let image = asset_manager.get_image_or_white(&self.main_texture);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &local_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0, // We'll use 1 for our local bindings.
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buf,
                        range: 0..material_uniform_size,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&image.view),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&image.sampler),
                },
            ],
            label: None,
        });

        self.bind_group_data = Some(BindGroupWithData {
            uniform_buf,
            bind_group,
        });
    }
}
