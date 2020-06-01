use legion::prelude::Resources;

use crate::{
    graphics::{
        pipeline_manager::{PipelineDesc, PipelineManager},
        resources::GPUResourceManager,
    },
    AssetManager,
};
use std::sync::Arc;

pub fn create(resources: &Resources, format: wgpu::TextureFormat) {
    let asset_manager = resources.get_mut::<AssetManager>().unwrap();
    let mut pipeline_manager = resources.get_mut::<PipelineManager>().unwrap();
    let mut resource_manager = resources.get_mut::<GPUResourceManager>().unwrap();
    let device = resources.get::<Arc<wgpu::Device>>().unwrap();
    let irradiance_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        component_type: wgpu::TextureComponentType::Float,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        component_type: wgpu::TextureComponentType::Float,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                },
            ],
            label: None,
        });
    resource_manager.add_bind_group_layout("irradiance", irradiance_bind_group_layout);

    let mut irradiance_desc = PipelineDesc::default();
    irradiance_desc.shader = "irradiance2.shader".to_string();
    irradiance_desc.color_state.format = format;

    irradiance_desc.layouts = vec!["irradiance".to_string()];
    irradiance_desc.cull_mode = wgpu::CullMode::None;
    irradiance_desc
        .vertex_state
        .set_index_format(wgpu::IndexFormat::Uint16);

    pipeline_manager.add_pipeline(
        "irradiance",
        &irradiance_desc,
        vec![],
        &device,
        &asset_manager,
        &resource_manager,
    );
}
