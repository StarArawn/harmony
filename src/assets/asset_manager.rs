use walkdir::WalkDir;
use std::collections::HashMap;

use crate::graphics::material::Shader;
use crate::gui::core::Font;

pub struct AssetManager {
    path: String,
    shaders: HashMap<String, Shader>,
    fonts: HashMap<String, Font>,
}

impl AssetManager {
    pub fn new(path: String) -> Self {
        AssetManager {
            path,
            shaders: HashMap::new(),
            fonts: HashMap::new(),
        }
    }

    pub fn load(&mut self, device: &wgpu::Device, console: &mut crate::gui::components::default::Console) {
        for entry in WalkDir::new(&self.path) {
            let entry = entry.expect("Error: Could not access file.");
            let file_name = entry.file_name().to_str().unwrap();
            let full_file_path = str::replace(entry.path().to_str().unwrap_or_else(|| panic!(format!("Error: could not get full file path: {}", file_name))), file_name, "");
            //let full_path = format!("{}{}", full_file_path, file_name);
            if file_name.ends_with(".shader") {
                let shader = Shader::new(device, full_file_path.to_string(), file_name.to_string());
                self.shaders.insert(file_name.to_string(), shader);
                console.info(crate::gui::components::default::ModuleType::Asset, format!("Compiled shader: {}", file_name));
            }
            if file_name.ends_with(".ttf") || file_name.ends_with(".otf") {
                let font = Font::new(device, format!("{}{}", full_file_path, file_name).to_string());
                self.fonts.insert(file_name.to_string(), font);
                console.info(crate::gui::components::default::ModuleType::Asset, format!("Loaded font: {}", file_name));
            }
        }
    }

    pub fn get_shader(&self, key: String) -> &Shader
    {
        self.shaders.get(&key).expect(&format!("Asset Error: Could not find {} shader asset!", key))
    }

    pub fn get_font(&self, key: String) -> &Font {
        self.fonts.get(&key).expect(&format!("Asset Error: Could not find {} font asset!", key))
    }

    pub fn get_font_mut(&mut self, key: String) -> &mut Font {
        self.fonts.get_mut(&key).expect(&format!("Asset Error: Could not find {} font asset!", key))
    }

    pub fn get_fonts(&self) -> Vec<&Font> {
        self.fonts.values().collect()
    }
}