use crate::Application;
use std::{fs, io};

pub struct Skybox {
    pub texture_asset: String,
    pub(crate) cubemap_texture: Option<wgpu::Texture>,
    pub(crate) cubemap_view: Option<wgpu::TextureView>,
    pub(crate) cubemap_sampler: wgpu::Sampler,
    pub(crate) cubemap_bind_group: Option<wgpu::BindGroup>,
    pub(crate) render_texture: Option<wgpu::Texture>,
}

impl Skybox {
    pub(crate) fn new(app: &mut Application) {
        use crate::graphics::{SimplePipeline, SimplePipelineDesc};

        // We need to convert our regular texture map to a cube texture map with 6 faces.
        // Should be straight forward enough if we use equirectangular projection.
        // First we need a custom pipeline that will run in here to do the conversion.
        let mut cube_projection_pipeline_desc =
            crate::graphics::pipelines::equirectangular::CubeProjectionPipelineDesc::default();
        let pipeline = cube_projection_pipeline_desc.pipeline(&app.asset_manager, &mut app.renderer);

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
    }
}
