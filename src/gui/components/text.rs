use ultraviolet::vec::Vec2;
use std::any::Any;

use crate::gui::components::Component;
use crate::gui::renderables::Renderable;
use crate::gui::core::{ Color, Rectangle };

pub struct Text {
    pub font: String,
    pub size: f32,
    pub color: Color,
    pub text: String,
    pub position: Vec2,
}

impl Component for Text {
    fn update(&mut self, _delta_time: f32) {
    }

    fn draw(&self, parent_bounds: Rectangle) -> Renderable {
        Renderable::Text(crate::gui::renderables::Text {
            text: self.text.clone(),
            size: self.size,
            bounds: Rectangle {
                x: parent_bounds.x + self.position.x,
                y: parent_bounds.y + self.position.y,
                width: parent_bounds.width,
                height: parent_bounds.height,
            },
            color: self.color,
            font: self.font.clone(),
        })
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}