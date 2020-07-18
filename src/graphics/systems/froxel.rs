use crate::{
    graphics::{
        pipeline_manager::{PipelineManager},
        CommandBufferQueue,
        CommandQueueItem,
        lighting::cluster::Clustering,
    },
    scene::components
};
use legion::prelude::*;
use std::sync::Arc;

pub fn create() -> Box<dyn Schedulable> {
    SystemBuilder::new("compute_froxels")
        .write_resource::<Clustering>()
        .write_resource::<CommandBufferQueue>()
        .read_resource::<PipelineManager>()
        .read_resource::<Arc<wgpu::Device>>()
        .with_query(<Read<components::CameraData>>::query())
        .build(
            | _, world,
            (
                clustering,
                command_buffer_queue,
                pipeline_manager,
                device,
            ),
             camera_query| {
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("froxel_command_buffer"),
                });

                // We need to find at least one active camera.
                let filtered_camera_data: Vec<_> = camera_query
                    .iter(&world)
                    .filter(|camera| camera.active)
                    .collect();
                let camera_data: Option<&
                    legion::borrow::Ref<'_, crate::scene::components::camera_data::CameraData>,
                > = filtered_camera_data.first();

                if camera_data.is_none() {
                    return;
                }
                let camera_data = camera_data.unwrap();

                clustering.resize(&mut encoder, device.clone(), camera_data.frustum, camera_data.get_inverse_proj());

                clustering.compute(&mut encoder, pipeline_manager);

                command_buffer_queue
                    .push(CommandQueueItem {
                        buffer: encoder.finish(),
                        name: "froxel_creation".to_string(),
                    })
                    .unwrap();
            },
        )
}
