use specs::RunNow;
use std::mem;

use crate::{
    graphics::{
        mesh::MeshVertexData,
        pipeline::{VertexStateBuilder},
        // renderer::DEPTH_FORMAT,
        Pipeline,
        SimplePipeline,
        SimplePipelineDesc, RenderTarget,
    },
    scene::systems::RenderPBR,
    AssetManager,
};
use super::{LightingUniform, GlobalUniforms};

#[derive(Debug)]
pub struct PBRPipeline {
    constants_buffer: wgpu::Buffer,
    lighting_buffer: wgpu::Buffer,
    global_bind_group: wgpu::BindGroup,
}

impl SimplePipeline for PBRPipeline 
{
    fn prepare(&mut self, _device: &mut wgpu::Device, _pipeline: &Pipeline, _encoder: &mut wgpu::CommandEncoder) {
        
    }

    fn render(
        &mut self,
        frame: Option<&wgpu::SwapChainOutput>,
        depth: Option<&wgpu::TextureView>,
        device: &wgpu::Device,
        pipeline: &Pipeline,
        asset_manager: Option<&mut AssetManager>,
        world: &mut Option<&mut specs::World>,
        _input: Option<&RenderTarget>,
        _output: Option<&RenderTarget>,
    ) -> (wgpu::CommandBuffer, Option<RenderTarget>) {
        // Buffers can/are stored per mesh.
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut render_pbr = RenderPBR {
                device,
                asset_manager: asset_manager.as_ref().unwrap(),
                encoder: &mut encoder,
                frame_view: &frame.as_ref().unwrap().view,
                pipeline,
                constants_buffer: &self.constants_buffer,
                lighting_buffer: &self.lighting_buffer,
                global_bind_group: &self.global_bind_group,
                depth: depth.as_ref().unwrap(),
            };
            RunNow::setup(&mut render_pbr, world.as_mut().unwrap());
            render_pbr.run_now(world.as_mut().unwrap());
        }

        (encoder.finish(), None)
    }
}

#[derive(Debug, Default)]
pub struct PBRPipelineDesc;

impl SimplePipelineDesc for PBRPipelineDesc {
    type Pipeline = PBRPipeline;

    fn load_shader<'a>(
        &self,
        asset_manager: &'a crate::AssetManager,
    ) -> &'a crate::graphics::material::Shader {
        asset_manager.get_shader("pbr.shader")
    }

    fn create_layout(
        &self,
        device: &mut wgpu::Device,
    ) -> Vec<wgpu::BindGroupLayout> {
        // We can create whatever layout we want here.
        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry { // CAMERA TRANSFORM
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    },
                    wgpu::BindGroupLayoutEntry { // LIGHTING DATA
                        binding: 1,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    }
                ],
                label: None,
            });

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

        let pbr_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::Cube,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::Cube,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                    },
                ],
                label: None,
            });

        vec![
            global_bind_group_layout,
            material_bind_group_layout,
            pbr_bind_group_layout,
        ]
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
        device: &wgpu::Device,
        bind_group_layouts: &Vec<wgpu::BindGroupLayout>,
    ) -> PBRPipeline {
        // This data needs to be saved and passed onto the pipeline.
        let constants_buffer = device.create_buffer_with_data(
            bytemuck::bytes_of(&GlobalUniforms::default()),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let lighting_buffer = device.create_buffer_with_data(
            bytemuck::bytes_of(&LightingUniform::default()),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layouts[0],
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &constants_buffer,
                        range: 0..std::mem::size_of::<GlobalUniforms>() as u64,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &lighting_buffer,
                        range: 0..std::mem::size_of::<LightingUniform>() as u64,
                    },
                }
            ],
            label: None,
        });

        PBRPipeline {
            constants_buffer,
            lighting_buffer,
            global_bind_group,
        }
    }
}
