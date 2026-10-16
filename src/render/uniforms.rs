use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use crate::render::camera::Camera;

/// Uniform buffer payload for camera view-projection and eye position.
/// Guaranteed 16-byte alignment for standard WGSL uniforms (`std140`/`uniform`).
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraUniform {
    /// 4x4 View-Projection Matrix (64 bytes, 16-byte aligned)
    pub view_proj: [[f32; 4]; 4],
    /// Camera world position [x, y, z, _padding] (16 bytes, 16-byte aligned)
    pub eye_pos: [f32; 4],
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            eye_pos: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

impl CameraUniform {
    pub fn update(&mut self, camera: &Camera, aspect: f32) {
        self.view_proj = camera.view_proj_matrix(aspect).to_cols_array_2d();
        self.eye_pos = [camera.eye.x, camera.eye.y, camera.eye.z, 1.0];
    }
}
