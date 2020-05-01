use crate::{
    graphics::{
        material::Skybox, pipelines::SkyboxUniforms, render_graph::RenderGraphNode,
        CommandBufferQueue, CommandQueueItem, RenderGraph, renderer::DepthTexture,
        resources::{RenderTargetDepth, CurrentRenderTarget}
    },
    scene::components::CameraData,
};
use legion::prelude::*;
use std::sync::Arc;

pub fn create() -> Box<dyn Schedulable> {
    SystemBuilder::new("render_skybox")
        .write_resource::<CommandBufferQueue>()
        .read_resource::<CurrentRenderTarget>()
        .read_resource::<RenderTargetDepth>()
        .read_resource::<RenderGraph>()
        .read_resource::<wgpu::Device>()
        .read_resource::<Arc<wgpu::SwapChainOutput>>()
        .read_resource::<DepthTexture>()
        .with_query(<(Read<Skybox>,)>::query())
        .with_query(<(Read<CameraData>,)>::query())
        .build(
            |_,
             world,
             (command_buffer_queue, current_render_target, render_target_depth, render_graph, device, output, depth_texture),
             (skyboxes, cameras)| {
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("skybox_clear_pass"),
                });

                let node: &RenderGraphNode = render_graph.get("skybox");

                let (bind_group, uniform_buffer) = node.simple_pipeline.get_uniforms().unwrap();

                for (camera_data,) in cameras.iter(&world) {
                    if camera_data.active {
                        let uniforms = SkyboxUniforms {
                            proj: camera_data.projection,
                            view: camera_data.view,
                        };

                        let constants_buffer = device.create_buffer_with_data(
                            bytemuck::bytes_of(&uniforms),
                            wgpu::BufferUsage::COPY_SRC,
                        );

                        encoder.copy_buffer_to_buffer(
                            &constants_buffer,
                            0,
                            uniform_buffer[0],
                            0,
                            std::mem::size_of::<SkyboxUniforms>() as u64,
                        );
                    }
                }

                let render_target_view = if current_render_target.0.is_some() {
                    Some(current_render_target.0.as_ref().unwrap().texture.create_view(&wgpu::TextureViewDescriptor {
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        dimension: wgpu::TextureViewDimension::D2,
                        aspect: wgpu::TextureAspect::default(),
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: render_target_depth.0,
                        array_layer_count: 1,
                    }))
                } else { None };

                for (skybox,) in skyboxes.iter(&world) {
                    let view_attachment = if current_render_target.0.is_some() {
                        render_target_view.as_ref().unwrap()
                    } else {
                        &output.view
                    };

                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: view_attachment,
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
                        // Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                        //     attachment: &depth_texture.0,
                        //     depth_load_op: wgpu::LoadOp::Clear,
                        //     depth_store_op: wgpu::StoreOp::Store,
                        //     stencil_load_op: wgpu::LoadOp::Clear,
                        //     stencil_store_op: wgpu::StoreOp::Store,
                        //     clear_depth: 1.0,
                        //     clear_stencil: 0,
                        // }),
                    });

                    render_pass.set_pipeline(&node.pipeline);
                    render_pass.set_bind_group(0, &bind_group[0], &[]);

                    render_pass.set_bind_group(1, skybox.cubemap_bind_group.as_ref().unwrap(), &[]);
                    render_pass.draw(0..3 as u32, 0..1);
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
