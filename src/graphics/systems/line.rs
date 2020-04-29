use legion::prelude::*;
use crate::{
    graphics::{
        RenderGraph,
        CommandBufferQueue,
        resources::GPUResourceManager,
        renderer::DepthTexture,
        render_graph::RenderGraphNode,
        CommandQueueItem
    },
    scene::components,
    AssetManager,
};
use std::sync::Arc;

pub fn create() -> Box<dyn Schedulable> {
    SystemBuilder::new("render_lines")
        .write_resource::<AssetManager>()
        .write_resource::<CommandBufferQueue>()
        .read_resource::<RenderGraph>()
        .read_resource::<wgpu::Device>()
        .read_resource::<Arc<wgpu::SwapChainOutput>>()
        .read_resource::<GPUResourceManager>()
        .read_resource::<DepthTexture>()
        .with_query(<Read<components::Mesh>>::query())
        .build(
            |_,
                mut world,
                (asset_manager, command_buffer_queue, render_graph, device, output, resource_manager, depth_texture),
                mesh_query,
            |{
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("line_renderer"),
                });

                let node: &RenderGraphNode = render_graph.get("line");

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &output.view,
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
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                            attachment: &depth_texture.0,
                            depth_load_op: wgpu::LoadOp::Clear,
                            depth_store_op: wgpu::StoreOp::Store,
                            stencil_load_op: wgpu::LoadOp::Clear,
                            stencil_store_op: wgpu::StoreOp::Store,
                            clear_depth: 1.0,
                            clear_stencil: 0,
                        }),
                    });

                    render_pass.set_pipeline(&node.pipeline);
                    render_pass.set_bind_group(1, &resource_manager.global_bind_group, &[]);
                    // draw lines
                    for mesh in mesh_query.iter(&world) {
                        let asset_mesh = asset_manager.get_mesh(mesh.mesh_name.clone());
                        for sub_mesh in asset_mesh.sub_meshes.iter() {
                            render_pass.set_vertex_buffer(0, &sub_mesh.vertex_buffer, 0, 0);
                            render_pass.draw(0..sub_mesh.tangent_lines.len() as u32, 0..1);
                        }
                    }
                }

                command_buffer_queue
                    .push(CommandQueueItem {
                        buffer: encoder.finish(),
                        name: "line".to_string(),
                    })
                    .unwrap();
        })
}