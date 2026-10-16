// REA Verified: net.minecraft.client.Camera (mapped: eut.class)
// REA Verified: net.minecraft.client.renderer.GameRenderer (mapped: fta.class)
// Bytecode constant: 0.017453292f (PI / 180.0f)
// Bytecode constant: zNear = 0.05f

use glam::{Mat4, Quat, Vec3};

/// Constant extracted from net.minecraft.client.Camera.setRotation(float, float)
/// Bytecode: ldc #231 // float 0.017453292f
pub const DEG_TO_RAD: f32 = 0.017453292;

/// Constant extracted from net.minecraft.client.renderer.GameRenderer.getProjectionMatrix(double)
/// Bytecode: ldc #50 // float 0.05f
pub const DEFAULT_Z_NEAR: f32 = 0.05;

/// First-Person Camera matching net.minecraft.client.Camera (eut.class)
#[derive(Debug, Clone)]
pub struct Camera {
    /// Eye position in world coordinates (net.minecraft.client.Camera.position: elt)
    pub eye: Vec3,
    /// Pitch angle in degrees (net.minecraft.client.Camera.xRot: F), clamped [-89.9, 89.9]
    pub pitch: f32,
    /// Yaw angle in degrees (net.minecraft.client.Camera.yRot: F)
    pub yaw: f32,
    /// Field of view in degrees (default vanilla FOV = 70.0)
    pub fov_degrees: f32,
    /// Near clipping plane (default 0.05)
    pub z_near: f32,
    /// Far clipping plane (default 1000.0)
    pub z_far: f32,

    // Rotation quaternion and directional vectors matching JOML rotation in eut.class
    rotation: Quat,
    forwards: Vec3,
    up: Vec3,
    left: Vec3,
}

impl Default for Camera {
    fn default() -> Self {
        let mut cam = Self {
            eye: Vec3::new(0.0, 64.0, 0.0),
            pitch: 0.0,
            yaw: 0.0,
            fov_degrees: 70.0,
            z_near: DEFAULT_Z_NEAR,
            z_far: 1000.0,
            rotation: Quat::IDENTITY,
            forwards: Vec3::Z,
            up: Vec3::Y,
            left: Vec3::X,
        };
        cam.update_vectors();
        cam
    }
}

impl Camera {
    pub fn new(eye: Vec3, yaw: f32, pitch: f32, fov_degrees: f32) -> Self {
        let mut cam = Self {
            eye,
            pitch: pitch.clamp(-89.9, 89.9),
            yaw,
            fov_degrees,
            z_near: DEFAULT_Z_NEAR,
            z_far: 1000.0,
            rotation: Quat::IDENTITY,
            forwards: Vec3::Z,
            up: Vec3::Y,
            left: Vec3::X,
        };
        cam.update_vectors();
        cam
    }

    /// REA Verified: net.minecraft.client.Camera::setRotation(float, float) -> eut.a(FF)V
    /// Bytecode:
    ///   aload_0
    ///   fload_1 (yaw)
    ///   putfield Field k:F
    ///   aload_0
    ///   fload_2 (pitch)
    ///   putfield Field j:F
    ///   rotation.rotationYXZ(-yaw * 0.017453292F, pitch * 0.017453292F, 0.0F)
    ///   forwards.set(0.0F, 0.0F, 1.0F).rotate(rotation)
    ///   up.set(0.0F, 1.0F, 0.0F).rotate(rotation)
    ///   left.set(1.0F, 0.0F, 0.0F).rotate(rotation)
    pub fn set_rotation(&mut self, yaw: f32, pitch: f32) {
        self.yaw = yaw;
        self.pitch = pitch.clamp(-89.9, 89.9);
        self.update_vectors();
    }

    /// Update internal rotation quaternion and directional unit vectors
    fn update_vectors(&mut self) {
        let yaw_rad = -self.yaw * DEG_TO_RAD;
        let pitch_rad = self.pitch * DEG_TO_RAD;

        // JOML rotationYXZ(y, x, z) order: Y then X then Z
        let q_y = Quat::from_rotation_y(yaw_rad);
        let q_x = Quat::from_rotation_x(pitch_rad);
        self.rotation = q_y * q_x;

        // Direction vectors matching eut.class lines 28-71
        self.forwards = self.rotation * Vec3::new(0.0, 0.0, 1.0);
        self.up = self.rotation * Vec3::new(0.0, 1.0, 0.0);
        self.left = self.rotation * Vec3::new(1.0, 0.0, 0.0);
    }

    /// Returns the normalized forward look vector (net.minecraft.client.Camera.getLookVector() -> eut.l())
    pub fn forwards(&self) -> Vec3 {
        self.forwards
    }

    /// Returns the normalized up vector (net.minecraft.client.Camera.getUpVector() -> eut.m())
    pub fn up(&self) -> Vec3 {
        self.up
    }

    /// Returns the normalized left vector (net.minecraft.client.Camera.getLeftVector() -> eut.n())
    pub fn left(&self) -> Vec3 {
        self.left
    }

    /// Returns the normalized right vector
    pub fn right(&self) -> Vec3 {
        -self.left
    }

    /// Computes the Camera View Matrix
    /// Transforms world space coordinates to camera space
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::from_quat(self.rotation.conjugate()) * Mat4::from_translation(-self.eye)
    }

    /// REA Verified: net.minecraft.client.renderer.GameRenderer::getProjectionMatrix(double) -> fta.a(D)Lorg/joml/Matrix4f;
    /// Bytecode:
    ///   fovRadians = fov * 0.01745329238474369d
    ///   setPerspective(fovRadians, aspect, 0.05f, depthFar)
    pub fn proj_matrix(&self, aspect: f32) -> Mat4 {
        let fov_rad = self.fov_degrees * DEG_TO_RAD;
        glam::camera::rh::proj::directx::perspective(fov_rad, aspect, self.z_near, self.z_far)
    }

    /// Combined View-Projection Matrix
    pub fn view_proj_matrix(&self, aspect: f32) -> Mat4 {
        self.proj_matrix(aspect) * self.view_matrix()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_initialization() {
        let cam = Camera::default();
        assert_eq!(cam.fov_degrees, 70.0);
        assert_eq!(cam.z_near, DEFAULT_Z_NEAR);
        assert!((cam.forwards().length() - 1.0).abs() < 1e-5);
        assert!((cam.up().length() - 1.0).abs() < 1e-5);
        assert!((cam.left().length() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_pitch_clamp() {
        let mut cam = Camera::default();
        cam.set_rotation(0.0, 100.0);
        assert!(cam.pitch <= 89.9);
        cam.set_rotation(0.0, -100.0);
        assert!(cam.pitch >= -89.9);
    }

    #[test]
    fn test_matrices() {
        let cam = Camera::new(Vec3::new(10.0, 64.0, 10.0), 45.0, -15.0, 70.0);
        let view = cam.view_matrix();
        let proj = cam.proj_matrix(16.0 / 9.0);
        let vp = cam.view_proj_matrix(16.0 / 9.0);
        assert!(!view.is_nan());
        assert!(!proj.is_nan());
        assert!(!vp.is_nan());
    }
}
