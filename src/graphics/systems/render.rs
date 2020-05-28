use crate::graphics::{pipeline_manager::PipelineManager, CommandBufferQueue};
use legion::prelude::*;

pub fn create() -> Box<dyn Fn(&mut World, &mut Resources) -> ()> {
    let thread = Box::new(|_world: &mut World, resources: &mut Resources| {
        let mut command_buffers = Vec::new();

        let device = resources.get::<wgpu::Device>().unwrap();
        let queue = resources.get::<wgpu::Queue>().unwrap();
        let pipeline_manager = resources.get::<PipelineManager>().unwrap();
        let mut command_queue = resources.get_mut::<CommandBufferQueue>().unwrap();
        command_buffers.extend(pipeline_manager.collect_buffers(&mut command_queue));

        let mut image_asset_manager = resources.get_mut::<crate::assets::ImageAssetManager>().unwrap();
        image_asset_manager.update();

        queue.submit(command_buffers);
    });
    thread
}
