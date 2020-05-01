use log::*;
use std::collections::HashMap;
use walkdir::WalkDir;

use crate::core::Font;
use crate::graphics::{
    material::{Image, Material, Shader},
    mesh::Mesh, resources::GPUResourceManager,
};

pub struct AssetManager {
    path: String,
    shaders: HashMap<String, Shader>,
    fonts: HashMap<String, Font>,
    meshes: HashMap<String, Mesh>,
    pub(crate) images: HashMap<String, Image>,
    pub(crate) materials: HashMap<u32, Material>,
}

impl AssetManager {
    pub fn new(path: String) -> Self {
        AssetManager {
            path,
            shaders: HashMap::new(),
            fonts: HashMap::new(),
            meshes: HashMap::new(),
            images: HashMap::new(),
            materials: HashMap::new(),
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
                let (mesh, materials) = Mesh::new(
                    &device,
                    format!("{}{}", full_file_path, file_name),
                    current_index,
                );
                let mut index = current_index;
                for material in materials {
                    self.materials.insert(index, material);
                    index += 1;
                }
                self.meshes.insert(file_name.to_string(), mesh);
                info!("Loaded mesh: {}", file_name);
            }
            if file_name.ends_with(".png")
                || file_name.ends_with(".jpg")
                || file_name.ends_with(".hdr")
            {
                let image = Image::new(
                    &device,
                    &mut init_encoder,
                    format!("{}{}", full_file_path, file_name),
                    file_name.to_string(),
                );
                self.images.insert(file_name.to_string(), image);
                info!("Loaded image: {}", file_name);
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

    pub fn get_mesh<T>(&self, key: T) -> &Mesh
    where
        T: Into<String>,
    {
        let key = key.into();
        self.meshes
            .get(&key)
            .expect(&format!("Asset Error: Could not find {} mesh asset!", &key))
    }

    pub fn get_meshes(&self) -> Vec<&Mesh> {
        self.meshes.values().collect()
    }

    pub fn get_meshes_mut(&mut self) -> Vec<&mut Mesh> {
        self.meshes.values_mut().collect()
    }

    pub fn get_material(&self, index: u32) -> &Material {
        self.materials.get(&index).expect(&format!(
            "Asset Error: Could not find material @index {} asset!",
            index
        ))
    }

    pub fn get_materials_mut(&mut self) -> Vec<&mut Material> {
        self.materials.values_mut().collect()
    }

    pub fn get_materials(&self) -> Vec<&Material> {
        self.materials.values().collect()
    }

    pub fn get_image<T>(&self, key: T) -> &Image
    where
        T: Into<String>,
    {
        let key = key.into();
        self.images.get(&key).expect(&format!(
            "Asset Error: Could not find {} image asset!",
            &key
        ))
    }

    pub fn get_image_option<T>(&self, key: T) -> Option<&Image>
    where
        T: Into<String>,
    {
        let key = key.into();
        self.images.get(&key)
    }

    pub fn get_images(&self) -> Vec<&Image> {
        self.images.values().collect()
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

    pub(crate) fn load_materials(&mut self, device: &wgpu::Device, resource_manager: &mut GPUResourceManager) {
        
        let mut current_bind_group = None;
        let mut current_index = 0;
        for material in self.materials.values_mut() {
            match material {
                crate::graphics::material::Material::Unlit(unlit_material) => {
                    let unlit_bind_group_layout = resource_manager.get_bind_group_layout("unlit_material");
                    unlit_material.create_bind_group(
                        &self.images,
                        &device,
                        unlit_bind_group_layout,
                    );
                }
                crate::graphics::material::Material::PBR(pbr_material) => {
                    let pbr_bind_group_layout = resource_manager.get_bind_group_layout("pbr_material");
                    current_bind_group = Some(pbr_material.create_bind_group(
                            &self.images,
                            device,
                            pbr_bind_group_layout,
                        ));
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
