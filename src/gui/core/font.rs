use std::io::prelude::*;
use std::fs::File;

pub struct Font{
    pub data: Vec<u8>,
}

impl Font {
    pub fn new(_device: &wgpu::Device, font_path: String) -> Self {
        let mut file = File::open(&font_path).expect("Font: Unable to open the file");
        let mut font_contents: Vec<u8> = Vec::new();
        file.read_to_end(&mut font_contents).unwrap_or_else(|err| panic!("Unable to read the file: {} with error: {}", font_path, err));

        Self {
            data: font_contents,
        }
    }
}