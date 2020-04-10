use wgpu_glyph::{ GlyphBrush, GlyphBrushBuilder };

use std::io::prelude::*;
use std::fs::File;

pub struct Font{
    pub glyph_brush: GlyphBrush<'static, ()>,
    pub measure_brush: glyph_brush::GlyphBrush<'static, ()>,
}

impl Font {
    pub fn new(device: &wgpu::Device, font_path: String) -> Self {
        let mut file = File::open(&font_path).expect("Font: Unable to open the file");
        let mut font_contents: Vec<u8> = Vec::new();
        file.read_to_end(&mut font_contents).unwrap_or_else(|err| panic!("Unable to read the file: {} with error: {}", font_path, err));

        
        let glyph_brush = GlyphBrushBuilder::using_font_bytes(font_contents.clone())
            .unwrap()
            .initial_cache_size((2048, 2048))
            .build(device, wgpu::TextureFormat::Bgra8UnormSrgb);

        Self {
            glyph_brush,
            measure_brush: glyph_brush::GlyphBrushBuilder::using_font_bytes(font_contents.clone()).build()
        }
    }
}