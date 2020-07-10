/// WIP
use nalgebra_glm::Vec4;
use std::{path::PathBuf, convert::TryFrom, sync::Arc, collections::HashMap, task::Poll};
use super::{texture::Texture, new_asset_manager::AssetManager, texture_manager::TextureFuture};
use crate::graphics::{material::PBRMaterialUniform, resources::{BindGroup, GPUResourceManager}};
use futures::{future::Shared, executor::ThreadPool, task::AtomicWaker, Future};
use futures::FutureExt;
use crossbeam::channel::{bounded, Receiver, TryRecvError};


#[derive(serde::Serialize, serde::Deserialize)]
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

pub trait Material {
    fn load_textures(&self) -> Vec<PathBuf>;
}

impl Material for PBRMaterialRon {
    fn load_textures(&self) -> Vec<PathBuf> {
        vec![
            self.main_texture.clone().into(),
            self.roughness_texture.clone().into(),
            self.normal_texture.clone().into(),
        ]
    }
}

// Handles transferring materials from CPU to GPU memory.
pub struct MaterialManager {
    gpu_materials: HashMap<PathBuf, BindGroup>
}

impl MaterialManager {
    pub fn new(asset_manager: &mut AssetManager) -> Self {
        asset_manager.register::<PBRMaterialRon>();

        Self {
            gpu_materials: HashMap::new(),
        }
    }


}

pub struct PBRMaterial {
    pub main_texture: Option<Arc<Texture>>,
    pub roughness_texture: Option<Arc<Texture>>,
    pub normal_texture: Option<Arc<Texture>>,
    pub roughness: f32,
    pub metallic: f32,
    pub color: Vec4,
}

impl From<PBRMaterialRon> for PBRMaterial {
    fn from(ron_material: PBRMaterialRon) -> Self {
        Self {
            main_texture: None,
            roughness_texture: None,
            normal_texture: None,
            roughness: ron_material.roughness,
            metallic: ron_material.metallic,
            color: ron_material.color,
        }
    }
}

pub trait BindTexture {
    fn bind_textures(&mut self, textures: Vec<Arc<Texture>>) { }
    fn create_bindgroup(&self, device: Arc<wgpu::Device>, gpu_resource_manager: &mut GPUResourceManager, asset_manager: &mut AssetManager) -> BindGroup;
}

impl BindTexture for PBRMaterial {
    fn bind_textures(&mut self, mut textures: Vec<Arc<Texture>>) {
        self.main_texture = Some(textures.remove(0));
        self.roughness_texture = Some(textures.remove(0));
        self.normal_texture = Some(textures.remove(0));
    }

    fn create_bindgroup(&self, device: Arc<wgpu::Device>, gpu_resource_manager: &mut GPUResourceManager, asset_manager: &mut AssetManager) -> BindGroup {
        let layout = gpu_resource_manager.get_bind_group_layout("pbr_material_layout").unwrap();

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
                    resource: wgpu::BindingResource::TextureView(&self.main_texture.as_ref().unwrap().view),
                },
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&self.normal_texture.as_ref().unwrap().view),
                },
                wgpu::Binding {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&self.roughness_texture.as_ref().unwrap().view),
                },
            ],
            label: None,
        });

        BindGroup::new(2, bind_group)
    }
}

/// The future that resolves to a Texture
pub struct MaterialFuture {
    material_ron: Arc<dyn Material>,
    texture_futures: Vec<Shared<TextureFuture>>,
    loaded_textures: Vec<Arc<Texture>>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pool: Arc<ThreadPool>,
    waker: Arc<AtomicWaker>,
    status: LoadStatus,
}
#[allow(unused)]
impl MaterialFuture {
    pub fn new(
        material_ron: Arc<dyn Material>,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        pool: Arc<ThreadPool>,
    ) -> Self {
        Self {
            material_ron,
            texture_futures: Vec::new(),
            loaded_textures: Vec::new(),
            device,
            queue,
            pool,
            waker: Arc::new(AtomicWaker::new()),
            status: LoadStatus::Start,
        }
    }
}

enum LoadStatus {
    Start,
    LoadingTextures(Receiver<Vec<Arc<Texture>>>),
    Uploading(Receiver<Arc<dyn Material>>),
}

impl Future for MaterialFuture {
    type Output = Result<Arc<dyn Material>, Arc<std::io::Error>>;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match &self.status {
            LoadStatus::Start => {
                let (tx, rx) = bounded(1);
                self.waker.register(cx.waker());
                let waker = self.waker.clone();
                // let imgdata = self.imgdata.clone();
                // let imgron = self.imgron.clone();
                let device = self.device.clone();
                let queue = self.queue.clone();
                self.pool.spawn_ok(async move {
                    // tx.send(Arc::new(Texture::new(device, queue, imgdata, imgron)))
                        // .expect("Error forwarding loaded data!");
                    waker.wake();
                });
                self.get_mut().status = LoadStatus::Uploading(rx);
                std::task::Poll::Pending
            }
            LoadStatus::LoadingTextures(rx) => match rx.try_recv() {
                Ok(textures) => {
                    Poll::Pending
                },
                Err(TryRecvError::Empty) => {
                    self.waker.register(cx.waker());
                    Poll::Pending
                }
                Err(e) => Poll::Ready(Err(Arc::new(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    e,
                )))),
            },
            LoadStatus::Uploading(rx) => match rx.try_recv() {
                Ok(material) => Poll::Ready(Ok(material)),
                Err(TryRecvError::Empty) => {
                    self.waker.register(cx.waker());
                    Poll::Pending
                }
                Err(e) => Poll::Ready(Err(Arc::new(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    e,
                )))),
            },
        }
    }
}