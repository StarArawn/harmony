use harmony::{
    graphics::{
        VertexStateBuilder,
        Pipeline,
        SimplePipeline,
        SimplePipelineDesc,
        RenderTarget,
    },
    AssetManager,
};

#[derive(Debug)]
pub struct TrianglePipeline {
    bind_group: wgpu::BindGroup,
}

impl SimplePipeline for TrianglePipeline {
    fn prepare(&mut self, _device: &mut wgpu::Device, _pipeline: &Pipeline, _encoder: &mut wgpu::CommandEncoder) {
        
    }

    fn render(
        &mut self,
        frame: Option<&wgpu::TextureView>,
        _depth: Option<&wgpu::TextureView>,
        device: &wgpu::Device,
        pipeline: &Pipeline,
        mut _asset_manager: Option<&mut AssetManager>,
        _world: &mut Option<&mut specs::World>,
        _input: Option<&RenderTarget>,
        _output: Option<&RenderTarget>,
    ) -> wgpu::CommandBuffer {
        // Buffers can/are stored per mesh.
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: frame.as_ref().unwrap(),
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::GREEN,
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&pipeline.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }

        encoder.finish()
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

    fn create_layout(
        &self,
        device: &mut wgpu::Device,
    ) -> Vec<wgpu::BindGroupLayout> {
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
    ) -> TrianglePipeline {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layouts[0],
            bindings: &[],
            label: None,
        });
        TrianglePipeline {
            bind_group,
        }
    }
}
