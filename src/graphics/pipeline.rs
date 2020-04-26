use super::{
    material::Shader,
    resources::{GPUResourceManager, RenderTarget},
};
use crate::AssetManager;

#[derive(Debug)]
pub struct BindGroupWithData {
    pub(crate) uniform_buf: wgpu::Buffer,
    pub(crate) bind_group: wgpu::BindGroup,
}

pub trait SimplePipeline: std::fmt::Debug + Send + Sync + 'static {
    fn prepare(
        &mut self,
        asset_manager: &mut AssetManager,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &wgpu::RenderPipeline,
        world: &mut legion::world::World,
    ) {
    }

    fn get_uniforms(&self) -> Option<(Vec<&wgpu::BindGroup>, Vec<&wgpu::Buffer>)> {
        None
    }

    fn render(
        &mut self,
        asset_manager: &mut AssetManager,
        depth: Option<&wgpu::TextureView>,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        frame: Option<&wgpu::SwapChainOutput>,
        input: Option<&RenderTarget>,
        output: Option<&RenderTarget>,
        pipeline: &wgpu::RenderPipeline,
        world: &mut legion::world::World,
        resource_manager: &mut GPUResourceManager,
    ) -> Option<RenderTarget> {
        None
    }
}

pub trait SimplePipelineDesc: std::fmt::Debug {
    type Pipeline: SimplePipeline;

    fn pipeline<'a>(
        &mut self,
        asset_manager: &'a AssetManager,
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        resource_manager: &mut GPUResourceManager,
        local_bind_group_layout: Option<&wgpu::BindGroupLayout>,
    ) -> wgpu::RenderPipeline {
        let shader = self.load_shader(asset_manager);
        let vertex_stage = wgpu::ProgrammableStageDescriptor {
            module: &shader.vertex,
            entry_point: "main",
        };
        let fragment_stage = Some(wgpu::ProgrammableStageDescriptor {
            module: &shader.fragment,
            entry_point: "main",
        });

        let mut bind_group_layouts = self.create_layout(&device, resource_manager);
        let rasterization_state = self.rasterization_state_desc();
        let primitive_topology = self.primitive_topology();
        let color_states = self.color_states_desc(&sc_desc);
        let depth_stencil_state = self.depth_stencil_state_desc();
        let vertex_state_builder = self.vertex_state_desc();
        let sample_count = self.create_samplers(&device);
        let sample_mask = self.sampler_mask();
        let alpha_to_coverage_enabled = self.alpha_to_coverage_enabled();

        if local_bind_group_layout.is_some() {
            bind_group_layouts.insert(0, local_bind_group_layout.as_ref().unwrap());
        }

        // Once we create the layout we don't need the bind group layout.
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &bind_group_layouts,
        });

        let vertex_buffers: Vec<wgpu::VertexBufferDescriptor<'_>> = vertex_state_builder
            .buffer_desc
            .iter()
            .map(|desc| wgpu::VertexBufferDescriptor {
                stride: desc.stride,
                step_mode: desc.step_mode,
                attributes: &desc.attributes,
            })
            .collect();

        let vertex_state = wgpu::VertexStateDescriptor {
            index_format: vertex_state_builder.index_format,
            vertex_buffers: &vertex_buffers,
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &layout,
            vertex_stage,
            fragment_stage,
            primitive_topology,
            color_states: &color_states,
            rasterization_state: Some(rasterization_state),
            depth_stencil_state,
            vertex_state,
            sample_count,
            sample_mask,
            alpha_to_coverage_enabled,
        });

        pipeline
    }

    // TODO: Support other types of shaders like compute.
    // Also support having only a vertex shader.
    fn load_shader<'a>(&self, asset_manager: &'a AssetManager) -> &'a Shader;
    fn create_layout<'a>(
        &self,
        _device: &wgpu::Device,
        _resource_manager: &'a mut GPUResourceManager,
    ) -> Vec<&'a wgpu::BindGroupLayout>;
    fn rasterization_state_desc(&self) -> wgpu::RasterizationStateDescriptor;
    fn primitive_topology(&self) -> wgpu::PrimitiveTopology;
    fn color_states_desc(
        &self,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Vec<wgpu::ColorStateDescriptor>;
    fn depth_stencil_state_desc(&self) -> Option<wgpu::DepthStencilStateDescriptor>;
    fn vertex_state_desc(&self) -> VertexStateBuilder;
    fn create_samplers(&self, _device: &wgpu::Device) -> u32 {
        1
    }
    fn sampler_mask(&self) -> u32 {
        !0
    }
    fn alpha_to_coverage_enabled(&self) -> bool {
        false
    }

    fn build<'a>(
        self,
        device: &wgpu::Device,
        resource_manager: &mut GPUResourceManager,
    ) -> Self::Pipeline;
}

pub struct VertexStateBuilder {
    pub(crate) index_format: wgpu::IndexFormat,
    pub(crate) buffer_desc: Vec<VertexBufferDescriptor>,
}

impl VertexStateBuilder {
    pub fn new() -> Self {
        Self {
            index_format: wgpu::IndexFormat::Uint32,
            buffer_desc: Vec::new(),
        }
    }

    pub fn set_index_format<'a>(&'a mut self, format: wgpu::IndexFormat) -> &'a mut Self {
        self.index_format = format;
        self
    }

    pub fn new_buffer_descriptor<'a>(
        &'a mut self,
        stride: wgpu::BufferAddress,
        step_mode: wgpu::InputStepMode,
        attributes: Vec<wgpu::VertexAttributeDescriptor>,
    ) -> &'a mut Self {
        self.buffer_desc.push(VertexBufferDescriptor {
            stride,
            step_mode,
            attributes,
        });
        self
    }
}

pub struct VertexBufferDescriptor {
    pub(crate) stride: wgpu::BufferAddress,
    pub(crate) step_mode: wgpu::InputStepMode,
    pub(crate) attributes: Vec<wgpu::VertexAttributeDescriptor>,
}
