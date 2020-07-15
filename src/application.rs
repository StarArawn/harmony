use std::{sync::Arc, time::Instant, path::PathBuf};
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
};

use imgui::*;
use legion::prelude::*;

use crate::{
    core::input::Input,
    graphics::{
        self,
        material::Skybox,
        pipeline_manager::PipelineManager,
        resources::{CurrentRenderTarget, GPUResourceManager, ProbeManager},
        systems::create_render_schedule_builder,
        RenderGraph, Renderer,
    },
    scene::Scene,
    AssetManager, TransformCount,
};
use graphics::{
    material::skybox::SkyboxType,
    // pipelines::{LinePipelineDesc, UnlitPipelineDesc},
    CommandBufferQueue, CommandQueueItem, renderer::{DEPTH_FORMAT, DepthTexture},
};
use nalgebra_glm::Vec2;

pub trait AppState {
    /// Is called after the engine has loaded an assets.
    fn load(&mut self, _app: &mut Application) {}
    /// Called to update app state.
    fn update(&mut self, _app: &mut Application) {}
    /// Called when the window resizes
    fn resize(&mut self, _app: &mut Application) {}
    /// Used to update your app state for the UI.
    // TODO: Maybe update should just be used instead.
    fn update_ui(&mut self, _app: &mut Application) {}
    /// A function to help draw your UI. PLease see hello-world for an example.
    fn draw_ui(&mut self, _ui: &mut imgui::Ui<'_>, _screen_size: Vec2) {}
}

pub struct Application {
    // TODO: Don't expose renderer outside of harmony?
    pub renderer: Renderer,
    clock: Instant,
    fixed_timestep: f32,
    elapsed_time: f32,
    /// Time last frame took.
    pub frame_time: f32,
    /// Current delta time.
    pub delta_time: f32,
    /// Current scene.
    pub current_scene: Scene,
    /// A legion schedule that contains the systems used to render.
    pub render_schedule: Schedule,
    /// Legion resources.
    pub resources: Resources,
    /// The probe manager.
    pub probe_manager: ProbeManager,
    pub(crate) imgui: imgui::Context,
    pub(crate) platform: imgui_winit_support::WinitPlatform,
    pub(crate) imgui_renderer: imgui_wgpu::Renderer,
    last_cursor: Option<imgui::MouseCursor>,
    last_frame: Instant,
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
        T: Into<PathBuf>,
    {
        let scene = Scene::new(None, None);
        let window = window_builder.build(event_loop).unwrap();
        let size = window.inner_size();

        // Add resources
        let mut resources = Resources::default();
        resources.insert(crate::scene::resources::DeltaTime(0.05));
        resources.insert(PipelineManager::new());

        let renderer = futures::executor::block_on(Renderer::new(window, size, &mut resources));

        let asset_manager = {
            let device = resources.get::<Arc<wgpu::Device>>().unwrap();
            let queue = resources.get::<Arc<wgpu::Queue>>().unwrap();
            AssetManager::new(asset_path.into(), device.clone(), queue.clone())
        };

        let mut render_schedule_builder = create_render_schedule_builder();
        render_schedule_builder =
            render_schedule_builder.add_system(crate::graphics::systems::mesh::create());

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

        let hidpi_factor = renderer.window.scale_factor();
        let mut imgui = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            &renderer.window,
            imgui_winit_support::HiDpiMode::Default,
        );
        imgui.set_ini_filename(None);

        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        let mut style = imgui.style_mut();
        let theme = super::core::Theme::default();
        theme.update_imgui(&mut style);

        // Fix incorrect colors with sRGB framebuffer
        fn imgui_gamma_to_linear(col: [f32; 4]) -> [f32; 4] {
            let x = col[0].powf(2.2);
            let y = col[1].powf(2.2);
            let z = col[2].powf(2.2);
            let w = 1.0 - (1.0 - col[3]).powf(2.2);
            [x, y, z, w]
        }

        for col in 0..style.colors.len() {
            style.colors[col] = imgui_gamma_to_linear(style.colors[col]);
        }

        let imgui_renderer = {
            let device = resources.get::<Arc<wgpu::Device>>().unwrap();
            let mut queue = resources.get_mut::<Arc<wgpu::Queue>>().unwrap();
            let sc_desc = resources.get::<wgpu::SwapChainDescriptor>().unwrap();
            imgui_wgpu::Renderer::new(&mut imgui, &device, &mut queue, sc_desc.format, None)
        };

        let last_frame = Instant::now();

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
            imgui,
            platform,
            imgui_renderer,
            last_frame,
            last_cursor: None,
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
            let render_graph = RenderGraph::new(&mut self.resources, true);
            self.resources.insert(render_graph);
        }

        {
            let asset_manager = self.resources.get_mut::<AssetManager>().unwrap();
            let mut render_graph = self.resources.get_mut::<RenderGraph>().unwrap();
            let mut resource_manager = self.resources.get_mut::<GPUResourceManager>().unwrap();
            let device = self.resources.get::<Arc<wgpu::Device>>().unwrap();
            let sc_desc = self.resources.get::<wgpu::SwapChainDescriptor>().unwrap();

            // Unlit pipeline
            // let unlit_pipeline_desc = UnlitPipelineDesc::default();
            // render_graph.add(
            //     &asset_manager,
            //     &device,
            //     &sc_desc,
            //     &mut resource_manager,
            //     "unlit",
            //     unlit_pipeline_desc,
            //     vec!["skybox"],
            //     true,
            //     None,
            //     false,
            // );

            // Line pipeline
            // let line_pipeline_desc = LinePipelineDesc::default();
            // render_graph.add(
            //     &asset_manager,
            //     &device,
            //     &sc_desc,
            //     &mut resource_manager,
            //     "line",
            //     line_pipeline_desc,
            //     vec!["skybox"],
            //     false,
            //     None,
            //     false,
            // );
        }

        // Global Node
        {
            let mut pipeline_manager = self.resources.get_mut::<PipelineManager>().unwrap();
            pipeline_manager.add_node("globals", vec![]);
        }

        // Create new pipelines
        crate::graphics::pipelines::skybox::create(&self.resources);
        crate::graphics::pipelines::realtime_sky::create(&self.resources);
        
        // PBR pipeline
        super::graphics::pipelines::pbr::create(&self.resources);

        {
            let mut asset_manager = self.resources.get_mut::<AssetManager>().unwrap();
            asset_manager.load();
        }

        // Run user code.
        app_state.load(self);

        {
            let resource_manager = self.resources.get_mut::<GPUResourceManager>().unwrap();
            let query = <(Write<Skybox>,)>::query();
            for (mut skybox,) in query.iter_mut(&mut self.current_scene.world) {
                if skybox.skybox_type == SkyboxType::HdrCubemap {
                    let device = self.resources.get::<Arc<wgpu::Device>>().unwrap();
                    let material_layout = resource_manager
                        .get_bind_group_layout("skybox_material")
                        .unwrap();
                    skybox.create_bind_group2(&device, material_layout);
                } else if skybox.skybox_type == SkyboxType::RealTime {
                    let device = self.resources.get::<Arc<wgpu::Device>>().unwrap();
                    let asset_manager = self.resources.get::<AssetManager>().unwrap();
                    let material_layout = resource_manager
                        .get_bind_group_layout("realtime_skybox_material")
                        .unwrap();
                    skybox.create_realtime_bind_group(&device, &asset_manager, material_layout);
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
                self.frame_time = frame_time * 1000.0;
                {
                    self.last_frame = self.imgui.io_mut().update_delta_time(self.last_frame);
                }

                while frame_time > 0.0 {
                    self.delta_time = f32::min(frame_time, self.fixed_timestep);

                    self.current_scene
                        .update(self.delta_time, &mut self.resources);

                    {
                        let mut input = self.resources.get_mut::<Input>().unwrap();
                        input.clear();
                    }

                    app_state.update_ui(self);

                    frame_time -= self.delta_time;
                    self.elapsed_time += self.delta_time;
                }

                self.platform
                    .prepare_frame(self.imgui.io_mut(), &self.renderer.window)
                    .expect("Failed to prepare frame");
                let mut ui = self.imgui.frame();

                // Store current frame buffer.
                {
                    let output = Arc::new(self.renderer.render().output);
                    self.resources.insert(output);
                }

                // First update our probes if we need to.
                {
                    self.probe_manager
                        .render(&mut self.resources, &mut self.current_scene);
                }

                // Allow user to render UI stuff.
                let scale = self.renderer.window.scale_factor() as f32;
                app_state.draw_ui(
                    &mut ui,
                    Vec2::new(
                        self.renderer.size.width as f32 / scale,
                        self.renderer.size.height as f32 / scale,
                    ),
                );

                // Draw UI.
                {
                    let device = self.resources.get::<Arc<wgpu::Device>>().unwrap();
                    let frame = self.resources.get::<Arc<wgpu::SwapChainTexture>>().unwrap();
                    let command_buffer_queue = self.resources.get::<CommandBufferQueue>().unwrap();
                    let mut encoder: wgpu::CommandEncoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("UI"),
                        });

                    if self.last_cursor != ui.mouse_cursor() {
                        self.last_cursor = ui.mouse_cursor();
                        self.platform.prepare_render(&ui, &self.renderer.window);
                    }

                    self.imgui_renderer
                        .render(ui.render(), &device, &mut encoder, &frame.view)
                        .expect("Rendering failed");

                    command_buffer_queue
                        .push(CommandQueueItem {
                            buffer: encoder.finish(),
                            name: "UI".to_string(),
                        })
                        .unwrap();
                }

                // Next render's our scene.
                self.render_schedule
                    .execute(&mut self.current_scene.world, &mut self.resources);

                // We need to let the swap drop so the frame renderers.
                let _swap_chain_output = self
                    .resources
                    .remove::<Arc<wgpu::SwapChainTexture>>()
                    .unwrap();

                self.renderer.window.request_redraw();
            }
            Event::WindowEvent {
                event: winit::event::WindowEvent::Resized(size),
                ..
            } => {
                {
                    let device = self.resources.get::<Arc<wgpu::Device>>().unwrap();
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

                // Resize depth buffer too
                let depth_texture = {
                    let device = self.resources.get::<Arc<wgpu::Device>>().unwrap();
                    device.create_texture(&wgpu::TextureDescriptor {
                        size: wgpu::Extent3d {
                            width: size.width,
                            height: size.height,
                            depth: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: DEPTH_FORMAT,
                        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                        label: None,
                    })
                };
                self.resources.insert(DepthTexture(depth_texture.create_default_view()));
                
                app_state.resize(self);
            }
            _ => (),
        }
        self.platform
            .handle_event(self.imgui.io_mut(), &self.renderer.window, &event);
    }
}
