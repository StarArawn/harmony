use crate::graphics::material::{
    image::{ImageBuilder, ImageInfo},
    Image,
};
use std::{collections::HashMap, error::Error, path::PathBuf, sync::Arc};

// Types
pub(crate) type ImageInfoAssetManager = assetmanage_rs::Manager<ImageInfo,assetmanage_rs::MemoryLoader>;
pub(crate) type ImageBuilderManager = assetmanage_rs::Manager<ImageBuilder,assetmanage_rs::MemoryLoader>;
pub(crate) type ImageStorage = HashMap<PathBuf, Option<Arc<Image>>>;
// TODO: If we were able to pass the device and queue into the ImageBuilderManager on creation, we could build the image while decoding. The ImageBuilderManager could then become a ImageManager. ImageStorage could be removed. 
//       Current Blocker: Imgui fetches the queue as mut in Application::new, so we cannot fetch it twice there.

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
        self.image_info_manager.insert(&full_path, path);
        self.image_info_manager.load(&full_path)?;
        Ok(())
    }

    pub fn get<T: Into<PathBuf>>(&self, path: T) -> Option<Arc<Image>> {
        let path = path.into();
        dbg!(self.image_storage.keys());
        dbg!(&path);
        self.image_storage.get(&path)?.clone()
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.image_info_manager.maintain();
        self.image_builder_manager.maintain();

        for path in self.image_info_manager.get_loaded_once() {
            let image_info = self.image_info_manager.get(&path).unwrap();
            log::info!("Loaded image info: {}", &path.to_str().unwrap());
            let mut image_path = path.clone();
            // We do need to erase the file name, but..
            image_path.set_file_name("");
            // set_file_name wont work here since we want image_info.file to be a relative path to the image from where the image_info is located.
            image_path = image_path.join(&image_info.file);
            self.image_builder_manager.insert(&image_path, image_info);
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
            log::info!("Loaded image: {}", &image.image_info.file);
            // This needs to be a relative path from the exe folder(or in the examples case from the crate's folder).
            // To make this even more complex it seems like it's more user friendly to use the path of the image_info...
            // Ex. User loads: `example/textures/georgentor.image.ron` > which loads `georgentor_4k.hdr`
            // User requests image from manager using get() as if it was the ron file: `example/textures/georgentor.image.ron`.
            if let Some(asset_relative_file_path) = image.image_info.path.as_ref(){
                self.image_storage.insert(asset_relative_file_path.into(), Some(Arc::new(image)));
            } else {
                //The Image has no associated ron -> The ImageInfo was constructed from memory and has the same path as the image.
                self.image_storage.insert(path, Some(Arc::new(image)));
                unimplemented!();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ImageAssetManager;
    use std::path::PathBuf;
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

            let mut asset_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            asset_path.push("assets");
            let mut iam = ImageAssetManager::new(asset_path.clone());
            asset_path.push("core");
            asset_path.push("white.image.ron");
            iam.insert(&asset_path).unwrap();
            async_std::task::sleep(std::time::Duration::from_millis(50)).await;
            iam.update(&device, &queue);
            async_std::task::sleep(std::time::Duration::from_millis(50)).await;
            iam.update(&device, &queue);
            assert!(iam.get(&asset_path).is_some());
        });
    }
}
