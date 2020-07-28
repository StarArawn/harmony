use legion::prelude::*;
use std::{convert::TryInto, sync::Arc};

use crate::{
    graphics::{
        pipeline_manager::PipelineManager,
        resources::GPUResourceManager,
        shadows::{OmniShadowManager, ShadowCamera},
        CommandBufferQueue, CommandQueueItem, pipelines::{PointLight, DirectionalLight, MAX_LIGHTS, LightingUniform}, lighting::cluster::{FROXELS_Y, FROXELS_X, FAR_PLANE_DISTANCE, FROXELS_Z},
    },
    scene::components,
};
use nalgebra_glm::Vec4;

pub fn create() -> Box<dyn Schedulable> {
    SystemBuilder::new("shadows")
        .read_resource::<Arc<GPUResourceManager>>()
        .write_resource::<crate::core::PerformanceMetrics>()
        .read_resource::<Arc<wgpu::Device>>()
        .write_resource::<ShadowCamera>()
        .write_resource::<CommandBufferQueue>()
        .read_resource::<Arc<GPUResourceManager>>()
        .read_resource::<PipelineManager>()
        .write_resource::<OmniShadowManager>()
        .with_query(<(Write<components::PointLightData>, Read<components::Transform>)>::query())
        .with_query(<(Read<components::Mesh>, Read<components::Transform>)>::query())
        .with_query(<(Read<components::CameraData>, )>::query())
        .with_query(<(Read<components::DirectionalLightData>,)>::query())
        .build(
            |_,
             mut world,
             (resource_manager, perf_metrics, device, shadow_camera, command_buffer_queue, gpu_resource_manager, pipeline_manager, omni_shadow_manager),
             (point_light_query, transform_mesh_query, camera_query, directional_light_query)| {

                // Get camera for update_globals function.
                let (cam_pos, camera_view) = {
                    let filtered_camera_data: Vec<_> = camera_query
                        .iter(&world)
                        .filter(|(camera,)| camera.active)
                        .collect();
                    let camera_data: Option<&(
                        legion::borrow::Ref<'_, crate::scene::components::camera_data::CameraData>,
                    )> = filtered_camera_data.first();
                    
                    // No camera no shadows
                    if camera_data.is_none() {
                        return;
                    }
                    let camera = &camera_data.as_ref().unwrap().0;
                    (camera.position, camera.view)
                };

                // Create shadow encoder
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("shadow"),
                });

                let shadow_sort_time = std::time::Instant::now();
                let point_lights = {
                    let mut point_lights = point_light_query.iter_mut(world)
                        .filter(|(light, _)| light.shadow)
                        .collect::<Vec<_>>();

                    
                    // First sort point lights by range / distance.
                    point_lights.sort_by(|(light_a, transform_a), (light_b, transform_b)| {
                        let distance_a = nalgebra_glm::distance2(&transform_a.position, &cam_pos);
                        let distance_b = nalgebra_glm::distance2(&transform_b.position, &cam_pos);
                        let importance_a = light_a.attenuation / distance_a;
                        let importance_b = light_b.attenuation / distance_b;
                        return importance_b.partial_cmp(&importance_a).unwrap();
                    });
                    
                    let mut total = 0;
                    let mut i = 0;
                    let mut j = 0;

                    // Allocate shadow maps
                    point_lights.iter_mut().for_each(|(light, transform)| {
                        light.shadow_texture_id = (j, i);
                        
                        i += 1;

                        // TODO: Uh we could do better..
                        if i >= 3 && j == 0 {
                            i = 0;
                            j += 1;
                        }
                        if i >= 10 && j == 1 {
                            i = 0;
                            j += 1;
                        }
                        if i >= 42 && j == 2 {
                            i = 0;
                            j += 1;
                        }
                        if i >= 170 && j == 3 {
                            i = 0;
                            j += 1;
                        }
                    });

                    // Sort point lights by distance and age.
                    point_lights.sort_by(|(light_a, transform_a), (light_b, transform_b)| {
                        let distance_a = nalgebra_glm::distance2(&transform_a.position, &cam_pos);
                        let distance_b = nalgebra_glm::distance2(&transform_b.position, &cam_pos);
                        let importance_a = light_a.attenuation / distance_a * (light_a.age + 1) as f32;
                        let importance_b = light_b.attenuation / distance_b * (light_b.age + 1) as f32;
                        return importance_b.partial_cmp(&importance_a).unwrap();
                    });

                    // Calculate or clear out ages.
                    for (light, _) in point_lights.iter_mut() {
                        if total < omni_shadow_manager.max_casters_per_frame {
                            light.age = 0;
                        } else {
                            light.age = light.age + 1;
                        }
                        total += 1;
                    }

                    // Collect data to pass to shadow renderer.
                    point_lights.iter().map(|(light, transform)| {
                        (light.attenuation, transform.position.clone(), light.shadow_texture_id)
                    })
                    .collect::<Vec<_>>()
                };
                perf_metrics.insert("shadow light sort", std::time::Instant::now().duration_since(shadow_sort_time));

                let shadow_time = std::time::Instant::now();
                omni_shadow_manager.update(
                    point_lights,
                    pipeline_manager,
                    gpu_resource_manager.clone(),
                    &mut encoder,
                    shadow_camera,
                    transform_mesh_query,
                    world,
                );
                perf_metrics.insert("shadow generation", std::time::Instant::now().duration_since(shadow_time));

                // ******************************************************************************
                // This section is where we upload our lighting uniforms to the GPU
                // ******************************************************************************
                if directional_light_query.iter(&world).count() > 0 || point_light_query.iter_mut(&mut world).count() > 0  {
                    let mut directional_light_data_vec: Vec<DirectionalLight> = directional_light_query
                        .iter(&world)
                        .map(|(data,)| DirectionalLight {
                            direction: Vec4::new(
                                data.direction.x,
                                data.direction.y,
                                data.direction.z,
                                0.0,
                            ),
                            color: Vec4::new(data.color.x, data.color.y, data.color.z, data.intensity),
                        })
                        .collect();

                    // TODO: Use some sort of distance calculation to get the closest lights.
                    let mut point_light_data_vec: Vec<PointLight> = point_light_query
                        .iter_mut(&mut world)
                        .map(|(data, transform)| {
                            let position = Vec4::new(
                                transform.position.x,
                                transform.position.y,
                                transform.position.z,
                                1.0,
                            );
                            PointLight {
                                attenuation: Vec4::new(data.attenuation, if data.shadow { 1.0 } else { 0.0 }, data.shadow_texture_id.0 as f32, data.shadow_texture_id.1 as f32),
                                color: Vec4::new(data.color.x, data.color.y, data.color.z, data.intensity),
                                position,
                                view_position: camera_view * position,
                                shadow_matrix: nalgebra_glm::perspective_fov_lh_no(
                                    90f32.to_radians(),
                                    512.0,
                                    512.0,
                                    0.1,
                                    data.attenuation,
                                ),
                                ..Default::default()
                            }
                        })
                        .collect();

                    let total_dir_lights = directional_light_data_vec.len() as u32;
                    let total_point_lights = point_light_data_vec.len() as u32;

                    // Fill in missing data if we don't have it.
                    point_light_data_vec.resize_with(MAX_LIGHTS, || PointLight::default());
                    directional_light_data_vec
                        .resize_with(4, || DirectionalLight::default());

                    let light_uniform = LightingUniform {
                        cluster_count: [FROXELS_X, FROXELS_Y, FROXELS_Z, 0],
                        light_num: Vec4::new(
                            total_dir_lights as f32,
                            total_point_lights as f32,
                            0.0,
                            FAR_PLANE_DISTANCE,
                        ),
                        directional_lights: directional_light_data_vec
                            .as_slice()
                            .try_into()
                            .unwrap(),
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

                command_buffer_queue
                    .push(CommandQueueItem {
                        buffer: encoder.finish(),
                        name: "shadow".to_string(),
                    })
                    .unwrap();
             },
        )
}
