use crate::graphics::{resources::{BindGroup}};
use bytemuck::{Pod, Zeroable};
use nalgebra_glm::Vec4;
use std::{mem, collections::HashMap};
use super::Image;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PBRMaterialUniform {
    pub color: Vec4,
}

unsafe impl Zeroable for PBRMaterialUniform {}
unsafe impl Pod for PBRMaterialUniform {}

#[derive(Debug)]
pub struct PBRMaterial {
    pub index: u32,
    pub main_texture: String,
    pub color: Vec4,
    pub uniform_buf: Option<wgpu::Buffer>,
}

impl PBRMaterial {
    pub fn new<T>(main_texture: T, color: Vec4, material_index: u32) -> Self
    where
        T: Into<String>,
    {
        let main_texture = main_texture.into();
        Self {
            index: material_index,
            main_texture: main_texture.clone(),
            color,
            uniform_buf: None,
        }
    }

    pub(crate) fn create_bind_group<'a>(
        &mut self,
        images: &HashMap<String, Image>,
        device: &wgpu::Device,
        pipeline_layouts: &'a Vec<wgpu::BindGroupLayout>
    ) -> BindGroup { 
        let material_uniform_size = mem::size_of::<PBRMaterialUniform>() as wgpu::BufferAddress;
        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            size: material_uniform_size,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            label: None,
        });
        self.uniform_buf = Some(uniform_buf);

        // Asset manager will panic if image doesn't exist, but we don't want that.
        // So use get_image_option instead.
        let image = images.get(&self.main_texture)
            .unwrap_or(
                images.get("white.png")
                    .unwrap_or_else(|| panic!("PBRMaterial Error: Couldn't find default white texture. Please make sure it exists in the asset folder or make sure your material's image can be found."))
            );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline_layouts[1],
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
                    resource: wgpu::BindingResource::TextureView(&image.view),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&image.sampler),
                },
            ],
            label: None,
        });

        BindGroup::new(2, bind_group)
    }
    
}