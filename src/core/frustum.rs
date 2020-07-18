use super::{bounding_sphere::BoundingSphere, plane::{GpuPlane, Plane}};
use nalgebra_glm::Mat4;
use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    pub planes: [Plane; 5],
}

impl Frustum {
    pub fn new() -> Self {
        Self {
            planes: [Plane::new(0.0, 0.0, 0.0, 0.0); 5],
        }
    }

    pub fn from_matrix(matrix: Mat4) -> Self {
        let mat_arr: [[f32; 4]; 4] = matrix.into();

        let left = Plane::new(
            mat_arr[0][3] + mat_arr[0][0],
            mat_arr[1][3] + mat_arr[1][0],
            mat_arr[2][3] + mat_arr[2][0],
            mat_arr[3][3] + mat_arr[3][0],
        );

        let right = Plane::new(
            mat_arr[0][3] - mat_arr[0][0],
            mat_arr[1][3] - mat_arr[1][0],
            mat_arr[2][3] - mat_arr[2][0],
            mat_arr[3][3] - mat_arr[3][0],
        );

        let top = Plane::new(
            mat_arr[0][3] - mat_arr[0][1],
            mat_arr[1][3] - mat_arr[1][1],
            mat_arr[2][3] - mat_arr[2][1],
            mat_arr[3][3] - mat_arr[3][1],
        );

        let bottom = Plane::new(
            mat_arr[0][3] + mat_arr[0][1],
            mat_arr[1][3] + mat_arr[1][1],
            mat_arr[2][3] + mat_arr[2][1],
            mat_arr[3][3] + mat_arr[3][1],
        );

        // no far plane as we have infinite depth

        // this is the far plane in the algorithm, but we're using inverse Z, so near and far
        // get flipped.
        let near = Plane::new(
            mat_arr[0][3] - mat_arr[0][2],
            mat_arr[1][3] - mat_arr[1][2],
            mat_arr[2][3] - mat_arr[2][2],
            mat_arr[3][3] - mat_arr[3][2],
        );

        Self {
            planes: [
                left.normalize(),
                right.normalize(),
                top.normalize(),
                bottom.normalize(),
                near.normalize(),
            ],
        }
    }

    pub fn contains_sphere(&self, sphere: BoundingSphere) -> bool {
        // ref: https://wiki.ogre3d.org/Frustum+Culling+In+Object+Space
        // the normals of the planes point into the frustum, so the distance to a visible object right on the edge of
        // the frustum would be just greater than -radius
        self.planes
            .iter()
            .all(|plane| plane.distance(sphere.center) >= -sphere.radius)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GpuFrustum {
    pub planes: [GpuPlane; 4],
}

unsafe impl Zeroable for GpuFrustum { }
unsafe impl Pod for GpuFrustum { }

impl From<Frustum> for GpuFrustum {
    fn from(frustum: Frustum) -> Self {
        Self {
            planes: [
                GpuPlane {
                    data: [frustum.planes[0].normal.x, frustum.planes[0].normal.y, frustum.planes[0].normal.z, frustum.planes[0].distance],
                },
                GpuPlane {
                    data: [frustum.planes[1].normal.x, frustum.planes[1].normal.y, frustum.planes[1].normal.z, frustum.planes[1].distance],
                },
                GpuPlane {
                    data: [frustum.planes[2].normal.x, frustum.planes[2].normal.y, frustum.planes[2].normal.z, frustum.planes[2].distance],
                },
                GpuPlane {
                    data: [frustum.planes[3].normal.x, frustum.planes[3].normal.y, frustum.planes[3].normal.z, frustum.planes[3].distance],
                },
            ],
        }
    }
}
