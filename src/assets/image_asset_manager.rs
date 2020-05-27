use crate::graphics::material::{
    image::{ImageBuilder, ImageInfo},
    Image,
};
use std::{collections::HashMap, error::Error, path::PathBuf, sync::Arc};

// Types
pub(crate) type ImageInfoAssetManager = assetmanage_rs::Manager<ImageInfo>;
pub(crate) type ImageBuilderManager = assetmanage_rs::Manager<ImageBuilder>;
pub(crate) type ImageStorage = HashMap<PathBuf, Option<Arc<Image>>>;

pub struct ImageAssetManager {
    asset_path: PathBuf,
    image_info_manager: ImageInfoAssetManager,
    image_builder_manager: ImageBuilderManager,
    image_storage: ImageStorage,
}

impl ImageAssetManager {
    pub fn new<T: Into<PathBuf>>(asset_path: T) -> Self {
        let mut builder = assetmanage_rs::Builder::new();
        let image_info_manager = builder.create_manager::<ImageInfo>(());
        let image_builder_manager = builder.create_manager::<ImageBuilder>(());
        let loader = builder.finish_loader();
        async_std::task::spawn(loader.run());
        Self {
            asset_path: asset_path.into(),
            image_info_manager,
            image_builder_manager,
            image_storage: HashMap::new(),
        }
    }

    pub fn insert<T: Into<PathBuf>>(&mut self, path: T) -> Result<(), Box<dyn Error>> {
        let path = path.into();
        let mut full_path = self.asset_path.clone();
        full_path = full_path.join(&path);
        dbg!(&full_path);
        self.image_info_manager.insert(&path,());
        self.image_info_manager.load(&path)?;
        Ok(())
    }

    pub fn get<T: Into<PathBuf>>(&self, path: T) -> Option<Arc<Image>> {
        let path = path.into();
        let mut full_path = self.asset_path.clone();
        full_path = full_path.join(&path);
        self.image_storage.get(&path)?.clone()
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.image_info_manager.maintain();
        self.image_builder_manager.maintain();

        for path in self.image_info_manager.get_loaded_once() {
            let image_info = self.image_info_manager.get(&path).unwrap();
            log::info!("Loaded image info: {}", &path.to_str().unwrap());
            let mut image_path = path.clone();
            image_path.set_file_name(&image_info.file);
            self.image_builder_manager.insert(&image_path,image_info);
            if self.image_builder_manager.load(&image_path).is_err() {
                log::warn!("Image info not found! {:?}", &image_path);
                // If we drop here the key will be reused. It may be cheaper to keep it and if the image gets requested by get(key) it returns none and default can be used
                // self.image_info_manager.drop(key);
                // self.image_builder_manager.drop(key);
            }
        }
        for path in self.image_builder_manager.get_loaded_once() {

            let image_builder = self.image_builder_manager.get(&path).unwrap();
            

            let image = image_builder.build(device, queue);
            // TODO: Store image somewhere that can be accessed by users easily.
            log::info!("Loaded image: {}", image.image_info.file);
            let asset_path = self.asset_path.to_str().unwrap().to_string();
            self.image_storage
                .insert(path, Some(Arc::new(image)));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ImageAssetManager;
    #[test]
    fn it_works() {
        env_logger::init_from_env(
            env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
        );

        async_std::task::block_on(async {
            let instance = wgpu::Instance::new();
            let adapter = instance
                .request_adapter(
                    &wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::Default,
                        compatible_surface: None,
                    },
                    wgpu::BackendBit::PRIMARY,
                )
                .await
                .unwrap();

            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor {
                    extensions: wgpu::Extensions {
                        anisotropic_filtering: false,
                    },
                    limits: wgpu::Limits::default(),
                }, None)
                .await
                .unwrap();

            let mut asset_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/").to_string();
            let mut iam = ImageAssetManager::new(asset_path.clone());
            asset_path.push_str("core/white.image.ron");
            iam.insert(&asset_path).unwrap();
            async_std::task::sleep(std::time::Duration::from_millis(50)).await;
            iam.update(&device, &queue);
            async_std::task::sleep(std::time::Duration::from_millis(50)).await;
            iam.update(&device, &queue);
            iam.get(&asset_path);
        });
    }
}
