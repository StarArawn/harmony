use log;
use legion::prelude::*;
use nalgebra_glm::{Vec3};

use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

use harmony::scene::{
    components::{CameraData},
    resources::DeltaTime,
    Scene,
};

use harmony::{
    core::input::{Input, MouseButton},
    WinitState
};

struct WindowSize {
    width: u32,
    height: u32,
}

const WINDOW_SIZE: WindowSize = WindowSize {
    width: 1024,
    height: 768,
};

fn create_camera_orbit_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("Camera Orbit")
        .read_resource::<DeltaTime>()
        .read_resource::<Input>()
        .with_query(<Write<CameraData>>::query())
        .build(|_, mut world, (delta_time, input), camera_query| {
            for mut camera in camera_query.iter_mut(&mut world) {
                if !input.is_mouse_button_down(MouseButton::Left) {
                    continue;
                }
                camera.yaw += input.mouse_delta.x * 0.5 * delta_time.0;
                camera.pitch += input.mouse_delta.y * 0.5 * delta_time.0;
                camera.pitch = camera
                    .pitch
                    .max(-std::f32::consts::FRAC_PI_2 + 0.0001)
                    .min(std::f32::consts::FRAC_PI_2 - 0.0001);
                let eye = Vec3::new(0.0, 0.0, 0.0)
                    + (5.0
                        * nalgebra::Vector3::new(
                            camera.yaw.sin() * camera.pitch.cos(),
                            camera.pitch.sin(),
                            camera.yaw.cos() * camera.pitch.cos(),
                        ));
                camera.position = eye;
                camera.update_view(eye, Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
            }
        })
}

struct AppState {
    #[allow(unused)]
    frame_time: f32,
}

impl AppState {
    pub fn new() -> Self {
        Self { frame_time: 0.0 }
    }
}

impl harmony::AppState for AppState {
    fn load(&mut self, app: &mut harmony::Application) {
        let scheduler_builder = Schedule::builder()
            .add_system(create_camera_orbit_system());
        app.current_scene = Scene::new(None, Some(scheduler_builder));
        
        // Create a clear color
        let clear_color = harmony::graphics::material::Skybox::create_clear_color(Vec3::new(
            0.0, 0.0, 0.0,
        ));
        // Clear color needs to be added as an entity in legion (we only should have one for now..).
        app.current_scene.world.insert((), vec![(clear_color,)]);

        {
            // Test load in some random textures.
            let mut image_asset_manager = app.resources.get_mut::<harmony::ImageAssetManager>().unwrap();
            image_asset_manager.insert("core/white.image.ron").unwrap();
            image_asset_manager.insert("core/empty_normal.image.ron").unwrap();
            image_asset_manager.insert("core/mie.image.ron").unwrap();
            image_asset_manager.insert("core/rayleigh.image.ron").unwrap();
            image_asset_manager.insert("core/brdf_texture.image.ron").unwrap();
            image_asset_manager.insert("example/textures/georgentor.image.ron").unwrap();
        }

        let skybox = harmony::graphics::material::Skybox::new_hdr(app, "example/textures/georgentor.image.ron", 2048.0);
        app.current_scene.world.insert((), vec![(skybox,)]);

        let actual_window_size = app.get_window_actual_size();
        let mut camera_data = CameraData::new_perspective(
            70.0,
            actual_window_size.width,
            actual_window_size.height,
            0.01,
            10.0,
        );
        camera_data.position = Vec3::new(0.0, 0.0, 5.0);
        camera_data.update_view(
            camera_data.position,     // This is our camera's "position".
            Vec3::new(0.0, 0.0, 0.0), // Where the camera is looking at.
            Vec3::new(0.0, 1.0, 0.0), // Our camera's up vector.
        );
        harmony::scene::entities::camera::create(&mut app.current_scene.world, camera_data);
    }

    fn resize(&mut self, app: &mut harmony::Application) {
        let world = &mut app.current_scene.world;
        // This is kinda of a hacky soultion. It might be better to have this be handled internally for each camera.
        let query = <(Write<CameraData>,)>::query();
        for mut camera in query.iter_mut(world) {
            camera.0.resize(
                app.renderer.size.width as f32,
                app.renderer.size.height as f32,
            );
        }
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
