use crate::{
    graphics::{
        pipeline::VertexStateBuilder,
        resources::{GPUResourceManager, RenderTarget},
        SimplePipeline, SimplePipelineDesc,
    },
    AssetManager,
};

pub struct CubeProjectionPipeline {
    texture: String,
    size: f32,
    bind_group: Option<wgpu::BindGroup>,
}

impl SimplePipeline for CubeProjectionPipeline {
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
        asset_manager: &AssetManager,
        _depth: Option<&wgpu::TextureView>,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        _frame: Option<&wgpu::SwapChainOutput>,
        _input: Option<&RenderTarget>,
        output: Option<&RenderTarget>,
        pipeline: &wgpu::RenderPipeline,
        _world: &mut legion::world::World,
        resource_manager: &mut GPUResourceManager,
    ) -> Option<RenderTarget> {
        {
            let image = asset_manager.get_image(self.texture.clone());

            let global_bind_group = resource_manager
                .get_bind_group_layout("equirectangular_globals")
                .unwrap();

            self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: global_bind_group,
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
            }));

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
            render_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
            render_pass.draw(0..6, 0..6);
        }

        let cube_map = RenderTarget::new(
            device,
            self.size,
            self.size,
            6,
            1,
            wgpu::TextureFormat::Rgba32Float,
            wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        );

        for i in 0..6 {
            encoder.copy_texture_to_texture(
                wgpu::TextureCopyView {
                    texture: &output.as_ref().unwrap().texture,
                    mip_level: 0,
                    array_layer: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: self.size as u32 * i,
                        z: 0,
                    },
                },
                wgpu::TextureCopyView {
                    texture: &cube_map.texture,
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

        Some(cube_map)
    }
}

#[derive(Debug, Default)]
pub struct CubeProjectionPipelineDesc {
    texture: String,
    size: f32,
}

impl CubeProjectionPipelineDesc {
    pub fn new(texture: String, size: f32) -> Self {
        Self { texture, size }
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
        resource_manager.add_bind_group_layout("equirectangular_globals", global_bind_group_layout);
        let global_bind_group_layout = resource_manager
            .get_bind_group_layout("equirectangular_globals")
            .unwrap();

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
        _resource_manager: &mut GPUResourceManager,
    ) -> CubeProjectionPipeline {
        CubeProjectionPipeline {
            texture: self.texture,
            size: self.size,
            bind_group: None,
        }
    }
}
