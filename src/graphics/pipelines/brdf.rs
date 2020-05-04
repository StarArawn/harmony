use legion::prelude::Resources;

use crate::{
    AssetManager, 
    graphics::{
        pipeline_manager::{PipelineDesc, PipelineManager}, 
        resources::{GPUResourceManager, RenderTarget}
    }
};

// mipmaps always run pretty much right away.
pub fn create(resources: &Resources, output: &RenderTarget, format: wgpu::TextureFormat) {
    let asset_manager = resources.get_mut::<AssetManager>().unwrap();
    let mut pipeline_manager = resources.get_mut::<PipelineManager>().unwrap();
    let resource_manager = resources.get_mut::<GPUResourceManager>().unwrap();
    let device = resources.get::<wgpu::Device>().unwrap();

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("brdf") });

    let mut pipeline = pipeline_manager.get("brdf", None);

    if pipeline.is_none() {
        let mut mipmap_desc = PipelineDesc::default();
        mipmap_desc.shader = "specular_brdf.shader".to_string();
        mipmap_desc.color_state.format = format;
        mipmap_desc.cull_mode = wgpu::CullMode::None;
        pipeline_manager.add("brdf", &mipmap_desc, vec![], &device, &asset_manager, &resource_manager);
        pipeline = pipeline_manager.get("brdf", None);
    }

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &output.texture_view,
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
        render_pass.set_pipeline(&pipeline.as_ref().unwrap().render_pipeline);
        render_pass.draw(0..3, 0..1);
    }

    let queue = resources.get::<wgpu::Queue>().unwrap();
    queue.submit(Some(encoder.finish()));

    device.poll(wgpu::Maintain::Wait);
}