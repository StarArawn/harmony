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

fn map_mouse_button(button: winit::event::MouseButton) -> Option<MouseButton> {
    match button {
        winit::event::MouseButton::Left => Some(MouseButton::Left),
        winit::event::MouseButton::Right => Some(MouseButton::Right),
        winit::event::MouseButton::Middle => Some(MouseButton::Middle),
        winit::event::MouseButton::Other(8) => Some(MouseButton::X1),
        winit::event::MouseButton::Other(9) => Some(MouseButton::X2),
        _ => None
    }
}

#[derive(Debug)]
pub struct Input {
    keys_down: HashSet<VirtualKeyCode>,
    keys_pressed: HashSet<VirtualKeyCode>,
    keys_released: HashSet<VirtualKeyCode>,

    mouse_buttons_down: HashSet<MouseButton>,
    mouse_buttons_pressed: HashSet<MouseButton>,
    mouse_buttons_released: HashSet<MouseButton>,
    pub mouse_position: Vec2,
    pub mouse_delta: Vec2,
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
            mouse_delta: Vec2::zeros(),
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

    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons_down.contains(&button)
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }

    pub fn is_mouse_button_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons_released.contains(&button)
    }

    pub(crate) fn update_events(&mut self, winit_event: &winit::event::Event<'_, ()>) {
        match winit_event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == winit::event::ElementState::Pressed {
                        if input.virtual_keycode.is_some() {
                            self.keys_down.insert(input.virtual_keycode.unwrap());
                            self.keys_pressed.insert(input.virtual_keycode.unwrap());
                        }
                    } else if input.state == winit::event::ElementState::Released {
                        if input.virtual_keycode.is_some() {
                            self.keys_down.remove(&input.virtual_keycode.unwrap());
                            self.keys_released.insert(input.virtual_keycode.unwrap());
                        }
                    }
                },
                winit::event::WindowEvent::MouseInput { device_id: _, state, button, ..} => {
                    if let Some(mouse_button) = map_mouse_button(*button) {
                        if *state == winit::event::ElementState::Pressed {
                            self.mouse_buttons_down.insert(mouse_button);
                            self.mouse_buttons_pressed.insert(mouse_button);
                        }
                        else if *state == winit::event::ElementState::Released {
                            self.mouse_buttons_down.remove(&mouse_button);
                            self.mouse_buttons_released.insert(mouse_button);
                        }
                    }
                }
                winit::event::WindowEvent::CursorMoved {
                    position,
                    ..
                } => {
                    self.mouse_position = Vec2::new(position.x as f32, position.y as f32);
                },
                _ => (),
            },
            winit::event::Event::DeviceEvent { event, .. } => match event {
                winit::event::DeviceEvent::MouseMotion { delta } => {
                    self.mouse_delta = Vec2::new(delta.0 as f32, delta.1 as f32);
                },
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
        self.mouse_delta = Vec2::zeros();
    }
}
