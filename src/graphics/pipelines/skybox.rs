use legion::prelude::Resources;
use nalgebra_glm::Mat4;
use bytemuck::{Pod, Zeroable};

use crate::{
    AssetManager, 
    graphics::{
        renderer::DEPTH_FORMAT,
        pipeline_manager::{PipelineDesc, PipelineManager}, 
        resources::{BindGroup, GPUResourceManager}
    }
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SkyboxUniforms {
    pub proj: Mat4,
    pub view: Mat4,
}

impl Default for SkyboxUniforms {
    fn default() -> Self {
        Self {
            proj: Mat4::identity(),
            view: Mat4::identity(),
        }
    }
}

unsafe impl Zeroable for SkyboxUniforms {}
unsafe impl Pod for SkyboxUniforms {}

pub fn create(resources: &Resources) {
    let asset_manager = resources.get_mut::<AssetManager>().unwrap();
    let mut pipeline_manager = resources.get_mut::<PipelineManager>().unwrap();
    let mut resource_manager = resources.get_mut::<GPUResourceManager>().unwrap();
    let device = resources.get::<wgpu::Device>().unwrap();
    let sc_desc = resources.get::<wgpu::SwapChainDescriptor>().unwrap();

    let mut skybox_desc = PipelineDesc::default();
    skybox_desc.shader = "skybox.shader".to_string();
    skybox_desc.color_state.format = sc_desc.format;
    skybox_desc.depth_state = Some(wgpu::DepthStencilStateDescriptor {
        format: DEPTH_FORMAT,
        depth_write_enabled: false,
        depth_compare: wgpu::CompareFunction::LessEqual,
        stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
        stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
        stencil_read_mask: 0,
        stencil_write_mask: 0,
    });

    // Create skybox bind group layouts.
    let skybox_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            }],
            label: None,
        });
    resource_manager.add_bind_group_layout("skybox_globals", skybox_layout);

    let skybox_material_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        component_type: wgpu::TextureComponentType::Float,
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::Cube,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                },
            ],
            label: None,
        });
    resource_manager.add_bind_group_layout("skybox_material", skybox_material_layout);
    skybox_desc.layouts = vec!["skybox_globals".to_string(), "skybox_material".to_string()];
    skybox_desc.cull_mode = wgpu::CullMode::None;
    skybox_desc.vertex_state.set_index_format(wgpu::IndexFormat::Uint16);
    
    let skybox_globals_buffer = device.create_buffer_with_data(
        bytemuck::bytes_of(&SkyboxUniforms::default()),
        wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
    );
    let skybox_layout = resource_manager.get_bind_group_layout("skybox_globals").unwrap();
    let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: skybox_layout,
        bindings: &[wgpu::Binding {
            binding: 0,
            resource: wgpu::BindingResource::Buffer {
                buffer: &skybox_globals_buffer,
                range: 0..std::mem::size_of::<SkyboxUniforms>() as u64,
            },
        }],
        label: None,
    });
    resource_manager.add_single_bind_group("skybox_globals", BindGroup::new(0, global_bind_group));
    resource_manager.add_buffer("skybox_buffer", skybox_globals_buffer);

    pipeline_manager.add("skybox", &skybox_desc, vec![], &device, &asset_manager, &resource_manager);
}