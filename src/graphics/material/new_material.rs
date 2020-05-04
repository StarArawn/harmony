//Materials are being Stored in a HashSet
use super::Image;
use nalgebra_glm::Vec4;
use std::{collections::HashSet, sync::Arc};
use walkdir::WalkDir;

/// Hash as identifier.
pub struct NewMaterialData {
    pub main_texture: Option<Arc<Image>>,
    pub roughness_texture: Option<Arc<Image>>,
    pub normal_texture: Option<Arc<Image>>,
    pub roughness: Option<f32>,
    pub metallic: Option<f32>,
    pub color: Option<[f32; 4]>,
    pub uniform_buf: Option<wgpu::Buffer>,
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug)]
pub struct NewMaterialHandle {
    main_texture: Option<String>,
    roughness_texture: Option<String>,
    normal_texture: Option<String>,
    roughness: Option<f32>,
    metallic: Option<f32>,
    color: Option<[f32; 4]>,
}

impl NewMaterialHandle {
    fn load_data(
        self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> NewMaterialData {
        NewMaterialData{
            main_texture: self.main_texture.map_or(None, |path| Some(Arc::new(Image::new_color(device, encoder, &path)))),
            roughness_texture: self.roughness_texture.map_or(None, |path| Some(Arc::new(Image::new_normal(device, encoder, &path)))),
            normal_texture: self.normal_texture.map_or(None, |path| Some(Arc::new(Image::new_normal(device, encoder, &path)))),
            roughness: self.roughness,
            metallic: self.metallic,
            color: self.color,
            uniform_buf: None,
        }
    }
}

/// load_material_handles reads all valid NewMaterialHandles from path
pub fn load_material_handles(path: &str) -> Vec<NewMaterialHandle> {
    let mut material_handles = Vec::new();
    for entry in WalkDir::new(path) {
        if let Some(entry) = entry.ok() {
            if let Some(bytes) = std::fs::read(entry.path()).ok() {
                // TODO: read could be smarter
                if let Some(handle) = ron::de::from_bytes::<NewMaterialHandle>(&bytes).ok() {
                    material_handles.push(handle);
                }
            }
        }
    }
    material_handles
}

#[test]
fn test_load_mat_nones() {
    let dummydata = "NewMaterialHandle(
            main_texture:None,
            roughness_texture:None,
            normal_texture:None,
            roughness:None,
            metallic:None,
            color:None,
        )";
    let dummystruct = NewMaterialHandle {
        main_texture: None,
        roughness_texture: None,
        normal_texture: None,
        roughness: None,
        metallic: None,
        color: None,
    };
    let buf = ron::de::from_str::<NewMaterialHandle>(dummydata).unwrap();
    let out = ron::ser::to_string(&buf).unwrap();
}
