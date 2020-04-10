use crate::gui::Scene;

use iced_wgpu::{
    wgpu, Primitive, Renderer, Settings, Target, Viewport
};
use iced_winit::{winit, Cache, Clipboard, MouseCursor, Size, UserInterface};

pub struct Gui {
    cache: Option<Cache>,
    clipboard: Option<Clipboard>,
    pub events: Vec<iced_native::Event>,
    pub renderer: Renderer,
    scenes: Vec<Box<dyn Scene>>,
    outputs: Vec<(Primitive, MouseCursor)>,
}

impl Gui {
    pub fn new(device: &mut wgpu::Device, window: &winit::window::Window) -> Self {
        // Initialize iced
        let events: Vec<iced_native::Event> = Vec::new();
        let cache = Some(Cache::default());
        let renderer = Renderer::new(device, Settings::default());
        let clipboard = Clipboard::new(window);

        Self {
            cache,
            clipboard,
            events,
            renderer,
            scenes: Vec::<Box<dyn Scene>>::new(),
            outputs: Vec::new(),
        }
    }


    pub fn add_scene<T: Scene + 'static>(&mut self, scene: T, size: winit::dpi::LogicalSize<f32>) {
        let scene_index = self.scenes.len();
        self.scenes.push(Box::new(scene));
        self.build_scenes(size, scene_index);
    }

    fn build_scenes(&mut self, size: winit::dpi::LogicalSize<f32>, scene_index: usize) {
        let scene = self.scenes[scene_index].as_mut();
        // Build our user interface.
        let user_interface = UserInterface::build(
            scene.view(),
            Size::new(size.width as f32, size.height as f32),
            self.cache.take().unwrap(),
            &mut self.renderer,
        );

        // Finally, we just need to draw a new output for our renderer,
        let output = user_interface.draw(&mut self.renderer);

        if self.outputs.len() > scene_index {
            self.outputs[scene_index] = output;
        } else {
            self.outputs.push(output);
        }

        // update our cache,
        self.cache = Some(user_interface.into_cache());
    }

    pub fn update(&mut self, size: winit::dpi::LogicalSize<f32>) {
        // If no relevant events happened, we can simply skip this
        if self.events.is_empty() {
            return;
        }

        // We need to:
        // 1. Process events of our user interface.
        // 2. Update state as a result of any interaction.
        // 3. Generate a new output for our renderer.

        // First, we build our user interface.
        let scene_index = 0;
        for scene in self.scenes.iter_mut() {
            let mut user_interface = UserInterface::build(
                scene.view(),
                Size::new(size.width as f32, size.height as f32),
                self.cache.take().unwrap(),
                &mut self.renderer,
            );

            // Then, we process the events, obtaining messages in return.
            let messages = user_interface.update(
                self.events.drain(..),
                self.clipboard.as_ref().map(|c| c as _),
                &self.renderer,
            );

            let user_interface = if messages.is_empty() {
                // If there are no messages, no interactions we care about have
                // happened. We can simply leave our user interface as it is.
                user_interface
            } else {
                // If there are messages, we need to update our state
                // accordingly and rebuild our user interface.
                // We can only do this if we drop our user interface first
                // by turning it into its cache.
                self.cache = Some(user_interface.into_cache());

                // In this example, `Controls` is the only part that cares
                // about messages, so updating our state is pretty
                // straightforward.
                for message in messages {
                    scene.update(message);
                }

                // Once the state has been changed, we rebuild our updated
                // user interface.
                UserInterface::build(
                    scene.view(),
                    Size::new(size.width as f32, size.height as f32),
                    self.cache.take().unwrap(),
                    &mut self.renderer,
                )
            };
            // Finally, we just need to draw a new output for our renderer,
            let output = user_interface.draw(&mut self.renderer);

            if self.outputs.len() > scene_index {
                self.outputs[scene_index] = output;
            } else {
                self.outputs.push(output);
            }

            // update our cache,
            self.cache = Some(user_interface.into_cache());
        }
    }

    pub fn draw(&mut self, device: &mut wgpu::Device, queue: &mut wgpu::Queue, size: winit::dpi::PhysicalSize<u32>, window: &winit::window::Window, view: &wgpu::TextureView) -> MouseCursor {
        let mut encoder = device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { todo: 0 },
        );

        let viewport = Viewport::new(size.width as u32, size.height as u32);

        let mut mouse_cursor: MouseCursor = MouseCursor::default();
        for output in self.outputs.iter() {
            mouse_cursor = self.renderer.draw(
                device,
                &mut encoder,
                Target {
                    texture: view,
                    viewport: &viewport,
                },
                &output,
                window.scale_factor(),
                &[""]
            );
        }

        // Then we submit the work
        queue.submit(&[encoder.finish()]);

        mouse_cursor
    }
}