use ultraviolet::vec::Vec2;
use crate::gui::components::Component;
use crate::gui::core::{ Background, Rectangle };
use crate::gui::renderables::Renderable;

pub struct WindowBuilder {
    background: Option<Background>,
    position: Option<Vec2>,
    size: Option<Vec2>,
    border_radius: Option<u16>,
    border_width: Option<u16>,
    border_color: Option<Color>,
}

impl WindowBuilder {
    pub fn new() -> Self {
        WindowBuilder { }
    }

    pub fn set_background<'a>(&'a mut self, background: Background) -> &'a mut Self {
        self.background = Some(background);
        self
    }

    pub fn set_position<'a>(&'a mut self, position: Vec2) -> &'a mut Self {
        self.position = Some(position);
        self
    }

    pub fn set_size<'a>(&'a mut self, size: Vec2) -> &'a mut Self {
        self.size = Some(size);
        self
    }

    pub fn set_border<'a>(&'a mut self, border_width: Option<u16>, border_color: Option<Color>, border_radius: Option<u16>) -> &'a mut Self {
        self.border_width = border_width;
        self.border_color = border_color;
        self.border_radius = border_radius;
        self
    }
}

#[derive(Default)]
pub struct Window {
    background: Background,
    rectangle: Rectangle,
    border_radius: u16,
    border_width: u16,
    border_color: Color,
}

impl Component for Window {
    fn draw(&self ) -> Renderable {
        Renderable::Quad {
            /// The bounds of the quad
            bounds: Rectangle,
            /// The background of the quad
            background: Background,
            /// The border radius of the quad
            border_radius: u16,
            /// The border width of the quad
            border_width: u16,
            /// The border color of the quad
            border_color: Color,
        }
    }
}