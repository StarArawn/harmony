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
    image_manager: ImageBuilderManager,
}

impl ImageAssetManager {
    pub fn new<T: Into<PathBuf>>(asset_path: T, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let mut builder = assetmanage_rs::Builder::new();
        let image_info_manager = builder.create_manager::<ImageInfo>(());
        let image_manager = builder.create_manager::<ImageBuilder>((device, queue));
        let loader = builder.finish_loader(());
        async_std::task::spawn(loader.run());
        Self {
            asset_path: asset_path.into(),
            image_info_manager,
            image_manager,
        }
    }

    pub fn insert<T: Into<PathBuf>>(&mut self, rel_path: T) -> Result<(), Box<dyn Error>> {
        let rel_path = rel_path.into();
        let full_path = self.asset_path.join(&rel_path);
        self.image_info_manager.insert(&full_path, rel_path);
        self.image_info_manager.load(&full_path)?;
        Ok(())
    }

    pub fn get<T: Into<PathBuf>>(&self, rel_path: T) -> Option<Arc<Image>> {
        let rel_path = rel_path.into();
        let image_info = self.image_info_manager.get(&self.asset_path.join(&rel_path))?;
        let rel_image_path = &image_info.file;
        let image = self.image_manager.get(self.asset_path.join(rel_image_path))?;
        Some(image)
    }
    pub fn status<T: Into<PathBuf>>(&self, rel_path: T) -> Option<assetmanage_rs::LoadStatus> {
        let rel_path = rel_path.into();
        let image_info = self.image_info_manager.get(&self.asset_path.join(&rel_path))?;
        let rel_image_path = &image_info.file;
        self.image_manager.status(self.asset_path.join(rel_image_path))
    }

    pub fn update(&mut self) {
        self.image_info_manager.maintain();
        self.image_manager.maintain();

        for path in self.image_info_manager.get_loaded_once() {
            let image_info = self.image_info_manager.get(&path).unwrap();
            log::info!("Loaded image info: {}", &path.to_str().unwrap());
            let rel_image_path = &image_info.file;
            let abs_image_path = self.asset_path.join(rel_image_path);
            self.image_manager.insert(&abs_image_path, image_info);
            println!("{:?}",&abs_image_path);

            if self.image_manager.load(&abs_image_path).is_err() {
                log::warn!("Image info not found! {:?}", &abs_image_path);
                // If we drop here the key will be reused. It may be cheaper to keep it and if the image gets requested by get(key) it returns none and default can be used
                // self.image_info_manager.drop(key);
                // self.image_builder_manager.drop(key);
            }
        }
        for path in self.image_manager.get_loaded_once(){
            log::info!("Loaded image:{:?}   {:?}", &path, self.image_manager.status(&path));
        }
        //for path in self.image_builder_manager.get_loaded_once() {
        //    let image = self.image_builder_manager.get(&path).unwrap();
        //    log::info!("Loaded image: {}", &image.image_info.file);
        //    // This needs to be a relative path from the exe folder(or in the examples case from the crate's folder).
        //    // To make this even more complex it seems like it's more user friendly to use the path of the image_info...
        //    // Ex. User loads: `example/textures/georgentor.image.ron` > which loads `georgentor_4k.hdr`
        //    // User requests image from manager using get() as if it was the ron file: `example/textures/georgentor.image.ron`.
        //    if let Some(asset_relative_file_path) = image.image_info.path.as_ref(){
        //        self.image_storage.insert(asset_relative_file_path.into(), Some(Arc::new(image)));
        //    } else {
        //        //The Image has no associated ron -> The ImageInfo was constructed from memory and has the same path as the image.
        //        self.image_storage.insert(path, Some(Arc::new(image)));
        //        unimplemented!();
        //    }
        //}
    }
}

#[cfg(test)]
mod tests {
    use crate::ImageAssetManager;
    use std::{sync::Arc, path::PathBuf};
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
            let arc_device = Arc::new(device);
            let arc_queue = Arc::new(queue);
            let mut asset_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            asset_path.push("assets");
            let mut iam = ImageAssetManager::new(asset_path.clone(), arc_device, arc_queue);
            let mut image_path = PathBuf::new();
            image_path.push("core");
            image_path.push("white.image.ron");
            iam.insert(&image_path).unwrap();
            async_std::task::sleep(std::time::Duration::from_millis(50)).await;
            iam.update();
            async_std::task::sleep(std::time::Duration::from_millis(50)).await;
            iam.update();
            
            println!("{:?}",iam.get(&image_path).is_some());
        });
    }
}
