use std::{sync::Arc, time::Instant};
use winit::{
    dpi::LogicalSize,
    event::Event,
    event_loop::{ControlFlow, EventLoop},
};

use legion::prelude::*;

use crate::{
    core::input::Input,
    graphics::{
        pipelines::{PBRPipelineDesc, SkyboxPipelineDesc, UnlitPipelineDesc},
        RenderGraph, Renderer, material::Skybox, self
    },
    scene::Scene,
    AssetManager,
};

pub trait AppState {
    /// Is called after the engine has loaded an assets.
    fn load(&mut self, _app: &mut Application) {}
    /// Called to update app state.
    fn update(&mut self, _app: &mut Application) {}

    /// Called when the window resizes
    fn resize(&mut self, _app: &mut Application) {}

    /// Draw's the gui.
    /// Return your current gui scene here or none if you don't have any.
    fn draw_gui(&mut self, _app: &mut Application) -> Option<&dyn crate::gui::Scene> {
        None
    }
}

pub struct Application {
    pub renderer: Renderer,
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
    pub current_scene: Scene,
    pub render_schedule: Schedule,
    pub resources: Resources,
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
        let scene = Scene::new(None, None);
        let window = window_builder.build(event_loop).unwrap();
        let size = window.inner_size();
        let surface = wgpu::Surface::create(&window);

        // Add resources
        let mut resources = Resources::default();
        resources.insert(crate::scene::resources::DeltaTime(0.05));

        let renderer = futures::executor::block_on(Renderer::new(window, size, surface, &mut resources));

        let asset_manager = AssetManager::new(asset_path.into());

        let console = crate::gui::components::default::Console::new();

        let render_schedule = Schedule::builder()
            .add_system(graphics::systems::skybox::create())
            .flush()
            .add_thread_local_fn(graphics::systems::render::create())
            .build();
        
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
            current_scene: scene,
            resources,
            render_schedule,
        }
    }

    /// Set's the current scene that harmony will use for rendering. Consider this a connivent place to store our specs world.
    /// # Arguments
    ///
    /// * `current_scene` - The current scene.
    ///
    /// *Note*: Once you've set the current scene you can access it using: `app.current_scene`.
    pub fn set_scene(&mut self, current_scene: Scene) {
        self.current_scene = current_scene;
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
            &self.resources,
            &mut self.console,
        );
        self.console.load(&self.asset_manager);

        {
            let render_graph = RenderGraph::new(&mut self.resources, true);
            self.resources.insert(render_graph);
        }
        
        {
            let mut render_graph = self.resources.get_mut::<RenderGraph>().unwrap();
            let device = self.resources.get::<wgpu::Device>().unwrap();
            let sc_desc = self.resources.get::<wgpu::SwapChainDescriptor>().unwrap();
            // Skybox pipeline
            let skybox_pipeline_desc = SkyboxPipelineDesc::default();
            render_graph.add(
                &self.asset_manager,
                &device,
                &sc_desc,
                "skybox",
                skybox_pipeline_desc,
                vec![],
                false,
                None,
                false,
            );
            // Unlit pipeline
            let unlit_pipeline_desc = UnlitPipelineDesc::default();
            render_graph.add(
                &self.asset_manager,
                &device,
                &sc_desc,
                "unlit",
                unlit_pipeline_desc,
                vec!["skybox"],
                true,
                None,
                false,
            );
            // PBR pipeline
            let pbr_pipeline_desc = PBRPipelineDesc::default();
            render_graph.add(
                &self.asset_manager,
                &device,
                &sc_desc,
                "pbr",
                pbr_pipeline_desc,
                vec!["skybox"],
                true,
                None,
                false,
            );
        }

        app_state.load(self);

        // Once materials have been created we need to create more info for them.
        let materials: Vec<&mut super::graphics::material::Material> =
            self.asset_manager.materials.values_mut().collect();
        {
            let mut render_graph = self.resources.get_mut::<RenderGraph>().unwrap();
            let device = self.resources.get::<wgpu::Device>().unwrap();
            
            let mut current_bind_group = None;
            let current_index = 0;
            for material in materials {
                match material {
                    super::graphics::material::Material::Unlit(unlit_material) => {
                        let unlit_bind_group_layout = &render_graph
                            .get("unlit")
                            .pipeline
                            .bind_group_layouts[1];
                        unlit_material.create_bind_group(
                            &self.asset_manager.images,
                            &device,
                            unlit_bind_group_layout,
                        );
                    }
                    super::graphics::material::Material::PBR(_pbr_material) => {
                        // let pbr_bind_group_layouts = &render_graph.get("pbr").pipeline.bind_group_layouts;
                        // current_bind_group = Some(pbr_material.create_bind_group(
                        //         &self.asset_manager.images,
                        //         &self.renderer.device,
                        //         pbr_bind_group_layouts,
                        //     ));
                        // current_index = pbr_material.index;
                    }
                }
                if current_bind_group.is_some() {
                    render_graph.binding_manager.add_multi_resource("pbr", current_bind_group.take().unwrap(), current_index);
                }

                current_bind_group = None;
            }
        }

        {
            let render_graph = self.resources.get_mut::<RenderGraph>().unwrap();
            let skybox_pipeline = render_graph.get("skybox");
            let material_layout = &skybox_pipeline.pipeline.bind_group_layouts[1];
            let query = <(Write<Skybox>,)>::query();
            for (mut skybox, ) in query.iter_mut(&mut self.current_scene.world) {
                let device = self.resources.get::<wgpu::Device>().unwrap();
                skybox.create_bind_group2(&device, material_layout);
                
                // let (pbr_node_name, bound_group) = {
                //     let pbr_node = render_graph.nodes.get_mut("pbr").unwrap();
                //     let bound_group = skybox.create_bind_group(&self.asset_manager, &device, &pbr_node.pipeline.bind_group_layouts);
                //     (pbr_node.name.clone(), bound_group)
                // };
                //render_graph.binding_manager.add_single_resource(pbr_node_name, bound_group);
            }
        }

        let size = self.renderer.window.inner_size();

        // Start up gui after load..
        {
            let device = self.resources.get::<wgpu::Device>().unwrap();
            let gui_renderer = crate::gui::Renderer::new(
                &self.asset_manager,
                &device,
                wgpu::TextureFormat::Bgra8UnormSrgb,
                LogicalSize::new(size.width, size.height),
            );
            self.gui_renderer = Some(gui_renderer);
        }
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
            width: self.renderer.size.width as f32 / self.renderer.window.scale_factor() as f32,
            height: self.renderer.size.height as f32 / self.renderer.window.scale_factor() as f32,
        };
        match event {
            Event::MainEventsCleared => {
                let mut frame_time = self.clock.elapsed().as_secs_f32() - self.elapsed_time;

                while frame_time > 0.0 {
                    self.delta_time = f32::min(frame_time, self.fixed_timestep);
                    
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

                    self.current_scene.update(self.delta_time, &mut self.resources);

                    self.input.clear();
                    frame_time -= self.delta_time;
                    self.elapsed_time += self.delta_time;
                }

                // Store current frame buffer.
                {
                    let output = Arc::new(self.renderer.render());
                    self.resources.insert(output);
                }

                // Render's the scene.
                self.render_schedule.execute(&mut self.current_scene.world, &mut self.resources);

                self.renderer.window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                let start = Instant::now();
                // let output = self.renderer.render();
                // let mut command_buffers = Vec::new();

                // Gather console components
                // let mut root_components: Vec<crate::gui::renderables::Renderable> = self
                //     .console
                //     .get_components()
                //     .iter()
                //     .map(|component| component.draw(bounds))
                //     .collect();
                // root_components.extend(self.gui_renderables.clone());

                // let root = crate::gui::renderables::Renderable::Group {
                //     bounds,
                //     renderables: root_components,
                // };

                // let gui_renderer = self.gui_renderer.as_mut().unwrap();
                // let device = self.current_scene.resources.get_mut::<wgpu::Device>().unwrap();
                // command_buffers.extend(gui_renderer.draw(
                //     &device,
                //     &output.view,
                //     root,
                //     Some(bounds),
                //     self.renderer.window.scale_factor() as f32,
                //     &mut self.asset_manager,
                // ));

                std::thread::yield_now();

                self.frame_time =
                    Instant::now().duration_since(start).subsec_millis() as f32 / 1000.0;
            }
            Event::WindowEvent {
                event: winit::event::WindowEvent::Resized(size),
                ..
            } => {
                {
                    let device = self.resources.get::<wgpu::Device>().unwrap();
                    let mut sc_desc = self.resources.get_mut::<wgpu::SwapChainDescriptor>().unwrap();

                    sc_desc.width = size.width;
                    sc_desc.height = size.height;
                    self.renderer.size = *size;
                    self.renderer.swap_chain = device
                        .create_swap_chain(&self.renderer.surface, &sc_desc);
                }
                app_state.resize(self);
            }
            _ => (),
        }
    }
}
