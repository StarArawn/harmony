use walkdir::WalkDir;
use std::collections::HashMap;

use crate::graphics::material::Shader;

pub struct AssetManager {
    path: String,
    shaders: HashMap<String, Shader>,
}

impl AssetManager {
    pub fn new(path: String) -> Self {
        AssetManager {
            path,
            shaders: HashMap::new(),
        }
    }

    pub fn load(&mut self) {
        for entry in WalkDir::new(&self.path) {
            let entry = entry.expect("Error: Could not access file.");
            let file_name = entry.file_name().to_str().unwrap();
            let full_file_path = str::replace(entry.path().to_str().unwrap_or_else(|| panic!(format!("Error: could not get full file path: {}", file_name))), file_name, "");
            //let full_path = format!("{}{}", full_file_path, file_name);
            if file_name.ends_with(".shader") {
                let shader = Shader::new(full_file_path.to_string(), file_name.to_string());
                self.shaders.insert(file_name.to_string(), shader);
                println!("Compiled shader: {}", file_name);
            }
        }
    }

    pub fn get_shader(&self, key: String) -> &Shader
    {
        self.shaders.get(&key).expect(&format!("Asset Error: Could not find {} shader asset!", key))
    }
}