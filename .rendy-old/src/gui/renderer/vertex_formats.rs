use rendy::mesh::{ AsVertex, Color, VertexFormat };
use crate::gui::renderer::vertex_attributes::*;

/// Vertex format for quads.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QuadVertex {
    pub position: Position2D,
}

impl AsVertex for QuadVertex {
    fn vertex() -> VertexFormat {
        VertexFormat::new(Position2D::vertex())
    }
}

/// Instanced format for quads.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InstanceQuadVertex {
    pub position: Position2D,
    pub scale: Position2D,
    pub color: Color,
    pub border_color: Color,
    pub border_radius: Float,
    pub border_width: Float,
}

impl AsVertex for InstanceQuadVertex {
    fn vertex() -> VertexFormat {
        VertexFormat::new((Position2D::vertex(), Position2D::vertex(), Color::vertex(), Color::vertex(), Float::vertex(), Float::vertex()))
    }
}