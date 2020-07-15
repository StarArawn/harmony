use super::{
    file_manager::{AssetCache, AssetHandle},
    shader::Shader,
};
use futures::executor::{ThreadPool, ThreadPoolBuilder};
use std::{path::PathBuf, sync::Arc};

pub struct ShaderManager {
    pool: Arc<ThreadPool>,
    cache: AssetCache<Shader>,
    device: Arc<wgpu::Device>,
}

impl ShaderManager {
    pub fn new(device: Arc<wgpu::Device>) -> Self {
        // TODO: One pool that we pass in is probably enough.
        let pool = Arc::new(ThreadPoolBuilder::new().pool_size(4).create().unwrap());
        let cache = Arc::new(dashmap::DashMap::new());
        Self {
            pool,
            cache,
            device,
        }
    }

    pub fn get<P: Into<PathBuf>>(&self, path: P) -> Arc<AssetHandle<Shader>> {
        let path = path.into();

        let asset_handle = Arc::new(AssetHandle::new(path.clone(), self.cache.clone()));

        if !self.cache.contains_key(&path) {
            let cache = self.cache.clone();

            let asset_thread_handle = asset_handle.clone();
            let device = self.device.clone();

            // TODO: Figure out why shaderc needs to be Send for this to use the pool..
            // TODO: Just fix this when naga comes out..
            // self.pool.spawn_ok(async move {
            // TODO: Make sure we return errors!!
            let shader = Shader::new(device, path.clone());

            cache.insert(asset_thread_handle.handle_id.clone(), Ok(shader));
            // });
        }

        asset_handle
    }
}

#[cfg(test)]
mod tests {
    use super::ShaderManager;
    use std::sync::Arc;

    #[test]
    fn should_load_shader() {
        let (_, device) = async_std::task::block_on(async {
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
            let (device, _) = adapter
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

            let device = Arc::new(device);

            (adapter, device)
        });

        let shader_manager = ShaderManager::new(device);
        let handle = shader_manager.get("./assets/core/shaders/pbr.shader");
        let shader = handle.get();
        assert!(shader.is_ok());
    }
}
