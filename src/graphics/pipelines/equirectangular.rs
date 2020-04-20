use crate::{
    graphics::{
        pipeline::{VertexStateBuilder},
        Pipeline, SimplePipeline, SimplePipelineDesc,
    },
    AssetManager,
};

#[derive(Debug)]
pub struct CubeProjectionPipeline {
    texture: String,
    size: f32,
    pub(crate) cubemap_view: Option<wgpu::TextureView>,
    pub(crate) cubemap_texture: Option<wgpu::Texture>,
}

impl SimplePipeline for CubeProjectionPipeline {
    fn prepare(&mut self, _device: &mut wgpu::Device, _pipeline: &Pipeline, _encoder: &mut wgpu::CommandEncoder) {
        
    }

    fn render(
        &mut self,
        _frame: Option<&wgpu::TextureView>,
        _depth: Option<&wgpu::TextureView>,
        device: &wgpu::Device,
        pipeline: &Pipeline,
        asset_manager: Option<&mut AssetManager>,
        _world: Option<&mut specs::World>,
    ) -> wgpu::CommandBuffer {
        // Buffers can/are stored per mesh.
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let image = asset_manager.as_ref().unwrap().get_image(self.texture.clone());

        let render_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.size as u32,
                height: self.size as u32 * 6,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: image.format,
            usage: wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            label: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline.bind_group_layouts[0],
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&image.view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&image.sampler),
                },
            ],
            label: None,
        });

        let render_texture_view = render_texture.create_default_view();

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &render_texture_view,
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
            render_pass.set_pipeline(&pipeline.pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..6, 0..6);
        }

        let cubemap_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.size as u32,
                height: self.size as u32,
                depth: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: image.format,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: None,
        });

        for i in 0..6 {
            encoder.copy_texture_to_texture(
                wgpu::TextureCopyView {
                    texture: &render_texture,
                    mip_level: 0,
                    array_layer: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: self.size as u32 * i,
                        z: 0,
                    },
                },
                wgpu::TextureCopyView {
                    texture: &cubemap_texture,
                    mip_level: 0,
                    array_layer: i,
                    origin: wgpu::Origin3d::ZERO,
                },
                wgpu::Extent3d {
                    width: self.size as u32,
                    height: self.size as u32,
                    depth: 1,
                },
            );
        }

        self.cubemap_view =
            Some(cubemap_texture.create_view(&wgpu::TextureViewDescriptor {
                format: wgpu::TextureFormat::Rgba32Float,
                dimension: wgpu::TextureViewDimension::Cube,
                aspect: wgpu::TextureAspect::default(),
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                array_layer_count: 6,
            }));
        self.cubemap_texture = Some(cubemap_texture);

        encoder.finish()
    }
}

#[derive(Debug, Default)]
pub struct CubeProjectionPipelineDesc {
    texture: String,
    size: f32,
}

impl CubeProjectionPipelineDesc {
    pub fn new(texture: String, size: f32) -> Self {
        Self {
            texture,
            size,
        }
    }
}

impl SimplePipelineDesc for CubeProjectionPipelineDesc {
    type Pipeline = CubeProjectionPipeline;

    fn load_shader<'a>(
        &self,
        asset_manager: &'a crate::AssetManager,
    ) -> &'a crate::graphics::material::Shader {
        asset_manager.get_shader("hdr_to_cubemap.shader")
    }

    fn create_layout(
        &self,
        device: &mut wgpu::Device,
    ) -> Vec<wgpu::BindGroupLayout> {
        // We can create whatever layout we want here.
        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2,
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
        _device: &wgpu::Device,
        _bind_group_layouts: &Vec<wgpu::BindGroupLayout>,
    ) -> CubeProjectionPipeline {
        CubeProjectionPipeline {
            texture: self.texture,
            size: self.size,
            cubemap_view: None,
            cubemap_texture: None,
        }
    }
}
