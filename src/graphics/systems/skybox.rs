use crate::graphics::{
    material::{skybox::SkyboxType, Skybox},
    pipeline_manager::{Pipeline, PipelineManager},
    renderer::DepthTexture,
    resources::{CurrentRenderTarget, GPUResourceManager},
    CommandBufferQueue, CommandQueueItem,
};
use legion::prelude::*;
use std::sync::Arc;

pub fn create() -> Box<dyn Schedulable> {
    SystemBuilder::new("render_skybox")
        .write_resource::<CommandBufferQueue>()
        .read_resource::<CurrentRenderTarget>()
        .read_resource::<Arc<GPUResourceManager>>()
        .read_resource::<PipelineManager>()
        .read_resource::<Arc<wgpu::Device>>()
        .read_resource::<Arc<wgpu::SwapChainTexture>>()
        .read_resource::<DepthTexture>()
        .with_query(<(Read<Skybox>,)>::query())
        .build(
            |_,
             world,
             (
                command_buffer_queue,
                current_render_target,
                resource_manager,
                pipeline_manager,
                device,
                output,
                depth_texture,
            ),
             skyboxes| {
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("skybox_clear_pass"),
                });
                
                let view_attachment = if current_render_target.0.is_some() {
                    &current_render_target.0.as_ref().unwrap().1
                } else {
                    &output.view
                };
                
                let depth_attachment = if current_render_target.0.is_some() {
                    current_render_target
                    .0
                    .as_ref()
                    .unwrap()
                    .0
                    .depth_texture_view
                    .as_ref()
                    .unwrap()
                } else {
                    &depth_texture.0
                };
                
                let pipeline: &Pipeline = pipeline_manager.get("skybox", None).unwrap();
                let pipeline_realtime: &Pipeline = pipeline_manager.get("realtime_skybox", None).unwrap();

                for (skybox,) in skyboxes.iter(&world) {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: view_attachment,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: skybox.clear_color.x as f64,
                                    g: skybox.clear_color.y as f64,
                                    b: skybox.clear_color.z as f64,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: Some(
                            wgpu::RenderPassDepthStencilAttachmentDescriptor {
                                attachment: depth_attachment,
                                depth_ops: Some(wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(1.0),
                                    store: true,
                                }),
                                stencil_ops: None,
                            },
                        ),
                    });

                    if skybox.skybox_type == SkyboxType::HdrCubemap {
                        render_pass.set_pipeline(&pipeline.render_pipeline);
                        render_pass.set_bind_group(0, &resource_manager.global_bind_group, &[]);

                        render_pass.set_bind_group(
                            1,
                            skybox.cubemap_bind_group.as_ref().unwrap(),
                            &[],
                        );
                        render_pass.draw(0..3 as u32, 0..1);
                    } else if skybox.skybox_type == SkyboxType::RealTime {
                        render_pass.set_pipeline(&pipeline_realtime.render_pipeline);
                        render_pass.set_bind_group(0, &resource_manager.global_bind_group, &[]);
                        render_pass.set_bind_group(
                            1,
                            skybox.cubemap_bind_group.as_ref().unwrap(),
                            &[],
                        );
                        render_pass.draw(0..3 as u32, 0..1);
                    }
                }

                command_buffer_queue
                    .push(CommandQueueItem {
                        buffer: encoder.finish(),
                        name: "skybox".to_string(),
                    })
                    .unwrap();
            },
        )
}
