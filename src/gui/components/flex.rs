use stretch::{
    Stretch,
    node::Node,
    geometry::{ Size },
    style::{ Dimension, Style, JustifyContent },
};

use crate::gui::components::Component;
use crate::gui::renderables::Renderable;

pub struct Flex {
    style: Style,
    children: Vec<Box<dyn Component>>,
    stretch: Stretch,
}

impl Flex {
    pub fn new() -> Self {
        Self {
            style: Style {
                padding: stretch::geometry::Rect {
                    start: stretch::style::Dimension::Points(10f32),
                    end: stretch::style::Dimension::Points(10f32),
                    top: stretch::style::Dimension::Points(10f32),
                    bottom: stretch::style::Dimension::Points(10f32),
                },
                size: Size { width: Dimension::Points(1024.0), height: Dimension::Points(768.0) },
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            children: Vec::new(),
            stretch: stretch::node::Stretch::new(),
        }
    }

    pub fn push<T>(&mut self, component: T) where T: Component + Sized + 'static {
        self.children.push(Box::new(component));
    }

    fn create_nodes_from_children(&self, stretch: &mut Stretch) -> Vec<(&Box<dyn Component>, Node)> {
        let children_nodes: Vec<(&Box<dyn Component>, Node)> = self.children.iter().map(|x: &Box<dyn Component>|
            (x, x.as_ref().node(stretch, stretch::style::PositionType::Relative))
        ).collect();
    
        children_nodes
    } 
}


impl Component for Flex {
    fn node(&self, stretch: &mut stretch::Stretch, _position_type: stretch::style::PositionType) -> stretch::node::Node {
        let children = self.create_nodes_from_children(stretch);
        let children_nodes = children.iter().map(|x| x.1).collect();
        stretch.new_node(self.style, children_nodes).unwrap()
    }

    fn draw(&self, stretch: &mut stretch::Stretch, node: stretch::node::Node) -> Renderable {
        // Compute layout
        stretch.compute_layout(node, Size::undefined()).unwrap();

        // TODO: Do something with this.
        let _layout = stretch.layout(node).unwrap();

        // Get children nodes.
        let children = stretch.children(node).unwrap();

        let renderables = self.children.iter().zip(children).map(|x| {
            x.0.as_ref().draw(stretch, x.1)
        }).collect();

        Renderable::Group {
            bounds: Default::default(),
            renderables,
        }
    }
}