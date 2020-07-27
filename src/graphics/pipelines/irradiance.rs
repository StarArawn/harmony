use legion::prelude::Resources;

use crate::{
    graphics::{
        pipeline_manager::{PipelineDesc, PipelineManager},
        resources::{GPUResourceManager, ProbeUniform},
    },
    AssetManager,
};
use std::{borrow::Cow, sync::Arc};

pub fn create(resources: &Resources, format: wgpu::TextureFormat) {
    let asset_manager = resources.get_mut::<AssetManager>().unwrap();
    let mut pipeline_manager = resources.get_mut::<PipelineManager>().unwrap();
    let resource_manager = resources.get::<Arc<GPUResourceManager>>().unwrap();
    let device = resources.get::<Arc<wgpu::Device>>().unwrap();
    let irradiance_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: Cow::Borrowed(&[
                wgpu::BindGroupLayoutEntry::new(
                    0,
                    wgpu::ShaderStage::FRAGMENT,
                    wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<ProbeUniform>() as _,
                        ),
                    },
                ),
                wgpu::BindGroupLayoutEntry::new(
                    1,
                    wgpu::ShaderStage::FRAGMENT,
                    wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        component_type: wgpu::TextureComponentType::Float,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                ),
                wgpu::BindGroupLayoutEntry::new(
                    2,
                    wgpu::ShaderStage::FRAGMENT,
                    wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        component_type: wgpu::TextureComponentType::Float,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                ),
                wgpu::BindGroupLayoutEntry::new(
                    3,
                    wgpu::ShaderStage::FRAGMENT,
                    wgpu::BindingType::Sampler { comparison: false },
                ),
            ]),
            label: None,
        });
    resource_manager.add_bind_group_layout("irradiance", irradiance_bind_group_layout);

    let mut irradiance_desc = PipelineDesc::default();
    irradiance_desc.shader = "core/shaders/calculations/irradiance2.shader".to_string();
    irradiance_desc.color_states[0].format = format;

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
        resource_manager.clone(),
    );
}
