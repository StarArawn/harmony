use nalgebra_glm::Mat4;
use std::convert::TryInto;
use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder};

pub struct TextRenderer {
    fonts: Vec<GlyphBrush<'static, ()>>,
}

impl TextRenderer {
    pub fn new(device: &wgpu::Device, asset_manager: &crate::AssetManager) -> Self {
        let fonts = asset_manager.get_fonts();

        let mut brushes = Vec::new();

        for font in fonts.iter() {
            let glyph_brush = GlyphBrushBuilder::using_font_bytes(font.data.clone())
                .unwrap()
                .initial_cache_size((2048, 2048))
                .build(device, wgpu::TextureFormat::Bgra8UnormSrgb);
            brushes.push(glyph_brush);
        }

        Self { fonts: brushes }
    }

    pub fn draw(
        &mut self,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        renderables: &Vec<crate::gui::renderables::Text>,
        transformation: Mat4,
        bounds: crate::gui::core::Rectangle<f32>,
        scale_factor: f32,
    ) {
        for asset_font in self.fonts.iter_mut() {
            for renderable in renderables.iter() {
                let section = wgpu_glyph::Section {
                    text: &*renderable.text,
                    screen_position: (
                        (renderable.bounds.x * scale_factor).round(),
                        (renderable.bounds.y * scale_factor).round(),
                    ),
                    color: renderable.color.into_linear(),
                    scale: wgpu_glyph::Scale {
                        x: renderable.size.clone() * scale_factor,
                        y: renderable.size.clone() * scale_factor,
                    },
                    bounds: (
                        renderable.bounds.width as f32 * scale_factor,
                        renderable.bounds.height as f32 * scale_factor,
                    ),
                    z: -1.0,
                    ..wgpu_glyph::Section::default()
                };
                asset_font.queue(section);
            }

            // Draw the text!
            // asset_font.draw_queued_with_transform_and_scissoring(
            //         device,
            //         encoder,
            //         &target,
            //         transformation.as_slice().try_into().unwrap(),
            //         wgpu_glyph::Region {
            //             x: bounds.x,
            //             y: bounds.y,
            //             width: bounds.width,
            //             height: bounds.height,
            //         },
            //     )
            //     .expect("Draw queued");

            // Draw the text!
            asset_font
                .draw_queued_with_transform_and_scissoring(
                    device,
                    encoder,
                    &target,
                    transformation.as_slice().try_into().unwrap(),
                    wgpu_glyph::Region {
                        x: bounds.x as u32,
                        y: bounds.y as u32,
                        width: bounds.width as u32,
                        height: bounds.height as u32,
                    },
                )
                .expect("Draw queued");
        }
    }
}
