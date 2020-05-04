use crate::{
    graphics::{
        material::skybox::SPEC_CUBEMAP_MIP_LEVELS,
        pipeline::VertexStateBuilder,
        resources::{GPUResourceManager, RenderTarget},
        SimplePipeline, SimplePipelineDesc,
    },
    AssetManager,
};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Uniforms {
    pub roughness: f32,
    pub resoultion: f32,
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            roughness: 1.0,
            resoultion: 1024.0,
        }
    }
}

unsafe impl Zeroable for Uniforms {}
unsafe impl Pod for Uniforms {}

pub struct SpecularPipeline {
    constants_buffer: wgpu::Buffer,
    resoultion: f32,
    mip_level: u32,
}

impl SimplePipeline for SpecularPipeline {
    fn prepare(
        &mut self,
        _asset_manager: &AssetManager,
        _device: &wgpu::Device,
        _encoder: &mut wgpu::CommandEncoder,
        _pipeline: &wgpu::RenderPipeline,
        _world: &mut legion::world::World,
    ) {
    }

    fn render(
        &mut self,
        _asset_manager: &AssetManager,
        _depth: Option<&wgpu::TextureView>,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        _frame: Option<&wgpu::SwapChainOutput>,
        input: Option<&RenderTarget>,
        output: Option<&RenderTarget>,
        pipeline: &wgpu::RenderPipeline,
        _world: &mut legion::world::World,
        resource_manager: &mut GPUResourceManager,
    ) -> Option<RenderTarget> {
        let global_bind_group_layout =
            resource_manager.get_bind_group_layout(format!("specular_globals_{}", self.mip_level)).unwrap();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: global_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &self.constants_buffer,
                        range: 0..std::mem::size_of::<Uniforms>() as u64,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &input.as_ref().unwrap().texture_view,
                    ),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&input.as_ref().unwrap().sampler),
                },
            ],
            label: None,
        });

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
        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..6, 0..6);

        None
    }
}

#[derive(Debug, Default)]
pub struct SpecularPipelineDesc {
    mip_level: u32,
    resoultion: f32,
}

impl SpecularPipelineDesc {
    pub fn new(mip_level: u32, resoultion: f32) -> Self {
        Self {
            mip_level,
            resoultion,
        }
    }
}

impl SimplePipelineDesc for SpecularPipelineDesc {
    type Pipeline = SpecularPipeline;

    fn load_shader<'a>(
        &self,
        asset_manager: &'a crate::AssetManager,
    ) -> &'a crate::graphics::material::Shader {
        asset_manager.get_shader("specular.shader")
    }

    fn create_layout<'a>(
        &self,
        device: &wgpu::Device,
        resource_manager: &'a mut GPUResourceManager,
    ) -> Vec<&'a wgpu::BindGroupLayout> {
        // We can create whatever layout we want here.
        let global_bind_group_layout =
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
                        ty: wgpu::BindingType::Sampler { comparison: false },
                    },
                ],
                label: None,
            });
        let binding_layout_name = format!("specular_globals_{}", self.mip_level);
        resource_manager
            .add_bind_group_layout(binding_layout_name.clone(), global_bind_group_layout);
        let global_bind_group_layout =
            resource_manager.get_bind_group_layout(binding_layout_name.clone()).unwrap();

        vec![global_bind_group_layout]
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
            format: wgpu::TextureFormat::Rgba32Float,
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
        device: &wgpu::Device,
        _resource_manager: &mut GPUResourceManager,
    ) -> SpecularPipeline {
        // This data needs to be saved and passed onto the pipeline.
        let constants_buffer = device.create_buffer_with_data(
            bytemuck::bytes_of(&Uniforms {
                roughness: self.mip_level as f32 / (SPEC_CUBEMAP_MIP_LEVELS - 1) as f32,
                resoultion: self.resoultion,
            }),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        SpecularPipeline {
            constants_buffer,
            resoultion: self.resoultion,
            mip_level: self.mip_level,
        }
    }
}
