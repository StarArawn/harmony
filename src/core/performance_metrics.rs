use std::collections::HashMap;
use imgui::{im_str, Condition};
use winit::event::VirtualKeyCode;

// Just a simple place to store performance metrics..
pub struct PerformanceMetrics {
    pub(crate) data: HashMap<String, std::time::Duration>,
    pub(crate) visible: bool,
    pub(crate) last_open_key_pressed: bool,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            visible: false,
            last_open_key_pressed: false,
        }
    }


    pub fn insert<T: Into<String>>(&mut self, key: T, duration: std::time::Duration) {
        self.data.insert(key.into(), duration);
    }

    pub fn display(&mut self, ui: &mut imgui::Ui<'_>, input: &crate::core::input::Input) {
        let open_key_pressed = input.is_key_down(VirtualKeyCode::Grave);
        if  open_key_pressed && !self.last_open_key_pressed {
            self.visible = !self.visible;
        }
        self.last_open_key_pressed = open_key_pressed;
        
        if self.visible {
            let window = imgui::Window::new(im_str!("Performance Metrics"));
            window
                .scroll_bar(true)
                .resizable(false)
                .size([300.0, 150.0], Condition::Always)
                .position([0.0, 0.0], Condition::Always)
                .build(&ui, || {
                    for (key, duration) in self.data.iter() {
                        ui.text(im_str!("{:?}: {:?}", key, duration));
                    }
                });
        }
    }
}