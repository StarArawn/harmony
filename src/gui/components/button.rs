use ultraviolet::vec::Vec2;
use stretch::{ Stretch, node::Node, geometry::{ Rect, Size}, style::{ Dimension, Style } };

use crate::gui::components::Component;
use crate::gui::core::{ Background, Color, Rectangle };
use crate::gui::renderables::Renderable;

pub struct ButtonBuilder {
    background: Option<Background>,
    position: Option<Vec2>,
    size: Option<Vec2>,
    border_radius: Option<u16>,
    border_width: Option<u16>,
    border_color: Option<Color>,
    children: Vec<Box<dyn Component>>,
}

impl ButtonBuilder {
    pub fn new() -> Self {
        ButtonBuilder {
            background: None,
            position: None,
            size: None,
            border_radius: None,
            border_width: None,
            border_color: None,
            children: Vec::new(),
        }
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

    pub fn with_child<'a, T>(&'a mut self, child: T) -> &'a mut Self where T: Component + Sized + 'static {
        self.children.push(Box::new(child));
        self
    }

    pub fn build(self) -> Button {
        let position = self.position.unwrap_or(Default::default());
        let size = self.position.unwrap_or(Default::default());
        let rectangle = Rectangle { 
            x: position.x,
            y: position.y,
            width: size.x,
            height: size.y,
        };
        Button {
            state: Default::default(),
            background: self.background.unwrap_or(Default::default()),
            rectangle,
            border_width: self.border_width.unwrap_or(Default::default()),
            border_color: self.border_color.unwrap_or(Default::default()),
            border_radius: self.border_radius.unwrap_or(Default::default()),
            children: self.children,
        }
    }
}

pub enum ButtonState {
    Default,
    Pressed,
    Hover,
}

impl Default for ButtonState { 
    fn default() -> Self {
        ButtonState::Default
    }
}

#[derive(Default)]
pub struct Button {
    state: ButtonState,
    background: Background,
    rectangle: Rectangle,
    border_radius: u16,
    border_width: u16,
    border_color: Color,
    children: Vec<Box<dyn Component>>,
}

impl Component for Button {

    fn layout(&self, stretch: &mut Stretch) -> Node {
        let position = Rect::<Dimension> {
            start: Dimension::Points(self.rectangle.x),
            end: Dimension::Points(self.rectangle.x + self.rectangle.width),
            top: Dimension::Points(self.rectangle.y),
            bottom: Dimension::Points(self.rectangle.y + self.rectangle.height),
        };

        stretch.new_node(
            Style { size: Size { width: Dimension::Points(self.rectangle.width), height: Dimension::Auto }, position, ..Default::default() },
            self.children.iter().map(|x: &Box<dyn Component>| x.as_ref().layout(stretch)).collect(),
        ).expect("Button Error: Something went wrong generating the layout!")
    }
    
    fn draw(&self, stretch: &mut Stretch) -> Renderable {
        let node = self.layout(stretch);

        stretch.compute_layout(node, Size::undefined());
        let layout = stretch.layout(node).unwrap();

        let rectangle = Rectangle {
            x: layout.location.x,
            y: layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        };

        // First take care of children
        if self.children.len() > 0 {
            Renderable::Group {
                renderables: self.children.iter().map(|x: &Box<dyn Component>| x.as_ref().draw(stretch)).collect(),
            }
        } else {
            // Then lets take care of the quad.
            Renderable::Quad {
                /// The bounds of the quad
                bounds: rectangle,
                /// The background of the quad
                background: self.background,
                /// The border radius of the quad
                border_radius: self.border_radius,
                /// The border width of the quad
                border_width: self.border_width,
                /// The border color of the quad
                border_color: self.border_color,
            }
        }
    }
}