use bytemuck::{Pod, Zeroable};
use nalgebra_glm::Mat4;

use crate::{
    graphics::{
        pipeline::VertexStateBuilder, resources::{BindingManager, RenderTarget}, Pipeline,
        SimplePipeline, SimplePipelineDesc,
    },
    AssetManager,
};

#[derive(Debug)]
pub struct SkyboxPipeline {
    constants_buffer: wgpu::Buffer,
    global_bind_group: wgpu::BindGroup,
}

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

impl SimplePipeline for SkyboxPipeline {

    fn get_uniforms(&self) -> Option<(Vec<&wgpu::BindGroup>, Vec<&wgpu::Buffer>)> {
        Some((vec![&self.global_bind_group], vec![&self.constants_buffer]))
    }

    fn prepare(
        &mut self,
        _asset_manager: &mut AssetManager,
        _device: &wgpu::Device,
        _encoder: &mut wgpu::CommandEncoder,
        _pipeline: &Pipeline,
        _world: &mut legion::world::World,
    ) {
    }

    fn render(
        &mut self,
        _asset_manager: &mut AssetManager,
        _depth: Option<&wgpu::TextureView>,
        _device: &wgpu::Device,
        _encoder: &mut wgpu::CommandEncoder,
        _frame: Option<&wgpu::SwapChainOutput>,
        _input: Option<&RenderTarget>,
        _output: Option<&RenderTarget>,
        _pipeline: &Pipeline,
        _world: &mut legion::world::World,
        _binding_manager: &mut BindingManager,
    ) -> Option<RenderTarget> {
        None
    }
}

#[derive(Debug, Default)]
pub struct SkyboxPipelineDesc;

impl SimplePipelineDesc for SkyboxPipelineDesc {
    type Pipeline = SkyboxPipeline;

    fn load_shader<'a>(
        &self,
        asset_manager: &'a crate::AssetManager,
    ) -> &'a crate::graphics::material::Shader {
        asset_manager.get_shader("skybox.shader")
    }

    fn create_layout(&self, device: &wgpu::Device) -> Vec<wgpu::BindGroupLayout> {
        // We can create whatever layout we want here.
        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                }],
                label: None,
            });

        let material_bind_group_layout =
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

        vec![global_bind_group_layout, material_bind_group_layout]
    }
    fn rasterization_state_desc(&self) -> wgpu::RasterizationStateDescriptor {
        wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Cw,
            cull_mode: wgpu::CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }
    }
    fn primitive_topology(&self) -> wgpu::PrimitiveTopology {
        wgpu::PrimitiveTopology::TriangleList
    }
    fn color_states_desc(
        &self,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Vec<wgpu::ColorStateDescriptor> {
        vec![wgpu::ColorStateDescriptor {
            format: sc_desc.format,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }]
    }

    fn depth_stencil_state_desc(&self) -> Option<wgpu::DepthStencilStateDescriptor> {
        // Some(wgpu::DepthStencilStateDescriptor {
        //     format: DEPTH_FORMAT,
        //     depth_write_enabled: true,
        //     depth_compare: wgpu::CompareFunction::Less,
        //     stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
        //     stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
        //     stencil_read_mask: 0,
        //     stencil_write_mask: 0,
        // })
        None
    }

    fn vertex_state_desc(&self) -> VertexStateBuilder {
        let mut vertex_state_builder = VertexStateBuilder::new();
        vertex_state_builder.set_index_format(wgpu::IndexFormat::Uint16);

        vertex_state_builder
    }

    fn build(
        self,
        device: &wgpu::Device,
        bind_group_layouts: &Vec<wgpu::BindGroupLayout>,
        _binding_manager: &mut BindingManager,
    ) -> SkyboxPipeline {
        // This data needs to be saved and passed onto the pipeline.
        let constants_buffer = device.create_buffer_with_data(
            bytemuck::bytes_of(&SkyboxUniforms::default()),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layouts[0],
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &constants_buffer,
                    range: 0..std::mem::size_of::<SkyboxUniforms>() as u64,
                },
            }],
            label: None,
        });

        SkyboxPipeline {
            constants_buffer,
            global_bind_group,
        }
    }
}
