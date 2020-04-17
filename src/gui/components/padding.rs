use nalgebra_glm::Vec4;
use crate::gui::components::Component;
use crate::gui::core::Rectangle;
use crate::gui::renderables::Renderable;
use std::any::Any;

pub struct PaddingBuilder {
    padding: Vec4,
    children: Vec<Box<dyn Component>>,
}

impl PaddingBuilder {
    pub fn new(padding: Vec4) -> Self {
        Self {
            padding,
            children: Vec::new(),
        }
    }

    pub fn with_child<'a, T>(&'a mut self, child: T) -> &'a mut Self where T: Component + Sized + 'static {
        self.children.push(Box::new(child));
        self
    }

    pub fn build(self) -> Padding {
        Padding {
            padding: self.padding,
            children: self.children,
        }
    }
}

pub struct Padding {
    padding: Vec4,
    children: Vec<Box<dyn Component>>,
}

impl Component for Padding {
    fn update(&mut self, _delta_time: f32) {
    }

    fn draw(&self, parent_bounds: Rectangle) -> Renderable {
        let bounds = Rectangle {
            x: self.padding.x,
            y: self.padding.z,
            width: parent_bounds.width - (self.padding.x + self.padding.y),
            height: parent_bounds.height - (self.padding.z + self.padding.w),
        };
        Renderable::Group {
            bounds: parent_bounds,
            renderables: self.children.iter().map(|x| x.draw(bounds)).collect(),
        }
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}