use super::{
    file_manager::{AssetHandle, FileManager},
    material::Material,
    material_manager::MaterialManager,
    mesh_manager::MeshManager,
    shader_manager::ShaderManager,
    texture::Texture,
    texture_manager::TextureManager,
    Shader,
};
use crate::graphics::resources::GPUResourceManager;
use legion::{prelude::Resources, systems::resource::Resource};
use std::{any::TypeId, convert::TryFrom, fmt::Debug, path::PathBuf, sync::Arc};
use walkdir::WalkDir;

pub struct AssetManager {
    loaders: Resources,
    texture_manager: Arc<TextureManager>,
    shader_manager: Arc<ShaderManager>,
    mesh_manager: Arc<MeshManager>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    path: PathBuf,
    gpu_resource_manager: Arc<GPUResourceManager>,
}

impl AssetManager {
    pub fn new(
        path: PathBuf,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        gpu_resource_manager: Arc<GPUResourceManager>,
    ) -> Self {
        let texture_manager = Arc::new(TextureManager::new(device.clone(), queue.clone()));
        let shader_manager = Arc::new(ShaderManager::new(device.clone()));
        let mut loaders = Resources::default();

        let material_manager = Arc::new(MaterialManager::new(
            device.clone(),
            queue.clone(),
            texture_manager.clone(),
            gpu_resource_manager.clone(),
        ));
        let mesh_manager = Arc::new(MeshManager::new(device.clone(), material_manager.clone()));

        loaders.insert(material_manager);
        Self {
            loaders,
            texture_manager,
            shader_manager,
            mesh_manager,
            device,
            queue,
            path,
            gpu_resource_manager,
        }
    }

    pub fn load(&mut self) {
        for entry in WalkDir::new(&self.path) {
            let entry = entry.expect("Error: Could not access asset directory.");
            let file_name = entry.file_name().to_str().unwrap().to_string();
            let path = entry.into_path();
            if file_name.ends_with(".png")
                || file_name.ends_with(".jpg")
                || file_name.ends_with(".hdr")
            {
                self.get_texture(path);
            } else if file_name.ends_with(".shader") {
                self.get_shader(path);
            } else if file_name.ends_with(".gltf") {
                // self.get_mesh(path);
            }
        }
    }

    pub fn register<T: Resource + TryFrom<(PathBuf, Vec<u8>)>>(&mut self) {
        if self.loaders.contains::<FileManager<T>>() {
            log::warn!("Duplicate registration of key: {:?}", TypeId::of::<T>());
            return;
        }

        let loader = FileManager::<T>::new();
        self.loaders.insert(loader);
    }

    pub fn register_material<
        T: TryFrom<(PathBuf, Vec<u8>)> + Debug + Material + Send + Sync + 'static,
    >(
        &mut self,
    ) {
        if self.loaders.contains::<Arc<MaterialManager<T>>>() {
            log::warn!(
                "Duplicate registration of material key: {:?}",
                TypeId::of::<T>()
            );
            return;
        }

        let loader = MaterialManager::<T>::new(
            self.device.clone(),
            self.queue.clone(),
            self.texture_manager.clone(),
            self.gpu_resource_manager.clone(),
        );
        self.loaders.insert(Arc::new(loader));
    }

    // Instantly returns Arc<AssetHandle<T>> from a path.
    // Note: You should only call `get` once per path.
    // TODO: Add better checking to make sure we don't load an asset more than once.
    pub fn get<T: Resource + TryFrom<(PathBuf, Vec<u8>)>, K: Into<PathBuf>>(
        &self,
        path: K,
    ) -> Arc<AssetHandle<T>> {
        let path = self.path.join(path.into());
        let loader = self.loaders.get::<FileManager<T>>();

        if loader.is_none() {
            panic!("Couldn't find asset loader for the requested file.");
        }

        let loader = loader.unwrap();

        loader.get(path)
    }

    // Instantly returns Arc<AssetHandle<Texture>> from a path.
    pub fn get_texture<K: Into<PathBuf>>(&self, path: K) -> Arc<AssetHandle<Texture>> {
        let path = self.path.join(path.into());
        self.texture_manager.get(path)
    }

    // Instantly returns Arc<AssetHandle<Shader>> from a path.
    pub fn get_shader<K: Into<PathBuf>>(&self, path: K) -> Arc<AssetHandle<Shader>> {
        let path = self.path.join(path.into());
        self.shader_manager.get(path)
    }

    // Instantly returns a Arc<AssetHandle<T::BindMaterialType>> from a path.
    // Note: If materials have textures they take longer to load as it'll await the loading of the textures.
    pub fn get_material<
        T: TryFrom<(PathBuf, Vec<u8>)> + Debug + Material + Send + Sync + 'static,
        K: Into<PathBuf>,
    >(
        &self,
        path: K,
    ) -> Arc<AssetHandle<T::BindMaterialType>> {
        let path = self.path.join(path.into());
        let loader = self.loaders.get::<Arc<MaterialManager<T>>>();
        if loader.is_none() {
            panic!("Couldn't find material asset loader for the requested file.");
        }

        let loader = loader.unwrap();

        loader.get(path)
    }
}

#[cfg(test)]
mod tests {
    use super::super::file_manager::AssetError;
    use super::AssetManager;
    use crate::{
        assets::material::PBRMaterialRon,
        graphics::{pipelines::pbr::create_pbr_bindgroup_layout, resources::GPUResourceManager},
    };
    use std::{path::PathBuf, sync::Arc};

    #[test]
    fn should_load_material() {
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
        let gpu_resource_manager = Arc::new(GPUResourceManager::new(device.clone()));

        let pbr_bind_group_layout = create_pbr_bindgroup_layout(device.clone());
        gpu_resource_manager.add_bind_group_layout("pbr_material_layout", pbr_bind_group_layout);

        let mut asset_manager = AssetManager::new(
            PathBuf::from(""),
            device.clone(),
            queue.clone(),
            gpu_resource_manager,
        );

        asset_manager.register_material::<PBRMaterialRon>();
        let material_handle =
            asset_manager.get_material::<PBRMaterialRon, _>("./assets/material.ron");
        let material = material_handle.get();
        assert!(match *material.err().unwrap() {
            AssetError::Loading => true,
            _ => false,
        });

        std::thread::sleep(std::time::Duration::from_secs(1));

        let material = material_handle.get();
        assert!(material.is_ok());
    }
}
