use crate::{graphics::{RenderTarget, RenderGraph}, Application};

#[derive(Debug)]
pub struct Skybox {
    pub size: f32,
    pub(crate) cubemap_texture: wgpu::Texture,
    pub(crate) cubemap_view: wgpu::TextureView,
    pub(crate) cubemap_sampler: wgpu::Sampler,
    pub(crate) cubemap_bind_group: Option<wgpu::BindGroup>,
}

impl Skybox {
    pub fn new<T>(app: &mut Application, texture: T, size: f32) -> Self where T: Into<String> {
        // Create a new render graph for this process..
        let mut graph = RenderGraph::new(&app.renderer.device);
        
        let cube_map_target = RenderTarget::new(&app.renderer.device, size, size * 6.0, 1, wgpu::TextureFormat::Rgba32Float);

        let cube_projection_pipeline_desc =
            crate::graphics::pipelines::equirectangular::CubeProjectionPipelineDesc::new(texture.into(), size);
        graph.add(&app.asset_manager, &mut app.renderer, "cube_projection", cube_projection_pipeline_desc, vec![], false, Some(cube_map_target), false);

        let irradiance_target = RenderTarget::new(&app.renderer.device, size, size * 6.0, 1, wgpu::TextureFormat::Rgba32Float);
        let irradiance_pipeline_desc = crate::graphics::pipelines::irradiance::IrradiancePipelineDesc::new(size);
        graph.add(&app.asset_manager, &mut app.renderer, "irradiance", irradiance_pipeline_desc, vec!["cube_projection"], false, Some(irradiance_target), true);

        // We need to convert our regular texture map to a cube texture map with 6 faces.
        // Should be straight forward enough if we use equirectangular projection.
        // First we need a custom pipeline that will run in here to do the conversion.
        let output = app.renderer.swap_chain.get_next_texture().unwrap();
        let command_buffers = graph.render(&mut app.renderer, &mut app.asset_manager, None, &output);

        let output = graph.pull_render_target("irradiance");

        // Add copy texture copy command buffer and push to all command buffers to the queue
        app.renderer.queue.submit(&command_buffers);

        let cubemap_view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            format: wgpu::TextureFormat::Rgba32Float,
            dimension: wgpu::TextureViewDimension::Cube,
            aspect: wgpu::TextureAspect::default(),
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            array_layer_count: 6,
        });
        let cubemap_texture = output.texture;

        let cubemap_sampler = app.renderer.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Undefined,
        });

        Self {
            size,
            cubemap_texture,
            cubemap_view,
            cubemap_sampler,
            cubemap_bind_group: None,
        }
    }

    pub(crate) fn create_bind_group(&mut self, device: &wgpu::Device, material_layout: &wgpu::BindGroupLayout) {
        let bind_group = device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &material_layout,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &self.cubemap_view,
                        ),
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            &self.cubemap_sampler,
                        ),
                    },
                ],
                label: None,
            });
        self.cubemap_bind_group = Some(bind_group);
    }
}
