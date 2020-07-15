use legion::prelude::*;
use log;
use nalgebra_glm::Vec3;

use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

use harmony::scene::{
    components::{CameraData, DirectionalLightData, LightType, Material, Mesh, Transform},
    resources::DeltaTime,
    Scene,
};
use harmony::{
    core::input::{Input, MouseButton},
    graphics::resources::{ProbeFormat, ProbeQuality},
    WinitState,
};

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

fn create_rotate_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("Rotate Cube")
        .read_resource::<DeltaTime>()
        .with_query(<Write<Transform>>::query())
        .build(|_, mut world, delta_time, transform_query| {
            for mut transform in transform_query.iter_mut(&mut world) {
                // transform.rotate_on_y(-0.5 * delta_time.0);
                // transform.rotate_on_x(-0.5 * delta_time.0);
            }
        })
}

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

impl harmony::AppState for AppState {
    fn load(&mut self, app: &mut harmony::Application) {
        let scheduler_builder = Schedule::builder()
            .add_system(create_rotate_system())
            .add_system(create_camera_orbit_system());
        app.current_scene = Scene::new(None, Some(scheduler_builder));

        // Here we create our game entity that contains 3 components.
        // 1. Mesh - This is our reference to let the renderer know which asset to use from the asset pipeline.
        // 2. Material - GLTF files come with their own materials this is a reference to which material globally
        // we are picking from the asset manager. In the future we'll have an API to retrieve the material index
        // in a friendly way. For now we only have 1 GLTF file and 1 material in the file so our material index is 0.
        // 3. The transform which allows us to render the mesh using it's world cords. This also includes stuff like
        // rotation and scale.
        let transform = Transform::new(app);
        app.current_scene.world.insert(
            (),
            vec![(
                Mesh::new("cube.gltf"),
                transform,
            )],
        );

        // Here we create our skybox entity and populate it with a HDR skybox texture.
        let skybox =
            harmony::graphics::material::Skybox::new_hdr(app, "example/textures/venice_sunrise_4k.hdr", 2048.0);
        // Or create a realtime skybox:
        // Note: realtime skybox will use the first directional light as the sun position.
        // let skybox =
        //     harmony::graphics::material::Skybox::create_realtime();
        // Skybox needs to be added as an entity in legion (we only should have one for now..).
        app.current_scene.world.insert((), vec![(skybox,)]);

        // Setup probe for PBR
        harmony::scene::entities::probe::create(
            app,
            Vec3::zeros(),
            ProbeQuality::Low,
            ProbeFormat::RGBA16,
        );

        // Add directional light to our scene.
        let light_transform = Transform::new(app);
        harmony::scene::entities::light::create(
            &mut app.current_scene.world,
            LightType::Directional(DirectionalLightData {
                direction: Vec3::new(0.0, 1.0, 0.0),
                color: Vec3::new(0.9, 0.55, 0.42),
            }),
            light_transform,
        );

        // Add red point light to our scene.
        // Uncomment this code to see point light.
        // Point lights currently don't work.
        // let mut transform = Transform::new(app);
        // transform.position = Vec3::new(-5.0, 0.0, 0.0);
        // harmony::scene::entities::light::create(
        //     &mut scene.world,
        //     LightType::Point(PointLightData {
        //         color: Vec3::new(1.0, 0.0, 0.0),
        //         attenuation: 10.0,
        //     }),
        //     transform,
        // );

        let actual_window_size = app.get_window_actual_size();

        // TODO: Add other camera types other than perspective.
        // We can't render anything without a camera. Add one here.
        // Thankfully we have a method to help that makes it easy!
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
        "Harmony - Hello Cube",
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
