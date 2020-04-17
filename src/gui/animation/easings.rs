#![allow(non_snake_case)]
pub struct EasingFunctions {}

impl EasingFunctions {
    // no easing, no acceleration
    pub fn linear(t: f32) -> f32 {
        t
    }
    // accelerating from zero velocity
    pub fn easeInQuad(t: f32) -> f32 {
        t * t
    }
    // decelerating to zero velocity
    pub fn easeOutQuad(t: f32) -> f32 {
        t * (2.0 - t)
    }
    // acceleration until halfway, then deceleration
    pub fn easeInOutQuad(t: f32) -> f32 {
        if t < 0.5 {
            2.0 * t * t
        } else {
            -1.0 + (4.0 - 2.0 * t) * t
        }
    }
    // accelerating from zero velocity
    pub fn easeInCubic(t: f32) -> f32 {
        t * t * t
    }
    // decelerating to zero velocity
    pub fn easeOutCubic(t: f32) -> f32 {
        (--t) * t * t + 1.0
    }
    // acceleration until halfway, then deceleration
    pub fn easeInOutCubic(t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            (t - 1.0) * (2.0 * t - 2.0) * (2.0 * t - 2.0) + 1.0
        }
    }
    // accelerating from zero velocity
    pub fn easeInQuart(t: f32) -> f32 {
        t * t * t * t
    }
    // decelerating to zero velocity
    pub fn easeOutQuart(t: f32) -> f32 {
        1.0 - (--t) * t * t * t
    }
    // acceleration until halfway, then deceleration
    pub fn easeInOutQuart(t: f32) -> f32 {
        if t < 0.5 {
            8.0 * t * t * t * t
        } else {
            1.0 - 8.0 * (--t) * t * t * t
        }
    }
    // accelerating from zero velocity
    pub fn easeInQuint(t: f32) -> f32 {
        t * t * t * t * t
    }
    // decelerating to zero velocity
    pub fn easeOutQuint(t: f32) -> f32 {
        1.0 + (--t) * t * t * t * t
    }
    // acceleration until halfway, then deceleration
    pub fn easeInOutQuint(t: f32) -> f32 {
        if t < 0.5 {
            16.0 * t * t * t * t * t
        } else {
            1.0 + 16.0 * (--t) * t * t * t * t
        }
    }
}
