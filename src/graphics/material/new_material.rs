//Materials are being Stored in a HashSet
use super::Image;
use nalgebra_glm::{vec4, Vec4};
use std::{sync::Arc, mem};
use walkdir::WalkDir;
use crate::graphics::resources::BindGroup;
use bytemuck::{Pod, Zeroable};

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

impl NewMaterialData{
    pub(crate) fn create_bind_group<'a>(
        &mut self,
        device: &wgpu::Device,
        pipeline_layout: &'a wgpu::BindGroupLayout,
    ) -> BindGroup {

        let metallic = self.metallic.map_or(0.0, |v|v);
        let roughness = self.roughness.map_or(0.0, |v|v);
        let color = self.color.map_or(vec4(0f32,0f32,0f32,0f32), |v| vec4(v[0],v[1],v[2],v[3]));
        
        let uniform = PBRMaterialUniform {
            color,
            info: Vec4::new(metallic, roughness, 0.0, 0.0),
        };
    
        let material_uniform_size = mem::size_of::<PBRMaterialUniform>() as wgpu::BufferAddress;
        let uniform_buf = device.create_buffer_with_data(bytemuck::bytes_of(&uniform), wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST);
        self.uniform_buf = Some(uniform_buf);
    
        // Asset manager will panic if image doesn't exist, but we don't want that.
        // So use get_image_option instead.
        let main_image = match &self.main_texture{
            Some(img) => img,
            None => unimplemented!()//return white
        };
        
        let normal_image = match &self.normal_texture{
            Some(img) => img,
            None => unimplemented!()//return white
        };
        
        let roughness_image = match &self.roughness_texture{
            Some(img) => img,
            None => unimplemented!()//return white
        };
    
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: self.uniform_buf.as_ref().unwrap(),
                        range: 0..material_uniform_size,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&main_image.sampler),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&main_image.view),
                },
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&normal_image.view),
                },
                wgpu::Binding {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&roughness_image.view),
                },
            ],
            label: None,
        });
    
        BindGroup::new(2, bind_group)
    }
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
            main_texture: self.main_texture.map_or(None, |path| Image::new_color(device, encoder, path.into()).ok()),
            roughness_texture: self.roughness_texture.map_or(None, |path| Image::new_normal(device, encoder, path.into()).ok()),
            normal_texture: self.normal_texture.map_or(None, |path| Image::new_normal(device, encoder, path.into()).ok()),
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PBRMaterialUniform {
    pub color: Vec4,
    pub info: Vec4,
}

unsafe impl Zeroable for PBRMaterialUniform {}
unsafe impl Pod for PBRMaterialUniform {}

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
