use legion::prelude::*;
use std::sync::Arc;

use crate::{
    graphics::{
        pipeline_manager::PipelineManager,
        resources::GPUResourceManager,
        shadows::{OmniShadowManager, ShadowCamera},
        CommandBufferQueue, CommandQueueItem,
    },
    scene::components,
};

pub fn create() -> Box<dyn Schedulable> {
    SystemBuilder::new("shadows")
        .read_resource::<Arc<wgpu::Device>>()
        .write_resource::<ShadowCamera>()
        .write_resource::<CommandBufferQueue>()
        .read_resource::<Arc<GPUResourceManager>>()
        .read_resource::<PipelineManager>()
        .write_resource::<OmniShadowManager>()
        .with_query(<(Write<components::PointLightData>, Read<components::Transform>)>::query())
        .with_query(<(Read<components::Mesh>, Read<components::Transform>)>::query())
        .with_query(<(Read<components::CameraData>, )>::query())
        .build(
            |_,
             world,
             (device, shadow_camera, command_buffer_queue, gpu_resource_manager, pipeline_manager, omni_shadow_manager),
             (point_light_query, transform_mesh_query, camera_query)| {

                // Get camera for update_globals function.
                // let cam_pos = {
                //     let filtered_camera_data: Vec<_> = camera_query
                //         .iter(&world)
                //         .filter(|(camera,)| camera.active)
                //         .collect();
                //     let camera_data: Option<&(
                //         legion::borrow::Ref<'_, crate::scene::components::camera_data::CameraData>,
                //     )> = filtered_camera_data.first();
                    
                //     // No camera no shadows
                //     if camera_data.is_none() {
                //         return;
                //     }
                //     let camera = &camera_data.as_ref().unwrap().0;
                //     camera.position
                // };

                // Create shadow encoder
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("shadow"),
                });


                let point_lights = {
                    let mut point_lights = point_light_query.iter_mut(world)
                        .filter(|(light, _)| light.shadow)
                        .collect::<Vec<_>>();
                    
                    // TODO: This doesn't really work because of the threaded nature of our application.
                    // Essentially we could change light.shadow_texture_id here by sorting based off of some criteria,
                    // but the "globals" system more than likely already finished running which will cause
                    // a "moment" or frame where the wrong shadow map could be used. This is actually highly
                    // noticeable as it flickers.
                    // Another thing to figure out here is mananging how many lights get updated per frame.
                    // Currently all of the lights that can render shadows do so every single frame.
                    // This leads to poor performance. Ideally we would want to change the sorting algorthim
                    // to: range / distance(light to camera) + (age + 1)
                    // age in this particular case would represent how old a shadow map is in frames.
                    // Then we simply only render x number of shadowed lights per frame, and
                    // older shadow maps will be sorted into the more important catgeory.
                    // I think we'd likely want to separate out the age logic from the shadow map
                    // resolution logic. Essentially updates should happen based on the formula
                    // above and resoultion should happen based on distance from the camera.

                    // point_lights.sort_by(|(light_a, transform_a), (light_b, transform_b)| {
                    //     let distance_a = nalgebra_glm::distance2(&transform_a.position, &cam_pos);
                    //     let distance_b = nalgebra_glm::distance2(&transform_b.position, &cam_pos);
                    //     let importance_a = light_a.attenuation / distance_a;
                    //     let importance_b = light_b.attenuation / distance_b;
                    //     return importance_a.partial_cmp(&importance_b).unwrap();
                    // });
                    
                    let mut i = 0;
                    let mut j = 0;
                    point_lights.iter_mut().map(|(light, transform)| {
                        light.shadow_texture_id = (j, i);
                        i += 1;

                        // TODO: Uh we could do better..
                        if i > 4 && j == 0 {
                            i = 0;
                            j += 1;
                        }
                        if i > 11 && j == 0 {
                            i = 0;
                            j += 1;
                        }
                        if i > 43 && j == 0 {
                            i = 0;
                            j += 1;
                        }
                        if i > 171 && j == 0 {
                            i = 0;
                            j += 1;
                        }
                        (light.attenuation, transform.position.clone())
                    })
                    .collect::<Vec<_>>()
                };

                omni_shadow_manager.update(
                    point_lights,
                    pipeline_manager,
                    gpu_resource_manager.clone(),
                    &mut encoder,
                    shadow_camera,
                    transform_mesh_query,
                    world,
                );

                command_buffer_queue
                    .push(CommandQueueItem {
                        buffer: encoder.finish(),
                        name: "shadow".to_string(),
                    })
                    .unwrap();
             },
        )
}
