use std::{any::{TypeId}, convert::TryFrom, path::PathBuf, sync::Arc};
use legion::{systems::resource::Resource, prelude::Resources};
use super::{texture_manager::{TextureManager}, texture::Texture, material::{PBRMaterialRon, Material, BindMaterial}, file_manager::{AssetError, FileManager, AssetHandle}};

pub struct AssetManager {
    loaders: Resources,
    texture_manager: TextureManager,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

impl AssetManager {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let texture_manager = TextureManager::new(device.clone(), queue.clone());
        Self{ 
            loaders: Resources::default(),
            texture_manager,
            device,
            queue,
        }
    }

    pub fn register<T: Resource + TryFrom<(PathBuf, Vec<u8>)> + Unpin>(&mut self) {
        
        if self.loaders.contains::<FileManager<T>>() {
            log::warn!("Duplicate registration of key: {:?}", TypeId::of::<T>());
            return;
        }

        let loader = FileManager::<T>::new();
        self.loaders.insert(loader);
    }

    // Instantly returns Arc<AssetHandle<T>> from a path.
    // Note: You should only call `get` once per path.
    // TODO: Add better checking to make sure we don't load an asset more than once.
    pub fn get<T: Resource + TryFrom<(PathBuf, Vec<u8>)>, K: Into<PathBuf>>(&self, path: K) -> Arc<AssetHandle<T>> {
        let path = path.into();
        let loader = self.loaders.get_mut::<FileManager<T>>();

        if loader.is_none() {
            log::error!("Couldn't find asset loader for the requested file.");
        }

        let loader = loader.unwrap();

        loader.get(path)
    }

    // Instantly returns Arc<AssetHandle<Texture>> from a path.
    pub fn get_texture<K: Into<PathBuf>>(&self, path: K) -> Arc<AssetHandle<Texture>> {
        self.texture_manager.get(path)
    }
}