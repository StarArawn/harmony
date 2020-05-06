use crate::{
    graphics::{
        pipelines::{LightingUniform, PointLight, DirectionalLight, GlobalUniform, MAX_LIGHTS}, resources::GPUResourceManager,
        CommandBufferQueue, CommandQueueItem, RenderGraph, material::Material, renderer::DepthTexture, pipeline_manager::PipelineManager,
    },
    scene::components,
    AssetManager,
};
use std::convert::TryInto;
use legion::prelude::*;
use std::sync::Arc;
use nalgebra_glm::Vec4;
use components::transform::LocalUniform;

pub fn create() -> Box<dyn Schedulable> {
    SystemBuilder::new("render_skybox")
        .write_resource::<AssetManager>()
        .write_resource::<CommandBufferQueue>()
        .read_resource::<RenderGraph>()
        .read_resource::<wgpu::Device>()
        .read_resource::<Arc<wgpu::SwapChainOutput>>()
        .read_resource::<GPUResourceManager>()
        .read_resource::<DepthTexture>()
        .read_resource::<PipelineManager>()
        .with_query(<(Read<components::CameraData>,)>::query())
        .with_query(<(Read<components::DirectionalLightData>,)>::query())
        .with_query(<(Read<components::PointLightData>, Read<components::Transform>)>::query())
        .with_query(<(Write<components::Transform>,)>::query())
        .with_query(<(Read<components::Mesh>, Read<components::Material>, Read<components::Transform>)>::query())
        .build(
            |_,
             mut world,
             (asset_manager, command_buffer_queue, render_graph, device, output, resource_manager, depth_texture, pipeline_manager),
             (
                camera_data,
                directional_lights,
                point_lights,
                transform_query,
                mesh_query,
            )| {
                // Create mesh encoder
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("mesh"),
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
                        camera_pos: Vec4::new(camera_data.position.x, camera_data.position.y, camera_data.position.z, 0.0),
                        view: camera_data.view,
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

                // ******************************************************************************
                // This section is where we upload our lighting uniforms to the GPU
                // ******************************************************************************
                if directional_lights.iter(&world).count() > 0 {
                    let mut directional_light_data_vec: Vec<DirectionalLight> = directional_lights
                        .iter(&world)
                        .map(|(data, )| DirectionalLight {
                            direction: Vec4::new(data.direction.x, data.direction.y, data.direction.z, 0.0),
                            color: Vec4::new(data.color.x, data.color.y, data.color.z, 1.0),
                        })
                        .collect();

                    // TODO: Use some sort of distance calculation to get the closest lights.
                    let mut point_light_data_vec: Vec<PointLight> = point_lights
                        .iter(&world)
                        .map(|(data, transform)| PointLight {
                            attenuation: Vec4::new(data.attenuation, 0.0, 0.0, 0.0),
                            color: Vec4::new(data.color.x, data.color.y, data.color.z, 1.0),
                            position: Vec4::new(
                                transform.position.x,
                                transform.position.y,
                                transform.position.z,
                                0.0,
                            ),
                        })
                        .collect();

                    let total_dir_lights = directional_light_data_vec.len() as u32;
                    let total_point_lights = point_light_data_vec.len() as u32;

                    // Fill in missing data if we don't have it.
                    point_light_data_vec.resize_with(MAX_LIGHTS / 2, || PointLight::default());
                    directional_light_data_vec.resize_with(MAX_LIGHTS / 2, || DirectionalLight::default());

                    let light_uniform = LightingUniform {
                        light_num: Vec4::new(total_dir_lights as f32, total_point_lights as f32, 0.0, 0.0),
                        directional_lights: directional_light_data_vec.as_slice().try_into().unwrap(),
                        point_lights: point_light_data_vec.as_slice().try_into().unwrap(),
                    };

                    let lighting_buffer = device.create_buffer_with_data(
                        bytemuck::bytes_of(&light_uniform),
                        wgpu::BufferUsage::COPY_SRC,
                    );

                    encoder.copy_buffer_to_buffer(
                        &lighting_buffer,
                        0,
                        &resource_manager.global_lighting_buffer,
                        0,
                        std::mem::size_of::<LightingUniform>() as u64,
                    );
                }

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
                    for ((mut transform, ), slot) in transform_query
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
                    for (transform, ) in transform_query.iter_mut(mut_world) {
                        let transform_buffer = resource_manager.get_multi_buffer("transform", transform.index);
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
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                            attachment: &depth_texture.0,
                            depth_load_op: wgpu::LoadOp::Load,
                            depth_store_op: wgpu::StoreOp::Store,
                            stencil_load_op: wgpu::LoadOp::Load,
                            stencil_store_op: wgpu::StoreOp::Store,
                            clear_depth: 1.0,
                            clear_stencil: 0,
                        }),
                    });

                    if mesh_query.iter(&world).count() > 0 {
                        // Collect materials in to their groups.
                        let asset_materials = asset_manager.get_materials();
                        let pbr_materials: Vec<_> = asset_materials.iter().filter(|material| match material { Material::PBR(_) => { true }, _ => { false }}).collect();
                        let unlit_materials: Vec<_> = asset_materials.iter().filter(|material| match material { Material::Unlit(_) => { true }, _ => { false }}).collect();
                        
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
                                    for (mesh, _, transform) in mesh_query.iter(&world)
                                        .filter(|(_, material, _)| material.index == data.index)
                                    {
                                        resource_manager.set_multi_bind_group(&mut render_pass, "transform", 0, transform.index);
                                        let asset_mesh = asset_manager.get_mesh(mesh.mesh_name.clone());
                                        for sub_mesh in asset_mesh.sub_meshes.iter() {
                                            render_pass.set_index_buffer(&sub_mesh.index_buffer, 0, 0);
                                            render_pass.set_vertex_buffer(0, sub_mesh.vertex_buffer.as_ref().unwrap(), 0, 0);
                                            render_pass.draw_indexed(0..sub_mesh.index_count as u32, 0, 0..1);
                                        }
                                    }

                                },
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
                                    resource_manager.set_multi_bind_group(&mut render_pass, "pbr", 2, data.index as u32);
                                    for (mesh, _, transform) in mesh_query.iter(&world)
                                        .filter(|(_, material, _)| material.index == data.index)
                                    {
                                        resource_manager.set_multi_bind_group(&mut render_pass, "transform", 0, transform.index);
                                        let asset_mesh = asset_manager.get_mesh(mesh.mesh_name.clone());
                                        for sub_mesh in asset_mesh.sub_meshes.iter() {
                                            render_pass.set_index_buffer(&sub_mesh.index_buffer, 0, 0);
                                            render_pass.set_vertex_buffer(0, sub_mesh.vertex_buffer.as_ref().unwrap(), 0, 0);
                                            render_pass.draw_indexed(0..sub_mesh.index_count as u32, 0, 0..1);
                                        }
                                    }

                                },
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
