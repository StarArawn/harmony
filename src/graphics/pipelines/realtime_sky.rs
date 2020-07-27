use legion::prelude::Resources;

use crate::{
    graphics::{
        pipeline_manager::{PipelineDesc, PipelineManager},
        renderer::DEPTH_FORMAT,
        resources::GPUResourceManager,
    },
    AssetManager,
};
use std::{borrow::Cow, sync::Arc};

pub fn create(resources: &Resources) {
    let asset_manager = resources.get::<AssetManager>().unwrap();
    let mut pipeline_manager = resources.get_mut::<PipelineManager>().unwrap();
    let resource_manager = resources.get::<Arc<GPUResourceManager>>().unwrap();
    let device = resources.get::<Arc<wgpu::Device>>().unwrap();
    let sc_desc = resources.get::<wgpu::SwapChainDescriptor>().unwrap();

    let mut skybox_desc = PipelineDesc::default();
    skybox_desc.shader = "core/shaders/sky/sky.shader".to_string();
    skybox_desc.color_states[0].format = sc_desc.format;
    skybox_desc.depth_state = Some(wgpu::DepthStencilStateDescriptor {
        format: DEPTH_FORMAT,
        depth_write_enabled: false,
        depth_compare: wgpu::CompareFunction::LessEqual,
        stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
        stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
        stencil_read_mask: 0,
        stencil_write_mask: 0,
    });

    let skybox_material_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: Cow::Borrowed(&[
                wgpu::BindGroupLayoutEntry::new(
                    0,
                    wgpu::ShaderStage::FRAGMENT,
                    wgpu::BindingType::Sampler { comparison: false },
                ),
                wgpu::BindGroupLayoutEntry::new(
                    1,
                    wgpu::ShaderStage::FRAGMENT,
                    wgpu::BindingType::SampledTexture {
                        component_type: wgpu::TextureComponentType::Float,
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                ),
                wgpu::BindGroupLayoutEntry::new(
                    2,
                    wgpu::ShaderStage::FRAGMENT,
                    wgpu::BindingType::SampledTexture {
                        component_type: wgpu::TextureComponentType::Float,
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                ),
            ]),
            label: None,
        });
    resource_manager.add_bind_group_layout("realtime_skybox_material", skybox_material_layout);
    skybox_desc.layouts = vec![
        "globals".to_string(),
        "realtime_skybox_material".to_string(),
    ];
    skybox_desc.cull_mode = wgpu::CullMode::None;
    skybox_desc
        .vertex_state
        .set_index_format(wgpu::IndexFormat::Uint16);

    pipeline_manager.add_pipeline(
        "realtime_skybox",
        &skybox_desc,
        vec!["globals"],
        &device,
        &asset_manager,
        resource_manager.clone(),
    );
}
