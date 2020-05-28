use crate::{
    graphics::{
        material::Material, pipeline_manager::PipelineManager, renderer::DepthTexture,
        resources::GPUResourceManager, CommandBufferQueue, CommandQueueItem, RenderGraph,
    },
    scene::components,
    AssetManager,
};
use components::transform::LocalUniform;
use legion::prelude::*;
use std::sync::Arc;

pub fn create() -> Box<dyn Schedulable> {
    SystemBuilder::new("render_mesh")
        .write_resource::<AssetManager>()
        .write_resource::<CommandBufferQueue>()
        .read_resource::<RenderGraph>()
        .read_resource::<Arc<wgpu::Device>>()
        .read_resource::<Arc<wgpu::SwapChainOutput>>()
        .read_resource::<GPUResourceManager>()
        .read_resource::<DepthTexture>()
        .read_resource::<PipelineManager>()
        .with_query(<(Write<components::Transform>,)>::query())
        .with_query(<(
            Read<components::Mesh>,
            Read<components::Material>,
            Read<components::Transform>,
        )>::query())
        .build(
            |_,
             mut world,
             (
                asset_manager,
                command_buffer_queue,
                render_graph,
                device,
                output,
                resource_manager,
                depth_texture,
                pipeline_manager,
            ),
             (transform_query, mesh_query)| {
                // Create mesh encoder
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("mesh"),
                });

                // ******************************************************************************
                // This section is where we upload our transforms to the GPU
                // ******************************************************************************
                if transform_query.iter_mut(&mut world).count() > 0 {
                    let size = std::mem::size_of::<LocalUniform>();
                    let mut_world = &mut world;
                    let mut temp_buf_data = device.create_buffer_mapped(&wgpu::BufferDescriptor {
                        size: (transform_query.iter_mut(mut_world).count() * size) as u64,
                        usage: wgpu::BufferUsage::COPY_SRC,
                        label: None,
                    });

                    // FIXME: Align and use `LayoutVerified`
                    for ((mut transform,), slot) in transform_query
                        .iter_mut(mut_world)
                        .zip(temp_buf_data.data().chunks_exact_mut(size))
                    {
                        transform.update();
                        slot.copy_from_slice(bytemuck::bytes_of(&LocalUniform {
                            world: transform.matrix,
                        }));
                    }

                    let temp_buf = temp_buf_data.finish();

                    let mut i = 0;
                    for (transform,) in transform_query.iter_mut(mut_world) {
                        let transform_buffer =
                            resource_manager.get_multi_buffer("transform", transform.index);
                        encoder.copy_buffer_to_buffer(
                            &temp_buf,
                            (i * size) as wgpu::BufferAddress,
                            &transform_buffer,
                            0,
                            size as wgpu::BufferAddress,
                        );
                        i += 1;
                    }
                }

                // ******************************************************************************
                // This section is where we actually render our meshes.
                // ******************************************************************************
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
                                a: 1.0,
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

                    if mesh_query.iter(&world).count() > 0 {
                        // Collect materials in to their groups.
                        let asset_materials = asset_manager.get_materials();
                        let pbr_materials: Vec<_> = asset_materials
                            .iter()
                            .filter(|material| match material {
                                Material::PBR(_) => true,
                                _ => false,
                            })
                            .collect();
                        let unlit_materials: Vec<_> = asset_materials
                            .iter()
                            .filter(|material| match material {
                                Material::Unlit(_) => true,
                                _ => false,
                            })
                            .collect();

                        // Render unlit materials.
                        let unlit_node = render_graph.get("unlit");
                        render_pass.set_pipeline(&unlit_node.pipeline);
                        render_pass.set_bind_group(1, &resource_manager.global_bind_group, &[]);
                        for material in unlit_materials.iter() {
                            match material {
                                Material::Unlit(data) => {
                                    render_pass.set_bind_group(
                                        2,
                                        &data.bind_group_data.as_ref().unwrap().bind_group,
                                        &[],
                                    );
                                    for (mesh, _, transform) in mesh_query
                                        .iter(&world)
                                        .filter(|(_, material, _)| material.index == data.index)
                                    {
                                        resource_manager.set_multi_bind_group(
                                            &mut render_pass,
                                            "transform",
                                            0,
                                            transform.index,
                                        );
                                        let asset_mesh =
                                            asset_manager.get_mesh(mesh.mesh_name.clone());
                                        for sub_mesh in asset_mesh.sub_meshes.iter() {
                                            render_pass.set_index_buffer(
                                                sub_mesh.index_buffer.slice(..)
                                            );
                                            render_pass.set_vertex_buffer(
                                                0,
                                                sub_mesh.vertex_buffer.as_ref().unwrap().slice(..),
                                            );
                                            render_pass.draw_indexed(
                                                0..sub_mesh.index_count as u32,
                                                0,
                                                0..1,
                                            );
                                        }
                                    }
                                }
                                _ => (),
                            }
                        }

                        // Render pbr materials.
                        let pbr_node = pipeline_manager.get("pbr", None).unwrap();
                        render_pass.set_pipeline(&pbr_node.render_pipeline);
                        render_pass.set_bind_group(1, &resource_manager.global_bind_group, &[]);
                        resource_manager.set_bind_group(&mut render_pass, "probe_material", 3);
                        for material in pbr_materials.iter() {
                            match material {
                                Material::PBR(data) => {
                                    resource_manager.set_multi_bind_group(
                                        &mut render_pass,
                                        "pbr",
                                        2,
                                        data.index as u32,
                                    );
                                    for (mesh, _, transform) in mesh_query
                                        .iter(&world)
                                        .filter(|(_, material, _)| material.index == data.index)
                                    {
                                        resource_manager.set_multi_bind_group(
                                            &mut render_pass,
                                            "transform",
                                            0,
                                            transform.index,
                                        );
                                        let asset_mesh =
                                            asset_manager.get_mesh(mesh.mesh_name.clone());
                                        for sub_mesh in asset_mesh.sub_meshes.iter() {
                                            render_pass.set_index_buffer(
                                                sub_mesh.index_buffer.slice(..)
                                            );
                                            render_pass.set_vertex_buffer(
                                                0,
                                                sub_mesh.vertex_buffer.as_ref().unwrap().slice(..),
                                            );
                                            render_pass.draw_indexed(
                                                0..sub_mesh.index_count as u32,
                                                0,
                                                0..1,
                                            );
                                        }
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                }

                command_buffer_queue
                    .push(CommandQueueItem {
                        buffer: encoder.finish(),
                        name: "pbr".to_string(),
                    })
                    .unwrap();
            },
        )
}
