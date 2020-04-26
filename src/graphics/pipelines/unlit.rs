use std::mem;

use crate::{
    graphics::{
        mesh::MeshVertexData,
        pipeline::VertexStateBuilder,
        renderer::DEPTH_FORMAT,
        resources::{GPUResourceManager, RenderTarget},
        // renderer::DEPTH_FORMAT,
        SimplePipeline,
        SimplePipelineDesc,
    },
    AssetManager,
};

#[derive(Debug)]
pub struct UnlitPipeline {}

impl SimplePipeline for UnlitPipeline {
    fn prepare(
        &mut self,
        _asset_manager: &mut AssetManager,
        _device: &wgpu::Device,
        _encoder: &mut wgpu::CommandEncoder,
        _pipeline: &wgpu::RenderPipeline,
        _world: &mut legion::world::World,
    ) {
        // let mut prepare_unlit = PrepareUnlit {
        //     device,
        //     encoder,
        //     constants_buffer: &self.constants_buffer,
        // };
        // RunNow::setup(&mut prepare_unlit, world);
        // prepare_unlit.run_now(world);
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
        _pipeline: &wgpu::RenderPipeline,
        _world: &mut legion::world::World,
        _binding_manager: &mut GPUResourceManager,
    ) -> Option<RenderTarget> {
        // let mut render_unlit = RenderUnlit {
        //     device,
        //     asset_manager: asset_manager,
        //     encoder,
        //     frame_view: &frame.as_ref().unwrap().view,
        //     pipeline,
        //     constants_buffer: &self.constants_buffer,
        //     global_bind_group: &self.global_bind_group,
        //     depth: depth.as_ref().unwrap(),
        // };
        // RunNow::setup(&mut render_unlit, world);
        // render_unlit.run_now(world);

        None
    }
}

#[derive(Debug, Default)]
pub struct UnlitPipelineDesc;

impl SimplePipelineDesc for UnlitPipelineDesc {
    type Pipeline = UnlitPipeline;

    fn load_shader<'a>(
        &self,
        asset_manager: &'a crate::AssetManager,
    ) -> &'a crate::graphics::material::Shader {
        asset_manager.get_shader("unlit.shader")
    }

    fn create_layout<'a>(
        &self,
        device: &wgpu::Device,
        resource_manager: &'a mut GPUResourceManager,
    ) -> Vec<&'a wgpu::BindGroupLayout> {
        let material_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
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
                        ty: wgpu::BindingType::Sampler { comparison: false },
                    },
                ],
                label: None,
            });

        resource_manager.add_bind_group_layout("unlit_material", material_bind_group_layout);
        let material_bind_group_layout = resource_manager.get_bind_group_layout("unlit_material");

        let global_bind_group_layout = resource_manager.get_bind_group_layout("globals");

        vec![global_bind_group_layout, material_bind_group_layout]
    }
    fn rasterization_state_desc(&self) -> wgpu::RasterizationStateDescriptor {
        wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Cw,
            cull_mode: wgpu::CullMode::Back,
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
        Some(wgpu::DepthStencilStateDescriptor {
            format: DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Greater,
            stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
            stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
            stencil_read_mask: 0,
            stencil_write_mask: 0,
        })
    }

    fn vertex_state_desc(&self) -> VertexStateBuilder {
        let vertex_size = mem::size_of::<MeshVertexData>();

        let mut vertex_state_builder = VertexStateBuilder::new();

        vertex_state_builder
            .set_index_format(wgpu::IndexFormat::Uint32)
            .new_buffer_descriptor(
                vertex_size as wgpu::BufferAddress,
                wgpu::InputStepMode::Vertex,
                vec![
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float3,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float3,
                        offset: 4 * 3,
                        shader_location: 1,
                    },
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float2,
                        offset: 4 * (3 + 3),
                        shader_location: 2,
                    },
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float4,
                        offset: 4 * (3 + 3 + 2),
                        shader_location: 3,
                    },
                ],
            );

        vertex_state_builder
    }

    fn build(
        self,
        _device: &wgpu::Device,
        _binding_manager: &mut GPUResourceManager,
    ) -> UnlitPipeline {
        UnlitPipeline {}
    }
}
