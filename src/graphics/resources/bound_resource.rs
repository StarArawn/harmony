use crate::AssetManager;
use super::BindGroup;

pub trait BoundResource {
    fn create_bind_group(&self, asset_manager: &AssetManager, device: &wgpu::Device, pipeline_layouts: &Vec<wgpu::BindGroupLayout>) -> BindGroup;
}