use shaderc;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufRead;
use std::io::BufReader;

pub struct Shader {
    pub fragment: wgpu::ShaderModule,
    pub vertex: wgpu::ShaderModule,
}

impl Shader {
    pub fn new(device: &wgpu::Device, path: String, file_name: String) -> Self {
        // Compiler
        let mut compiler = shaderc::Compiler::new().unwrap();
        let mut options = shaderc::CompileOptions::new().unwrap();
        options.add_macro_definition("EP", Some("main"));

        let shader_path = format!("{}{}", path, file_name);
        let file = File::open(&shader_path).expect("Shader: Unable to open the file");

        let shader_file = BufReader::new(&file);
        let mut vert_file_name = String::new();
        let mut frag_file_name = String::new();
        for (_num, line) in shader_file.lines().enumerate() {
            let current_line = line.unwrap();
            if current_line.contains("frag") {
                frag_file_name = current_line;
            } else {
                vert_file_name = current_line;
            }
        }

        // Pixel
        let shader_path = format!("{}{}", path, frag_file_name);
        let mut file = File::open(&shader_path)
            .unwrap_or_else(|_| panic!("Unable to open the file: {}", shader_path));
        let mut frag_contents = String::new();
        file.read_to_string(&mut frag_contents)
            .unwrap_or_else(|_| panic!("Unable to read the file: {}", shader_path));

        // Vertex
        let shader_path = format!("{}/{}", path, vert_file_name);
        file = File::open(&shader_path)
            .unwrap_or_else(|_| panic!("Unable to read the file: {}", shader_path));
        let mut vert_contents = String::new();
        file.read_to_string(&mut vert_contents)
            .unwrap_or_else(|_| panic!("Unable to read the file: {}", shader_path));

        options.add_macro_definition("EP", Some("main"));

        let vertex = {
            let spirv = compiler
                .compile_into_spirv(
                    &vert_contents,
                    shaderc::ShaderKind::Vertex,
                    "vertex.glsl",
                    "main",
                    Some(&options),
                )
                .unwrap();
            device.create_shader_module(&spirv.as_binary())
        };

        let fragment = {
            let spirv = compiler
                .compile_into_spirv(
                    &frag_contents,
                    shaderc::ShaderKind::Fragment,
                    "pixel.glsl",
                    "main",
                    Some(&options),
                )
                .unwrap();
            device.create_shader_module(spirv.as_binary())
        };

        Shader { fragment, vertex }
    }
}
