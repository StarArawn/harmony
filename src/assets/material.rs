use super::{file_manager::AssetHandle, texture::Texture};
use crate::graphics::resources::{BindGroup, GPUResourceManager};
use bytemuck::{Pod, Zeroable};
use nalgebra_glm::Vec4;
use std::{convert::TryFrom, fmt::Debug, path::PathBuf, sync::Arc};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PBRMaterialUniform {
    pub color: Vec4,
    pub info: Vec4,
}

unsafe impl Zeroable for PBRMaterialUniform {}
unsafe impl Pod for PBRMaterialUniform {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PBRMaterialRon {
    pub main_texture: String,
    pub roughness_texture: String,
    pub normal_texture: String,
    pub roughness: f32,
    pub metallic: f32,
    pub color: Vec4,
}

impl TryFrom<(PathBuf, Vec<u8>)> for PBRMaterialRon {
    type Error = ron::de::Error;
    fn try_from((_p, v): (PathBuf, Vec<u8>)) -> Result<Self, Self::Error> {
        ron::de::from_bytes(&v)
    }
}

pub trait Material: Clone {
    type BindMaterialType: BindMaterial + Debug + Send + Sync;

    fn load_textures(&self) -> Vec<PathBuf>;
    fn create_material(&self, textures: Vec<Arc<AssetHandle<Texture>>>) -> Self::BindMaterialType;
    fn get_layout(gpu_resource_manager: Arc<GPUResourceManager>) -> Arc<wgpu::BindGroupLayout>;
}

impl Material for PBRMaterialRon {
    type BindMaterialType = PBRMaterial;

    fn load_textures(&self) -> Vec<PathBuf> {
        vec![
            self.main_texture.clone().into(),
            self.roughness_texture.clone().into(),
            self.normal_texture.clone().into(),
        ]
    }

    fn create_material(&self, mut textures: Vec<Arc<AssetHandle<Texture>>>) -> PBRMaterial {
        PBRMaterial {
            main_texture: textures.remove(0),
            roughness_texture: textures.remove(0),
            normal_texture: textures.remove(0),
            roughness: self.roughness,
            metallic: self.metallic,
            color: self.color,
            bind_group: None,
        }
    }

    fn get_layout(gpu_resource_manager: Arc<GPUResourceManager>) -> Arc<wgpu::BindGroupLayout> {
        gpu_resource_manager
            .get_bind_group_layout("pbr_material_layout")
            .unwrap()
            .clone()
    }
}

#[derive(Clone)]
pub struct PBRMaterial {
    pub main_texture: Arc<AssetHandle<Texture>>,
    pub roughness_texture: Arc<AssetHandle<Texture>>,
    pub normal_texture: Arc<AssetHandle<Texture>>,
    pub roughness: f32,
    pub metallic: f32,
    pub color: Vec4,
    pub(crate) bind_group: Option<Arc<BindGroup>>,
}

impl std::fmt::Debug for PBRMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubMesh")
            .field("main_texture", &self.main_texture)
            .field("roughness_texture", &self.roughness_texture)
            .field("normal_texture", &self.normal_texture)
            .field("roughness", &self.roughness)
            .field("metallic", &self.metallic)
            .field("roughness", &self.color)
            .finish()
    }
}

pub trait BindMaterial {
    fn create_bindgroup(&mut self, device: Arc<wgpu::Device>, layout: Arc<wgpu::BindGroupLayout>);
}

impl BindMaterial for PBRMaterial {
    fn create_bindgroup(&mut self, device: Arc<wgpu::Device>, layout: Arc<wgpu::BindGroupLayout>) {
        let uniform = PBRMaterialUniform {
            color: self.color,
            info: Vec4::new(self.metallic, self.roughness, 0.0, 0.0),
        };

        // let material_uniform_size = std::mem::size_of::<PBRMaterialUniform>() as wgpu::BufferAddress;
        let uniform_buf = device.create_buffer_with_data(
            bytemuck::bytes_of(&uniform),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        // Asset manager will panic if image doesn't exist, but we don't want that.
        // So use get_image_option instead.

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("PBRMaterialSampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let main_texture = self.main_texture.get();
        let normal_texture = self.normal_texture.get();
        let roughness_texture = self.roughness_texture.get();

        // By this point these should be loaded. Panicing here is probably good.
        let main_texture = main_texture.unwrap();
        let normal_texture = normal_texture.unwrap();
        let roughness_texture = roughness_texture.unwrap();

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
                    resource: wgpu::BindingResource::TextureView(&main_texture.view),
                },
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::Binding {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&roughness_texture.view),
                },
            ],
            label: None,
        });

        self.bind_group = Some(Arc::new(BindGroup::new(2, bind_group)));
    }
}
