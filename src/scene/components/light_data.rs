use nalgebra_glm::Vec3;

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
}

impl Default for DirectionalLightData {
    fn default() -> Self {
        Self {
            direction: Vec3::zeros(),
            color: Vec3::zeros(),
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
}

impl Default for PointLightData {
    fn default() -> Self {
        Self {
            color: Vec3::zeros(),
            attenuation: 0.0,
        }
    }
}
