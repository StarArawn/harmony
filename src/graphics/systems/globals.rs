use legion::prelude::*;
use nalgebra_glm::{Mat4, Vec4};
use std::{sync::Arc};

use crate::{
    graphics::{
        pipelines::{GlobalUniform},
        resources::GPUResourceManager,
        CommandBufferQueue, CommandQueueItem,
    },
    scene::components,
};

// ******************************************************************************
// This section is meant to prepare our global uniforms and pass them to the GPU.
// ******************************************************************************
pub fn update_globals<'a>(camera_data: &components::CameraData, encoder: &'a mut wgpu::CommandEncoder, device: Arc<wgpu::Device>, resource_manager: Arc<GPUResourceManager>) -> Mat4 {
    let camera_matrix = camera_data.get_matrix();

    let camera_view = camera_data.view;

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

    return camera_view;
}

pub fn create() -> Box<dyn Schedulable> {
    SystemBuilder::new("encoder_globals")
        .write_resource::<crate::core::PerformanceMetrics>()
        .write_resource::<CommandBufferQueue>()
        .read_resource::<Arc<GPUResourceManager>>()
        .read_resource::<Arc<wgpu::Device>>()
        .with_query(<(Read<components::CameraData>,)>::query())
        .build(
            |_,
             world,
             (perf_metrics, command_buffer_queue, resource_manager, device),
             camera_query| {
                let global_time = std::time::Instant::now();
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("globals"),
                });

                // Get camera for update_globals function.
                let filtered_camera_data: Vec<_> = camera_query
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

                update_globals(camera_data, &mut encoder, device.clone(), resource_manager.clone());

                command_buffer_queue
                    .push(CommandQueueItem {
                        buffer: encoder.finish(),
                        name: "globals".to_string(),
                    })
                    .unwrap();
                perf_metrics.insert("transform calculations", std::time::Instant::now().duration_since(global_time));
            },
        )
}
