use crate::{
    graphics::{
        pipeline::VertexStateBuilder, resources::{BindingManager, RenderTarget}, Pipeline, SimplePipeline,
        SimplePipelineDesc,
    },
    AssetManager,
};

#[derive(Debug)]
pub struct SpecularBRDFPipeline {
    size: f32,
}

impl SimplePipeline for SpecularBRDFPipeline {
    fn prepare(
        &mut self,
        _asset_manager: &mut AssetManager,
        _device: &mut wgpu::Device,
        _encoder: &mut wgpu::CommandEncoder,
        _pipeline: &Pipeline,
        _world: &mut specs::World,
    ) {
    }

    fn render(
        &mut self,
        _asset_manager: &mut AssetManager,
        _depth: Option<&wgpu::TextureView>,
        _device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        _frame: Option<&wgpu::SwapChainOutput>,
        _input: Option<&RenderTarget>,
        output: Option<&RenderTarget>,
        pipeline: &Pipeline,
        _world: &mut specs::World,
        _binding_manager: &mut BindingManager,
    ) -> Option<RenderTarget> {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &output.as_ref().unwrap().texture_view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
            }],
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&pipeline.pipeline);
        render_pass.draw(0..3, 0..1);

        None
    }
}

#[derive(Debug, Default)]
pub struct SpecularBRDFPipelineDesc {
    size: f32,
}

impl SpecularBRDFPipelineDesc {
    pub fn new(size: f32) -> Self {
        Self { size }
    }
}

impl SimplePipelineDesc for SpecularBRDFPipelineDesc {
    type Pipeline = SpecularBRDFPipeline;

    fn load_shader<'a>(
        &self,
        asset_manager: &'a crate::AssetManager,
    ) -> &'a crate::graphics::material::Shader {
        asset_manager.get_shader("specular_brdf.shader")
    }

    fn create_layout(&self, _device: &mut wgpu::Device) -> Vec<wgpu::BindGroupLayout> {
        // No bindings? No problem! Just remember that later on!
        vec![]
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
        _sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Vec<wgpu::ColorStateDescriptor> {
        vec![wgpu::ColorStateDescriptor {
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }]
    }

    fn depth_stencil_state_desc(&self) -> Option<wgpu::DepthStencilStateDescriptor> {
        None
    }

    fn vertex_state_desc(&self) -> VertexStateBuilder {
        let vertex_state_builder = VertexStateBuilder::new();
        vertex_state_builder
    }

    fn build(
        self,
        _device: &wgpu::Device,
        _bind_group_layouts: &Vec<wgpu::BindGroupLayout>,
        _binding_manager: &mut BindingManager,
    ) -> SpecularBRDFPipeline {
        SpecularBRDFPipeline { size: self.size }
    }
}
