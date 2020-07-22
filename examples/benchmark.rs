use legion::prelude::*;
use nalgebra_glm::{Quat, Vec3, Vec4};

use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent, VirtualKeyCode},
    event_loop::ControlFlow,
};

use harmony::scene::components::{CameraData, DirectionalLightData, LightType, Mesh, Transform};
use harmony::scene::{resources::DeltaTime, Scene};
use harmony::{
    graphics::resources::{ProbeFormat, ProbeQuality},
    AssetManager, WinitState, core::input::Input,
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

#[allow(dead_code)]
fn create_camera_fps_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("Camera Orbit")
        .read_resource::<DeltaTime>()
        .read_resource::<Input>()
        .with_query(<Write<CameraData>>::query())
        .build(|_, mut world, (delta_time, input), camera_query| {
            for mut camera in camera_query.iter_mut(&mut world) {
                
                let forward = input.is_key_down(VirtualKeyCode::W);
                let back = input.is_key_down(VirtualKeyCode::S);
                let right = input.is_key_down(VirtualKeyCode::D);
                let left = input.is_key_down(VirtualKeyCode::A);
                
                let frontv = Vec3::z();
                let rightv = Vec3::x();
        
                let mut movement = Vec3::zeros();
        
                if forward {
                    movement = movement + frontv
                }
        
                if back {
                    movement = movement - frontv
                }
        
                if right {
                    movement = movement - rightv
                }
        
                if left {
                    movement = movement + rightv
                }
                
                camera.yaw -= input.mouse_delta.x * 0.5 * delta_time.0;
                camera.pitch -= input.mouse_delta.y * 0.5 * delta_time.0;  
                camera.pitch = camera
                    .pitch
                    .max(-std::f32::consts::FRAC_PI_2 + 0.0001)
                    .min(std::f32::consts::FRAC_PI_2 - 0.0001);
                
                let camera_dir: Vec3 = Vec3::new(
                    camera.yaw.sin() * camera.pitch.cos(),
                    camera.pitch.sin(),
                    camera.yaw.cos() * camera.pitch.cos(),
                );

                let axis_x = nalgebra_glm::quat_angle_axis(-camera.yaw, &Vec3::new(0.0, -1.0, 0.0));
                let axis_y = nalgebra_glm::quat_angle_axis(-camera.pitch, &Vec3::new(1.0, 0.0, 0.0));

                let camera_rot: Quat = axis_x * axis_y;
                
                let camera_rot = nalgebra_glm::quat_to_mat4(&camera_rot);
                
                let movement = camera_rot * Vec4::new(movement.x, movement.y, movement.z, 1.0);

                let move_speed = 1.0;

                camera.position += movement.xyz() * move_speed;
                let camera_position = camera.position;
                let look_at = camera_position + camera_dir;
                camera.update_view(
                    camera_position,     // This is our camera's "position".
                    look_at, // Where the camera is looking at.
                    Vec3::new(0.0, 1.0, 0.0), // Our camera's up vector.
                );
            }
        })
}

fn create_rotate_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("Rotate Cube")
        .read_resource::<DeltaTime>()
        .with_query(<Write<Transform>>::query())
        .build(|_, mut world, delta_time, transform_query| {
            for mut transform in transform_query.iter_mut(&mut world) {
                transform.rotate_on_y(-2.0 * delta_time.0);
            }
        })
}

impl harmony::AppState for AppState {
    fn load(&mut self, app: &mut harmony::Application) {
        let scheduler_builder = Schedule::builder()
            // .add_system(create_camera_fps_system()) // Uncomment for a fps style camera
            .add_system(create_rotate_system());
        app.current_scene = Scene::new(None, Some(scheduler_builder));

        let mesh_handle = {
            // asset manager lets you retrieve files from disk.
            let asset_manager = app.resources.get_mut::<AssetManager>().unwrap();

            // Retrieves a mesh handle from disk.
            // Note: This could be loading still, but in our case we don't care as the system that renders the meshes
            // will wait until the mesh is finished loading before displaying it. If the loading fails you wont
            // see it in the scene and the app wont crash. You will see an error message appear in the console.
            asset_manager.get_mesh("example/meshes/cube/cube.gltf")
        };

        // Here we create our game entity that contains 3 components.
        // 1. Mesh - This is our reference to let the renderer know which asset to use from the asset pipeline.
        // 2. Material - GLTF files come with their own materials this is a reference to which material globally
        // we are picking from the asset manager. In the future we'll have an API to retrieve the material index
        // in a friendly way. For now we only have 1 GLTF file and 1 material in the file so our material index is 0.
        // 3. The transform which allows us to render the mesh using it's world cords. This also includes stuff like
        // rotation and scale.
        let size = 50;
        let scale = 3.0;
        for x in 0..size {
            for y in 0..size {
                let mut transform = Transform::new(app);
                transform.position.x = x as f32 * scale;
                transform.position.y = y as f32 * scale;
                transform.update();
                app.current_scene.world.insert(
                    (),
                    vec![(
                        Mesh::new(mesh_handle.clone()),
                        transform, // Transform
                    )],
                );
            }
        }

        // Here we create our skybox entity and populate it with a HDR skybox texture.
        // create skybox first for now this *has* to be done in load.
        let skybox = harmony::graphics::material::Skybox::new_hdr(
            app,
            "example/textures/venice_sunrise_4k.hdr",
            2048.0,
        );
        // Skybox needs to be added as an entity in legion. (we only should have one).
        app.current_scene.world.insert((), vec![(skybox,)]);

        // let clear_color = harmony::graphics::material::Skybox::create_clear_color(Vec3::new(
        //     0.0125, 0.0125, 0.0125,
        // ));
        // Clear color needs to be added as an entity in legion (we only should have one for now..).
        //app.current_scene.world.insert((), vec![(clear_color,)]);

        // Setup probe for PBR
        harmony::scene::entities::probe::create(
            app,
            Vec3::zeros(),
            ProbeQuality::Low,
            ProbeFormat::RGBA32,
        );

        // Add directional light to our scene.
        let light_transform = Transform::new(app);
        harmony::scene::entities::light::create(
            &mut app.current_scene.world,
            LightType::Directional(DirectionalLightData {
                direction: Vec3::new(0.0, 1.0, -0.5),
                color: Vec3::new(1.0, 1.0, 1.0),
                intensity: 5.0,
            }),
            light_transform,
        );

        let actual_window_size = app.get_window_actual_size();

        // TODO: Add other camera types other than perspective.
        // We can't render anything without a camera. Add one here.
        // Thankfully we have a method to help that makes it easy!
        let mut camera_data = CameraData::new_perspective(
            70.0,
            actual_window_size.width,
            actual_window_size.height,
            0.01,
            200.0,
        );
        camera_data.cull = true;
        camera_data.update_view(
            Vec3::new(
                (size as f32 * scale) / 2.0,
                (size as f32 * scale) / 2.0,
                -100.0,
            ), // This is our camera's "position".
            Vec3::new(
                (size as f32 * scale) / 2.0,
                (size as f32 * scale) / 2.0,
                0.0,
            ), // Where the camera is looking at.
            Vec3::new(0.0, 1.0, 0.0), // Our camera's up vector.
        );
        harmony::scene::entities::camera::create(&mut app.current_scene.world, camera_data);
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Error)
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
