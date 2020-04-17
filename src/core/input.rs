use nalgebra_glm::Vec2;
use std::collections::HashSet;
use winit::event::VirtualKeyCode;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
#[allow(missing_docs)]
/// A button on a mouse.
pub enum MouseButton {
    Left,
    Middle,
    Right,
    X1,
    X2,
}

#[derive(Debug)]
pub struct Input {
    keys_down: HashSet<VirtualKeyCode>,
    keys_pressed: HashSet<VirtualKeyCode>,
    keys_released: HashSet<VirtualKeyCode>,

    mouse_buttons_down: HashSet<MouseButton>,
    mouse_buttons_pressed: HashSet<MouseButton>,
    mouse_buttons_released: HashSet<MouseButton>,
    mouse_position: Vec2,
    mouse_wheel_movement: Vec2,
    // current_text_input: Option<String>,

    // pads: Vec<Option<GamepadState>>,
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),

            mouse_buttons_down: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            mouse_buttons_released: HashSet::new(),
            mouse_position: Vec2::zeros(),
            mouse_wheel_movement: Vec2::zeros(),
            // current_text_input: None,

            // pads: Vec::new(),
        }
    }

    pub fn is_key_down(&self, key: VirtualKeyCode) -> bool {
        self.keys_down.contains(&key)
    }

    pub fn is_key_pressed(&self, key: VirtualKeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn is_key_released(&self, key: VirtualKeyCode) -> bool {
        self.keys_released.contains(&key)
    }

    pub(crate) fn update_events(&mut self, event: &winit::event::Event<'_, ()>) {
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == winit::event::ElementState::Pressed {
                        self.keys_pressed.insert(input.virtual_keycode.unwrap());
                    } else if input.state == winit::event::ElementState::Released {
                        self.keys_released.insert(input.virtual_keycode.unwrap());
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_buttons_pressed.clear();
        self.mouse_buttons_released.clear();
        self.mouse_wheel_movement = Vec2::zeros();
    }
}
