use std::{path::PathBuf, sync::Arc, convert::TryFrom};
use futures::executor::{ThreadPoolBuilder, ThreadPool};
use super::{Image, file_manager::{AssetHandle, AssetCache, AssetError}, image::ImageRon, texture::Texture};

pub struct TextureManager {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pool: Arc<ThreadPool>,
    image_cache: AssetCache<Image>,
    ron_cache: AssetCache<ImageRon>,
    texture_cache: AssetCache<Texture>,
}

impl TextureManager {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
    ) -> Self {
        let pool = Arc::new(ThreadPoolBuilder::new().pool_size(4).create().unwrap());
        let image_cache = Arc::new(dashmap::DashMap::new());
        let ron_cache = Arc::new(dashmap::DashMap::new());
        let texture_cache = Arc::new(dashmap::DashMap::new());
        Self {
            device,
            queue,
            pool,
            image_cache,
            ron_cache,
            texture_cache,
        }
    }

    pub fn get<P: Into<PathBuf>>(&self, path: P) -> Arc<AssetHandle<Texture>> {
        let path = path.into();
        let texture_handle = Arc::new(AssetHandle::new(path.clone(), self.texture_cache.clone()));
        
        if !self.texture_cache.contains_key(&path) {
            let ext = path.extension().unwrap().to_str().unwrap().to_string();

            // Cross thread arcs passed to new thread.
            let image_cache = self.image_cache.clone();
            let ron_cache = self.ron_cache.clone();
            let texture_cache = self.texture_cache.clone();
            let texture_thread_handle = texture_handle.clone();
            let device = self.device.clone();
            let queue = self.queue.clone();

            self.pool.spawn_ok(async move {
                let mut ron_path = path.clone();
                ron_path.set_extension(format!("{}{}", ext,".ron"));
                let image_file = async_std::fs::read(path.clone()).await;
                let ron_file = async_std::fs::read(ron_path).await;

                let result = match image_file {
                    Ok(image_data) => {
                        // Attempt to load ron file..
                        let image_ron = if ron_file.is_ok() {
                            Some(ImageRon::try_from((path.clone(), ron_file.unwrap())).unwrap())
                        } else {
                            None
                        };

                        let image = Arc::new(Image::try_from((image_ron, path.clone(), image_data)).unwrap());
                        // Store image in cache.
                        image_cache.insert(texture_thread_handle.handle_id.clone(), Ok(image.clone()));

                        // TODO: Separate out loading into CPU from loading into the GPU.
                        let result = Ok(Arc::new(Texture::new(device, queue, image, image_ron, path.clone())));

                        let image_ron = match image_ron {
                            Some(ron) => {
                                Ok(Arc::new(ron))
                            },
                            None => {
                                Err(Arc::new(AssetError::FileNotFound))
                            }
                        };

                        ron_cache.insert(texture_thread_handle.handle_id.clone(), image_ron);

                        result
                    },
                    Err(error) => {
                        match error.kind() {
                            std::io::ErrorKind::NotFound => {
                                Err(Arc::new(AssetError::FileNotFound))
                            },
                            _ => { Err(Arc::new(AssetError::OtherError(error))) }
                        }
                    }
                };

                texture_cache.insert(texture_thread_handle.handle_id.clone(), result);
            });
        }

        texture_handle
    }

    // Assures the asset is loaded before returning the asset handle.
    pub async fn get_async<P: Into<PathBuf>>(&self, path: P) -> Arc<AssetHandle<Texture>> {
        let path = path.into();
        let texture_handle = Arc::new(AssetHandle::new(path.clone(), self.texture_cache.clone()));
        
        if !self.texture_cache.contains_key(&path) {
            let ext = path.extension().unwrap().to_str().unwrap().to_string();

            // Cross thread arcs passed to new thread.
            let image_cache = self.image_cache.clone();
            let ron_cache = self.ron_cache.clone();
            let texture_cache = self.texture_cache.clone();
            let texture_thread_handle = texture_handle.clone();
            let device = self.device.clone();
            let queue = self.queue.clone();

            let mut ron_path = path.clone();
            ron_path.set_extension(format!("{}{}", ext,".ron"));
            let image_file = async_std::fs::read(path.clone()).await;
            let ron_file = async_std::fs::read(ron_path).await;

            let result = match image_file {
                Ok(image_data) => {
                    // Attempt to load ron file..
                    let image_ron = if ron_file.is_ok() {
                        Some(ImageRon::try_from((path.clone(), ron_file.unwrap())).unwrap())
                    } else {
                        None
                    };

                    let image = Arc::new(Image::try_from((image_ron, path.clone(), image_data)).unwrap());
                    // Store image in cache.
                    image_cache.insert(texture_thread_handle.handle_id.clone(), Ok(image.clone()));

                    // TODO: Separate out loading into CPU from loading into the GPU.
                    let result = Ok(Arc::new(Texture::new(device, queue, image, image_ron, path.clone())));

                    let image_ron = match image_ron {
                        Some(ron) => {
                            Ok(Arc::new(ron))
                        },
                        None => {
                            Err(Arc::new(AssetError::FileNotFound))
                        }
                    };

                    ron_cache.insert(texture_thread_handle.handle_id.clone(), image_ron);

                    result
                },
                Err(error) => {
                    match error.kind() {
                        std::io::ErrorKind::NotFound => {
                            Err(Arc::new(AssetError::FileNotFound))
                        },
                        _ => { Err(Arc::new(AssetError::OtherError(error))) }
                    }
                }
            };

            texture_cache.insert(texture_thread_handle.handle_id.clone(), result);
        } else {
            loop {
                let result = texture_handle.get();
                if result.is_ok() {
                    break;
                }
            }
        }

        texture_handle
    }
}

#[cfg(test)]
mod tests {
    use super::TextureManager;
    use super::{AssetError};
    use std::sync::Arc;

    #[test]
    fn should_load_texture() {
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

        let texture_manager = TextureManager::new(device, queue);

        let handle = texture_manager.get("./assets/core/white.png");
        let asset = handle.get();
        assert!(match *asset.err().unwrap() { AssetError::Loading => true, _ => false });

        std::thread::sleep(std::time::Duration::from_secs(1));

        let asset = handle.get();
        assert!(asset.is_ok());
    }
}