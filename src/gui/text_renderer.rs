use ultraviolet::mat::Mat4;
use std::convert::TryInto;
use winit::dpi::LogicalSize;

pub struct TextRenderer {
}

impl TextRenderer {
    pub fn new(
    ) -> Self {
        
        Self {
        }
    }

    pub fn draw(
        &self,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        asset_manager: &mut crate::AssetManager,
        target: &wgpu::TextureView,
        renderable: crate::gui::renderables::Text,
        transformation: Mat4,
        bounds: crate::gui::core::Rectangle<u32>,
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

        let mut fonts = asset_manager.get_fonts_mut();

        for asset_font in fonts.iter_mut() { 
            let section = wgpu_glyph::Section {
                text: &*renderable.text,
                screen_position: (renderable.bounds.x, renderable.bounds.y),
                color: renderable.color.into_linear(),
                scale: wgpu_glyph::Scale { x: renderable.size.clone(), y: renderable.size.clone() },
                bounds: (renderable.bounds.width as f32, renderable.bounds.height as f32),
                ..wgpu_glyph::Section::default()
            };
            dbg!(section);
            asset_font.glyph_brush.queue(section);

            // Draw the text!
            asset_font.glyph_brush
                .draw_queued_with_transform_and_scissoring(
                    device,
                    encoder,
                    &target,
                    transformation.as_slice().try_into().unwrap(),
                    wgpu_glyph::Region {
                        x: bounds.x,
                        y: bounds.y,
                        width: bounds.width,
                        height: bounds.height,
                    },
                )
                .expect("Draw queued");
        }
    }
}