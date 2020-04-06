// This file uses wgpu to render our different renderable objects. Note they should be collected first.
use ultraviolet::mat::Mat4;
use std::mem;
use std::convert::TryInto;
use zerocopy::{AsBytes, FromBytes};

use crate::AssetManager;
use crate::gui::core::Rectangle;
use crate::gui::renderables::Quad;

pub struct QuadRenderer {
    pipeline: wgpu::RenderPipeline,
    constants: wgpu::BindGroup,
    constants_buffer: wgpu::Buffer,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    instances: wgpu::Buffer,
}

#[repr(C)]
#[derive(AsBytes, Debug, Clone, Copy)]
struct Uniforms {
    transform: [f32; 16],
    scale: f32,
}

impl Uniforms {
    fn new(transformation: Mat4, scale: f32) -> Uniforms {
        Self {
            transform: transformation.as_slice().try_into().unwrap(),
            scale,
        }
    }
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            transform: Mat4::identity().as_slice().try_into().unwrap(),
            scale: 1.0,
        }
    }
}


#[repr(C)]
#[derive(AsBytes, Clone, Copy)]
pub struct Vertex {
    _position: [f32; 2],
}

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

const QUAD_VERTS: [Vertex; 4] = [
    Vertex {
        _position: [0.0, 0.0],
    },
    Vertex {
        _position: [1.0, 0.0],
    },
    Vertex {
        _position: [1.0, 1.0],
    },
    Vertex {
        _position: [0.0, 1.0],
    },
];


impl QuadRenderer {
    pub fn new(
        asset_mananger: &AssetManager,
        device: &mut wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Self {
        let constant_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                }],
                label: None,
            });

        let constants_buffer = device
            .create_buffer_with_data([Uniforms::default()].to_vec().as_bytes(), wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST);
        
        let constants = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &constant_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &constants_buffer,
                    range: 0..std::mem::size_of::<Uniforms>() as u64,
                },
            }],
            label: None,
        });

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&constant_layout],
            });

        let shader = asset_mananger.get_shader(String::from("gui_quad.shader"));

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                layout: &layout,
                vertex_stage: wgpu::ProgrammableStageDescriptor {
                    module: &shader.vertex,
                    entry_point: "main",
                },
                fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                    module: &shader.fragment,
                    entry_point: "main",
                }),
                rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: wgpu::CullMode::None,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                }),
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                color_states: &[wgpu::ColorStateDescriptor {
                    format,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask: wgpu::ColorWrite::ALL,
                }],
                depth_stencil_state: None,
                vertex_state: wgpu::VertexStateDescriptor {
                    index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[
                    wgpu::VertexBufferDescriptor {
                        stride: mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::InputStepMode::Vertex,
                        attributes: &[wgpu::VertexAttributeDescriptor {
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float2,
                            offset: 0,
                        }],
                    },
                    wgpu::VertexBufferDescriptor {
                        stride: mem::size_of::<Quad>() as u64,
                        step_mode: wgpu::InputStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttributeDescriptor {
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float2,
                                offset: 0,
                            },
                            wgpu::VertexAttributeDescriptor {
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float2,
                                offset: 4 * 2,
                            },
                            wgpu::VertexAttributeDescriptor {
                                shader_location: 3,
                                format: wgpu::VertexFormat::Float4,
                                offset: 4 * (2 + 2),
                            },
                            wgpu::VertexAttributeDescriptor {
                                shader_location: 4,
                                format: wgpu::VertexFormat::Float4,
                                offset: 4 * (2 + 2 + 4),
                            },
                            wgpu::VertexAttributeDescriptor {
                                shader_location: 5,
                                format: wgpu::VertexFormat::Float,
                                offset: 4 * (2 + 2 + 4 + 4),
                            },
                            wgpu::VertexAttributeDescriptor {
                                shader_location: 6,
                                format: wgpu::VertexFormat::Float,
                                offset: 4 * (2 + 2 + 4 + 4 + 1),
                            },
                        ],
                    },
                ],
                },
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            });

        let vertices = device.create_buffer_with_data(&QUAD_VERTS.as_bytes(), wgpu::BufferUsage::VERTEX);

        let indices = device.create_buffer_with_data(&QUAD_INDICES.as_bytes(), wgpu::BufferUsage::INDEX);

        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            size: mem::size_of::<Quad>() as u64 * Quad::MAX as u64,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            label: None,
        });

        Self {
            pipeline,
            constants,
            constants_buffer,
            vertices,
            indices,
            instances,
        }
    }

    pub fn draw(
        &self,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        instances: &[Quad],
        transformation: Mat4,
        scale: f32,
        bounds: Rectangle<u32>,
        target: &wgpu::TextureView,
    ) {
        let uniforms = Uniforms::new(transformation, scale);

        let constants_buffer = device.create_buffer_with_data(&[uniforms].as_bytes(), wgpu::BufferUsage::COPY_SRC);

        encoder.copy_buffer_to_buffer(
            &constants_buffer,
            0,
            &self.constants_buffer,
            0,
            std::mem::size_of::<Uniforms>() as u64,
        );

        let mut i = 0;
        let total = instances.len();

        while i < total {
            let end = (i + Quad::MAX).min(total);
            let amount = end - i;

            let instance_buffer = device.create_buffer_with_data(&instances[i..end].as_bytes(), wgpu::BufferUsage::COPY_SRC);

            encoder.copy_buffer_to_buffer(
                &instance_buffer,
                0,
                &self.instances,
                0,
                (mem::size_of::<Quad>() * amount) as u64,
            );

            {
                let mut render_pass =
                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[
                            wgpu::RenderPassColorAttachmentDescriptor {
                                attachment: target,
                                resolve_target: None,
                                load_op: wgpu::LoadOp::Load,
                                store_op: wgpu::StoreOp::Store,
                                clear_color: wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 0.0,
                                },
                            },
                        ],
                        depth_stencil_attachment: None,
                    });
                    
                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_bind_group(0, &self.constants, &[]);
                render_pass.set_index_buffer(&self.indices, 0, 0);
                render_pass.set_vertex_buffer(0, &self.vertices, 0, 0);
                render_pass.set_vertex_buffer(1, &self.instances, 0, 0);
                
                // render_pass.set_scissor_rect(
                //     bounds.x,
                //     bounds.y,
                //     bounds.width,
                //     // TODO: Address anti-aliasing adjustments properly
                //     bounds.height + 1,
                // );

                render_pass.draw_indexed(
                    0..QUAD_INDICES.len() as u32,
                    0,
                    0..amount as u32,
                );
            }

            i += Quad::MAX;
        }
    }
}