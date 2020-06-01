use nalgebra_glm::Vec4;
use std::{sync::Arc, path::PathBuf};

use super::{image::{ImageFormat, ImageInfo}, Image};
use assetmanage_rs::{Loader, MemoryLoader, Source};
use crate::graphics::resources::{GPUImageHandle, ImageManager};

pub(crate) enum NewMaterial {
    PBRMaterial {
        main_texture: Arc<GPUImageHandle>,
        roughness_texture: Arc<GPUImageHandle>,
        normal_texture: Arc<GPUImageHandle>,
        roughness: f32,
        metallic: f32,
        color: Vec4,
        uniform_buf: Option<wgpu::Buffer>,
    }, 
    Test{
        id: u32
    },
}

#[derive(serde::Deserialize,serde::Serialize)]
pub(crate) enum MaterialRon{
    PBRMaterial{
        main_texture: PathBuf,
        main_texture_info: ImageInfo,
        roughness_texture: PathBuf,
        roughness_texture_info: ImageInfo,
        normal_texture: PathBuf,
        normal_texture_info: ImageInfo,
        roughness: f32,
        metallic: f32,
        color: [f32;4],
    },
}
impl MaterialRon{
    pub(crate) fn try_construct(&self,base:PathBuf, iam: &ImageManager) -> Option<NewMaterial>{
        match self{
            MaterialRon::PBRMaterial {
                 main_texture, 
                 main_texture_info, 
                 roughness_texture, 
                 roughness_texture_info, 
                 normal_texture, 
                 normal_texture_info, 
                 roughness, 
                 metallic, 
                 color } => {
                    let main_texture = iam.get(base.join(main_texture))?;
                    let roughness_texture = iam.get(base.join(roughness_texture))?;
                    let normal_texture = iam.get(base.join(normal_texture))?;
                    Some(NewMaterial::PBRMaterial{
                        main_texture,
                        roughness_texture,
                        normal_texture,
                        roughness: *roughness,
                        metallic: *metallic,
                        color: Vec4::from_column_slice(color),
                        uniform_buf: None
                    })
                 }
            }
        
    }
}

impl assetmanage_rs::Asset<MemoryLoader> for MaterialRon{
    type ManagerSupplement = ();
    type AssetSupplement = ();
    type Structure = MaterialRon;
    fn construct(
        data_load: Vec<u8>,
        data_ass: &Self::AssetSupplement,
        data_mgr: &Self::ManagerSupplement,
    ) -> Result<Self::Structure, std::io::Error> {
        ron::de::from_bytes(&data_load).map_err(|e| 
            std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}


