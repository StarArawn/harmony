use log::*;
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    error::Error,
    hash::Hash,
    io::ErrorKind,
    sync::Arc,
};
use walkdir::WalkDir;

use crate::core::Font;
use crate::graphics::{
    material::{Image, Material, NewMaterialData, NewMaterialHandle, Shader},
    mesh::{GltfData, SubMesh},
    resources::GPUResourceManager,
};

pub struct AssetManager {
    path: String,
    shaders: HashMap<String, Shader>,
    fonts: HashMap<String, Font>,
    meshes: HashMap<String, SubMesh>,
    pub(crate) materials: HashMap<NewMaterialHandle, Option<Arc<NewMaterialData>>>,
    pub(crate) images: HashMap<String, Arc<Image>>,
    //TODO: store samplers
}

impl AssetManager {
    pub fn new(path: String) -> Self {
        AssetManager {
            path,
            shaders: HashMap::new(),
            fonts: HashMap::new(),
            meshes: HashMap::new(),
            materials: HashMap::new(),
            images: HashMap::new(),
        }
    }

    pub(crate) fn load(&mut self, device: &wgpu::Device, queue: &mut wgpu::Queue) {
        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        for entry in WalkDir::new(&self.path) {
            let entry = entry.expect("Error: Could not access file.");
            let file_name = entry.file_name().to_str().unwrap();
            let full_file_path = str::replace(
                entry.path().to_str().unwrap_or_else(|| {
                    panic!(format!(
                        "Error: could not get full file path: {}",
                        file_name
                    ))
                }),
                file_name,
                "",
            );
            if file_name.ends_with(".shader") {
                let shader =
                    Shader::new(&device, full_file_path.to_string(), file_name.to_string());
                self.shaders.insert(file_name.to_string(), shader);
                info!("Compiled shader: {}", file_name);
            }
            if file_name.ends_with(".ttf") || file_name.ends_with(".otf") {
                let font = Font::new(
                    &device,
                    format!("{}{}", full_file_path, file_name).to_string(),
                );
                self.fonts.insert(file_name.to_string(), font);
                info!("Loaded font: {}", file_name);
            }
            if file_name.ends_with(".gltf") {
                let current_index = self.materials.len() as u32;
                let gltf_data =
                    GltfData::load(&device, format!("{}{}", full_file_path, file_name)).unwrap();
                let mesh = gltf_data.mesh;
                for handle in mesh.material_handles {
                    self.materials.insert(handle, None);
                }
                for sub in mesh.sub_meshes {
                    self.meshes.insert(file_name.to_string(), sub);
                    info!("Loaded mesh: {}", file_name);
                }
            }
            if file_name.ends_with(".png") || file_name.ends_with(".jpg") {
                let image;
                if file_name.to_lowercase().contains("_normal")
                    || file_name.to_lowercase().contains("metallic")
                {
                    image =
                        Image::new_normal(&device, &mut init_encoder, entry.path().into()).unwrap();
                } else {
                    image =
                        Image::new_color(&device, &mut init_encoder, entry.path().into()).unwrap();
                }
                self.images.insert(image.name, image);
                info!("Loaded image: {}", file_name);
            } else if file_name.ends_with(".hdr") {
                let image =
                    Image::new_hdr(&device, &mut init_encoder, entry.path().into()).unwrap();
                self.images.insert(image.name, image);
                info!("Loaded skybox: {}", file_name);
            }
        }
        queue.submit(Some(init_encoder.finish()));
    }

    pub fn get_shader<'a, T>(&'a self, key: T) -> &'a Shader
    where
        T: Into<String>,
    {
        let key = key.into();
        self.shaders.get(&key).expect(&format!(
            "Asset Error: Could not find {} shader asset!",
            &key
        ))
    }

    pub fn get_mesh<T>(&self, key: T) -> &SubMesh
    where
        T: Into<String>,
    {
        let key = key.into();
        self.meshes
            .get(&key)
            .expect(&format!("Asset Error: Could not find {} mesh asset!", &key))
    }

    pub fn get_meshes(&self) -> Vec<&SubMesh> {
        self.meshes.values().collect()
    }

    pub fn get_meshes_mut(&mut self) -> Vec<&mut SubMesh> {
        self.meshes.values_mut().collect()
    }

    fn get_material_or_load(
        &mut self,
        handle: NewMaterialHandle,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Arc<NewMaterialData> {
        let t = self
            .materials
            .entry(handle)
            .and_modify(|e| {
                if e.is_none() {
                    *e = Some(Arc::new(handle.load_data(device, encoder)))
                }
            })
            .or_insert(Some(Arc::new(handle.load_data(device, encoder))));
            t.unwrap()
    }
    //
    //pub fn get_materials_mut(&mut self) -> Vec<&mut Material> {
    //    self.materials.values_mut().collect()
    //}
    //
    pub fn get_loaded_material_data(&self) -> Vec<Arc<NewMaterialData>> {
        self.materials.values().filter(|opt| opt.is_some()).map(|opt| opt.unwrap().clone()).collect()
    }
    
    pub fn get_image_or_white(&self, key: &str) -> Arc<Image> {
        self.images.get(key).unwrap_or(self.images.get("white").unwrap()).clone()
    }

    pub fn get_images(&self) -> Vec<Arc<Image>> {
        self.images.values().map(|img| img.clone()).collect()
    }

    pub fn get_font<T>(&self, key: T) -> &Font
    where
        T: Into<String>,
    {
        let key = key.into();
        self.fonts
            .get(&key)
            .expect(&format!("Asset Error: Could not find {} font asset!", &key))
    }

    pub fn get_font_mut<T>(&mut self, key: T) -> &mut Font
    where
        T: Into<String>,
    {
        let key = key.into();
        self.fonts
            .get_mut(&key)
            .expect(&format!("Asset Error: Could not find {} font asset!", &key))
    }

    pub fn get_fonts(&self) -> Vec<&Font> {
        self.fonts.values().collect()
    }

    pub(crate) fn load_materials(
        &mut self,
        device: &wgpu::Device,
        resource_manager: &mut GPUResourceManager,
    ) {
        let mut current_bind_group = None;
        let mut current_index = 0;
        for material in self.materials.values_mut() {
            match material {
                crate::graphics::material::Material::Unlit(unlit_material) => {
                    let unlit_bind_group_layout = resource_manager
                        .get_bind_group_layout("unlit_material")
                        .unwrap();
                    unlit_material.create_bind_group(&self, &device, unlit_bind_group_layout);
                }
                crate::graphics::material::Material::PBR(pbr_material) => {
                    let pbr_bind_group_layout = resource_manager
                        .get_bind_group_layout("pbr_material")
                        .unwrap();
                    current_bind_group =
                        Some(pbr_material.create_bind_group(&self, device, pbr_bind_group_layout));
                    current_index = pbr_material.index;
                }
            }
            if current_bind_group.is_some() {
                resource_manager.add_multi_bind_group(
                    "pbr",
                    current_bind_group.take().unwrap(),
                    current_index,
                );
            }

            current_bind_group = None;
        }
    }
}
