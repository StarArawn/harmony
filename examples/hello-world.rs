use nalgebra_glm::Vec2;
use log;

use winit::{ 
    dpi::LogicalSize,
    event::{ Event, ModifiersState, WindowEvent },
    event_loop::{ ControlFlow },
};

use harmony::WinitState;
use harmony::gui::{
    core::{ Color },
    components::{ Component, Text }
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
    gui_scene: GuiScene,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            gui_scene: GuiScene::new(),
        }
    }
}

impl harmony::AppState for AppState {
    fn load(&mut self, _app: &mut harmony::Application) { }
    fn update(&mut self, app: &mut harmony::Application) {
        self.gui_scene.update(app.delta_time);
    }
    fn draw_gui(&mut self, _app: &mut harmony::Application) -> Option<&dyn harmony::gui::Scene> {
        Some(&self.gui_scene)
    }
    fn draw(&mut self, _app: &mut harmony::Application) { }
}

struct GuiScene {
    components: Vec<Box<dyn Component>>,
}

impl GuiScene {
    pub fn new() -> Self {
        let text = Text {
            color: Color::from_rgb(1.0, 1.0, 1.0),
            text: "Hello World!".to_string(),
            size: 40.0,
            font: "fantasque.ttf".to_string(),
            // TODO: Expose the methods for measuring text to the end user.
            // We would need to likely wait to create GuiScene until after we've loaded our assets as well
            // since we need the font data to determine the size of the text.
            position: Vec2::new((WINDOW_SIZE.width as f32 / 2.0) - 100.0, (WINDOW_SIZE.height as f32 / 2.0) - 40.0),
        };
        
        Self {
            components: vec![Box::new(text)],
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        for component in self.components.iter_mut() {
            component.update(delta_time);
        }
    }
}

// This let's the gui system pull the components out of your scene.
impl harmony::gui::Scene for GuiScene {
    fn get_components(&self) -> &Vec<Box<dyn Component>> { &self.components }
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("harmony", log::LevelFilter::Info)
        .init();

    let mut modifiers = ModifiersState::default();

    let (wb, event_loop) = WinitState::create("Harmony - Hello World", LogicalSize::new(WINDOW_SIZE.width, WINDOW_SIZE.height));

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