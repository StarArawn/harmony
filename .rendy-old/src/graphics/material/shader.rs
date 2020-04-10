use rendy::{
    shader::{PathBufShaderInfo, ShaderKind, SourceLanguage, SpirvShader},
};

use rendy::shader::SpirvReflection;

use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;


pub struct Shader {
    pub builder: rendy::shader::ShaderSetBuilder,
    pub reflection: SpirvReflection,    
}

impl Shader {
    pub fn new(path: String, file_name: String) -> Self {
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
        // let mut file = File::open(&shader_path).unwrap_or_else(|_| panic!("Unable to open the file: {}", shader_path));
        // let mut frag_contents = String::new();
        // file.read_to_string(&mut frag_contents).unwrap_or_else(|_| panic!("Unable to read the file: {}", shader_path));
        
        let frag: SpirvShader = PathBufShaderInfo::new(
            std::path::PathBuf::from(shader_path),
            ShaderKind::Fragment,
            SourceLanguage::GLSL,
            "main",
        ).precompile().unwrap();
    
        // Vertex
        let shader_path = format!("{}/{}", path, vert_file_name);
        // file = File::open(&shader_path).unwrap_or_else(|_| panic!("Unable to read the file: {}", shader_path));
        // let mut vert_contents = String::new();
        // file.read_to_string(&mut vert_contents).unwrap_or_else(|_| panic!("Unable to read the file: {}", shader_path));
        
        let vertex: SpirvShader = PathBufShaderInfo::new(
            std::path::PathBuf::from(shader_path),
            ShaderKind::Vertex,
            SourceLanguage::GLSL,
            "main",
        ).precompile().unwrap();
        
        let builder: rendy::shader::ShaderSetBuilder = rendy::shader::ShaderSetBuilder::default()
            .with_vertex(&vertex).unwrap()
            .with_fragment(&frag).unwrap();

        let reflection = builder.reflect().unwrap();
        Shader {
            builder,
            reflection,
        }
    }
}