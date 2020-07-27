use shaderc;
use std::io::BufRead;
use std::path::PathBuf;
use std::{borrow::Cow, sync::Arc};

pub enum Shader {
    Core(CoreShader),
    Compute(ComputeShader),
}

pub struct CoreShader {
    pub fragment: wgpu::ShaderModule,
    pub vertex: wgpu::ShaderModule,
}

pub struct ComputeShader {
    pub compute: wgpu::ShaderModule,
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
        let mut comp_file_name = String::new();
        let mut lines = shader_file.lines();

        while let Some(line) = lines.next() {
            let current_line = line.unwrap();
            if current_line.contains("frag") {
                frag_file_name = current_line;
            } else if current_line.contains("vert") {
                vert_file_name = current_line;
            } else if current_line.contains("comp") {
                comp_file_name = current_line;
            }
        }

        // Pixel
        let shader_path = path.join(frag_file_name.clone());
        let frag_contents = std::fs::read_to_string(&shader_path);

        // Vertex
        let shader_path = path.join(vert_file_name.clone());
        let vert_contents = std::fs::read_to_string(&shader_path);

        // Vertex
        let shader_path = path.join(comp_file_name.clone());
        let comp_contents = std::fs::read_to_string(&shader_path);

        options.add_macro_definition("EP", Some("main"));

        let vertex = if vert_contents.is_ok() {
            let spirv = compiler
                .compile_into_spirv(
                    &vert_contents.unwrap(),
                    shaderc::ShaderKind::Vertex,
                    "vertex.glsl",
                    "main",
                    Some(&options),
                )
                .unwrap();
            Some(device.create_shader_module(wgpu::ShaderModuleSource::SpirV(Cow::Borrowed(spirv.as_binary()))))
        } else { None };

        let fragment = if frag_contents.is_ok() {
            let spirv = compiler
                .compile_into_spirv(
                    &frag_contents.unwrap(),
                    shaderc::ShaderKind::Fragment,
                    "pixel.glsl",
                    "main",
                    Some(&options),
                )
                .unwrap();
            Some(device.create_shader_module(wgpu::ShaderModuleSource::SpirV(Cow::Borrowed(spirv.as_binary()))))
        } else { None };

        let compute = if comp_contents.is_ok() {
            let spirv = compiler
                .compile_into_spirv(
                    &comp_contents.unwrap(),
                    shaderc::ShaderKind::Compute,
                    "compute.glsl",
                    "main",
                    Some(&options),
                )
                .unwrap();
            Some(device.create_shader_module(wgpu::ShaderModuleSource::SpirV(Cow::Borrowed(spirv.as_binary()))))
        } else { None };

        if fragment.is_some() && vertex.is_some() {
            return Arc::new(Shader::Core(CoreShader {
                fragment: fragment.unwrap(),
                vertex: vertex.unwrap()
            }));
        } else if compute.is_some() {
            return Arc::new(Shader::Compute(ComputeShader {
                compute: compute.unwrap(),
            }));
        } else {
            panic!("Couldn't figure out shader type!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Shader;
    use std::sync::Arc;

    #[test]
    fn should_load_shader() {
        async_std::task::block_on(async {
            let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
            let adapter = instance
                .request_adapter(
                    &wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::Default,
                        compatible_surface: None,
                    },
                )
                .await
                .unwrap();

            let adapter_features = adapter.features();
            let (device, _) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        features: adapter_features,
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
