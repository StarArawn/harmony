use std::{path::PathBuf, sync::Arc, collections::HashMap};
use crossbeam::queue::ArrayQueue;
use crate::graphics::material::{Image, image::{ImageInfo, ImageBuilder}};

// Types
pub(crate) type ImagePathQueue = ArrayQueue<String>;
pub(crate) type ImageInfoQueue = ArrayQueue<(String, usize)>;
pub(crate) type ImageBuilders = ArrayQueue<(String, String, Arc<ImageInfo>, usize)>; 
pub(crate) type ImageInfoAssetManager = assetmanage_rs::Manager<ImageInfo>;
pub(crate) type ImageBuilderManager = assetmanage_rs::Manager<ImageBuilder>;
pub(crate) type ImageStorage = HashMap<String, Option<Arc<Image>>>;

pub struct ImageAssetManager {
    pub(crate) asset_path: String,
    pub(crate) image_path_queue: ImagePathQueue,
    pub(crate) image_info_queue: ImageInfoQueue,
    pub(crate) image_builders_queue: ImageBuilders,
    pub(crate) image_info_manager: ImageInfoAssetManager,
    pub(crate) image_builder_manager: ImageBuilderManager,
    pub(crate) image_storage: ImageStorage,
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
            image_path_queue: ArrayQueue::new(100),
            image_info_queue: ArrayQueue::new(100),
            image_builders_queue: ArrayQueue::new(100),
            image_info_manager,
            image_builder_manager,
            image_storage: HashMap::new(),
        }
    }

    pub fn insert<T: Into<String>>(&mut self, path: T) {
        let path: String = path.into();
        let final_path = format!("{}{}", self.asset_path, path);
        self.image_path_queue.push(final_path.clone()).unwrap();
        self.image_storage.insert(final_path.clone(), None);
    }

    pub fn get<T: Into<String>>(&self, path: T) -> &Option<Arc<Image>> {
        let path: String = path.into();
        let final_path = format!("{}{}", self.asset_path, path);
        self.image_storage.get(&final_path).unwrap()
    }

    pub fn update(&mut self, device: &wgpu::Device) -> wgpu::CommandBuffer {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ImageUpload")
        });

        for _ in 0..self.image_path_queue.len() {
            let path = self.image_path_queue.pop().unwrap();
            let id = self.image_info_manager.insert(PathBuf::from(path.clone()));
            self.image_info_manager.load(id).unwrap();
            self.image_info_queue.push((path.clone(), id)).unwrap();
        }

        let mut image_info_still_loading = Vec::new();
        for _ in 0..self.image_info_queue.len() {
            let item = self.image_info_queue.pop().unwrap();
            let image_info = self.image_info_manager.get(item.1);
            if image_info.is_none() {
                image_info_still_loading.push(item);
            } else {
                // Get path to image.
                let full_path = PathBuf::from(&item.0);
                let original_file_name = full_path.file_name().unwrap().to_str().unwrap();
                let image_info: Arc<ImageInfo> = image_info.unwrap();
                let image_path = format!("{}{}", str::replace(&item.0, original_file_name, ""), image_info.file);
                dbg!(&image_path);
                let image_builder_id = self.image_builder_manager.insert(PathBuf::from(image_path.clone()));
                self.image_builder_manager.load(image_builder_id).unwrap();
                self.image_builders_queue.push((item.0, image_path, image_info.clone(), image_builder_id)).unwrap();
            }
        }

        let mut image_builders_still_loading = Vec::new();
        for _ in 0..self.image_builders_queue.len() {
            let data = self.image_builders_queue.pop().unwrap();

            let image_builder = self.image_builder_manager.get(data.3);

            if image_builder.is_none() {
                image_builders_still_loading.push(data);
            } else {
                let image_builder: Arc<ImageBuilder> = image_builder.unwrap();
                let image = image_builder.build(&device, &mut encoder, &data.2);
                // TODO: Store image somewhere that can be accessed by users easily.
                log::info!("Loaded image: {}", data.2.file);
                self.image_storage.insert(data.0, Some(Arc::new(image)));
            }
        }

        for _ in 0..image_info_still_loading.len() {
            let item = image_info_still_loading.pop().unwrap();
            self.image_info_queue.push(item).unwrap();
        }

        for _ in 0..image_builders_still_loading.len() {
            let item = image_builders_still_loading.pop().unwrap();
            self.image_builders_queue.push(item).unwrap();
        }

        self.image_info_manager.maintain();
        self.image_builder_manager.maintain();

        encoder.finish()
    }
}
