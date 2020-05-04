use legion::prelude::Resources;

use crate::{
    AssetManager, 
    graphics::{
        pipeline_manager::{PipelineDesc, PipelineManager}, 
        resources::{GPUResourceManager, ProbeUniform}
    }
};

pub fn create(resources: &Resources, format: wgpu::TextureFormat) {
    let asset_manager = resources.get_mut::<AssetManager>().unwrap();
    let mut pipeline_manager = resources.get_mut::<PipelineManager>().unwrap();
    let mut resource_manager = resources.get_mut::<GPUResourceManager>().unwrap();
    let device = resources.get::<wgpu::Device>().unwrap();
    let specular_bind_group_layout =
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
    resource_manager.add_bind_group_layout("specular_globals", specular_bind_group_layout);

    let mut skybox_desc = PipelineDesc::default();
    skybox_desc.shader = "specular2.shader".to_string();
    skybox_desc.color_state.format = format;
    
    skybox_desc.layouts = vec!["specular_globals".to_string()];
    skybox_desc.cull_mode = wgpu::CullMode::None;
    skybox_desc.vertex_state.set_index_format(wgpu::IndexFormat::Uint16);
    
    let specular_globals_buffer = device.create_buffer_with_data(
        bytemuck::bytes_of(&ProbeUniform::default()),
        wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
    );
    resource_manager.add_buffer("specular", specular_globals_buffer);

    pipeline_manager.add("specular", &skybox_desc, vec![], &device, &asset_manager, &resource_manager);
}