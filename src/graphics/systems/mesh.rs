use legion::prelude::*;
use crate::{
    scene::components::{
        CameraData,
        Mesh,
        Transform,
        Material,
        DirectionalLightData,
        PointLightData
    },
    graphics::{
        CommandBufferQueue, CommandQueueItem, RenderGraph,
        render_graph::RenderGraphNode, pipelines::GlobalUniform, resources::GPUResourceManager
    }
};
use std::sync::Arc;

pub fn create() -> Box<dyn Schedulable> {
    SystemBuilder::new("render_skybox")
        .write_resource::<CommandBufferQueue>()
        .read_resource::<RenderGraph>()
        .read_resource::<wgpu::Device>()
        .read_resource::<Arc<wgpu::SwapChainOutput>>()
        .read_resource::<GPUResourceManager>()
        .with_query(<(Read<CameraData>, )>::query())
        .with_query(<(Read<DirectionalLightData>, )>::query())
        .with_query(<(Read<PointLightData>, )>::query())
        .with_query(<(Write<Transform>, )>::query())
        .with_query(<(Read<Mesh>, Read<Material>, Read<Transform>)>::query())
        .build(|
            _,
            mut world,
            (command_buffer_queue, render_graph, device, output, resource_manager),
            (camera_data, directional_light_data, point_light_data, transform_query, mesh_query)
        | {
            // ******************************************************************************
            // This section is meant to prepare our global uniforms and pass them to the GPU.
            // ******************************************************************************
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("mesh") });
            
            let filtered_camera_data: Vec<_> = camera_data.iter(&world)
                .filter(|(camera, )| camera.active)
                .collect();
            let camera_data: Option<&(legion::borrow::Ref<'_, crate::scene::components::camera_data::CameraData>,)> = filtered_camera_data.first();

            if camera_data.is_none() {
                return;
            }

            let camera_data = &camera_data.as_ref().unwrap().0;
            let camera_matrix = camera_data.get_matrix();

            let uniforms = GlobalUniform {
                view_projection: camera_matrix,
            };

            let constants_buffer = device
                .create_buffer_with_data(bytemuck::bytes_of(&uniforms), wgpu::BufferUsage::COPY_SRC);

            // encoder.copy_buffer_to_buffer(
            //     &constants_buffer,
            //     0,
            //     &self.constants_buffer,
            //     0,
            //     std::mem::size_of::<GlobalUniform>() as u64,
            // );

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
                    depth_stencil_attachment: None,
                });


            }

            command_buffer_queue
            .push(CommandQueueItem {
                buffer: encoder.finish(),
                name: "pbr".to_string(),
            })
            .unwrap();
        })
}
