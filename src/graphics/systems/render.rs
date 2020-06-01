use crate::graphics::{pipeline_manager::PipelineManager, CommandBufferQueue};
use legion::prelude::*;
use std::sync::Arc;

pub fn create() -> Box<dyn Fn(&mut World, &mut Resources) -> ()> {
    let thread = Box::new(|_world: &mut World, resources: &mut Resources| {
        let mut command_buffers = Vec::new();

        let device = resources.get::<Arc<wgpu::Device>>().unwrap();
        let queue = resources.get::<Arc<wgpu::Queue>>().unwrap();
        let pipeline_manager = resources.get::<PipelineManager>().unwrap();
        let mut command_queue = resources.get_mut::<CommandBufferQueue>().unwrap();
        command_buffers.extend(pipeline_manager.collect_buffers(&mut command_queue));

        queue.submit(command_buffers);
    });
    thread
}
