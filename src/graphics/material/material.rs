use nalgebra_glm::Vec4;
use std::{path::PathBuf, sync::Arc};

use super::{
    image::ImageInfo,
     PBRMaterialUniform,
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
            }
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) enum MaterialRon {
    PBRMaterial {
        //TODO: PathBuf into info
        main_texture: PathBuf,
        main_texture_info: ImageInfo,
        roughness_texture: PathBuf,
        roughness_texture_info: ImageInfo,
        normal_texture: PathBuf,
        normal_texture_info: ImageInfo,
        roughness: f32,
        metallic: f32,
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
                main_texture_info,
                roughness_texture,
                roughness_texture_info,
                normal_texture,
                normal_texture_info,
                roughness,
                metallic,
                color,
            } => {
                let main_texture = iam.get(base.join(main_texture))?;
                let roughness_texture = iam.get(base.join(roughness_texture))?;
                let normal_texture = iam.get(base.join(normal_texture))?;

                let uniform = PBRMaterialUniform {
                    color: Vec4::from_column_slice(color),
                    info: Vec4::new(*metallic, *roughness, 0.0, 0.0),
                };

                let material_uniform_size =
                    std::mem::size_of::<PBRMaterialUniform>() as wgpu::BufferAddress;
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
