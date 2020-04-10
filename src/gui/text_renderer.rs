use wgpu_glyph::{ GlyphBrush, GlyphBrushBuilder };
use ultraviolet::mat::Mat4;
use std::convert::TryInto;
use winit::dpi::LogicalSize;

pub struct TextRenderer {
    fonts: Vec<GlyphBrush<'static, ()>>
}

impl TextRenderer {
    pub fn new(
        device: &wgpu::Device,
        asset_manager: &crate::AssetManager,
    ) -> Self {

        let fonts = asset_manager.get_fonts();

        let mut brushes = Vec::new();

        for font in fonts.iter() {
            let glyph_brush = GlyphBrushBuilder::using_font_bytes(font.data.clone())
                .unwrap()
                .initial_cache_size((2048, 2048))
                .build(device, wgpu::TextureFormat::Bgra8UnormSrgb);
            brushes.push(glyph_brush);
        }
        
        Self {
            fonts: brushes,
        }
    }

    pub fn draw(
        &mut self,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        renderable: crate::gui::renderables::Text,
        transformation: Mat4,
        bounds: crate::gui::core::Rectangle<u32>,
        scale_factor: f32,
    ) {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[
                wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &target,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Load,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    },
                },
            ],
            depth_stencil_attachment: None,
        });

        for asset_font in self.fonts.iter_mut() { 
            let section = wgpu_glyph::Section {
                text: &*renderable.text,
                screen_position: (
                    (renderable.bounds.x * scale_factor).round(),
                    (renderable.bounds.y * scale_factor).round(),
                ),
                color: renderable.color.into_linear(),
                scale: wgpu_glyph::Scale {
                    x: renderable.size.clone() * scale_factor,
                    y: renderable.size.clone() * scale_factor
                },
                bounds: (
                    renderable.bounds.width as f32 * scale_factor,
                    renderable.bounds.height as f32 * scale_factor
                ),
                ..wgpu_glyph::Section::default()
            };
            asset_font.queue(section);

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

            asset_font.queue(wgpu_glyph::Section {
                text: "Hello wgpu_glyph!",
                screen_position: (30.0, 30.0),
                color: [0.0, 0.0, 0.0, 1.0],
                scale: wgpu_glyph::Scale { x: 40.0, y: 40.0 },
                bounds: (1024.0 * 2.0, 768.0 * 2.0),
                ..wgpu_glyph::Section::default()
            });

            // Draw the text!
            asset_font.draw_queued_with_transform_and_scissoring(
                    device,
                    encoder,
                    &target,
                    transformation.as_slice().try_into().unwrap(),
                    wgpu_glyph::Region {
                        x: 0,
                        y: 0,
                        width: 1024 * 2,
                        height: 768 * 2,
                    },
                )
                .expect("Draw queued");
        }
    }
}