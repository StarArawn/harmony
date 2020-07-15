use super::{
    file_manager::{AssetCache, AssetHandle},
    material::PBRMaterialRon,
    material_manager::MaterialManager,
    mesh::Gltf,
};
use futures::executor::{ThreadPool, ThreadPoolBuilder};
use std::{path::PathBuf, sync::Arc};

pub struct MeshManager {
    device: Arc<wgpu::Device>,
    pool: Arc<ThreadPool>,
    cache: AssetCache<Gltf>,
    material_manager: Arc<MaterialManager<PBRMaterialRon>>,
}

impl MeshManager {
    pub fn new(
        device: Arc<wgpu::Device>,
        material_manager: Arc<MaterialManager<PBRMaterialRon>>,
    ) -> Self {
        // TODO: One pool that we pass in is probably enough.
        let pool = Arc::new(ThreadPoolBuilder::new().pool_size(4).create().unwrap());
        let cache = Arc::new(dashmap::DashMap::new());
        Self {
            device,
            pool,
            cache,
            material_manager,
        }
    }

    pub fn get<P: Into<PathBuf>>(&self, path: P) -> Arc<AssetHandle<Gltf>> {
        let path = path.into();

        let asset_handle = Arc::new(AssetHandle::new(path.clone(), self.cache.clone()));

        if !self.cache.contains_key(&path) {
            let cache = self.cache.clone();

            let asset_thread_handle = asset_handle.clone();

            let device = self.device.clone();
            let material_manager = self.material_manager.clone();

            self.pool.spawn_ok(async move {
                let gltf = Gltf::from_gltf(device, material_manager, path).await;

                cache.insert(asset_thread_handle.handle_id.clone(), Ok(Arc::new(gltf)));
            });
        }

        asset_handle
    }
}
