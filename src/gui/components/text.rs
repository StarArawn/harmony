use crate::gui::components::Component;
use crate::gui::renderables::Renderable;
use crate::gui::core::{ Color, Rectangle };
use std::any::Any;

pub struct Text {
    pub font: String,
    pub size: f32,
    pub color: Color,
    pub text: String,
}

impl Component for Text {
    fn update(&mut self, _delta_time: f32) {
    }

    fn draw(&self, parent_bounds: Rectangle) -> Renderable {
        Renderable::Text(crate::gui::renderables::Text {
            text: self.text.clone(),
            size: self.size,
            bounds: parent_bounds,
            color: self.color,
            font: self.font.clone(),
        })
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}