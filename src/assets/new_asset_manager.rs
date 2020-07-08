use async_filemanager::AsyncFileManager;
use std::{any::{TypeId}, convert::TryFrom, path::PathBuf, sync::Arc};
use legion::{systems::resource::Resource, prelude::Resources};
use super::{image::ImageRon, Image};

pub struct AssetManager {
    pool: Arc<futures::executor::ThreadPool>,
    loaders: Resources,
}

impl AssetManager {
    pub fn new() -> Self {
        let pool = Arc::new(futures::executor::ThreadPoolBuilder::new().create().unwrap());
        Self{ 
            pool,
            loaders: Resources::default(),
        }
    }

    pub fn register<T: Resource + TryFrom<(PathBuf, Vec<u8>)> + Unpin>(&mut self) {
        
        if self.loaders.contains::<AsyncFileManager<T>>() {
            log::warn!("Duplicate registration of key: {:?}", TypeId::of::<T>());
            return;
        }
        
        let loader = AsyncFileManager::<T>::new(self.pool.clone());
        self.loaders.insert(loader);
    }

    pub fn load<T: Resource + TryFrom<(PathBuf, Vec<u8>)> + Unpin, K: Into<PathBuf>>(&mut self, path: K) {
        let path: PathBuf = path.into();
        {
            let loader = self.loaders.get_mut::<AsyncFileManager<T>>();

            if loader.is_none() {
                panic!("Couldn't find asset loader for the requested file.");
            }
            let mut loader = loader.unwrap();
            futures::executor::block_on(loader.load(path.clone()));
        }
        // If the loaded asset is detected as an image based off of extension attempt to load a descriptor ron file.
        // The ron file may fail, but we don't really care as we can use default values.
        let ext = path.extension().unwrap().to_str().unwrap().to_string();
        if (ext.contains("jpg") || ext.contains("png") || ext.contains("hdr")) && !ext.contains("ron") {
            let mut path = path.clone();
            path.set_extension(format!("{}{}", ext,".ron"));
            self.load::<ImageRon, _>(path);
        }
    }

    pub fn get<T: Resource + TryFrom<(PathBuf, Vec<u8>)> + Unpin, K: Into<PathBuf>>(&mut self, path: K) -> async_filemanager::LoadStatus<T, async_filemanager::FileLoadFuture<T>>{
        let path = path.into();
        let loader = self.loaders.get_mut::<AsyncFileManager<T>>();

        if loader.is_none() {
            panic!("Couldn't find asset loader for the requested file.");
        }

        let mut loader = loader.unwrap();
        futures::executor::block_on(loader.get(path))
    }
}

#[cfg(test)]
mod tests {
    use super::AssetManager;
    use crate::assets::{image::ImageRon, Image};

    #[test]
    fn should_register() {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Warn)
            .filter_module("harmony", log::LevelFilter::Info)
            .init();

        let mut asset_manager = AssetManager::new();
        asset_manager.register::<Image>();
        asset_manager.register::<ImageRon>();
        asset_manager.load::<Image, _>("./assets/core/white.png");

        let image = asset_manager.get::<Image, _>("./assets/core/white.png");
        match image {
            async_filemanager::LoadStatus::NotLoading => {

            },
            async_filemanager::LoadStatus::Loading(_) => {
            },
            _ => panic!("Failed to load image correctly!"),
        }

        std::thread::sleep(std::time::Duration::from_millis(1000));

        let image = asset_manager.get::<Image, _>("./assets/core/white.png");
        match image {
            async_filemanager::LoadStatus::Loaded(data) => {
                assert!(data.width == 1);
                assert!(data.height == 1);
                assert!(data.data == [255, 255, 255, 255]);
            },
            async_filemanager::LoadStatus::Error(error) => {
                dbg!(error);
            },
            _ => panic!("Failed to load image correctly!"),
        }
    }
}