use log;
use nalgebra_glm::{Vec2, Vec3};

use harmony::WinitState;
use imgui::{im_str, Condition};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

struct WindowSize {
    width: u32,
    height: u32,
}

const WINDOW_SIZE: WindowSize = WindowSize {
    width: 1024,
    height: 768,
};

struct AppState {
    frame_time: f32,
}

impl AppState {
    pub fn new() -> Self {
        Self { frame_time: 0.0 }
    }
}

impl harmony::AppState for AppState {
    fn load(&mut self, app: &mut harmony::Application) {
        // Create a clear color
        let clear_color = harmony::graphics::material::Skybox::create_clear_color(Vec3::new(
            0.0, 0.0, 0.0,
        ));
        // Clear color needs to be added as an entity in legion (we only should have one for now..).
        app.current_scene.world.insert((), vec![(clear_color,)]);

        {
            let mut image_asset_manager = app.resources.get_mut::<harmony::ImageAssetManager>().unwrap();
            image_asset_manager.insert("/core/white.image.ron");
            image_asset_manager.insert("/core/empty_normal.image.ron");
            image_asset_manager.insert("/core/mie.image.ron");
            image_asset_manager.insert("/core/rayleigh.image.ron");
            image_asset_manager.insert("/core/brdf_texture.image.ron");
            image_asset_manager.insert("/example/textures/georgentor.image.ron");
        }

        let skybox = harmony::graphics::material::Skybox::new_hdr(app, "/example/textures/georgentor.image.ron", 2048.0);
        app.current_scene.world.insert((), vec![(skybox,)]);
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("harmony", log::LevelFilter::Info)
        .init();

    let (wb, event_loop) = WinitState::create(
        "Harmony - Hello World",
        LogicalSize::new(WINDOW_SIZE.width, WINDOW_SIZE.height),
    );

    // Tell harmony where our asset path is.
    let asset_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/").to_string();
    let mut application = harmony::Application::new(wb, &event_loop, asset_path, vec![]);
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
