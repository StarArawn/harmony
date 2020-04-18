use std::time::Instant;
use winit::{
    dpi::LogicalSize,
    event::Event,
    event_loop::{ControlFlow, EventLoop},
};

use crate::{
    core::input::Input,
    graphics::{RenderGraph, Renderer},
    gui::Scene as GuiScene,
    scene::Scene,
    AssetManager,
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
    pub(crate) renderer: Renderer,
    gui_renderer: Option<crate::gui::Renderer>,
    gui_renderables: Vec<crate::gui::renderables::Renderable>,
    pub asset_manager: AssetManager,
    clock: Instant,
    fixed_timestep: f32,
    elapsed_time: f32,
    pub frame_time: f32,
    pub delta_time: f32,
    pub(crate) console: crate::gui::components::default::Console,
    pub input: Input,
    pub current_scene: Option<Scene<'static>>,
    pub(crate) render_graph: Option<RenderGraph>,
}

impl Application {
    /// Creates a new application.
    /// # Arguments
    ///
    /// * `window_builder` - The winit WindowBuilder that harmony can use to setup the window for rendering.
    /// * `event_loop` - A reference to winit's event loop.
    /// * `asset_path` - Path to the asset folder.
    ///
    /// *Note*: This returns a new instance of Application.
    pub fn new<T>(
        window_builder: winit::window::WindowBuilder,
        event_loop: &EventLoop<()>,
        asset_path: T,
    ) -> Self
    where
        T: Into<String>,
    {
        let window = window_builder.build(event_loop).unwrap();
        let size = window.inner_size();
        let surface = wgpu::Surface::create(&window);

        let renderer = futures::executor::block_on(Renderer::new(window, size, surface));

        let asset_manager = AssetManager::new(asset_path.into());

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
            current_scene: None,
            render_graph: None,
        }
    }

    /// Set's the current scene that harmony will use for rendering. Consider this a connivent place to store our specs world.
    /// # Arguments
    ///
    /// * `current_scene` - The current scene.
    ///
    /// *Note*: Once you've set the current scene you can access it using: `app.current_scene`.
    pub fn set_scene(&mut self, current_scene: Scene<'static>) {
        self.current_scene = Some(current_scene);
    }

    /// A function to help get the actual screen size as a LogicalSize<f32>
    pub fn get_window_actual_size(&self) -> winit::dpi::LogicalSize<f32> {
        let size = self.renderer.window.inner_size();
        winit::dpi::LogicalSize {
            width: size.width as f32,
            height: size.height as f32,
        }
    }

    /// Load's the entire application up. This also calls asset_manager.load and creates some default rendering pipelines.
    /// # Arguments
    ///
    /// * `app_state` - The app state you created which should implement the AppState trait.
    ///
    pub fn load<T>(&mut self, app_state: &mut T)
    where
        T: AppState,
    {
        self.asset_manager.load(
            &self.renderer.device,
            &mut self.renderer.queue,
            &mut self.console,
        );
        self.console.load(&self.asset_manager);

        super::graphics::material::HDRImage::create_cube_map(self);

        self.render_graph = Some(RenderGraph::new(self));

        app_state.load(self);

        // Once materials have been created we need to create more info for them.
        let materials: Vec<&mut super::graphics::material::Material> =
            self.asset_manager.materials.values_mut().collect();
        {
            let images = &self.asset_manager.images;
            for material in materials {
                match material {
                    super::graphics::material::Material::Unlit(unlit_material) => {
                        let local_bind_group_layout = &self
                            .render_graph
                            .as_ref()
                            .unwrap()
                            .get("unlit")
                            .pipeline
                            .bind_group_layouts[2];
                        unlit_material.create_bind_group(
                            images,
                            &self.renderer.device,
                            local_bind_group_layout,
                        );
                    }
                }
            }
        }

        let size = self.renderer.window.inner_size();

        // Start up gui after load..
        let gui_renderer = crate::gui::Renderer::new(
            &self.asset_manager,
            &mut self.renderer.device,
            wgpu::TextureFormat::Bgra8UnormSrgb,
            LogicalSize::new(size.width, size.height),
        );
        self.gui_renderer = Some(gui_renderer);
    }

    /// Run's the application which means two things.
    /// 1. Update all internal state and call app_state.update()
    /// 2. Draw all rendering data to the current screen and call app_state.update()
    /// 
    /// # Arguments
    ///
    /// * `app_state` - The app state you created which should implement the AppState trait.
    /// * `event` - The event data as a reference from winit. 
    /// * `control_flow` - a mutable reference to winit's control flow. 
    ///
    pub fn run<T>(
        &mut self,
        app_state: &mut T,
        event: &Event<'_, ()>,
        _control_flow: &mut ControlFlow, // TODO: Figure out if we actually will use this...
    ) where
        T: AppState,
    {
        self.input.update_events(event);
        let bounds = crate::gui::core::Rectangle {
            x: 0.0,
            y: 0.0,
            width: self.renderer.size.width as f32 / self.renderer.window.scale_factor() as f32, // TODO: Use window scale NOT 2.0.
            height: self.renderer.size.height as f32 / self.renderer.window.scale_factor() as f32, 
        };
        match event {
            Event::MainEventsCleared => {
                let mut frame_time = self.clock.elapsed().as_secs_f32() - self.elapsed_time;
                while frame_time > 0.0 {
                    self.delta_time = f32::min(frame_time, self.fixed_timestep);
                    
                    app_state.update(self);
                    let gui_scene = app_state.draw_gui(self);
                    if gui_scene.is_some() {
                        self.gui_renderables = gui_scene
                            .unwrap()
                            .get_components()
                            .iter()
                            .map(|component| component.draw(bounds))
                            .collect()
                    }
                    self.console.update(&self.input, self.delta_time);

                    if self.current_scene.is_some() {
                        self.current_scene.as_mut().unwrap().update(self.delta_time);
                    }

                    self.input.clear();
                    frame_time -= self.delta_time;
                    self.elapsed_time += self.delta_time;
                }

                self.renderer.window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                let start = Instant::now();
                let output = self.renderer.render();
                let mut command_buffers = Vec::new();

                // Render the graph.
                if self.render_graph.is_some() {
                    let render_graph = self.render_graph.as_mut().unwrap();
                    if self.current_scene.is_some() {
                        command_buffers.extend(render_graph.render(
                            &mut self.renderer,
                            &mut self.asset_manager,
                            &mut self.current_scene.as_mut().unwrap().world,
                            &output,
                        ));
                    }
                }

                // Gather console components
                let mut root_components: Vec<crate::gui::renderables::Renderable> = self
                    .console
                    .get_components()
                    .iter()
                    .map(|component| component.draw(bounds))
                    .collect();
                root_components.extend(self.gui_renderables.clone());

                let root = crate::gui::renderables::Renderable::Group {
                    bounds,
                    renderables: root_components,
                };

                let gui_renderer = self.gui_renderer.as_mut().unwrap();
                command_buffers.extend(gui_renderer.draw(
                    &mut self.renderer.device,
                    &output.view,
                    root,
                    Some(bounds),
                    self.renderer.window.scale_factor() as f32,
                    &mut self.asset_manager,
                ));

                app_state.draw(self);

                // Then we submit the work
                self.renderer.queue.submit(&command_buffers);

                std::thread::yield_now();

                self.frame_time =
                    Instant::now().duration_since(start).subsec_millis() as f32 / 1000.0;
            }
            _ => (),
        }
    }
}
