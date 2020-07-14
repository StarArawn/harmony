
use std::{sync::Arc, path::PathBuf, convert::{TryFrom}, hash::Hash};
use futures::{executor::{ThreadPool, ThreadPoolBuilder}};

pub type AssetCache<T> = Arc<dashmap::DashMap<PathBuf, Result<Arc<T>, Arc<AssetError>>>>;

/// A handle to a texture that will eventually resolve to Result<Arc<T>, Arc<AssetError>>
#[derive(Debug, Clone)]
pub struct AssetHandle<T> {
    pub(crate) handle_id: PathBuf,
    cache: AssetCache<T>,
}

impl<T> Hash for AssetHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.handle_id.hash(state);
    }   
}

impl<T> PartialEq for AssetHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.handle_id == other.handle_id
    }
}

impl<T> Eq for AssetHandle<T> {
    
}

impl<T> AssetHandle<T>
where T: Send + Sync + 'static {
    pub(crate) fn new(id: PathBuf, cache: AssetCache<T>) -> Self {
        Self {
            handle_id: id,
            cache,
        }
    }

    // Retreves some result from the cache which could be the requested asset if loaded.
    // Will return AssetError in other cases.
    // If the asset doesn't exist in the cache this will return AssetError::Loading
    pub fn get(&self) -> Result<Arc<T>, Arc<AssetError>> {
        let asset_result = self.cache.get(&self.handle_id);
        if asset_result.is_none() {
            return Err(Arc::new(AssetError::Loading));
        }
        let asset_result = asset_result.unwrap(); 
        let asset = asset_result.as_ref();
        match asset {
            Ok(asset) => {
                return Ok(asset.clone());
            },
            Err(error) => {
                return Err(error.clone());
            },
        }
    }
}

#[derive(Debug)]
pub enum AssetError {
    // Thrown when the file wasn't found
    FileNotFound,
    // Thrown when the try_from fails.
    InvalidData,
    // Thrown when the asset hasn't loaded yet.
    Loading,
    // Thrown on some other IO error.
    OtherError(std::io::Error),
}

pub struct FileManager<T> {
    pool: Arc<ThreadPool>,
    cache: AssetCache<T>,
}

impl<T> FileManager<T> 
where T: TryFrom<(PathBuf, Vec<u8>)> + Send + Sync + 'static {
    pub fn new() -> Self {
        // TODO: One pool that we pass in is probably enough.
        let pool = Arc::new(ThreadPoolBuilder::new().pool_size(4).create().unwrap());
        let cache = Arc::new(dashmap::DashMap::new());
        Self {
            pool,
            cache,
        }
    }

    pub fn get<P: Into<PathBuf>>(&self, path: P) -> Arc<AssetHandle<T>> {
        let path = path.into();

        let asset_handle = Arc::new(AssetHandle::new(path.clone(), self.cache.clone()));

        if !self.cache.contains_key(&path) {
            let cache = self.cache.clone();

            let asset_thread_handle = asset_handle.clone();

            self.pool.spawn_ok(async move {
                let file = async_std::fs::read(path.clone()).await;
                let result = if file.is_ok() {
                    // Do something
                    let file = file.unwrap();
                    match T::try_from((path.clone(), file)) {
                        Ok(f) => Ok(Arc::new(f)),
                        Err(_e) => {
                            Err(Arc::new(AssetError::InvalidData))
                        }
                    }
                } else {
                    let error = file.err().unwrap();
                    match error.kind() {
                        std::io::ErrorKind::NotFound => {
                            Err(Arc::new(AssetError::FileNotFound))
                        },
                        _ => { Err(Arc::new(AssetError::OtherError(error))) }
                    }
                };

                cache.insert(asset_thread_handle.handle_id.clone(), result);
            });
        }

        asset_handle
    }
}

#[cfg(test)]
mod tests {
    use super::{AssetError, FileManager};
    use crate::assets::image::ImageRon;
    use crate::assets::image::ImageFormat;
    use crate::assets::material::PBRMaterialRon;
    use nalgebra_glm::Vec4;

    #[test]
    fn should_load_image_ron_file() {
        let file_manager = FileManager::<ImageRon>::new();
        let asset_handle = file_manager.get("./assets/core/white.png.ron");

        let asset = asset_handle.get();
        assert!(match *asset.err().unwrap() { AssetError::Loading => true, _ => false });

        std::thread::sleep(std::time::Duration::from_secs(1));

        let asset = asset_handle.get();
        assert!(asset.is_ok());

        let asset = asset.unwrap();
        assert!(asset.format == ImageFormat::SRGB);
    }

    #[test]
    fn should_load_material_ron_file() {
        let file_manager = FileManager::<PBRMaterialRon>::new();
        let asset_handle = file_manager.get("./assets/material.ron");

        let asset = asset_handle.get();
        assert!(match *asset.err().unwrap() { AssetError::Loading => true, _ => false });

        std::thread::sleep(std::time::Duration::from_secs(1));

        let asset = asset_handle.get();
        assert!(asset.is_ok());

        let asset = asset.unwrap();
        assert!(asset.color == Vec4::new(1.0, 1.0, 1.0, 1.0));
    }

    #[test]
    fn should_only_load_once() {
        let file_manager = FileManager::<PBRMaterialRon>::new();
        let asset_handle = file_manager.get("./assets/material.ron");

        let asset = asset_handle.get();
        assert!(match *asset.err().unwrap() { AssetError::Loading => true, _ => false });

        std::thread::sleep(std::time::Duration::from_secs(1));

        let asset = asset_handle.get();
        assert!(asset.is_ok());

        let asset_handle = file_manager.get("./assets/material.ron");
        let asset = asset_handle.get();
        assert!(asset.is_ok());
    }
}