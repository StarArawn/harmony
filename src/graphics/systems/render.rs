use crate::graphics::{CommandBufferQueue, RenderGraph};
use legion::prelude::*;
use std::sync::Arc;

pub fn create() -> Box<dyn Fn(&mut World, &mut Resources) -> ()> {
    let thread = Box::new(|_world: &mut World, resources: &mut Resources| {
        let mut command_buffers = Vec::new();

        let _swap_chain_output = resources.remove::<Arc<wgpu::SwapChainOutput>>().unwrap();
        let queue = resources.get::<wgpu::Queue>().unwrap();
        let render_graph = resources.get::<RenderGraph>().unwrap();
        let mut command_queue = resources.get_mut::<CommandBufferQueue>().unwrap();
        command_buffers.extend(render_graph.collect_buffers(&mut command_queue));

        queue.submit(&command_buffers);
    });
    thread
}
