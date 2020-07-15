use shaderc;
use std::io::BufRead;
use std::path::PathBuf;
use std::sync::Arc;

pub struct Shader {
    pub fragment: wgpu::ShaderModule,
    pub vertex: wgpu::ShaderModule,
}

impl Shader {
    pub fn new<T: Into<PathBuf>>(device: Arc<wgpu::Device>, path: T) -> Arc<Self> {
        let path = path.into();
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        let path = path.parent().unwrap();

        // Compiler
        let mut compiler = shaderc::Compiler::new().unwrap();
        let mut options = shaderc::CompileOptions::new().unwrap();

        #[cfg(not(debug_assertions))]
        {
            options.set_optimization_level(shaderc::OptimizationLevel::Performance);
        }
        #[cfg(debug_assertions)]
        {
            options.set_optimization_level(shaderc::OptimizationLevel::Zero);
        }

        options.add_macro_definition("EP", Some("main"));
        options.set_include_callback(|file_path, _include_type, _, _| {
            let shader_path = path.clone().join(file_path);
            // let mut contents: String = "".into();
            let contents = std::fs::read_to_string(&shader_path).unwrap();
            Result::Ok(shaderc::ResolvedInclude {
                resolved_name: file_path.to_string(),
                content: contents,
            })
        });

        let shader_path = path.join(file_name);
        let file = std::fs::File::open(&shader_path).unwrap();

        let shader_file = std::io::BufReader::new(&file);
        let mut vert_file_name = String::new();
        let mut frag_file_name = String::new();
        let mut lines = shader_file.lines();

        while let Some(line) = lines.next() {
            let current_line = line.unwrap();
            if current_line.contains("frag") {
                frag_file_name = current_line;
            } else {
                vert_file_name = current_line;
            }
        }

        // Pixel
        let shader_path = path.join(frag_file_name.clone());
        let frag_contents = std::fs::read_to_string(&shader_path)
            .unwrap_or_else(|_| panic!("Unable to read the file: {}", frag_file_name));

        // Vertex
        let shader_path = path.join(vert_file_name.clone());
        let vert_contents = std::fs::read_to_string(&shader_path)
            .unwrap_or_else(|_| panic!("Unable to read the file: {}", vert_file_name));

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
            device.create_shader_module(wgpu::ShaderModuleSource::SpirV(&spirv.as_binary()))
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
            device.create_shader_module(wgpu::ShaderModuleSource::SpirV(spirv.as_binary()))
        };

        Arc::new(Shader { fragment, vertex })
    }
}

#[cfg(test)]
mod tests {
    use super::Shader;
    use std::sync::Arc;

    #[test]
    fn should_load_shader() {
        async_std::task::block_on(async {
            let (needed_features, unsafe_features) =
                (wgpu::Features::empty(), wgpu::UnsafeFeatures::disallow());

            let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
            let adapter = instance
                .request_adapter(
                    &wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::Default,
                        compatible_surface: None,
                    },
                    unsafe_features,
                )
                .await
                .unwrap();

            let adapter_features = adapter.features();
            let (device, _) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        features: adapter_features & needed_features,
                        limits: wgpu::Limits::default(),
                        shader_validation: true,
                    },
                    None,
                )
                .await
                .unwrap();

            let device = Arc::new(device);

            Shader::new(device, "./assets/core/shaders/pbr.shader");
        });
    }
}
