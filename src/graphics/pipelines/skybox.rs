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
    },
    scene::systems::RenderSkybox
};

#[derive(Debug)]
pub struct SkyboxPipeline {
    constants_buffer: wgpu::Buffer,
    global_bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct SkyboxUniforms {
    pub view_projection: Mat4,
}

unsafe impl Zeroable for SkyboxUniforms { }
unsafe impl Pod for SkyboxUniforms { }

impl SimplePipeline for SkyboxPipeline {
    fn prepare(&mut self) -> PrepareResult { 
        PrepareResult::Reuse
    }

    fn render(&mut self, frame_view: Option<&wgpu::TextureView>, device: &wgpu::Device, pipeline: &Pipeline, asset_manager: Option<&mut AssetManager>, mut world: Option<&mut specs::World>) -> wgpu::CommandBuffer {
        // Buffers can/are stored per mesh.
        let mut encoder = device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None },
        );
        {
            let mut render_skybox = RenderSkybox {
                device,
                asset_manager: asset_manager.as_ref().unwrap(),
                encoder: &mut encoder,
                frame_view: frame_view.as_ref().unwrap(),
                pipeline,
                constants_buffer: &self.constants_buffer,
                global_bind_group: &self.global_bind_group,
            };
            RunNow::setup(&mut render_skybox, world.as_mut().unwrap());
            render_skybox.run_now(world.as_mut().unwrap());
        }
    

        encoder.finish()
    }
}

#[derive(Debug, Default)]
pub struct SkyboxPipelineDesc;

impl SimplePipelineDesc for SkyboxPipelineDesc {
    type Pipeline = SkyboxPipeline;
    
    fn load_shader<'a>(&self, asset_manager: &'a crate::AssetManager) -> &'a crate::graphics::material::Shader {
        asset_manager.get_shader("skybox.shader")
    }

    fn create_layout(&self, device: &mut wgpu::Device) -> (Vec<wgpu::BindGroupLayout>, wgpu::PipelineLayout) {
        // We can create whatever layout we want here.
        let global_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings:  &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
            ],
            label: None,
        });

        let material_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings:  &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        component_type: wgpu::TextureComponentType::Float,
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::Cube,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                },
            ],
            label: None,
        });

        // Once we create the layout we don't need the bind group layout.
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&global_bind_group_layout, &material_bind_group_layout],
        });

        (vec![global_bind_group_layout, material_bind_group_layout], layout)
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
            .set_index_format(wgpu::IndexFormat::Uint16);

        vertex_state_builder
    }

    fn build(self, device: &wgpu::Device, bind_group_layouts: &Vec<wgpu::BindGroupLayout>) -> SkyboxPipeline {
        // This data needs to be saved and passed onto the pipeline.
        let constants_buffer = device
            .create_buffer_with_data(bytemuck::bytes_of(&SkyboxUniforms::default()), wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST);

        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layouts[0],
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &constants_buffer,
                        range: 0..std::mem::size_of::<SkyboxUniforms>() as u64,
                    },
                },
            ],
            label: None,
        });

        SkyboxPipeline {
            constants_buffer,
            global_bind_group,
        }
    }
}