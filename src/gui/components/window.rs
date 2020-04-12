use ultraviolet::vec::{ Vec2, Vec4 };
use std::any::Any;

use crate::gui::components::Component;
use crate::gui::core::{ Background, Color, Rectangle };
use crate::gui::renderables::Renderable;

pub struct WindowBuilder {
    background: Option<Background>,
    position: Option<Vec2>,
    size: Option<Vec2>,
    border_radius: Option<u16>,
    border_width: Option<u16>,
    border_color: Option<Color>,
    padding: Option<Vec4>,
    margin: Option<Vec4>,
    title: Option<String>,
    content: Option<Box<dyn Component>>,
}

impl WindowBuilder {
    pub fn new() -> Self {
        WindowBuilder {
            background: None,
            position: None,
            size: None,
            border_radius: None,
            border_width: None,
            border_color: None,
            padding: None,
            margin: None,
            title: None,
            content: None,
        }
    }
    pub fn set_title<'a, T>(&'a mut self, title: T) -> &'a mut Self where T: Into<String> {
        self.title = Some(title.into());
        self
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
        self.size = Some(Vec2::new(size.x, size.y));
        self
    }

    pub fn set_flex_size<'a>(&'a mut self, width: f32, height: f32) -> &'a mut Self {
        self.size = Some(Vec2::new(width, height));
        self
    }

    pub fn set_border<'a>(&'a mut self, border_width: Option<u16>, border_color: Option<Color>, border_radius: Option<u16>) -> &'a mut Self {
        self.border_width = border_width;
        self.border_color = border_color;
        self.border_radius = border_radius;
        self
    }

    pub fn set_padding<'a>(&'a mut self, left: f32, right: f32, top: f32, bottom: f32) -> &'a mut Self {
        self.padding = Some(Vec4::new(left, right, top, bottom));
        self
    }

    pub fn set_margin<'a>(&'a mut self, left: f32, right: f32, top: f32, bottom: f32) -> &'a mut Self {
        self.margin = Some(Vec4::new(left, right, top, bottom));
        self
    }

    pub fn set_content<'a, T>(&'a mut self, content: T) -> &'a mut Self where T: Component + Sized + 'static {
        self.content = Some(Box::new(content));
        self
    }
    
    pub fn build(self) -> Window {
        let title = self.title.unwrap_or(Default::default());
        let background = self.background.unwrap_or(Default::default());
        let position = self.position.unwrap_or(Default::default());
        let size = self.size.unwrap_or(Default::default());
        let border_color = self.border_color.unwrap_or(Default::default());
        let border_radius = self.border_radius.unwrap_or(Default::default());
        let border_width = self.border_width.unwrap_or(Default::default());
        let padding = self.padding.unwrap_or(Default::default());
        let margin = self.margin.unwrap_or(Default::default());
        let content = self.content;
        Window {
            title,
            background,
            position,
            border_radius,
            border_width,
            border_color,
            size,
            padding,
            margin,
            content,
        }
    }
}

#[derive(Default)]
pub struct Window {
    title: String,
    position: Vec2,
    size: Vec2,
    background: Background,
    border_radius: u16,
    border_width: u16,
    border_color: Color,
    padding: Vec4,
    margin: Vec4,
    content: Option<Box<dyn Component>>,
}

impl Component for Window {
    fn update(&mut self, _delta_time: f32) {

    }

    fn draw(&self, bounds: Rectangle) -> Renderable {
        let bounds = Rectangle {
            x: bounds.x - self.margin.x,
            y: bounds.y - self.margin.z,
            width: self.size.x - self.margin.y,
            height: self.size.y - self.margin.w,
        };
        let content_bounds = Rectangle {
            x: 0.0,
            y: 25.0,
            width: self.size.x,
            height: self.size.y - 25.0,
        };
        Renderable::Group {
            bounds: bounds,
            renderables: vec![
                Renderable::Quad {
                    /// The bounds of the quad
                    bounds: Rectangle {
                        x: 0.0,
                        y: 0.0,
                        width: self.size.x,
                        height: self.size.y,
                    },
                    /// The background of the quad
                    background: self.background,
                    /// The border radius of the quad
                    border_radius: self.border_radius,
                    /// The border width of the quad
                    border_width: self.border_width,
                    /// The border color of the quad
                    border_color: self.border_color,
                },
                Renderable::Quad {
                    bounds: Rectangle {
                        x: 0.0,
                        y: 0.0,
                        width: self.size.x,
                        height: 25.0,
                    },
                    background: Background::from(Color::from_rgb(0.1, 0.1, 0.1)),
                    border_radius: 0,
                    border_width: 0,
                    border_color: Default::default(),
                },
                Renderable::Text(crate::gui::renderables::Text {
                    bounds: Rectangle {
                        x: 10.0,
                        y: 5.0,
                        width: self.size.x,
                        height: 25.0,
                    },
                    size: 16.0,
                    text: self.title.clone(),
                    color: Color::WHITE,
                    font: "moon.otf".to_string(),
                }),
                Renderable::Clip {
                    bounds: Rectangle {
                        x: bounds.x + content_bounds.x,
                        y: bounds.y + content_bounds.y - 25.0,
                        ..content_bounds
                    },
                    offset: Vec2::new(0.0, 0.0),
                    content: Box::new(self.content.as_ref().unwrap().draw(content_bounds)),
                }
            ]
        }
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}