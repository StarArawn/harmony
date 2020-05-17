use crate::{
    graphics::{
        pipelines::GlobalUniform, render_graph::RenderGraphNode, renderer::DepthTexture,
        resources::GPUResourceManager, CommandBufferQueue, CommandQueueItem, RenderGraph,
    },
    scene::components,
    AssetManager,
};
use legion::prelude::*;
use nalgebra_glm::Vec4;
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
        .with_query(<(Read<components::CameraData>,)>::query())
        .with_query(<Read<components::Mesh>>::query())
        .build(
            |_,
             world,
             (
                asset_manager,
                command_buffer_queue,
                render_graph,
                device,
                output,
                resource_manager,
                depth_texture,
            ),
             (camera_data, mesh_query)| {
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("line_renderer"),
                });

                // ******************************************************************************
                // This section is meant to prepare our global uniforms and pass them to the GPU.
                // ******************************************************************************
                {
                    let filtered_camera_data: Vec<_> = camera_data
                        .iter(&world)
                        .filter(|(camera,)| camera.active)
                        .collect();
                    let camera_data: Option<&(
                        legion::borrow::Ref<'_, crate::scene::components::camera_data::CameraData>,
                    )> = filtered_camera_data.first();

                    if camera_data.is_none() {
                        return;
                    }

                    let camera_data = &camera_data.as_ref().unwrap().0;
                    let camera_matrix = camera_data.get_matrix();

                    let uniforms = GlobalUniform {
                        view_projection: camera_matrix,
                        camera_pos: Vec4::new(
                            camera_data.position.x,
                            camera_data.position.y,
                            camera_data.position.z,
                            0.0,
                        ),
                        view: camera_data.view,
                        projection: camera_data.projection,
                    };

                    let constants_buffer = device.create_buffer_with_data(
                        bytemuck::bytes_of(&uniforms),
                        wgpu::BufferUsage::COPY_SRC,
                    );

                    encoder.copy_buffer_to_buffer(
                        &constants_buffer,
                        0,
                        &resource_manager.global_uniform_buffer,
                        0,
                        std::mem::size_of::<GlobalUniform>() as u64,
                    );
                }

                let node: &RenderGraphNode = render_graph.get("line");

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &output.view,
                            resolve_target: None,
                            load_op: wgpu::LoadOp::Load,
                            store_op: wgpu::StoreOp::Store,
                            clear_color: wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0,
                            },
                        }],
                        depth_stencil_attachment: Some(
                            wgpu::RenderPassDepthStencilAttachmentDescriptor {
                                attachment: &depth_texture.0,
                                depth_load_op: wgpu::LoadOp::Load,
                                depth_store_op: wgpu::StoreOp::Store,
                                stencil_load_op: wgpu::LoadOp::Load,
                                stencil_store_op: wgpu::StoreOp::Store,
                                clear_depth: 1.0,
                                clear_stencil: 0,
                            },
                        ),
                    });

                    render_pass.set_pipeline(&node.pipeline);
                    render_pass.set_bind_group(0, &resource_manager.global_bind_group, &[]);
                    // draw lines
                    for mesh in mesh_query.iter(&world) {
                        let asset_mesh = asset_manager.get_mesh(mesh.mesh_name.clone());
                        for sub_mesh in asset_mesh.sub_meshes.iter() {
                            render_pass.set_vertex_buffer(
                                0,
                                sub_mesh.tangent_line_buffer.as_ref().unwrap(),
                                0,
                                0,
                            );
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
            },
        )
}
