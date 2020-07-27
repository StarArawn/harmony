use legion::prelude::Resources;

use crate::{
    graphics::{
        pipeline_manager::{PipelineDesc, PipelineManager},
        resources::{GPUResourceManager, RenderTarget},
    },
    AssetManager,
};
use std::{borrow::Cow, sync::Arc};

// mipmaps always run pretty much right away.
pub fn create(resources: &Resources, output: &RenderTarget, format: wgpu::TextureFormat) {
    let asset_manager = resources.get_mut::<AssetManager>().unwrap();
    let mut pipeline_manager = resources.get_mut::<PipelineManager>().unwrap();
    let resource_manager = resources.get::<Arc<GPUResourceManager>>().unwrap();
    let device = resources.get::<Arc<wgpu::Device>>().unwrap();

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("brdf"),
    });

    let mut pipeline = pipeline_manager.get("brdf", None);

    if pipeline.is_none() {
        let mut mipmap_desc = PipelineDesc::default();
        mipmap_desc.shader = "core/shaders/calculations/specular_brdf.shader".to_string();
        mipmap_desc.color_states[0].format = format;
        mipmap_desc.cull_mode = wgpu::CullMode::None;
        pipeline_manager.add_pipeline(
            "brdf",
            &mipmap_desc,
            vec![],
            &device,
            &asset_manager,
            resource_manager.clone(),
        );
        pipeline = pipeline_manager.get("brdf", None);
    }

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: Cow::Borrowed(&[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &output.texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: true,
                },
            }]),
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&pipeline.as_ref().unwrap().render_pipeline);
        render_pass.draw(0..3, 0..1);
    }

    let queue = resources.get::<Arc<wgpu::Queue>>().unwrap();
    queue.submit(Some(encoder.finish()));

    device.poll(wgpu::Maintain::Wait);
}
