use std::{sync::Arc, time::Instant};
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
};

use legion::prelude::*;

use crate::{
    core::input::Input,
    graphics::{
        self,
        material::Skybox,
        RenderGraph, Renderer,
        resources::{CurrentRenderTarget, GPUResourceManager, ProbeManager},
        systems::create_render_schedule_builder, 
        pipeline_manager::PipelineManager
    },
    scene::Scene,
    AssetManager, TransformCount,
};
use graphics::{material::skybox::SkyboxType, pipelines::{UnlitPipelineDesc, LinePipelineDesc}};

pub trait AppState {
    /// Is called after the engine has loaded an assets.
    fn load(&mut self, _app: &mut Application) {}
    /// Called to update app state.
    fn update(&mut self, _app: &mut Application) {}

    /// Called when the window resizes
    fn resize(&mut self, _app: &mut Application) {}
}

pub struct Application {
    pub renderer: Renderer,
    clock: Instant,
    fixed_timestep: f32,
    elapsed_time: f32,
    pub frame_time: f32,
    pub delta_time: f32,
    pub current_scene: Scene,
    pub render_schedule: Schedule,
    pub resources: Resources,
    pub probe_manager: ProbeManager,
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
        mut render_systems: Vec<Box<dyn Schedulable>>,
    ) -> Self
    where
        T: Into<String>,
    {
        let scene = Scene::new(None, None);
        let window = window_builder.build(event_loop).unwrap();
        let size = window.inner_size();

        // Add resources
        let mut resources = Resources::default();
        resources.insert(crate::scene::resources::DeltaTime(0.05));
        resources.insert(PipelineManager::new());

        let renderer =
            futures::executor::block_on(Renderer::new(window, size, &mut resources));

        let asset_manager = AssetManager::new(asset_path.into());

        let mut render_schedule_builder = create_render_schedule_builder();
        render_schedule_builder = render_schedule_builder.add_system(crate::graphics::systems::mesh::create());

        for index in 0..render_systems.len() {
            let system = render_systems.remove(index);
            render_schedule_builder = render_schedule_builder.add_system(system);
        }

        let render_schedule = render_schedule_builder
            .flush()
            .add_thread_local_fn(graphics::systems::render::create())
            .build();
        resources.insert(asset_manager);

        resources.insert(TransformCount(0));
        resources.insert(CurrentRenderTarget(None));

        resources.insert(Input::new());

        Application {
            renderer,
            clock: Instant::now(),
            fixed_timestep: 1.0 / 60.0,
            elapsed_time: 0.0,
            frame_time: 0.0,
            delta_time: 0.0,
            current_scene: scene,
            resources,
            render_schedule,
            probe_manager: ProbeManager::new(),
        }
    }

    /// Set's the current scene that harmony will use for rendering.
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
        {
            let mut asset_manager = self.resources.get_mut::<AssetManager>().unwrap();
            let device = self.resources.get::<wgpu::Device>().unwrap();
            let mut queue = self.resources.get_mut::<wgpu::Queue>().unwrap();
            asset_manager.load(&device, &mut queue);
        }

        {
            let render_graph = RenderGraph::new(&mut self.resources, true);
            self.resources.insert(render_graph);
        }

        {
            let asset_manager = self.resources.get_mut::<AssetManager>().unwrap();
            let mut render_graph = self.resources.get_mut::<RenderGraph>().unwrap();
            let mut resource_manager = self.resources.get_mut::<GPUResourceManager>().unwrap();
            let device = self.resources.get::<wgpu::Device>().unwrap();
            let sc_desc = self.resources.get::<wgpu::SwapChainDescriptor>().unwrap();

            crate::graphics::pipelines::skybox::create(&self.resources);

            // Unlit pipeline
            let unlit_pipeline_desc = UnlitPipelineDesc::default();
            render_graph.add(
                &asset_manager,
                &device,
                &sc_desc,
                &mut resource_manager,
                "unlit",
                unlit_pipeline_desc,
                vec!["skybox"],
                true,
                None,
                false,
            );

            // PBR pipeline
            super::graphics::pipelines::pbr::create(&self.resources);

            // PBR pipeline
            let line_pipeline_desc = LinePipelineDesc::default();
            render_graph.add(
                &asset_manager,
                &device,
                &sc_desc,
                &mut resource_manager,
                "line",
                line_pipeline_desc,
                vec!["skybox"],
                false,
                None,
                false,
            );
        }

        app_state.load(self);

        // Once materials have been created we need to create more info for them.
        {
            let mut asset_manager = self.resources.get_mut::<AssetManager>().unwrap();
            let device = self.resources.get::<wgpu::Device>().unwrap();
            let mut resource_manager = self.resources.get_mut::<GPUResourceManager>().unwrap();
            asset_manager.load_materials(&device, &mut resource_manager);
        }
        
        {
            let resource_manager = self.resources.get_mut::<GPUResourceManager>().unwrap();
            let query = <(Write<Skybox>,)>::query();
            for (mut skybox,) in query.iter_mut(&mut self.current_scene.world) {
                if skybox.skybox_type == SkyboxType::HdrCubemap {
                    let device = self.resources.get::<wgpu::Device>().unwrap();
                    let material_layout = resource_manager.get_bind_group_layout("skybox_material").unwrap();
                    skybox.create_bind_group2(&device, material_layout);
                }
            }
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
        {
            let mut input = self.resources.get_mut::<Input>().unwrap();
            input.update_events(event);
        }

        match event {
            Event::MainEventsCleared => {
                let mut frame_time = self.clock.elapsed().as_secs_f32() - self.elapsed_time;

                while frame_time > 0.0 {
                    self.delta_time = f32::min(frame_time, self.fixed_timestep);

                    self.current_scene
                        .update(self.delta_time, &mut self.resources);

                    {
                        let mut input = self.resources.get_mut::<Input>().unwrap();
                        input.clear();
                    }
                    
                    frame_time -= self.delta_time;
                    self.elapsed_time += self.delta_time;
                }

                // Store current frame buffer.
                {
                    let output = Arc::new(self.renderer.render());
                    self.resources.insert(output);
                }

                // First update our probes if we need to.
                {
                    self.probe_manager.render(&mut self.resources, &mut self.current_scene);
                }

                // Next render's our scene.
                self.render_schedule.execute(&mut self.current_scene.world, &mut self.resources);

                // We need to let the swap drop so the frame renderers.
                let _swap_chain_output = self.resources.remove::<Arc<wgpu::SwapChainOutput>>().unwrap();

                self.renderer.window.request_redraw();
            }
            Event::WindowEvent {
                event: winit::event::WindowEvent::Resized(size),
                ..
            } => {
                {
                    let device = self.resources.get::<wgpu::Device>().unwrap();
                    let mut sc_desc = self
                        .resources
                        .get_mut::<wgpu::SwapChainDescriptor>()
                        .unwrap();

                    sc_desc.width = size.width;
                    sc_desc.height = size.height;
                    self.renderer.size = *size;
                    self.renderer.swap_chain =
                        device.create_swap_chain(&self.renderer.surface, &sc_desc);
                }
                app_state.resize(self);
            }
            _ => (),
        }
    }
}
