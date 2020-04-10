use crate::AssetManager;
use crate::gui::QuadRenderer;
use crate::gui::TextRenderer;
use crate::gui::renderables;
use crate::gui::core::{ Background, Rectangle, Viewport };

struct Layer {
    bounds: Rectangle<u32>,
    quads: Vec<renderables::Quad>,
}

impl Layer {
    pub fn new(bounds: Rectangle<u32>) -> Self {
        Self {
            bounds,
            quads: Vec::new(),
        }
    }
}


pub struct Renderer {
    quad_renderer: QuadRenderer,
    text_renderer: TextRenderer,
    viewport: Viewport,
}

impl Renderer {
    pub fn new(
        asset_mananger: &AssetManager,
        device: &mut wgpu::Device,
        format: wgpu::TextureFormat,
        size: winit::dpi::LogicalSize<u32>,
    ) -> Self {

        Self {
            quad_renderer: QuadRenderer::new(asset_mananger, device, format),
            text_renderer: TextRenderer::new(device, asset_mananger),
            viewport: Viewport::new(size.width, size.height),
        }
    }

    pub fn draw(
        &mut self,
        device: &mut wgpu::Device,
        queue: &mut wgpu::Queue,
        target: &wgpu::TextureView,
        renderable: renderables::Renderable,
        bounds: Option<Rectangle>,
        scale_factor: f32,
        asset_manager: &mut AssetManager,
    ) {
        let mut encoder = device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None },
        );

        let (width, height) = self.viewport.dimensions();
        let transformation = self.viewport.transformation();

        let mut layers = Vec::new();
        layers.push(Layer::new(Rectangle {
            x: 0,
            y: 0,
            width,
            height,
        }));

        let parent_bounds = bounds.unwrap_or(Default::default());

        match renderable {
            renderables::Renderable::Group { bounds, renderables } => {
                for grouped_renderable in renderables {
                    self.draw(device, queue, target, grouped_renderable, Some(bounds), scale_factor, asset_manager)
                }
            },
            renderables::Renderable::Quad { bounds, background, border_radius, border_width, border_color } => {
                let layer = layers.last_mut().unwrap();
                
                // TODO: Move some of these computations to the GPU (?)
                let quad = renderables::Quad {
                    position: [
                        parent_bounds.x + bounds.x,
                        parent_bounds.y + bounds.y,
                    ],
                    scale: [bounds.width, bounds.height],
                    color: match background {
                        Background::Color(color) => color.into_linear(),
                    },
                    border_radius: border_radius as f32,
                    border_width: border_width as f32,
                    border_color: border_color.into_linear(),
                };
                layer.quads.push(quad);
            },
            renderables::Renderable::Text(text) => {
                self.text_renderer.draw(
                    device,
                    &mut encoder,
                    target,
                    text,
                    transformation,
                    parent_bounds.into(),
                    scale_factor,
                );
            }
            _ => {}
        }

        for layer in layers {
            let bounds = layer.bounds * scale_factor;
            
            if !layer.quads.is_empty() {
                self.quad_renderer.draw(
                    device,
                    &mut encoder,
                    &layer.quads,
                    transformation,
                    scale_factor,
                    bounds,
                    target,
                );
            }
        }

        // Then we submit the work
        queue.submit(&[encoder.finish()]);
    }
}