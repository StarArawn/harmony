use crate::Application;

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
        use crate::graphics::{SimplePipeline, SimplePipelineDesc};

        // We need to convert our regular texture map to a cube texture map with 6 faces.
        // Should be straight forward enough if we use equirectangular projection.
        // First we need a custom pipeline that will run in here to do the conversion.
        let mut cube_projection_pipeline_desc =
            crate::graphics::pipelines::equirectangular::CubeProjectionPipelineDesc::new(texture.into(), size);
        let pipeline = cube_projection_pipeline_desc.pipeline(&app.asset_manager, &mut app.renderer, None);

        let mut final_pipeline =
            cube_projection_pipeline_desc.build(&app.renderer.device, &pipeline.bind_group_layouts);

        let command_buffer = final_pipeline.render(
            None,
            None,
            &app.renderer.device,
            &pipeline,
            Some(&mut app.asset_manager),
            None,
        );

        app.renderer.queue.submit(&[command_buffer]);

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
            cubemap_texture: final_pipeline.cubemap_texture.unwrap(),
            cubemap_view: final_pipeline.cubemap_view.unwrap(),
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
