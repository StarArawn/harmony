use super::Image;
use crate::graphics::pipeline::BindGroupWithData;
use bytemuck::{Pod, Zeroable};
use nalgebra_glm::Vec4;
use std::{collections::HashMap, mem};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UnlitUniform {
    pub color: Vec4,
}

unsafe impl Zeroable for UnlitUniform {}
unsafe impl Pod for UnlitUniform {}

#[derive(Debug)]
pub struct UnlitMaterial {
    pub index: i32,
    pub main_texture: String,
    pub color: Vec4,
    pub(crate) bind_group_data: Option<BindGroupWithData>,
}

impl UnlitMaterial {
    pub fn new<T>(main_texture: T, color: Vec4, material_index: i32) -> Self
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
        images: &HashMap<String, Image>,
        device: &wgpu::Device,
        local_bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        let material_uniform_size = mem::size_of::<UnlitUniform>() as wgpu::BufferAddress;
        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            size: material_uniform_size,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            label: None,
        });

        // Asset manager will panic if image doesn't exist, but we don't want that.
        // So use get_image_option instead.
        let image = images
            .get(&self.main_texture)
            .unwrap_or(
                images.get("white.png")
                    .unwrap_or_else(|| panic!("UnlitMaterial Error: Couldn't find default white texture. Please make sure it exists in the asset folder or make sure your material's image can be found."))
            );

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
