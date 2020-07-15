use super::{
    file_manager::{AssetCache, AssetError, AssetHandle},
    material::{BindMaterial, Material},
    texture_manager::TextureManager,
};
use crate::graphics::resources::GPUResourceManager;
use futures::executor::{ThreadPool, ThreadPoolBuilder};
use std::{convert::TryFrom, fmt::Debug, path::PathBuf, sync::Arc};

pub struct MaterialManager<T: Material> {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pool: Arc<ThreadPool>,
    ron_cache: AssetCache<T>,
    material_cache: AssetCache<T::BindMaterialType>,
    texture_manager: Arc<TextureManager>,
    gpu_resource_manager: Arc<GPUResourceManager>,
    asset_path: PathBuf,
}

impl<T> MaterialManager<T>
where
    T: TryFrom<(PathBuf, Vec<u8>)> + Debug + Material + Send + Sync + 'static,
{
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        texture_manager: Arc<TextureManager>,
        gpu_resource_manager: Arc<GPUResourceManager>,
        asset_path: PathBuf,
    ) -> Self {
        let pool = Arc::new(ThreadPoolBuilder::new().pool_size(4).create().unwrap());
        let material_cache = Arc::new(dashmap::DashMap::new());
        let ron_cache = Arc::new(dashmap::DashMap::new());
        Self {
            device,
            queue,
            pool,
            material_cache,
            ron_cache,
            texture_manager,
            gpu_resource_manager,
            asset_path,
        }
    }

    pub fn insert<K: Into<PathBuf>>(&self, material: T, relative_path: K) -> Arc<AssetHandle<T::BindMaterialType>> {
        let path = PathBuf::new();
        let path = path.join(
            uuid::Builder::nil()
                .set_version(uuid::Version::Random)
                .build()
                .to_string(),
        );

        let relative_path: PathBuf = relative_path.into();

        let material_handle = Arc::new(AssetHandle::new(path, self.material_cache.clone()));

        let material_cache = self.material_cache.clone();
        let ron_cache = self.ron_cache.clone();
        let texture_manager = self.texture_manager.clone();
        let material_thread_handle = material_handle.clone();
        let device = self.device.clone();
        let layout = T::get_layout(self.gpu_resource_manager.clone());
        let asset_path = self.asset_path.clone();

        self.pool.spawn_ok(async move {
            let material_arc = Arc::new(material);
            // Store ron material in cache.
            ron_cache.insert(
                material_thread_handle.handle_id.clone(),
                Ok(material_arc.clone()),
            );

            // TODO: Separate out loading into CPU from loading into the GPU?

            let texture_paths = material_arc.load_textures();
            let mut textures = Vec::new();
            for texture_path in texture_paths {
                let texture_handle = texture_manager.get_async(&relative_path.parent().unwrap().join(texture_path)).await;
                textures.push(texture_handle);
            }

            // TODO: Create bind_group possible here?
            let mut material = material_arc.create_material(textures);
            material.create_bindgroup(device.clone(), layout);

            material_cache.insert(
                material_thread_handle.handle_id.clone(),
                Ok(Arc::new(material)),
            );
        });

        material_handle
    }

    pub fn get<P: Into<PathBuf>>(&self, path: P) -> Arc<AssetHandle<T::BindMaterialType>> {
        let path = path.into();
        let material_handle = Arc::new(AssetHandle::new(path.clone(), self.material_cache.clone()));

        if !self.material_cache.contains_key(&path) {
            // Cross thread arcs passed to new thread.
            let material_cache = self.material_cache.clone();
            let ron_cache = self.ron_cache.clone();
            let texture_manager = self.texture_manager.clone();
            let material_thread_handle = material_handle.clone();
            let device = self.device.clone();
            let layout = T::get_layout(self.gpu_resource_manager.clone());
            let asset_path = self.asset_path.clone();

            self.pool.spawn_ok(async move {
                let ron_file = async_std::fs::read(path.clone()).await;

                let result = match ron_file {
                    Ok(data) => {
                        let material = match T::try_from((path.clone(), data)) {
                            Ok(f) => Ok(Arc::new(f)),
                            Err(_e) => Err(Arc::new(AssetError::InvalidData)),
                        };

                        match material {
                            Ok(material) => {
                                let material_arc = material.clone();

                                // Store ron material in cache.
                                ron_cache
                                    .insert(material_thread_handle.handle_id.clone(), Ok(material));

                                // TODO: Separate out loading into CPU from loading into the GPU?

                                let texture_paths = material_arc.load_textures();
                                let mut textures = Vec::new();
                                for texture_path in texture_paths {
                                    let texture_handle =
                                        texture_manager.get_async(&asset_path.clone().join(texture_path)).await;
                                    textures.push(texture_handle);
                                }

                                let mut material = material_arc.create_material(textures);
                                material.create_bindgroup(device.clone(), layout);

                                Ok(Arc::new(material))
                            }
                            Err(err) => {
                                // Store ron material in cache.
                                ron_cache.insert(
                                    material_thread_handle.handle_id.clone(),
                                    Err(err.clone()),
                                );
                                Err(err)
                            }
                        }
                    }
                    Err(error) => match error.kind() {
                        std::io::ErrorKind::NotFound => Err(Arc::new(AssetError::FileNotFound)),
                        _ => Err(Arc::new(AssetError::OtherError(error))),
                    },
                };

                material_cache.insert(material_thread_handle.handle_id.clone(), result);
            });
        }

        material_handle
    }

    pub fn get_all(&self) -> Vec<Arc<AssetHandle<T::BindMaterialType>>> {
        let material_cache = self.material_cache.clone();
        self.material_cache
            .iter()
            .map(|item| {
                Arc::new(AssetHandle::<T::BindMaterialType>::new(
                    item.key().clone(),
                    material_cache.clone(),
                ))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::AssetError;
    use super::MaterialManager;
    use crate::{
        assets::{material::PBRMaterialRon, texture_manager::TextureManager},
        graphics::{pipelines::pbr::create_pbr_bindgroup_layout, resources::GPUResourceManager},
    };
    use std::sync::Arc;

    #[test]
    fn should_load_material() {
        let (_, device, queue) = async_std::task::block_on(async {
            let (needed_features, unsafe_features) =
                (wgpu::Features::empty(), wgpu::UnsafeFeatures::disallow());

            let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
            let adapter = instance
                .request_adapter(
                    &wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::Default,
                        compatible_surface: None,
                    },
                    unsafe_features,
                )
                .await
                .unwrap();

            let adapter_features = adapter.features();
            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        features: adapter_features & needed_features,
                        limits: wgpu::Limits::default(),
                        shader_validation: true,
                    },
                    None,
                )
                .await
                .unwrap();
            let arc_device = Arc::new(device);
            let arc_queue = Arc::new(queue);
            (adapter, arc_device, arc_queue)
        });

        let texture_manager = TextureManager::new(device.clone(), queue.clone());

        let gpu_resource_manager = Arc::new(GPUResourceManager::new(device.clone()));

        let pbr_bind_group_layout = create_pbr_bindgroup_layout(device.clone());
        gpu_resource_manager.add_bind_group_layout("pbr_material_layout", pbr_bind_group_layout);

        let material_manager = MaterialManager::<PBRMaterialRon>::new(
            device,
            queue,
            Arc::new(texture_manager),
            gpu_resource_manager,
        );
        let material_handle = material_manager.get("./assets/material.ron");
        let material = material_handle.get();
        assert!(match *material.err().unwrap() {
            AssetError::Loading => true,
            _ => false,
        });

        std::thread::sleep(std::time::Duration::from_secs(1));

        let material = material_handle.get();
        assert!(material.is_ok());
    }
}
