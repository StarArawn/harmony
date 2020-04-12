use ultraviolet::{
    lerp::Lerp,
    vec::Vec2
};
use crate::gui::components::Component;
use crate::gui::core::{ Rectangle };
use crate::gui::renderables::Renderable;
use std::any::Any;

pub struct AnimationBuilder {
    children: Vec<Box<dyn Component>>,
    easing_function: Option<Box<dyn Fn(f32) -> f32>>,
    duration: f32,
    destination: Option<Vec2>,
    position: Option<Vec2>,
}

impl AnimationBuilder {
    pub fn new() -> Self {
        AnimationBuilder {
            children: Vec::new(),
            easing_function: None,
            duration: 0.0,
            destination: None,
            position: None,
        }
    }
    
    pub fn set_easing_function<'a, T>(&'a mut self, function: T) -> &'a mut Self where T: Fn(f32) -> f32 + Sized + 'static {
        self.easing_function = Some(Box::new(function));
        self
    }

    pub fn with_duration<'a>(&'a mut self, duration: f32) -> &'a mut Self {
        self.duration = duration;
        self
    }

    pub fn with_destination<'a>(&'a mut self, position: Vec2) -> &'a mut Self {
        self.destination = Some(position);
        self
    }

    pub fn with_position<'a>(&'a mut self, position: Vec2) -> &'a mut Self {
        self.position = Some(position);
        self
    }

    pub fn with_child<'a, T>(&'a mut self, child: T) -> &'a mut Self where T: Component + Sized + 'static {
        self.children.push(Box::new(child));
        self
    }

    pub fn build(self) -> Animation {
        Animation {
            children: self.children,
            easing_function: self.easing_function.unwrap_or(Box::new(crate::gui::animation::EasingFunctions::linear)),
            start_time: std::time::Instant::now(),
            duration: self.duration,
            position: self.position.unwrap_or(Vec2::new(0.0, 0.0)),
            destination: self.destination.unwrap_or(Vec2::new(0.0, 0.0)),
        }
    }
}

pub struct Animation {
    children: Vec<Box<dyn Component>>,
    easing_function: Box<dyn Fn(f32) -> f32>,
    start_time: std::time::Instant,
    duration: f32,
    position: Vec2,
    destination: Vec2,
}

impl Animation {
    pub fn start(&mut self, duration: f32, destination: Vec2) {
        self.start_time = std::time::Instant::now();
        self.duration = duration;
        self.destination = destination;
    }
}

impl Component for Animation {
    fn update(&mut self, _delta_time: f32) {
        let mut time = self.start_time.elapsed().as_secs_f32() / self.duration;
        if time > 1.0 { time = 1.0; }

        time = self.easing_function.as_ref()(time);
        let new_position = self.position.lerp(self.destination, time);
        self.position = new_position;
    }

    fn draw(&self, _bounds: Rectangle) -> Renderable {
        // Compute layout
        // TODO: Move this out of here.
        //stretch.compute_layout(node, Size::undefined()).unwrap();
        
        let bounds =Rectangle {
            x: self.position.x,
            y: self.position.y,
            width: 1.0,
            height: 1.0,
        };

        let renderables = self.children.iter().map(|x| {
            x.as_ref().draw(bounds)
        }).collect();

        let group = Renderable::Group {
            bounds,
            renderables,
        };
        
        group
    }
    
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}