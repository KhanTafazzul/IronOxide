// REA Verified: net.minecraft.client.Camera (mapped: eut.class)
// REA Verified: net.minecraft.client.renderer.GameRenderer (mapped: fta.class)
// Bytecode constant: 0.017453292f (PI / 180.0f)
// Bytecode constant: zNear = 0.05f
// Bytecode constant: 4.0d (maxZoom for third-person perspective offset)

use glam::{Mat4, Quat, Vec3};
use crate::render::mesher::BlockReader;

/// Constant extracted from net.minecraft.client.Camera.setRotation(float, float)
/// Bytecode: ldc #231 // float 0.017453292f
pub const DEG_TO_RAD: f32 = 0.017453292;

/// Constant extracted from net.minecraft.client.renderer.GameRenderer.getProjectionMatrix(double)
/// Bytecode: ldc #50 // float 0.05f
pub const DEFAULT_Z_NEAR: f32 = 0.05;

/// Camera perspective modes matching net.minecraft.client.CameraType
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraMode {
    /// Locked to player eye coordinate (y + 1.62) looking along player direction
    #[default]
    FirstPerson,
    /// Offset 4.0 blocks behind player direction vector looking forward
    ThirdPersonBack,
    /// Offset 4.0 blocks ahead of player, rotated 180 degrees looking back at player
    ThirdPersonFront,
}

impl CameraMode {
    /// Cycle camera mode: FirstPerson -> ThirdPersonBack -> ThirdPersonFront -> FirstPerson
    pub fn cycle(self) -> Self {
        match self {
            Self::FirstPerson => Self::ThirdPersonBack,
            Self::ThirdPersonBack => Self::ThirdPersonFront,
            Self::ThirdPersonFront => Self::FirstPerson,
        }
    }
}

/// Multi-perspective Camera matching net.minecraft.client.Camera (eut.class)
#[derive(Debug, Clone)]
pub struct Camera {
    /// Camera world coordinate (net.minecraft.client.Camera.position: elt)
    pub eye: Vec3,
    /// Player pitch angle in degrees (net.minecraft.client.Camera.xRot: F), clamped [-89.9, 89.9]
    pub pitch: f32,
    /// Player yaw angle in degrees (net.minecraft.client.Camera.yRot: F)
    pub yaw: f32,
    /// Field of view in degrees (default vanilla FOV = 70.0)
    pub fov_degrees: f32,
    /// Near clipping plane (default 0.05)
    pub z_near: f32,
    /// Far clipping plane (default 1000.0)
    pub z_far: f32,
    /// Current camera perspective mode
    pub mode: CameraMode,

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
            mode: CameraMode::FirstPerson,
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
            mode: CameraMode::FirstPerson,
            rotation: Quat::IDENTITY,
            forwards: Vec3::Z,
            up: Vec3::Y,
            left: Vec3::X,
        };
        cam.update_vectors();
        cam
    }

    /// Returns current camera perspective mode
    #[inline]
    pub fn mode(&self) -> CameraMode {
        self.mode
    }

    /// Sets camera perspective mode
    pub fn set_mode(&mut self, mode: CameraMode) {
        self.mode = mode;
        self.update_vectors();
    }

    /// Cycles through the 3 camera modes on F5 press
    pub fn cycle_mode(&mut self) -> CameraMode {
        self.mode = self.mode.cycle();
        self.update_vectors();
        self.mode
    }

    /// REA Verified: net.minecraft.client.Camera::setRotation(float, float) -> eut.a(FF)V
    pub fn set_rotation(&mut self, yaw: f32, pitch: f32) {
        self.yaw = yaw;
        self.pitch = pitch.clamp(-89.9, 89.9);
        self.update_vectors();
    }

    /// Update internal rotation quaternion and directional unit vectors
    fn update_vectors(&mut self) {
        match self.mode {
            CameraMode::FirstPerson | CameraMode::ThirdPersonBack => {
                let yaw_rad = -self.yaw * DEG_TO_RAD;
                let pitch_rad = self.pitch * DEG_TO_RAD;

                // JOML rotationYXZ(y, x, z) order: Y then X then Z
                let q_y = Quat::from_rotation_y(yaw_rad);
                let q_x = Quat::from_rotation_x(pitch_rad);
                self.rotation = q_y * q_x;
            }
            CameraMode::ThirdPersonFront => {
                // Bytecode lines 117-125: setRotation(yaw + 180.0f, -pitch)
                let yaw_rad = -(self.yaw + 180.0) * DEG_TO_RAD;
                let pitch_rad = -self.pitch * DEG_TO_RAD;

                let q_y = Quat::from_rotation_y(yaw_rad);
                let q_x = Quat::from_rotation_x(pitch_rad);
                self.rotation = q_y * q_x;
            }
        }

        // Direction vectors matching eut.class lines 28-71
        self.forwards = self.rotation * Vec3::new(0.0, 0.0, 1.0);
        self.up = self.rotation * Vec3::new(0.0, 1.0, 0.0);
        self.left = self.rotation * Vec3::new(1.0, 0.0, 0.0);
    }

    /// Returns the player's true look vector (regardless of camera mode)
    #[inline]
    pub fn player_forwards(&self) -> Vec3 {
        let q_y = Quat::from_rotation_y(-self.yaw * DEG_TO_RAD);
        let q_x = Quat::from_rotation_x(self.pitch * DEG_TO_RAD);
        (q_y * q_x) * Vec3::new(0.0, 0.0, 1.0)
    }

    /// REA Verified: net.minecraft.client.Camera.setup(...) -> eut.a(csv, blv, boolean, boolean, float)
    /// Setup camera position, orientation, perspective offsets, and block occlusion raycasting
    pub fn setup(&mut self, player_pos: Vec3, world: Option<&dyn BlockReader>) {
        let eye_pos = Vec3::new(player_pos.x, player_pos.y + 1.62, player_pos.z);
        self.setup_from_eye(eye_pos, world);
    }

    /// Setup camera from given player eye coordinates
    pub fn setup_from_eye(&mut self, eye_pos: Vec3, world: Option<&dyn BlockReader>) {
        self.update_vectors();
        let player_fwd = self.player_forwards();

        match self.mode {
            CameraMode::FirstPerson => {
                // Locked to player eye coordinate
                self.eye = eye_pos;
            }
            CameraMode::ThirdPersonBack => {
                // Offset 4.0 blocks behind player direction vector with occlusion raycast
                let max_dist = 4.0;
                let actual_dist = if let Some(reader) = world {
                    // Raycast backwards from player eye position
                    if let Some(hit) = crate::game::raycast::raycast_voxel(eye_pos, -player_fwd, max_dist, reader) {
                        // Clamp directly in front of hit voxel face
                        (hit.distance - 0.1).max(0.0)
                    } else {
                        max_dist
                    }
                } else {
                    max_dist
                };
                self.eye = eye_pos - player_fwd * actual_dist;
            }
            CameraMode::ThirdPersonFront => {
                // Offset 4.0 blocks ahead of player with occlusion raycast
                let max_dist = 4.0;
                let actual_dist = if let Some(reader) = world {
                    // Raycast forwards from player eye position
                    if let Some(hit) = crate::game::raycast::raycast_voxel(eye_pos, player_fwd, max_dist, reader) {
                        // Clamp directly in front of hit voxel face
                        (hit.distance - 0.1).max(0.0)
                    } else {
                        max_dist
                    }
                } else {
                    max_dist
                };
                self.eye = eye_pos + player_fwd * actual_dist;
            }
        }
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
    pub fn proj_matrix(&self, aspect: f32) -> Mat4 {
        let fov_rad = self.fov_degrees * DEG_TO_RAD;
        glam::camera::lh::proj::directx::perspective(fov_rad, aspect, self.z_near, self.z_far)
    }

    /// Combined View-Projection Matrix
    pub fn view_proj_matrix(&self, aspect: f32) -> Mat4 {
        self.proj_matrix(aspect) * self.view_matrix()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::atlas::BlockType;

    struct MockBlockReader {
        solid_blocks: Vec<glam::IVec3>,
    }

    impl BlockReader for MockBlockReader {
        fn get_block_at(&self, x: i32, y: i32, z: i32) -> BlockType {
            if self.solid_blocks.contains(&glam::IVec3::new(x, y, z)) {
                BlockType::Stone
            } else {
                BlockType::Air
            }
        }
    }

    #[test]
    fn test_camera_initialization() {
        let cam = Camera::default();
        assert_eq!(cam.fov_degrees, 70.0);
        assert_eq!(cam.z_near, DEFAULT_Z_NEAR);
        assert_eq!(cam.mode(), CameraMode::FirstPerson);
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
    fn test_camera_mode_cycle() {
        let mut cam = Camera::default();
        assert_eq!(cam.mode, CameraMode::FirstPerson);
        assert_eq!(cam.cycle_mode(), CameraMode::ThirdPersonBack);
        assert_eq!(cam.cycle_mode(), CameraMode::ThirdPersonFront);
        assert_eq!(cam.cycle_mode(), CameraMode::FirstPerson);
    }

    #[test]
    fn test_first_person_setup() {
        let mut cam = Camera::default();
        let player_pos = Vec3::new(10.0, 64.0, 10.0);
        cam.setup(player_pos, None);
        assert_eq!(cam.eye, Vec3::new(10.0, 65.62, 10.0));
    }

    #[test]
    fn test_third_person_back_setup_unobstructed() {
        let mut cam = Camera::default();
        cam.set_mode(CameraMode::ThirdPersonBack);
        // Player looking along +Z (yaw = 0, pitch = 0)
        cam.set_rotation(0.0, 0.0);
        let player_pos = Vec3::new(0.0, 64.0, 0.0);
        cam.setup(player_pos, None);

        let eye_pos = Vec3::new(0.0, 65.62, 0.0);
        // 4 blocks behind along -Z
        assert!((cam.eye.z - (eye_pos.z - 4.0)).abs() < 1e-4);
        assert!((cam.eye.x - eye_pos.x).abs() < 1e-4);
        assert!((cam.eye.y - eye_pos.y).abs() < 1e-4);
    }

    #[test]
    fn test_third_person_front_setup_unobstructed() {
        let mut cam = Camera::default();
        cam.set_mode(CameraMode::ThirdPersonFront);
        // Player looking along +Z (yaw = 0, pitch = 0)
        cam.set_rotation(0.0, 0.0);
        let player_pos = Vec3::new(0.0, 64.0, 0.0);
        cam.setup(player_pos, None);

        let eye_pos = Vec3::new(0.0, 65.62, 0.0);
        // 4 blocks ahead along +Z
        assert!((cam.eye.z - (eye_pos.z + 4.0)).abs() < 1e-4);
        // Camera rotated 180 degrees looking backward along -Z
        assert!((cam.forwards().z - (-1.0)).abs() < 1e-4);
    }

    #[test]
    fn test_camera_occlusion_raycast_clamps_distance() {
        let mut cam = Camera::default();
        cam.set_mode(CameraMode::ThirdPersonBack);
        cam.set_rotation(0.0, 0.0); // Looking towards +Z, camera offsets along -Z

        // Place a solid wall at z = -3 (face is at z = -2.0; player eye is at (0, 65.62, 0))
        let mock_world = MockBlockReader {
            solid_blocks: vec![glam::IVec3::new(0, 65, -3)],
        };

        let player_pos = Vec3::new(0.0, 64.0, 0.0);
        cam.setup(player_pos, Some(&mock_world));

        // Unobstructed distance is 4.0, but wall face at z = -2.0 is 2.0 blocks away.
        // Raycast must clamp camera distance to 1.9 (2.0 - 0.1)
        assert!(cam.eye.z > -2.0, "Camera eye must stay in front of the wall");
        assert!((cam.eye.z - (-1.9)).abs() < 1e-4, "Camera must be clamped at z = -1.9");
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
