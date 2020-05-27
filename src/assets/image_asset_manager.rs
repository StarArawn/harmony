use crate::graphics::material::{
    image::{ImageBuilder, ImageInfo},
    Image,
};
use std::{collections::HashMap, error::Error, path::PathBuf, sync::Arc};

// Types
pub(crate) type ImageInfoAssetManager = assetmanage_rs::Manager<ImageInfo>;
pub(crate) type ImageBuilderManager = assetmanage_rs::Manager<ImageBuilder>;
pub(crate) type ImageStorage = HashMap<String, Option<Arc<Image>>>;

pub struct ImageAssetManager {
    asset_path: String,
    image_info_manager: ImageInfoAssetManager,
    image_builder_manager: ImageBuilderManager,
    image_storage: ImageStorage,
    temp_image_info: HashMap<usize, usize>,
}

impl ImageAssetManager {
    pub fn new(asset_path: String) -> Self {
        let mut builder = assetmanage_rs::Builder::new();
        let image_info_manager = builder.create_manager::<ImageInfo>();
        let image_builder_manager = builder.create_manager::<ImageBuilder>();
        let loader = builder.finish_loader();
        async_std::task::spawn(loader.run());
        Self {
            asset_path,
            image_info_manager,
            image_builder_manager,
            image_storage: HashMap::new(),
            temp_image_info: HashMap::new(),
        }
    }

    pub fn insert<T: Into<String>>(&mut self, path: T) -> Result<(), Box<dyn Error>> {
        let path = path.into();
        let mut full_path = self.asset_path.clone();
        full_path.push_str(&path);
        dbg!(&full_path);
        let id = self.image_info_manager.insert(PathBuf::from(full_path));
        self.image_info_manager.load(id)?;
        Ok(())
    }

    pub fn get<T: Into<String>>(&self, path: T) -> Option<Arc<Image>> {
        let path = path.into();
        let mut full_path = self.asset_path.clone();
        full_path.push_str(&path);
        self.image_storage.get(&path)?.clone()
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.image_info_manager.maintain();
        self.image_builder_manager.maintain();

        for key in self.image_info_manager.get_loaded_once() {
            let image_info = self.image_info_manager.get(key).unwrap();
            let mut path = self.image_info_manager.path(key).unwrap().clone();
            log::info!("Loaded image info: {}", path.to_str().unwrap());
            path.set_file_name(&image_info.file);
            let image_path = PathBuf::from(&self.asset_path).join(path);
            let image_builder_key = self.image_builder_manager.insert(PathBuf::from(image_path));
            if self.image_builder_manager.load(image_builder_key).is_err() {
                log::warn!("Image info not found! {:?}", image_info.file);
                // If we drop here the key will be reused. It may be cheaper to keep it and if the image gets requested by get(key) it returns none and default can be used
                // self.image_info_manager.drop(key);
                // self.image_builder_manager.drop(key);
            } else {
                self.temp_image_info.entry(image_builder_key).or_insert(key);
            }
        }
        for key in self.image_builder_manager.get_loaded_once() {
            let image_builder = self.image_builder_manager.get(key).unwrap();
            let image_info_key = self.temp_image_info.remove(&key).unwrap();
            let image_info = self.image_info_manager.get(image_info_key).unwrap();
            let image = image_builder.build(device, queue, image_info);
            // TODO: Store image somewhere that can be accessed by users easily.
            log::info!("Loaded image: {}", image.image_info.file);
            let mut image_path = self.image_info_manager.path(image_info_key).unwrap().to_str().unwrap().to_string();
            image_path = image_path.replace(&self.asset_path, "");
            self.image_storage
                .insert(image_path, Some(Arc::new(image)));
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
