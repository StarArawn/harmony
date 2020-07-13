/// WIP
use nalgebra_glm::Vec4;
use std::{path::PathBuf, convert::TryFrom, sync::Arc, collections::HashMap, task::Poll};
use super::{texture::Texture, new_asset_manager::AssetManager, texture_manager::TextureFuture};
use crate::graphics::{material::PBRMaterialUniform, resources::{BindGroup, GPUResourceManager}};
use futures::{future::Shared, executor::ThreadPool, task::AtomicWaker, Future};
use futures::{stream::FuturesUnordered, StreamExt};
use crossbeam::channel::{bounded, Receiver, TryRecvError};


#[derive(Clone, serde::Serialize, serde::Deserialize)]
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

pub trait Material<T: BindMaterial>: Clone {
    fn load_textures(&self) -> Vec<PathBuf>;
    fn create_material(&self, textures: Vec<Arc<Texture>>) -> T;
    fn get_layout(&self, gpu_resource_manager: &GPUResourceManager) -> Arc<wgpu::BindGroupLayout>;
}

impl Material<PBRMaterial> for PBRMaterialRon {
    fn load_textures(&self) -> Vec<PathBuf> {
        vec![
            self.main_texture.clone().into(),
            self.roughness_texture.clone().into(),
            self.normal_texture.clone().into(),
        ]
    }

    fn create_material(&self, mut textures: Vec<Arc<Texture>>) -> PBRMaterial {
       PBRMaterial {
            main_texture: Some(textures.remove(0)),
            roughness_texture: Some(textures.remove(0)),
            normal_texture: Some(textures.remove(0)),
            roughness: self.roughness,
            metallic: self.metallic,
            color: self.color,
        }
    }

    fn get_layout(&self, gpu_resource_manager: &GPUResourceManager) -> Arc<wgpu::BindGroupLayout> {
        gpu_resource_manager.get_bind_group_layout("pbr_material_layout").unwrap().clone()
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

#[derive(Clone)]
pub struct PBRMaterial {
    pub main_texture: Option<Arc<Texture>>,
    pub roughness_texture: Option<Arc<Texture>>,
    pub normal_texture: Option<Arc<Texture>>,
    pub roughness: f32,
    pub metallic: f32,
    pub color: Vec4,
}

pub trait BindMaterial {
    fn create_bindgroup(&self, device: Arc<wgpu::Device>, layout: Arc<wgpu::BindGroupLayout>) -> BindGroup;
}

impl BindMaterial for PBRMaterial {
    fn create_bindgroup(&self, device: Arc<wgpu::Device>, layout: Arc<wgpu::BindGroupLayout>) -> BindGroup {
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

// /// The future that resolves to a Texture
// pub struct MaterialFuture<T: Material + Send + Sync> {
//     bind_group_layout: Arc<wgpu::BindGroupLayout>,
//     ron_material: Arc<T>,
//     texture_futures: Option<FuturesUnordered<Shared<TextureFuture>>>,
//     loaded_textures: Vec<Arc<Texture>>,
//     device: Arc<wgpu::Device>,
//     queue: Arc<wgpu::Queue>,
//     pool: Arc<ThreadPool>,
//     waker: Arc<AtomicWaker>,
//     status: LoadStatus,
// }
// #[allow(unused)]
// impl<T> MaterialFuture<T>
// where T: Material + Send + Sync {
//     pub fn new(
//         bind_group_layout: Arc<wgpu::BindGroupLayout>,
//         ron_material: Arc<T>,
//         device: Arc<wgpu::Device>,
//         queue: Arc<wgpu::Queue>,
//         pool: Arc<ThreadPool>,
//         /*
//             TODO: This doesn't make sense if the texture is already loaded.. 
//             TODO: I'm not 100% sure why the asset manager `get` requires `&mut` if we could solve that we could share a
//             TODO: Arc<AssetManager> with this instead and I think some things will be much easier..
//             TODO: maybe Arc<Mutex<AssetManager>> though because we still need to mutate with load? :(
//         */
//         texture_futures: FuturesUnordered<Shared<TextureFuture>>,
//     ) -> Self {
//         Self {
//             bind_group_layout,
//             ron_material,
//             texture_futures: Some(texture_futures),
//             loaded_textures: Vec::new(),
//             device,
//             queue,
//             pool,
//             waker: Arc::new(AtomicWaker::new()),
//             status: LoadStatus::Start,
//         }
//     }
// }

// enum LoadStatus {
//     Start,
//     Uploading(Receiver<Arc<dyn BindMaterial>>),
// }

// impl<T> Future for MaterialFuture<T>
// where T: Material + Send + Sync + 'static {
//     type Output = Result<Arc<dyn BindMaterial>, Arc<std::io::Error>>;
//     fn poll(
//         mut self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Self::Output> {
//         match &self.status {
//             LoadStatus::Start => {
//                 let (tx, rx) = bounded(1);
//                 self.waker.register(cx.waker());
//                 let waker = self.waker.clone();
//                 let device = self.device.clone();
//                 let queue = self.queue.clone();
//                 let mut texture_futures = self.texture_futures.take().unwrap();
//                 let ron_material = self.ron_material.clone();
//                 let bind_group_layout = self.bind_group_layout.clone();
//                 self.pool.spawn_ok(async move {
//                     // First load textures
//                     // TODO: Again texture futures don't make sense it seems like it would be better to just have..
//                     // TODO: loaded_textures.push(asset_manager.get_async::<Texture>(path).await); which would always resolve to a Arc<Texture> ??
//                     let mut loaded_textures = Vec::new();
//                     while let Some(result) = texture_futures.next().await {
//                         if result.is_ok() {
//                             let texture = result.unwrap();
//                             loaded_textures.push(texture);
//                         } else {
//                             panic!("Texture failed to load!");
//                         }
//                     }

//                     let bind_material = ron_material.create_material(loaded_textures);

//                     // TODO: How to get `&mut gpu_resource_manager` here?
//                     // TODO: Maybe a Arc<Mutex<GPUResourceManager>>? Again not great to have mutexs everywhere, but perhaps its okay?
//                     let bind_group = bind_material.create_bindgroup(device, bind_group_layout);

//                     waker.wake();
//                 });
//                 self.get_mut().status = LoadStatus::Uploading(rx);
//                 std::task::Poll::Pending
//             }
//             LoadStatus::Uploading(rx) => match rx.try_recv() {
//                 Ok(material) => Poll::Ready(Ok(material)),
//                 Err(TryRecvError::Empty) => {
//                     self.waker.register(cx.waker());
//                     Poll::Pending
//                 }
//                 Err(e) => Poll::Ready(Err(Arc::new(std::io::Error::new(
//                     std::io::ErrorKind::BrokenPipe,
//                     e,
//                 )))),
//             },
//         }
//     }
// }
