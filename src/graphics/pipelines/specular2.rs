use legion::prelude::Resources;

use crate::{
    graphics::{
        pipeline_manager::{PipelineDesc, PipelineManager},
        resources::{GPUResourceManager, ProbeUniform},
    },
    AssetManager,
};
use std::sync::Arc;

pub fn create(resources: &Resources, format: wgpu::TextureFormat) {
    let asset_manager = resources.get_mut::<AssetManager>().unwrap();
    let mut pipeline_manager = resources.get_mut::<PipelineManager>().unwrap();
    let resource_manager = resources.get::<Arc<GPUResourceManager>>().unwrap();
    let device = resources.get::<Arc<wgpu::Device>>().unwrap();
    let specular_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutEntry::new(
                    0,
                    wgpu::ShaderStage::FRAGMENT,
                    wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<ProbeUniform>() as _
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
            ],
            label: None,
        });
    resource_manager.add_bind_group_layout("specular_globals", specular_bind_group_layout);

    let mut skybox_desc = PipelineDesc::default();
    skybox_desc.shader = "core/shaders/calculations/specular2.shader".to_string();
    skybox_desc.color_state.format = format;

    skybox_desc.layouts = vec!["specular_globals".to_string()];
    skybox_desc.cull_mode = wgpu::CullMode::None;
    skybox_desc
        .vertex_state
        .set_index_format(wgpu::IndexFormat::Uint16);

    let specular_globals_buffer = device.create_buffer_with_data(
        bytemuck::bytes_of(&ProbeUniform::default()),
        wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
    );
    resource_manager.add_buffer("specular", specular_globals_buffer);

    pipeline_manager.add_pipeline(
        "specular",
        &skybox_desc,
        vec![],
        &device,
        &asset_manager,
        resource_manager.clone(),
    );
}
