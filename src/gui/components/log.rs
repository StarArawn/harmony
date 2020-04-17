use crate::gui::components::Component;
use crate::gui::core::{Color, Rectangle};
use crate::gui::renderables::Renderable;
use std::any::Any;

#[derive(Clone)]
pub struct LogLine {
    color: Color,
    text: String,
}

#[derive(Clone)]
pub struct Log {
    lines: Vec<Renderable>,
}

impl Log {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }

    pub fn add_line(&mut self, renderable: Renderable) {
        self.lines.push(renderable);
    }
}

impl Component for Log {
    fn update(&mut self, _delta_time: f32) {}

    fn draw(&self, parent_bounds: Rectangle) -> Renderable {
        let mut y = 0.0;
        let renderables: Vec<Renderable> = self
            .lines
            .iter()
            .map(|line| {
                let renderable = line.clone();
                let renderable = match renderable {
                    Renderable::Group {
                        bounds,
                        renderables,
                    } => Renderable::Group {
                        bounds: Rectangle {
                            x: bounds.x,
                            y: y,
                            width: parent_bounds.width,
                            height: bounds.height,
                        },
                        renderables,
                    },
                    v => v,
                };
                y += 18.0;
                renderable
            })
            .collect();

        Renderable::Group {
            bounds: parent_bounds,
            renderables,
        }
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
