use log;
use specs::prelude::*;

use winit::{ 
    dpi::LogicalSize,
    event::{ Event, ModifiersState, WindowEvent },
    event_loop::{ ControlFlow },
};

use harmony::WinitState;
use harmony::scene::Scene;
use harmony::scene::components::Mesh;

struct WindowSize {
    width: u32,
    height: u32,
}

const WINDOW_SIZE: WindowSize = WindowSize {
    width: 1024,
    height: 768,
};

struct AppState {
}

impl AppState {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl harmony::AppState for AppState {
    fn load(&mut self, app: &mut harmony::Application) {
        let mut scene = Scene::new(None, None);
        scene.world.create_entity().with(Mesh {
            mesh_name: "cube.gltf".into(),
        }).build();

        app.current_scene = Some(scene);
    }
    fn update(&mut self, _app: &mut harmony::Application) {
    }
    fn draw_gui(&mut self, _app: &mut harmony::Application) -> Option<&dyn harmony::gui::Scene> {
        None
    }
    fn draw(&mut self, _app: &mut harmony::Application) { }
}


fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("harmony", log::LevelFilter::Info)
        .init();

    let mut modifiers = ModifiersState::default();

    let (wb, event_loop) = WinitState::create("Harmony - Hello Cube", LogicalSize::new(WINDOW_SIZE.width, WINDOW_SIZE.height));

    let mut application = harmony::Application::new(wb, &event_loop);
    
    let mut app_state = AppState::new();

    application.load(&mut app_state);

    event_loop.run(move |event, _, control_flow| {
        application.run(&mut app_state, &event, control_flow);
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = new_modifiers;
                    }
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
                }
            },
            _ => (),
        };
    });
}