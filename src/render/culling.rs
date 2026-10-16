// REA Verified: net.minecraft.client.renderer.culling.Frustum (mapped: fts.class)
// 6-Plane View Frustum Extraction from View-Projection Matrix (Gribb-Hartmann algorithm)
// Used to cull 16x16x16 ChunkSections before submitting WGPU draw calls.

use glam::{Mat4, Vec3, Vec4};

/// Normalized Frustum Plane represented by normal vector and distance from origin:
/// $ax + by + cz + d = 0$, where $\|normal\| = 1.0$.
/// Points with $normal \cdot p + distance \ge 0$ are on or inside the half-space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrustumPlane {
    pub normal: Vec3,
    pub distance: f32,
}

impl FrustumPlane {
    pub fn new(normal: Vec3, distance: f32) -> Self {
        let len = normal.length();
        if len > 1e-6 {
            Self {
                normal: normal / len,
                distance: distance / len,
            }
        } else {
            Self {
                normal: Vec3::ZERO,
                distance: 0.0,
            }
        }
    }

    /// From raw plane equation coefficients $(A, B, C, D)$
    #[inline]
    pub fn from_vec4(v: Vec4) -> Self {
        Self::new(Vec3::new(v.x, v.y, v.z), v.w)
    }

    /// Signed distance from a 3D point to this plane
    #[inline]
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }
}

/// 6-Plane View Frustum extracted from combined View-Projection Matrix
/// Order: Left, Right, Bottom, Top, Near, Far
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frustum {
    pub planes: [FrustumPlane; 6],
}

impl Frustum {
    pub const LEFT: usize = 0;
    pub const RIGHT: usize = 1;
    pub const BOTTOM: usize = 2;
    pub const TOP: usize = 3;
    pub const NEAR: usize = 4;
    pub const FAR: usize = 5;

    /// Extract frustum planes from combined View-Projection matrix
    /// Compatible with DirectX/WGPU depth clip space $[0, 1]$
    pub fn from_view_proj(vp: Mat4) -> Self {
        let row0 = vp.row(0);
        let row1 = vp.row(1);
        let row2 = vp.row(2);
        let row3 = vp.row(3);

        let planes = [
            // Left plane: row3 + row0
            FrustumPlane::from_vec4(row3 + row0),
            // Right plane: row3 - row0
            FrustumPlane::from_vec4(row3 - row0),
            // Bottom plane: row3 + row1
            FrustumPlane::from_vec4(row3 + row1),
            // Top plane: row3 - row1
            FrustumPlane::from_vec4(row3 - row1),
            // Near plane: row2 (DirectX clip depth: 0 <= z <= w)
            FrustumPlane::from_vec4(row2),
            // Far plane: row3 - row2
            FrustumPlane::from_vec4(row3 - row2),
        ];

        Self { planes }
    }

    /// Test if an axis-aligned bounding box (defined by min and max coordinates)
    /// intersects or is inside the view frustum.
    ///
    /// Returns `false` if the bounding box is completely outside any of the 6 planes.
    pub fn intersects_aabb(&self, min: Vec3, max: Vec3) -> bool {
        for plane in &self.planes {
            // Find p-vertex (box corner furthest in the direction of the plane normal)
            let px = if plane.normal.x >= 0.0 { max.x } else { min.x };
            let py = if plane.normal.y >= 0.0 { max.y } else { min.y };
            let pz = if plane.normal.z >= 0.0 { max.z } else { min.z };

            if plane.distance_to_point(Vec3::new(px, py, pz)) < 0.0 {
                // Entire bounding box is outside this frustum plane
                return false;
            }
        }
        true
    }

    /// Test if a 16x16x16 ChunkSection intersects the view frustum
    /// `sec_pos` contains the chunk section coordinates (e.g. `(0, 0, 0)` for `[0..16, 0..16, 0..16]`).
    #[inline]
    pub fn intersects_chunk_section(&self, sec_pos: glam::IVec3) -> bool {
        let min = Vec3::new(
            (sec_pos.x * 16) as f32,
            (sec_pos.y * 16) as f32,
            (sec_pos.z * 16) as f32,
        );
        let max = min + Vec3::splat(16.0);
        self.intersects_aabb(min, max)
    }

    /// Test an AABB against the view frustum
    #[inline]
    pub fn intersects_box(&self, aabb: &crate::game::physics::Aabb) -> bool {
        self.intersects_aabb(aabb.min, aabb.max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::Camera;

    #[test]
    fn test_frustum_planes_normalized() {
        let cam = Camera::default();
        let vp = cam.view_proj_matrix(16.0 / 9.0);
        let frustum = Frustum::from_view_proj(vp);

        for plane in &frustum.planes {
            let len = plane.normal.length();
            assert!((len - 1.0).abs() < 1e-4, "Plane normal must be unit length, got {len}");
        }
    }

    #[test]
    fn test_chunk_section_inside_frustum() {
        // Camera at (8, 64, -10) looking towards +Z
        let cam = Camera::new(Vec3::new(8.0, 64.0, -10.0), 0.0, 0.0, 70.0);
        let vp = cam.view_proj_matrix(16.0 / 9.0);
        let frustum = Frustum::from_view_proj(vp);

        // Section (0, 4, 0) spans x: [0, 16], y: [64, 80], z: [0, 16] - directly in front
        let sec = glam::IVec3::new(0, 4, 0);
        assert!(frustum.intersects_chunk_section(sec), "Section directly in front must pass frustum test");
    }

    #[test]
    fn test_chunk_section_behind_camera_culled() {
        // Camera at (8, 64, 50) looking towards +Z (yaw = 0)
        let cam = Camera::new(Vec3::new(8.0, 64.0, 50.0), 0.0, 0.0, 70.0);
        let vp = cam.view_proj_matrix(16.0 / 9.0);
        let frustum = Frustum::from_view_proj(vp);

        // Section (0, 4, 0) spans z: [0, 16] - completely behind camera at z = 50
        let sec = glam::IVec3::new(0, 4, 0);
        assert!(!frustum.intersects_chunk_section(sec), "Section behind camera must be culled");
    }

    #[test]
    fn test_chunk_section_far_lateral_culled() {
        // Camera at (8, 64, 0) looking towards +Z (yaw = 0)
        let cam = Camera::new(Vec3::new(8.0, 64.0, 0.0), 0.0, 0.0, 70.0);
        let vp = cam.view_proj_matrix(16.0 / 9.0);
        let frustum = Frustum::from_view_proj(vp);

        // Section (50, 4, 1) spans x: [800, 816], far to the right of the FOV cone
        let sec = glam::IVec3::new(50, 4, 1);
        assert!(!frustum.intersects_chunk_section(sec), "Far lateral section outside FOV must be culled");
    }
}
