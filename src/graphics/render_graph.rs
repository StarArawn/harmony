use std::sync::Arc;
use crate::graphics::Drawable;

pub struct RenderGraph {
    pub drawables: Vec<Arc<dyn Drawable>>,
}

impl RenderGraph {
    pub fn new() -> Self {
        RenderGraph {
            drawables: Vec::new(),
        }
    }

    pub fn add<T: Drawable + Sized + 'static>(&mut self, drawable: T) {
        let _id = drawable.id();
        self.drawables.push(Arc::new(drawable));
    }

    pub fn remove(&mut self, drawable: &dyn Drawable) {
        self.drawables.remove(drawable.id());
    }

    pub fn length(&self) -> usize {
        self.drawables.len()
    }

    pub fn draw(&self) {
        for drawable in self.drawables.iter() {
            drawable.draw();
        }
    }
}