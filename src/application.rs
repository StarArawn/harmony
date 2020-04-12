use std::time::Instant;
use winit::{ 
    dpi::LogicalSize,
    event::{ Event },
    event_loop::{ ControlFlow, EventLoop },
};

use crate::gui::Scene as GuiScene;
use crate::graphics::Renderer;
use crate::{
    core::input::Input, 
    AssetManager
};

pub trait AppState {
    /// Is called after the engine has loaded an assets.
    fn load(&mut self, _app: &mut Application);
    /// Called to update app state.
    fn update(&mut self, _app: &mut Application);
    /// Draw's the gui.
    /// Return your current gui scene here or none if you don't have any.
    fn draw_gui(&mut self, _app: &mut Application) -> Option<&dyn crate::gui::Scene>;
    /// TODO: we might remove this so investigate what use this really has?
    /// Called when we draw stuff to the screen
    /// essentially this lets you draw more stuff to the screen then you might normally
    /// within the engine.
    fn draw(&mut self, _app: &mut Application);
}

pub struct Application {
    renderer: Renderer,
    gui_renderer: Option<crate::gui::Renderer>,
    gui_renderables: Vec<crate::gui::renderables::Renderable>,
    asset_manager: AssetManager,
    clock: Instant,
    fixed_timestep: f32,
    elapsed_time: f32,
    pub frame_time: f32,
    pub delta_time: f32,
    pub(crate) console: crate::gui::components::default::Console,
    pub input: Input,
}

impl Application {
    pub fn new(window_builder: winit::window::WindowBuilder, event_loop: &EventLoop<()>) -> Self {  
        let window = window_builder.build(event_loop).unwrap();
        let size = window.inner_size();
        let surface = wgpu::Surface::create(&window);

        let renderer = futures::executor::block_on(Renderer::new(window, size, surface));

        let asset_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/").to_string();
        let asset_manager = AssetManager::new(asset_path);

        let console = crate::gui::components::default::Console::new();

        Application {
            renderer,
            gui_renderer: None,
            gui_renderables: Vec::new(),
            asset_manager,
            clock: Instant::now(),
            fixed_timestep: 1.0 / 60.0,
            elapsed_time: 0.0,
            frame_time: 0.0,
            delta_time: 0.0,
            console,
            input: Input::new(),
        }
    }

    pub fn load<T>(&mut self, app_state: &mut T) where T: AppState { 
        self.asset_manager.load(&self.renderer.device, &mut self.console);
        self.console.load(&self.asset_manager);
        app_state.load(self);

        let size = self.renderer.window.inner_size();
        
        // Start up gui after load..
        let gui_renderer = crate::gui::Renderer::new(
            &self.asset_manager,
            &mut self.renderer.device,
            wgpu::TextureFormat::Bgra8UnormSrgb,
            LogicalSize::new(size.width, size.height) // TODO: Remove 2 because this wrong.
        );
        self.gui_renderer = Some(gui_renderer);
    }

    pub fn run<T>(&mut self, app_state: &mut T, event: &Event<'_, ()>, _control_flow: &mut ControlFlow) where T: AppState {
        self.input.update_events(event);
        let bounds = crate::gui::core::Rectangle {
            x: 0.0,
            y: 0.0,
            width: self.renderer.size.width as f32 / 2.0,
            height: self.renderer.size.height as f32 / 2.0,
        };
        match event {
            Event::MainEventsCleared => {
                let mut dt = self.clock.elapsed().as_secs_f32() - self.elapsed_time;
                while dt >= self.fixed_timestep {
                    dt -= self.fixed_timestep;
                    self.delta_time = dt;
                    self.elapsed_time += self.fixed_timestep;
                    
                    app_state.update(self);
                    let gui_scene = app_state.draw_gui(self);
                    if gui_scene.is_some() {
                        self.gui_renderables = gui_scene.unwrap().get_components().iter().map(|component| component.draw(bounds)).collect()
                    }
                    self.console.update(&self.input, self.delta_time);

                    self.input.clear();
                }
                
                self.renderer.window.request_redraw();
            },
            Event::RedrawRequested(_) => {
                let start = Instant::now();
                let output = self.renderer.render();
                
                // Gather console components
                let mut root_components: Vec<crate::gui::renderables::Renderable> = self.console.get_components().iter().map(|component| component.draw(bounds)).collect();
                root_components.extend(self.gui_renderables.clone());

                let root = crate::gui::renderables::Renderable::Group {
                    bounds,
                    renderables: root_components,
                };

                let gui_renderer = self.gui_renderer.as_mut().unwrap();
                gui_renderer.draw(
                    &mut self.renderer.device,
                    &mut self.renderer.queue,
                    &output.view,
                    root,
                    Some(bounds),
                    self.renderer.window.scale_factor() as f32,
                    &mut self.asset_manager,
                );

                app_state.draw(self);

                std::thread::yield_now();

                self.frame_time = Instant::now().duration_since(start).subsec_millis() as f32 / 1000.0;
            },
            _ => (),
        }
    }

}