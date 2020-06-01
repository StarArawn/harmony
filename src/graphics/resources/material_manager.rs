use super::ImageManager;
use crate::graphics::material::{MaterialRon, NewMaterial};
use assetmanage_rs::*;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
pub(crate) struct MaterialManager {
    base_path: PathBuf,

    ron_manager: Manager<MaterialRon, MemoryLoader>,
    image_manager: ImageManager,
    material_cache: HashMap<PathBuf, Arc<NewMaterial>>,
}
impl MaterialManager {
    pub(crate) fn new<T: Into<PathBuf>>(
        base_path: T,
        ron_manager: Manager<MaterialRon, MemoryLoader>,
        image_manager: ImageManager,
    ) -> Self {
        Self {
            base_path: base_path.into(),
            ron_manager,
            image_manager,
            material_cache: HashMap::new(),
        }
    }

    pub fn insert<T: Into<PathBuf>>(&mut self, base_rel_path: T) {
        let path = self.base_path.join(base_rel_path.into());
        self.ron_manager.insert(path, ());
    }
    pub fn insert_raw<T: Into<PathBuf>>(&mut self, base_rel_path: T, material: Arc<NewMaterial>) {
        let path = self.base_path.join(base_rel_path.into());
        self.material_cache.insert(path, material);
    }
    pub fn load<T: Into<PathBuf>>(&mut self, base_rel_path: T) -> Result<(), std::io::Error> {
        let path = self.base_path.join(base_rel_path.into());
        self.ron_manager.load(path, ())
    }
    pub fn get<T: Into<PathBuf>>(&mut self, base_rel_path: T) -> Option<Arc<NewMaterial>> {
        let path = self.base_path.join(base_rel_path.into());
        self.material_cache.get(&path).cloned()
        //if Ron not inserted return None
        //if Ron not loaded return None
        //if Ron loading return None
        //if Ron loaded dont construct here. Will be constructed on next call to maintain. return None
    }
    pub fn maintain(&mut self, device: &Arc<wgpu::Device>, queue: &Arc<wgpu::Queue>) {
        self.ron_manager.maintain();
        self.image_manager.maintain();
        for mat_ron in self.ron_manager.get_loaded_once() {
            match self.ron_manager.get(&mat_ron).unwrap().as_ref() {
                #[allow(unused)]
                MaterialRon::PBRMaterial {
                    main_texture,
                    main_texture_info,
                    roughness_texture,
                    roughness_texture_info,
                    normal_texture,
                    normal_texture_info,
                    roughness,
                    metallic,
                    color,
                } => {
                    let mut abs_path = mat_ron.clone();
                    abs_path.pop();
                    let main_texture = abs_path.join(main_texture);
                    let roughness_texture = abs_path.join(roughness_texture);
                    let normal_texture = abs_path.join(normal_texture);
                    self.image_manager.insert(&main_texture, mat_ron.clone());
                    self.image_manager
                        .insert(&roughness_texture, mat_ron.clone());
                    self.image_manager.insert(&normal_texture, mat_ron);
                    self.image_manager
                        .load(
                            &main_texture,
                            (
                                Arc::new(main_texture_info.to_owned()),
                                device.clone(),
                                queue.clone(),
                            ),
                        )
                        .map_err(|e| log::warn!("Image not loaded! {:?}", &e))
                        .ok();
                    self.image_manager
                        .load(
                            &roughness_texture,
                            (
                                Arc::new(roughness_texture_info.to_owned()),
                                device.clone(),
                                queue.clone(),
                            ),
                        )
                        .map_err(|e| log::warn!("Image not loaded! {:?}", &e))
                        .ok();
                    self.image_manager
                        .load(
                            &normal_texture,
                            (
                                Arc::new(normal_texture_info.to_owned()),
                                device.clone(),
                                queue.clone(),
                            ),
                        )
                        .map_err(|e| log::warn!("Image not loaded! {:?}", &e))
                        .ok();
                }
            }
        }

        for img in self.image_manager.get_loaded_once() {
            let mat_path = self.image_manager.data_asset(img).unwrap();
            if let Some(mat_ron) = self.ron_manager.get(mat_path) {
                let mut base = mat_path.clone();
                base.pop();
                if let Some(mat) = mat_ron.try_construct(base, &self.image_manager, &device) {
                    self.material_cache
                        .entry(mat_path.into())
                        .or_insert(Arc::new(mat));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MaterialManager;
    use crate::graphics::{
        material::{
            image::{ImageData},
            MaterialRon,
        },
        resources::GPUImageHandle,
    };
    use std::{path::PathBuf, sync::Arc, time::Duration};

    #[test]
    fn initial() {
        env_logger::init_from_env(
            env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
        );

        let (device, queue) = async_std::task::block_on(async {
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
                .request_device(
                    &wgpu::DeviceDescriptor {
                        extensions: wgpu::Extensions {
                            anisotropic_filtering: false,
                        },
                        limits: wgpu::Limits::default(),
                    },
                    None,
                )
                .await
                .unwrap();
            let arc_device = Arc::new(device);
            let arc_queue = Arc::new(queue);

            (arc_device, arc_queue)
        });

        let mut asset_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        asset_path.push("assets");

        let mut builder_memory = assetmanage_rs::Builder::new();
        let image_file_manager = builder_memory.create_manager::<ImageData>(());

        let mut builder_gpu = assetmanage_rs::Builder::new();
        let image_manager = builder_gpu.create_manager::<GPUImageHandle>(());
        let loader = builder_gpu.finish_loader(image_file_manager);
        async_std::task::spawn(loader.run());

        let ron_manager = builder_memory.create_manager::<MaterialRon>(());
        let mut material_manager = MaterialManager::new(asset_path, ron_manager, image_manager);
        let loader = builder_memory.finish_loader(());
        async_std::task::spawn(loader.run());

        let mut rel_image_path = PathBuf::new();
        rel_image_path.push("core");
        rel_image_path.push("material_test.ron");
        material_manager.insert(&rel_image_path);
        material_manager.load(&rel_image_path).unwrap();

        std::thread::sleep(Duration::from_millis(16));
        material_manager.maintain(&device, &queue);
        std::thread::sleep(Duration::from_millis(16));
        material_manager.maintain(&device, &queue);

        let t = material_manager.get(&rel_image_path);
        assert!(t.is_some());
    }
}
