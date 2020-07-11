use async_filemanager::{AsyncFileManager};
use std::{any::{TypeId}, convert::TryFrom, path::PathBuf, sync::Arc};
use legion::{systems::resource::Resource, prelude::Resources};
use super::{image::ImageRon, Image, texture_manager::{TextureFuture, TextureManager}, texture::Texture};
use futures::{stream::FuturesUnordered, future::Shared, StreamExt};

pub struct AssetManager {
    pool: Arc<futures::executor::ThreadPool>,
    loaders: Resources,
    image_futures: FuturesUnordered<Shared<async_filemanager::FileLoadFuture<Image>>>,
    texture_manager: TextureManager,
    
}

impl AssetManager {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let pool = Arc::new(futures::executor::ThreadPoolBuilder::new().create().unwrap());
        let texture_manager = TextureManager::new(pool.clone(), device, queue);
        Self{ 
            pool,
            loaders: Resources::default(),
            image_futures: FuturesUnordered::new(),
            texture_manager,
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
        if TypeId::of::<Image>() == TypeId::of::<T>() {
            // Load ron file.
            let mut ron_path = path.clone();
            ron_path.set_extension(format!("{}{}", ext,".ron"));
            self.load::<ImageRon, _>(ron_path);
            
            let mut ron_image_loader = self.loaders.get_mut::<AsyncFileManager<ImageRon>>().unwrap();

            // Grab image.
            dbg!("Getting image future!");
            let mut loader = self.loaders.get_mut::<AsyncFileManager<Image>>().unwrap();
            let image_result = futures::executor::block_on(loader.get(&path));

            match image_result {
                async_filemanager::LoadStatus::Loading(img_future) => {
                    dbg!("Storing image future!");
                    self.image_futures.push(img_future);
                },
                async_filemanager::LoadStatus::Error(error) => {
                    dbg!(error);
                    panic!("Some sort of error");
                },
                _ => {}
            };
        }
    }

    pub fn get_texture<T: Into<PathBuf>>(&mut self, path: T) -> async_filemanager::LoadStatus<Texture, TextureFuture> {
        futures::executor::block_on(self.texture_manager.get(&path.into()))
    }

    pub fn maintain(&mut self) {
        let mut ron_image_loader = self.loaders.get_mut::<AsyncFileManager<ImageRon>>().unwrap();
        let image_futures = self.image_futures.by_ref();
        let texture_manager = &mut self.texture_manager;
        dbg!(image_futures.len());
        
        // Instead of block should this be a thread pool?
        futures::executor::block_on(async {
            while let Some(result) = image_futures.next().await {
                if result.is_ok() {
                    let image = result.unwrap();

                    // We need to grab the image ron file if its A loaded and B exists.
                    // If it doesn't exist Texture will use defaults.
                    let mut ron_path = image.path.clone();
                    let ext = ron_path.extension().unwrap().to_str().unwrap().to_string();
                    ron_path.set_extension(format!("{}{}", ext,".ron"));
                    let img_ron = match ron_image_loader.get(ron_path).await {
                        async_filemanager::LoadStatus::Loaded(img_ron) => {
                            Some(img_ron)
                        },
                        _ => None,
                    };

                    dbg!("Loaded texture!");
                    dbg!(&image.path);
                    texture_manager.load(&image.path.clone(), image, img_ron).await;
                }
            }
        });
    }

    pub fn get<T: Resource + TryFrom<(PathBuf, Vec<u8>)> + Unpin, K: Into<PathBuf>>(&mut self, path: K) -> async_filemanager::LoadStatus<T, async_filemanager::FileLoadFuture<T>>{
        let path = path.into();
        let loader = self.loaders.get_mut::<AsyncFileManager<T>>();

        if loader.is_none() {
            panic!("Couldn't find asset loader for the requested file.");
        }

        let mut loader = loader.unwrap();
        return futures::executor::block_on(loader.get(path));
    }
}

#[cfg(test)]
mod tests {
    // use super::AssetManager;
    // use crate::assets::{image::ImageRon, Image};

    #[test]
    fn should_register() {
        // env_logger::Builder::from_default_env()
        //     .filter_level(log::LevelFilter::Warn)
        //     .filter_module("harmony", log::LevelFilter::Info)
        //     .init();

        // let mut asset_manager = AssetManager::new();
        // asset_manager.register::<Image>();
        // asset_manager.register::<ImageRon>();
        // asset_manager.load::<Image, _>("./assets/core/white.png");

        // let image = asset_manager.get::<Image, _>("./assets/core/white.png");
        // match image {
        //     async_filemanager::LoadStatus::NotLoading => {

        //     },
        //     async_filemanager::LoadStatus::Loading(_) => {
        //     },
        //     _ => panic!("Failed to load image correctly!"),
        // }

        // std::thread::sleep(std::time::Duration::from_millis(1000));

        // let image = asset_manager.get::<Image, _>("./assets/core/white.png");
        // match image {
        //     async_filemanager::LoadStatus::Loaded(data) => {
        //         assert!(data.width == 1);
        //         assert!(data.height == 1);
        //         assert!(data.data == [255, 255, 255, 255]);
        //     },
        //     async_filemanager::LoadStatus::Error(error) => {
        //         dbg!(error);
        //     },
        //     _ => panic!("Failed to load image correctly!"),
        // }
    }
}