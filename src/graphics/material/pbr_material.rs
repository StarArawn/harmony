use super::Image;
use crate::graphics::resources::BindGroup;
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
    pub fn new<T>(
        main_texture: T,
        normal_texture: T,
        roughness_texture: T,
        color: Vec4,
        material_index: u32,
    ) -> Self
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
        images: &HashMap<String, Image>,
        device: &wgpu::Device,
        pipeline_layout: &'a wgpu::BindGroupLayout,
    ) -> BindGroup {
        let uniform = PBRMaterialUniform {
            color: self.color,
            info: Vec4::new(self.metallic, self.roughness, 0.0, 0.0),
        };

        let _material_uniform_size = mem::size_of::<PBRMaterialUniform>() as wgpu::BufferAddress;
        let uniform_buf = device.create_buffer_with_data(
            bytemuck::bytes_of(&uniform),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );
        self.uniform_buf = Some(uniform_buf);

        // Asset manager will panic if image doesn't exist, but we don't want that.
        // So use get_image_option instead.
        let _main_image = images.get(&self.main_texture)
            .unwrap_or(
                images.get("white.png")
                    .unwrap_or_else(|| panic!("PBRMaterial Error: Couldn't find default white texture. Please make sure it exists in the asset folder or make sure your material's image can be found."))
            );

        let _normal_image = images.get(&self.normal_texture)
            .unwrap_or(
                images.get("white.png")
                    .unwrap_or_else(|| panic!("PBRMaterial Error: Couldn't find default white texture. Please make sure it exists in the asset folder or make sure your material's image can be found."))
            );

        let _roughness_image = images.get(&self.roughness_texture)
            .unwrap_or(
                images.get("white.png")
                    .unwrap_or_else(|| panic!("PBRMaterial Error: Couldn't find default white texture. Please make sure it exists in the asset folder or make sure your material's image can be found."))
            );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        self.uniform_buf.as_ref().unwrap().slice(..),
                    ),
                },
                // wgpu::Binding {
                //     binding: 1,
                //     resource: wgpu::BindingResource::Sampler(&main_image.sampler),
                // },
                // wgpu::Binding {
                //     binding: 2,
                //     resource: wgpu::BindingResource::TextureView(&main_image.view),
                // },
                // wgpu::Binding {
                //     binding: 3,
                //     resource: wgpu::BindingResource::TextureView(&normal_image.view),
                // },
                // wgpu::Binding {
                //     binding: 4,
                //     resource: wgpu::BindingResource::TextureView(&roughness_image.view),
                // },
            ],
            label: None,
        });

        BindGroup::new(2, bind_group)
    }
}
