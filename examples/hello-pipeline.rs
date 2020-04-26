use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};
use legion::prelude::*;

use harmony::{graphics::{resources::GPUResourceManager, RenderGraph, CommandBufferQueue, CommandQueueItem}, WinitState};

mod helpers;
pub use helpers::*;
use std::sync::Arc;

struct WindowSize {
    width: u32,
    height: u32,
}

const WINDOW_SIZE: WindowSize = WindowSize {
    width: 1024,
    height: 768,
};

struct AppState {}

impl AppState {
    pub fn new() -> Self {
        Self {}
    }
}

// This is an example of how to render a custom pipeline.
// You can swap between different pipelines as much as you'd like within this system.
// Note: It's important to remember that this system runs on a SEPARATE thread from main.
// So watch out for issues with things being parallel.
pub fn create_triangle_render_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("render_skybox")
        .write_resource::<CommandBufferQueue>()
        .read_resource::<RenderGraph>()
        .read_resource::<wgpu::Device>()
        .read_resource::<Arc<wgpu::SwapChainOutput>>()
        .read_resource::<GPUResourceManager>()
        .build(
            |_,
             _world,
             (command_buffer_queue, render_graph, device, output, resource_manager),
             _| {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Triangle Pass") });

            // Name of our node we created in app state "load".
            let node = render_graph.get("triangle");

            {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &output.view,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color::GREEN,
                    }],
                    depth_stencil_attachment: None,
                });
                rpass.set_pipeline(&node.pipeline);
                resource_manager.set_bind_group(&mut rpass, "triangle", 0);
                rpass.draw(0..3, 0..1);
            }

            // Name here should match node name.
            command_buffer_queue
                .push(CommandQueueItem {
                    buffer: encoder.finish(),
                    name: "triangle".to_string(),
                })
                .unwrap();
        })
}



impl harmony::AppState for AppState {
    fn load(&mut self, app: &mut harmony::Application) {

        let mut render_graph = app.resources.get_mut::<RenderGraph>().unwrap();
        let mut resource_manager = app.resources.get_mut::<GPUResourceManager>().unwrap();
        let device = app.resources.get::<wgpu::Device>().unwrap();
        let sc_desc = app.resources.get::<wgpu::SwapChainDescriptor>().unwrap();

        // Setup our custom render pass.
        let pipeline_desc = triangle_pipeline::TrianglePipelineDesc::default();
        render_graph.add(
            &app.asset_manager,
            &device,
            &sc_desc,
            &mut resource_manager,
            "triangle", // The name of the pipeline node.
            pipeline_desc, // A description of what our pipeline is which should match the shader in the pipeline as well.
            vec![], // Can be used to pass in dependencies which dictate draw order.
            false, // Automatically include local bindings from transforms these get applied to slot 0.
            None, // Optional output target useful rendering to a texture.
            false, // This option will pass the previous output as the input. Useful for chaining stuff together.
        );
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("harmony", log::LevelFilter::Info)
        .init();

    let (wb, event_loop) = WinitState::create(
        "Harmony - Hello pipeline!",
        LogicalSize::new(WINDOW_SIZE.width, WINDOW_SIZE.height),
    );

    // Tell harmony where our asset path is.
    let asset_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/").to_string();
    // When we create our application we tell it we have some custom render systems.
    // It is IMPORTANT to remember NOT to add render systems to a scene's scheduler.
    // As the scene scheduler runs potentially multiple times per frame.
    let mut application = harmony::Application::new(wb, &event_loop, asset_path, vec![create_triangle_render_system()]);
    let mut app_state = AppState::new();
    // Call application load to have harmony load all the required assets.
    application.load(&mut app_state);

    // Standard winit event loop here.
    event_loop.run(move |event, _, control_flow| {
        // Here is where the harmony does most of the work and it accepts events from winit.
        application.run(&mut app_state, &event, control_flow);
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                winit::event::WindowEvent::KeyboardInput {
                    input:
                        winit::event::KeyboardInput {
                            virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = winit::event_loop::ControlFlow::Exit,
                _ => {}
            },
            _ => (),
        };
    });
}
