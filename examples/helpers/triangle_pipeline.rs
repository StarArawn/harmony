use harmony::{
    graphics::{
        resources::{BindGroup, GPUResourceManager, RenderTarget},
        SimplePipeline, SimplePipelineDesc, VertexStateBuilder,
    },
    AssetManager,
};

#[derive(Debug)]
pub struct TrianglePipeline;

impl SimplePipeline for TrianglePipeline {
    fn prepare(
        &mut self,
        _asset_manager: &mut AssetManager,
        _device: &wgpu::Device,
        _encoder: &mut wgpu::CommandEncoder,
        _pipeline: &wgpu::RenderPipeline,
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
        _pipeline: &wgpu::RenderPipeline,
        _world: &mut legion::world::World,
        _resource_manager: &mut GPUResourceManager,
    ) -> Option<RenderTarget> {
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

    fn create_layout<'a>(&self, device: &wgpu::Device, resource_manager: &'a mut GPUResourceManager) -> Vec<&'a wgpu::BindGroupLayout> {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[],
            label: None,
        });
        resource_manager.add_bind_group_layout("triangle_layout", bind_group_layout);
        let bind_group_layout = resource_manager.get_bind_group_layout("triangle_layout");

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
        resource_manager: &mut GPUResourceManager,
    ) -> TrianglePipeline {
        let layout = resource_manager.get_bind_group_layout("triangle_layout");
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            bindings: &[],
            label: None,
        });
        resource_manager.add_single_bind_group("triangle", BindGroup::new(0, bind_group));
        TrianglePipeline {}
    }
}
