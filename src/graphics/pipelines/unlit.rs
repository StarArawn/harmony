use ultraviolet::Mat4;
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
    bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct UnlitUniforms {
    pub view_projection: Mat4,
}

unsafe impl Zeroable for UnlitUniforms { }
unsafe impl Pod for UnlitUniforms { }

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
            let mut render_mesh = RenderMesh {
                device,
                asset_manager,
                encoder: &mut encoder,
                frame,
                pipeline,
                bind_group: &self.bind_group,
                constants_buffer: &self.constants_buffer,
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
    fn create_layout(&self, device: &mut wgpu::Device) -> (wgpu::BindGroupLayout, wgpu::PipelineLayout) {
        // We can create whatever layout we want here.
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings:  &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            }],
            label: None,
        });

        // Once we create the layout we don't need the bind group layout.
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        (bind_group_layout, layout)
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

    fn build(self, device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout ) -> UnlitPipeline {
        // This data needs to be saved and passed onto the pipeline.
        let constants_buffer = device
            .create_buffer_with_data(bytemuck::bytes_of(&UnlitUniforms::default()), wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST);
        
        // Create bind group.
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &constants_buffer,
                    range: 0..std::mem::size_of::<UnlitUniforms>() as u64,
                },
            }],
            label: None,
        });

        UnlitPipeline {
            constants_buffer,
            bind_group,
        }
    }
}