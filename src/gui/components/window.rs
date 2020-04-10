use ultraviolet::vec::Vec2;
use stretch::{
    style::{ JustifyContent, Dimension },
    geometry::{ Size }
};
use crate::gui::components::Component;
use crate::gui::core::{ Background, Color, Rectangle };
use crate::gui::renderables::Renderable;

pub struct WindowBuilder {
    background: Option<Background>,
    position: Option<Vec2>,
    size: Option<(Dimension, Dimension)>,
    border_radius: Option<u16>,
    border_width: Option<u16>,
    border_color: Option<Color>,
    padding: Option<(f32, f32, f32, f32)>,
    margin: Option<(f32, f32, f32, f32)>,
    title: Option<String>,
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
        self.size = Some((Dimension::Points(size.x), Dimension::Points(size.y)));
        self
    }

    pub fn set_flex_size<'a>(&'a mut self, width: Dimension, height: Dimension) -> &'a mut Self {
        self.size = Some((width, height));
        self
    }

    pub fn set_border<'a>(&'a mut self, border_width: Option<u16>, border_color: Option<Color>, border_radius: Option<u16>) -> &'a mut Self {
        self.border_width = border_width;
        self.border_color = border_color;
        self.border_radius = border_radius;
        self
    }

    pub fn set_padding<'a>(&'a mut self, left: f32, right: f32, top: f32, bottom: f32) -> &'a mut Self {
        self.padding = Some((left, right, top, bottom));
        self
    }

    pub fn set_margin<'a>(&'a mut self, left: f32, right: f32, top: f32, bottom: f32) -> &'a mut Self {
        self.margin = Some((left, right, top, bottom));
        self
    }
    
    pub fn build(self) -> Window {
        let title = self.title.unwrap_or(Default::default());
        let background = self.background.unwrap_or(Default::default());
        //let position = self.position.unwrap_or(Default::default());
        let size = self.size.unwrap_or(Default::default());
        let border_color = self.border_color.unwrap_or(Default::default());
        let border_radius = self.border_radius.unwrap_or(Default::default());
        let border_width = self.border_width.unwrap_or(Default::default());
        let padding = self.padding.unwrap_or(Default::default());
        let margin = self.margin.unwrap_or(Default::default());
        Window {
            title,
            background,
            border_radius,
            border_width,
            border_color,
            size,
            padding: stretch::geometry::Rect {
                start: stretch::style::Dimension::Points(padding.0),
                end: stretch::style::Dimension::Points(padding.1),
                top: stretch::style::Dimension::Points(padding.2),
                bottom: stretch::style::Dimension::Points(padding.3),
            },
            margin: stretch::geometry::Rect {
                start: stretch::style::Dimension::Points(margin.0),
                end: stretch::style::Dimension::Points(margin.1),
                top: stretch::style::Dimension::Points(margin.2),
                bottom: stretch::style::Dimension::Points(margin.3),
            },
            style: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct Window {
    title: String,
    style: stretch::style::Style,
    size: (Dimension, Dimension),
    background: Background,
    border_radius: u16,
    border_width: u16,
    border_color: Color,
    padding: stretch::geometry::Rect<Dimension>,
    margin: stretch::geometry::Rect<Dimension>,
}

impl Component for Window {
    fn node(&self, stretch: &mut stretch::Stretch, position_type: stretch::style::PositionType) -> stretch::node::Node {
        stretch.new_node(stretch::style::Style {
            position_type,
            size: Size { width: self.size.0, height: self.size.1 },
            padding: self.padding,
            margin: self.margin,
            ..Default::default()
        }, vec![]).unwrap()
    }

    fn draw(&self, stretch: &mut stretch::Stretch, node: stretch::node::Node) -> Renderable {
        //stretch.compute_layout(node, Size::undefined());
        let layout = stretch.layout(node).unwrap();
        Renderable::Group {
            bounds: Rectangle {
                x: layout.location.x,
                y: layout.location.y,
                width: layout.size.width,
                height: layout.size.height,
            },
            renderables: vec![
                Renderable::Quad {
                    /// The bounds of the quad
                    bounds: Rectangle {
                        x: 0.0,
                        y: 0.0,
                        width: layout.size.width,
                        height: layout.size.height,
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
                        width: layout.size.width,
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
                        width: layout.size.width,
                        height: 25.0,
                    },
                    size: 1.0,
                    text: self.title.clone(),
                    color: Color::WHITE,
                    font: "moon.otf".to_string(),
                })
            ]
        }
    }
}