use nalgebra_glm::{Vec3};

/// An enum representing different light types
pub enum LightType {
    /// Directional Light
    Directional(DirectionalLightData),
    /// Point Light
    Point(PointLightData),
}

/// Directional light information
pub struct DirectionalLightData {
    /// The direction of the light.
    pub direction: Vec3,
    /// The color of the light.
    pub color: Vec3,
    /// Light intensity
    pub intensity: f32,
}

impl Default for DirectionalLightData {
    fn default() -> Self {
        Self {
            direction: Vec3::zeros(),
            color: Vec3::zeros(),
            intensity: 10.0,
        }
    }
}

/// Point light information
/// Position is defined by the transform.
/// Currently point lights do not render.
/// TODO: Fix point lighting rendering.
pub struct PointLightData {
    /// Color of the light.
    pub color: Vec3,
    /// Light attenuation.
    pub attenuation: f32,
    /// Light intensity
    pub intensity: f32,
    // Will generate a shadow map.
    pub shadow: bool,
    // Auto calculated by the omni shadow manager.
    pub(crate) shadow_texture_id: (u32, u32),
    // The age of the shadow map in frames.
    pub(crate) age: u32,
}

impl Default for PointLightData {
    fn default() -> Self {
        Self {
            color: Vec3::zeros(),
            attenuation: 0.0,
            intensity: 10.0,
            shadow: false,
            shadow_texture_id: (0, 0),
            age: 0,
        }
    }
}

impl PointLightData {
    pub fn new(color: Vec3, attenuation: f32, intensity: f32, shadow: bool) -> Self {
        Self {
            color,
            attenuation,
            intensity,
            shadow,
            shadow_texture_id: (0, 0),
            age: 0,
        }
    }
}
