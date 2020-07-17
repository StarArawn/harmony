use legion::prelude::*;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

use harmony::{
    graphics::{
        pipeline_manager::{PipelineDesc, PipelineManager},
        resources::{BindGroup, GPUResourceManager},
        CommandBufferQueue, CommandQueueItem,
    },
    AssetManager, WinitState,
};
use nalgebra_glm::Vec3;
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
    SystemBuilder::new("render_triangle")
        .write_resource::<CommandBufferQueue>()
        .read_resource::<PipelineManager>()
        .read_resource::<Arc<wgpu::Device>>()
        .read_resource::<Arc<wgpu::SwapChainTexture>>()
        .read_resource::<Arc<GPUResourceManager>>()
        .build(
            |_,
             _world,
             (command_buffer_queue, pipeline_manager, device, output, resource_manager),
             _| {
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Triangle Pass"),
                });

                // Name of our node we created in app state "load".
                let node = pipeline_manager.get("triangle", None).unwrap();

                let triangle_bind_group = resource_manager.get_bind_group("triangle", 0).unwrap();
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &output.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });
                    rpass.set_pipeline(&node.render_pipeline);
                    rpass.set_bind_group(0, &triangle_bind_group.group, &[]);
                    rpass.draw(0..3, 0..1);
                }

                // Name here should match node name.
                command_buffer_queue
                    .push(CommandQueueItem {
                        buffer: encoder.finish(),
                        name: "triangle".to_string(),
                    })
                    .unwrap();
            },
        )
}

impl harmony::AppState for AppState {
    fn load(&mut self, app: &mut harmony::Application) {
        // First we need to access some of the internal data.
        let device = app.resources.get::<Arc<wgpu::Device>>().unwrap();
        let asset_manager = app.resources.get::<AssetManager>().unwrap();
        let gpu_resource_manager = app.resources.get::<Arc<GPUResourceManager>>().unwrap();
        let mut pipeline_manager = app.resources.get_mut::<PipelineManager>().unwrap();

        // Setup our bind groups and layouts
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[],
            label: None,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[],
            label: Some("triangle"),
        });
        gpu_resource_manager.add_single_bind_group("triangle", BindGroup::new(0, bind_group));
        gpu_resource_manager.add_bind_group_layout("triangle_layout", bind_group_layout);

        // Setup our custom pipeline
        let mut triangle_desc = PipelineDesc::default();
        triangle_desc.shader = "example/shader/triangle.shader".to_string(); // Make sure we reference the right shader!
        triangle_desc.layouts = vec!["triangle_layout".to_string()];
        triangle_desc
            .vertex_state
            .set_index_format(wgpu::IndexFormat::Uint16);
        triangle_desc.cull_mode = wgpu::CullMode::None;

        // The pipeline manager helps manage pipelines. It's somewhat smart and will cache your pipeline.
        // Remember that adding new pipelines is expensive and should be avoided at runtime.
        pipeline_manager.add_pipeline(
            "triangle",                   // Name of pipeline.
            &triangle_desc,               // Pipeline description
            vec!["skybox"], // Dependencies list as names. Uses skybox so that the triangle draws "after" the clear pass.
            &device,        // The wgpu device.
            &asset_manager, // asset manager from where we can load shaders.
            gpu_resource_manager.clone(), // The gpu resource manager.
        );

        // Pipeline manager is smart enough to not add a new pipeline even if we call pipeline_manager.add again!
        // Note: There are ways to add a variation of a pipeline by cloning the description modifying it and adding
        // it with the same name. This is useful for example if you want to render your pipeline/shader to the
        // frame buffer and to a render target(with a different format).
        pipeline_manager.add_pipeline(
            "triangle",                   // Name of pipeline.
            &triangle_desc,               // Pipeline description
            vec!["skybox"],               // Dependencies list as names.
            &device,                      // The wgpu device.
            &asset_manager,               // asset manager from where we can load shaders.
            gpu_resource_manager.clone(), // The gpu resource manager.
        );

        // Create a clear color
        let clear_color =
            harmony::graphics::material::Skybox::create_clear_color(Vec3::new(0.0, 1.0, 0.0));
        // Clear color needs to be added as an entity in legion (we only should have one for now..).
        app.current_scene.world.insert((), vec![(clear_color,)]);
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Error)
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
    let mut application = harmony::Application::new(
        wb,
        &event_loop,
        asset_path,
        vec![create_triangle_render_system()],
    );
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
