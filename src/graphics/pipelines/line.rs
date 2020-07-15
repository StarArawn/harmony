use crate::graphics::{
    mesh::MeshTangentLine, pipeline::VertexStateBuilder, renderer::DEPTH_FORMAT,
    resources::GPUResourceManager, SimplePipeline, SimplePipelineDesc,
};

#[derive(Debug)]
pub struct LinePipeline {}

impl SimplePipeline for LinePipeline {}

#[derive(Debug, Default)]
pub struct LinePipelineDesc;

impl SimplePipelineDesc for LinePipelineDesc {
    type Pipeline = LinePipeline;

    fn load_shader<'a>(
        &self,
        asset_manager: &'a crate::AssetManager,
    ) -> &'a crate::graphics::material::Shader {
        asset_manager.get_shader("core/shaders/line.shader")
    }

    fn create_layout<'a>(
        &self,
        _device: &wgpu::Device,
        resource_manager: &'a mut GPUResourceManager,
    ) -> Vec<&'a wgpu::BindGroupLayout> {
        let global_bind_group_layout = resource_manager.get_bind_group_layout("globals").unwrap();
        vec![global_bind_group_layout]
    }
    fn rasterization_state_desc(&self) -> wgpu::RasterizationStateDescriptor {
        wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }
    }
    fn primitive_topology(&self) -> wgpu::PrimitiveTopology {
        wgpu::PrimitiveTopology::LineList
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
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
            stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
            stencil_read_mask: 0,
            stencil_write_mask: 0,
        })
    }

    fn vertex_state_desc(&self) -> VertexStateBuilder {
        let vertex_size = std::mem::size_of::<MeshTangentLine>();
        let mut vertex_state_builder = VertexStateBuilder::new();
        vertex_state_builder.new_buffer_descriptor(
            vertex_size as wgpu::BufferAddress,
            wgpu::InputStepMode::Vertex,
            wgpu::vertex_attr_array![0 => Float3, 1 => Float3].to_vec(),
        );

        vertex_state_builder
    }

    fn build(
        self,
        _device: &wgpu::Device,
        _resource_manager: &mut GPUResourceManager,
    ) -> LinePipeline {
        LinePipeline {}
    }
}
