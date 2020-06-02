use nalgebra_glm::Vec4;
use std::{path::PathBuf, sync::Arc};

use super::{
    image::ImageInfo,
     PBRMaterialUniform, UnlitUniform,
};
use crate::graphics::resources::{BindGroup, GPUImageHandle, ImageManager};
use assetmanage_rs::MemoryLoader;


pub(crate) enum NewMaterial {
    PBRMaterial {
        main_texture: Arc<GPUImageHandle>,
        roughness_texture: Arc<GPUImageHandle>,
        normal_texture: Arc<GPUImageHandle>,
        roughness: f32,
        metallic: f32,
        color: Vec4,
        uniform_buf: wgpu::Buffer,
    },
    UnlitMaterial{
        main_texture: Arc<GPUImageHandle>,
        color: Vec4,
        uniform_buf: wgpu::Buffer,
    },
}

impl NewMaterial {
    pub(crate) fn create_bind_group(
        &self,
        device: &wgpu::Device,
        pipeline_layout: &wgpu::BindGroupLayout,
    ) -> BindGroup {
        match self {
            #[allow(unused)]
            NewMaterial::PBRMaterial {
                main_texture,
                roughness_texture,
                normal_texture,
                roughness,
                metallic,
                color,
                uniform_buf,
            } => {
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &pipeline_layout,
                    bindings: &[
                        wgpu::Binding {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),
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
            },
            #[allow(unused)]
            NewMaterial::UnlitMaterial { 
                main_texture, 
                color ,
                uniform_buf,
            } => {
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &pipeline_layout,
                    bindings: &[
                        wgpu::Binding {
                            binding: 0, // We'll use 1 for our local bindings.
                            resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),
                        },
                        // wgpu::Binding {
                        //     binding: 1,
                        //     resource: wgpu::BindingResource::TextureView(&image.view),
                        // },
                        // wgpu::Binding {
                        //     binding: 2,
                        //     resource: wgpu::BindingResource::Sampler(&image.sampler),
                        // },
                    ],
                    label: None,
                });
                BindGroup::new(2, bind_group)
            }
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) enum MaterialRon {
    PBRMaterial {
        //TODO: PathBuf into info
        main_texture: ImageInfo,
        roughness_texture: ImageInfo,
        normal_texture: ImageInfo,
        roughness: f32,
        metallic: f32,
        color: [f32; 4],
    },
    UnlitMaterial{
        main_texture: ImageInfo,
        color: [f32; 4],
    },
}

impl MaterialRon {
    pub(crate) fn try_construct(
        &self,
        base: PathBuf,
        iam: &ImageManager,
        device: &wgpu::Device,
    ) -> Option<NewMaterial> {
        match self {
            #[allow(unused)]
            MaterialRon::PBRMaterial {
                main_texture,
                roughness_texture,
                normal_texture,
                roughness,
                metallic,
                color,
            } => {
                let main_texture = iam.get(base.join(&main_texture.path))?;
                let roughness_texture = iam.get(base.join(&roughness_texture.path))?;
                let normal_texture = iam.get(base.join(&normal_texture.path))?;

                let uniform = PBRMaterialUniform {
                    color: Vec4::from_column_slice(color),
                    info: Vec4::new(*metallic, *roughness, 0.0, 0.0),
                };

                //let material_uniform_size =
                //    std::mem::size_of::<PBRMaterialUniform>() as wgpu::BufferAddress;
                let uniform_buf = device.create_buffer_with_data(
                    bytemuck::bytes_of(&uniform),
                    wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                );

                Some(NewMaterial::PBRMaterial {
                    main_texture,
                    roughness_texture,
                    normal_texture,
                    roughness: *roughness,
                    metallic: *metallic,
                    color: Vec4::from_column_slice(color),
                    uniform_buf: uniform_buf,
                })
            },
            MaterialRon::UnlitMaterial { 
                main_texture, 
                color 
            } => {
                let main_texture = iam.get(base.join(&main_texture.path))?;

                let color = Vec4::from_column_slice(color);

                let material_uniform_size = std::mem::size_of::<UnlitUniform>() as wgpu::BufferAddress;
                let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
                    size: material_uniform_size,
                    usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                    label: None,
                });
                Some(NewMaterial::UnlitMaterial{
                    main_texture,
                    color,
                    uniform_buf,
                })
            }
        }
    }
}

impl assetmanage_rs::Asset<MemoryLoader> for MaterialRon {
    type ManagerSupplement = ();
    type AssetSupplement = ();
    type Structure = MaterialRon;
    fn construct(
        data_load: Vec<u8>,
        _data_ass: &Self::AssetSupplement,
        _data_mgr: &Self::ManagerSupplement,
    ) -> Result<Self::Structure, std::io::Error> {
        ron::de::from_bytes(&data_load)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}
