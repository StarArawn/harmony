use legion::prelude::Resources;

use crate::assets::{material::PBRMaterialUniform, mesh::MeshVertexData};

use crate::{
    graphics::{
        pipeline_manager::{PipelineDesc, PipelineManager},
        renderer::DEPTH_FORMAT,
        resources::GPUResourceManager,
    },
    AssetManager,
};
use std::sync::Arc;

pub fn create_pbr_bindgroup_layout(device: Arc<wgpu::Device>) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            wgpu::BindGroupLayoutEntry::new(
                0,
                wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<PBRMaterialUniform>() as _,
                    ),
                },
            ),
            wgpu::BindGroupLayoutEntry::new(
                1,
                wgpu::ShaderStage::FRAGMENT,
                wgpu::BindingType::Sampler { comparison: false },
            ),
            wgpu::BindGroupLayoutEntry::new(
                2,
                wgpu::ShaderStage::FRAGMENT,
                wgpu::BindingType::Sampler { comparison: false },
            ),
            wgpu::BindGroupLayoutEntry::new(
                3,
                wgpu::ShaderStage::FRAGMENT,
                wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    component_type: wgpu::TextureComponentType::Float,
                    dimension: wgpu::TextureViewDimension::D2,
                },
            ),
            wgpu::BindGroupLayoutEntry::new(
                4,
                wgpu::ShaderStage::FRAGMENT,
                wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    component_type: wgpu::TextureComponentType::Float,
                    dimension: wgpu::TextureViewDimension::D2,
                },
            ),
            wgpu::BindGroupLayoutEntry::new(
                5,
                wgpu::ShaderStage::FRAGMENT,
                wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    component_type: wgpu::TextureComponentType::Float,
                    dimension: wgpu::TextureViewDimension::D2,
                },
            ),
        ],
        label: Some("pbr_material_layout"),
    })
}

pub fn create(resources: &Resources) {
    let asset_manager = resources.get_mut::<AssetManager>().unwrap();
    let mut pipeline_manager = resources.get_mut::<PipelineManager>().unwrap();
    let resource_manager = resources.get::<Arc<GPUResourceManager>>().unwrap();
    let device = resources.get::<Arc<wgpu::Device>>().unwrap();
    let sc_desc = resources.get::<wgpu::SwapChainDescriptor>().unwrap();

    let mut pbr_desc = PipelineDesc::default();
    pbr_desc.shader = "core/shaders/pbr.shader".to_string();
    pbr_desc.color_state.format = sc_desc.format;
    pbr_desc.depth_state = Some(wgpu::DepthStencilStateDescriptor {
        format: DEPTH_FORMAT,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
        stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
        stencil_read_mask: 0,
        stencil_write_mask: 0,
    });

    // Create skybox bind group layouts.
    let pbr_material_layout = create_pbr_bindgroup_layout(device.clone());
    resource_manager.add_bind_group_layout("pbr_material_layout", pbr_material_layout);

    let probe_material_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            wgpu::BindGroupLayoutEntry::new(
                0,
                wgpu::ShaderStage::FRAGMENT,
                wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    component_type: wgpu::TextureComponentType::Float,
                    dimension: wgpu::TextureViewDimension::Cube,
                },
            ),
            wgpu::BindGroupLayoutEntry::new(
                1,
                wgpu::ShaderStage::FRAGMENT,
                wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    component_type: wgpu::TextureComponentType::Float,
                    dimension: wgpu::TextureViewDimension::Cube,
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
        ],
        label: Some("probe_material_layout"),
    });

    resource_manager.add_bind_group_layout("probe_material_layout", probe_material_layout);

    pbr_desc.layouts = vec![
        "locals".to_string(),
        "globals".to_string(),
        "pbr_material_layout".to_string(),
        "probe_material_layout".to_string(),
    ];
    pbr_desc.cull_mode = wgpu::CullMode::Back;
    let vertex_size = std::mem::size_of::<MeshVertexData>();
    pbr_desc
        .vertex_state
        .set_index_format(wgpu::IndexFormat::Uint32)
        .new_buffer_descriptor(
            vertex_size as wgpu::BufferAddress,
            wgpu::InputStepMode::Vertex,
            wgpu::vertex_attr_array![0 => Float3, 1 => Float3, 2 => Float2, 3 => Float4].to_vec(),
        );

    pipeline_manager.add_pipeline(
        "pbr",
        &pbr_desc,
        vec!["globals", "skybox", "froxel_cull"],
        &device,
        &asset_manager,
        resource_manager.clone(),
    );
}
