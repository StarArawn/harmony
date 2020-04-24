use harmony::{
    graphics::{
        resources::{BindingManager, RenderTarget, BindGroup}, Pipeline, SimplePipeline, SimplePipelineDesc, VertexStateBuilder,
    },
    AssetManager,
};

#[derive(Debug)]
pub struct TrianglePipeline;

impl SimplePipeline for TrianglePipeline {
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
        frame: Option<&wgpu::SwapChainOutput>,
        _input: Option<&RenderTarget>,
        _output: Option<&RenderTarget>,
        pipeline: &Pipeline,
        _world: &mut specs::World,
        binding_manager: &mut BindingManager,
    ) -> Option<RenderTarget> {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.as_ref().unwrap().view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color::GREEN,
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&pipeline.pipeline);
        binding_manager.set_bind_group(&mut rpass, "triangle", 0);
        rpass.draw(0..3, 0..1);

        None
    }
}

#[derive(Debug, Default)]
pub struct TrianglePipelineDesc {}

impl SimplePipelineDesc for TrianglePipelineDesc {
    type Pipeline = TrianglePipeline;

    fn load_shader<'a>(
        &self,
        asset_manager: &'a harmony::AssetManager,
    ) -> &'a harmony::graphics::material::Shader {
        asset_manager.get_shader("triangle.shader")
    }

    fn create_layout(&self, device: &mut wgpu::Device) -> Vec<wgpu::BindGroupLayout> {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[],
            label: None,
        });

        vec![bind_group_layout]
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
        wgpu::PrimitiveTopology::TriangleList
    }
    fn color_states_desc(
        &self,
        _sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Vec<wgpu::ColorStateDescriptor> {
        vec![wgpu::ColorStateDescriptor {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }]
    }

    fn depth_stencil_state_desc(&self) -> Option<wgpu::DepthStencilStateDescriptor> {
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
        binding_manager: &mut BindingManager,
    ) -> TrianglePipeline {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layouts[0],
            bindings: &[],
            label: None,
        });
        binding_manager.add_single_resource("triangle", BindGroup::new(0, bind_group));
        TrianglePipeline { }
    }
}
