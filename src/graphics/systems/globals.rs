use legion::prelude::*;
use nalgebra_glm::Vec4;
use std::{convert::TryInto, sync::Arc};

use crate::{
    graphics::{
        pipelines::{DirectionalLight, GlobalUniform, LightingUniform, PointLight, MAX_LIGHTS},
        resources::GPUResourceManager,
        CommandBufferQueue, CommandQueueItem,
    },
    scene::components,
};

pub fn create() -> Box<dyn Schedulable> {
    SystemBuilder::new("encoder_globals")
        .write_resource::<CommandBufferQueue>()
        .read_resource::<Arc<GPUResourceManager>>()
        .read_resource::<Arc<wgpu::Device>>()
        .with_query(<(Read<components::CameraData>,)>::query())
        .with_query(<(Read<components::DirectionalLightData>,)>::query())
        .with_query(<(
            Read<components::PointLightData>,
            Read<components::Transform>,
        )>::query())
        .build(
            |_,
             world,
             (command_buffer_queue, resource_manager, device),
             (camera_data, directional_lights, point_lights)| {
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("globals"),
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

                // ******************************************************************************
                // This section is where we upload our lighting uniforms to the GPU
                // ******************************************************************************
                if directional_lights.iter(&world).count() > 0 {
                    let mut directional_light_data_vec: Vec<DirectionalLight> = directional_lights
                        .iter(&world)
                        .map(|(data,)| DirectionalLight {
                            direction: Vec4::new(
                                data.direction.x,
                                data.direction.y,
                                data.direction.z,
                                0.0,
                            ),
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
                    directional_light_data_vec
                        .resize_with(MAX_LIGHTS / 2, || DirectionalLight::default());

                    let light_uniform = LightingUniform {
                        light_num: Vec4::new(
                            total_dir_lights as f32,
                            total_point_lights as f32,
                            0.0,
                            0.0,
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
                        name: "globals".to_string(),
                    })
                    .unwrap();
            },
        )
}
