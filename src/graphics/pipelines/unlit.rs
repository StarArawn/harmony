use specs::RunNow;
use bytemuck::{ Pod, Zeroable };
use std::mem;

use crate::{
    AssetManager,
    graphics::{
        mesh::MeshVertexData,
        Pipeline,
        pipeline::{ VertexStateBuilder, PrepareResult },
        SimplePipeline,
        SimplePipelineDesc,
    }, scene::systems::RenderMesh
};

#[derive(Debug)]
pub struct UnlitPipeline {
    constants_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
struct Uniforms {
    transform: [f32; 16],
}

unsafe impl Zeroable for Uniforms { }
unsafe impl Pod for Uniforms { }

impl SimplePipeline for UnlitPipeline {
    fn prepare(&mut self) -> PrepareResult { 
        PrepareResult::Reuse
    }

    fn render(&mut self, frame: &wgpu::SwapChainOutput, device: &wgpu::Device, asset_manager: &AssetManager, world: &mut specs::World, pipeline: &Pipeline) -> wgpu::CommandBuffer {
        // Buffers can/are stored per mesh.
        let mut encoder = device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None },
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        },
                    },
                ],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&pipeline.pipeline);
            render_pass.set_bind_group(0, &pipeline.bind_group, &[]);
            
            let mut render_mesh = RenderMesh {
                asset_manager,
                render_pass,    
            };
            RunNow::setup(&mut render_mesh, world);
            render_mesh.run_now(world);
        }
        

        encoder.finish()
    }
}

#[derive(Debug, Default)]
pub struct UnlitPipelineDesc;

impl SimplePipelineDesc for UnlitPipelineDesc {
    type Pipeline = UnlitPipeline;
    fn load_shader<'a>(&self, asset_manager: &'a crate::AssetManager) -> &'a crate::graphics::material::Shader {
        asset_manager.get_shader("unlit.shader")
    }
    fn create_layout(&self, device: &mut wgpu::Device) -> (wgpu::BindGroup, wgpu::PipelineLayout) {
        // We can create whatever layout we want here.
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[],
            label: None,
        });

        // Create bind group.
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[],
            label: None,
        });

        // Once we create the layout we don't need the bind group layout.
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        (bind_group, layout)
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
    fn color_states_desc(&self, sc_desc: &wgpu::SwapChainDescriptor) -> Vec<wgpu::ColorStateDescriptor> {
        vec![wgpu::ColorStateDescriptor {
            format: sc_desc.format,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }]
    }

    fn depth_stencil_state_desc(&self) -> Option<wgpu::DepthStencilStateDescriptor> {
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

    fn build(self, device: &wgpu::Device) -> UnlitPipeline {
        // This data needs to be saved and passed onto the pipeline.
        let constants_buffer = device
            .create_buffer_with_data(bytemuck::bytes_of(&Uniforms::default()), wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST);
        

        UnlitPipeline {
            constants_buffer,
        }
    }
}