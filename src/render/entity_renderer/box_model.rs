// REA Verified: net.minecraft.client.model.geom.ModelPart (mapped: fmx.class)
// REA Verified: net.minecraft.client.model.geom.ModelPart$Cube (mapped: fmx$a.class)
// REA Verified: net.minecraft.client.model.HierarchicalModel (mapped: fkg.class)
//
// Hierarchical Box Model Tree & Forward Kinematics:
// - Minecraft 1/16th block unit coordinates (16 pixels = 1.0 block = 1 meter).
// - Tree hierarchy: Parent Part -> Child Part -> Child Part (Root -> Body -> Head, Limbs).
// - Forward Kinematics: Child transform = Parent transform * Local Pivot * Euler ZYX rotation * Scale.
// - Standard Minecraft Cube Face UV unwrapping across 6 faces with normal vectors.

use std::collections::HashMap;
use glam::{Mat4, Vec3, Vec4};

/// Direction of a cube face matching vanilla Minecraft Direction enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CubeFace {
    Up,
    Down,
    North,
    South,
    West,
    East,
}

impl CubeFace {
    pub fn normal(&self) -> [f32; 3] {
        match self {
            CubeFace::Up => [0.0, 1.0, 0.0],
            CubeFace::Down => [0.0, -1.0, 0.0],
            CubeFace::North => [0.0, 0.0, -1.0],
            CubeFace::South => [0.0, 0.0, 1.0],
            CubeFace::West => [-1.0, 0.0, 0.0],
            CubeFace::East => [1.0, 0.0, 0.0],
        }
    }
}

/// Represents a single face of a textured cube with 4 corner vertices and UV bounds
#[derive(Debug, Clone, PartialEq)]
pub struct FaceQuad {
    pub face: CubeFace,
    pub corners: [[f32; 3]; 4],
    pub uvs: [[f32; 2]; 4],
    pub normal: [f32; 3],
}

/// Definition of a textured rectangular box/cube within a model part
#[derive(Debug, Clone, PartialEq)]
pub struct CubeDefinition {
    /// Origin coordinate in pixels [x, y, z] relative to part pivot
    pub origin: [f32; 3],
    /// Dimensions in pixels [dx, dy, dz]
    pub dimensions: [f32; 3],
    /// Texture coordinate offset [u, v] in pixels
    pub tex_coord: [f32; 2],
    /// Total texture dimensions [w, h], e.g. [64.0, 64.0]
    pub tex_size: [f32; 2],
    /// Dilation (inflation/deflation) in pixels [gx, gy, gz]
    pub dilation: [f32; 3],
    /// Whether the texture is mirrored horizontally
    pub mirror: bool,
}

impl CubeDefinition {
    pub fn new(
        tex_u: f32,
        tex_v: f32,
        origin_x: f32,
        origin_y: f32,
        origin_z: f32,
        dx: f32,
        dy: f32,
        dz: f32,
        tex_w: f32,
        tex_h: f32,
    ) -> Self {
        Self {
            origin: [origin_x, origin_y, origin_z],
            dimensions: [dx, dy, dz],
            tex_coord: [tex_u, tex_v],
            tex_size: [tex_w, tex_h],
            dilation: [0.0, 0.0, 0.0],
            mirror: false,
        }
    }

    pub fn with_dilation(mut self, gx: f32, gy: f32, gz: f32) -> Self {
        self.dilation = [gx, gy, gz];
        self
    }

    pub fn with_mirror(mut self, mirror: bool) -> Self {
        self.mirror = mirror;
        self
    }

    /// Computes the 6 textured faces of the cube in model meters (1/16m per pixel)
    pub fn build_faces(&self, atlas_uv_rect: Option<[f32; 4]>) -> Vec<FaceQuad> {
        let x0 = (self.origin[0] - self.dilation[0]) / 16.0;
        let y0 = (self.origin[1] - self.dilation[1]) / 16.0;
        let z0 = (self.origin[2] - self.dilation[2]) / 16.0;

        let x1 = (self.origin[0] + self.dimensions[0] + self.dilation[0]) / 16.0;
        let y1 = (self.origin[1] + self.dimensions[1] + self.dilation[1]) / 16.0;
        let z1 = (self.origin[2] + self.dimensions[2] + self.dilation[2]) / 16.0;

        let dx = self.dimensions[0];
        let dy = self.dimensions[1];
        let dz = self.dimensions[2];

        let u0 = self.tex_coord[0];
        let v0 = self.tex_coord[1];
        let tw = self.tex_size[0];
        let th = self.tex_size[1];

        // Standard Minecraft Box UV Layout:
        // u coordinates:
        // u0: left of west
        // u1: u0 + dz (between west and north/up)
        // u2: u1 + dx (between north/up and east/down)
        // u3: u2 + dz (between east/down and south)
        // u4: u3 + dx (right of south)
        let u_west_left = u0;
        let u_north_left = u0 + dz;
        let u_east_left = u0 + dz + dx;
        let u_south_left = u0 + dz + dx + dz;
        let u_south_right = u0 + dz + dx + dz + dx;

        let v_top = v0;
        let v_mid = v0 + dz;
        let v_bottom = v0 + dz + dy;

        // Map local texture pixel [u, v] to normalized UV [0.0, 1.0] or into Atlas sub-rect
        let map_uv = |u: f32, v: f32| -> [f32; 2] {
            let local_u = u / tw;
            let local_v = v / th;
            if let Some(atlas) = atlas_uv_rect {
                let au_min = atlas[0];
                let av_min = atlas[1];
                let au_max = atlas[2];
                let av_max = atlas[3];
                [
                    au_min + local_u * (au_max - au_min),
                    av_min + local_v * (av_max - av_min),
                ]
            } else {
                [local_u, local_v]
            }
        };

        // Vertices for 8 corners:
        let p_000 = [x0, y0, z0]; // min
        let p_100 = [x1, y0, z0];
        let p_110 = [x1, y1, z0];
        let p_010 = [x0, y1, z0];
        let p_001 = [x0, y0, z1];
        let p_101 = [x1, y0, z1];
        let p_111 = [x1, y1, z1]; // max
        let p_011 = [x0, y1, z1];

        let mut faces = Vec::with_capacity(6);

        // 1. Up Face (+Y):
        // Corners: [p_110, p_010, p_011, p_111]
        // UV: [u_north_left, v_top] to [u_east_left, v_mid]
        faces.push(FaceQuad {
            face: CubeFace::Up,
            corners: [p_110, p_010, p_011, p_111],
            uvs: [
                map_uv(u_east_left, v_mid),
                map_uv(u_north_left, v_mid),
                map_uv(u_north_left, v_top),
                map_uv(u_east_left, v_top),
            ],
            normal: CubeFace::Up.normal(),
        });

        // 2. Down Face (-Y):
        // Corners: [p_101, p_001, p_000, p_100]
        // UV: [u_east_left, v_top] to [u_south_left, v_mid]
        faces.push(FaceQuad {
            face: CubeFace::Down,
            corners: [p_101, p_001, p_000, p_100],
            uvs: [
                map_uv(u_south_left, v_top),
                map_uv(u_east_left, v_top),
                map_uv(u_east_left, v_mid),
                map_uv(u_south_left, v_mid),
            ],
            normal: CubeFace::Down.normal(),
        });

        // 3. North Face (-Z):
        // Corners: [p_110, p_100, p_000, p_010]
        // UV: [u_north_left, v_mid] to [u_east_left, v_bottom]
        faces.push(FaceQuad {
            face: CubeFace::North,
            corners: [p_110, p_100, p_000, p_010],
            uvs: [
                map_uv(u_north_left, v_mid),
                map_uv(u_north_left, v_bottom),
                map_uv(u_east_left, v_bottom),
                map_uv(u_east_left, v_mid),
            ],
            normal: CubeFace::North.normal(),
        });

        // 4. South Face (+Z):
        // Corners: [p_011, p_001, p_101, p_111]
        // UV: [u_south_left, v_mid] to [u_south_right, v_bottom]
        faces.push(FaceQuad {
            face: CubeFace::South,
            corners: [p_011, p_001, p_101, p_111],
            uvs: [
                map_uv(u_south_left, v_mid),
                map_uv(u_south_left, v_bottom),
                map_uv(u_south_right, v_bottom),
                map_uv(u_south_right, v_mid),
            ],
            normal: CubeFace::South.normal(),
        });

        // 5. West Face (-X):
        // Corners: [p_010, p_000, p_001, p_011]
        // UV: [u_west_left, v_mid] to [u_north_left, v_bottom]
        faces.push(FaceQuad {
            face: CubeFace::West,
            corners: [p_010, p_000, p_001, p_011],
            uvs: [
                map_uv(u_west_left, v_mid),
                map_uv(u_west_left, v_bottom),
                map_uv(u_north_left, v_bottom),
                map_uv(u_north_left, v_mid),
            ],
            normal: CubeFace::West.normal(),
        });

        // 6. East Face (+X):
        // Corners: [p_111, p_101, p_100, p_110]
        // UV: [u_east_left, v_mid] to [u_south_left, v_bottom]
        faces.push(FaceQuad {
            face: CubeFace::East,
            corners: [p_111, p_101, p_100, p_110],
            uvs: [
                map_uv(u_east_left, v_mid),
                map_uv(u_east_left, v_bottom),
                map_uv(u_south_left, v_bottom),
                map_uv(u_south_left, v_mid),
            ],
            normal: CubeFace::East.normal(),
        });

        faces
    }
}

/// Initial pose / rest transformation for a ModelPart
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartPose {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub x_rot: f32,
    pub y_rot: f32,
    pub z_rot: f32,
}

impl PartPose {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        x_rot: 0.0,
        y_rot: 0.0,
        z_rot: 0.0,
    };

    pub fn offset_and_rotation(x: f32, y: f32, z: f32, x_rot: f32, y_rot: f32, z_rot: f32) -> Self {
        Self { x, y, z, x_rot, y_rot, z_rot }
    }

    pub fn offset(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z, x_rot: 0.0, y_rot: 0.0, z_rot: 0.0 }
    }

    pub fn rotation(x_rot: f32, y_rot: f32, z_rot: f32) -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0, x_rot, y_rot, z_rot }
    }
}

/// A node in the hierarchical model forward kinematics tree.
#[derive(Debug, Clone)]
pub struct ModelPart {
    /// Pivot position in pixels (1/16 block) relative to parent
    pub pivot: Vec3,
    /// Rotation angles in radians: pitch (x), yaw (y), roll (z)
    pub rotation: Vec3,
    /// Local scaling factor
    pub scale: Vec3,
    /// Visibility toggle
    pub visible: bool,
    /// Skip drawing cubes but still propagate transforms to children
    pub skip_draw: bool,
    /// Textured cubes belonging to this limb
    pub cubes: Vec<CubeDefinition>,
    /// Child limbs attached hierarchically to this limb
    pub children: HashMap<String, ModelPart>,
    /// Initial rest pose for resetting animations
    pub initial_pose: PartPose,
}

impl ModelPart {
    pub fn new(cubes: Vec<CubeDefinition>, children: HashMap<String, ModelPart>) -> Self {
        Self {
            pivot: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            visible: true,
            skip_draw: false,
            cubes,
            children,
            initial_pose: PartPose::ZERO,
        }
    }

    pub fn with_pose(mut self, pose: PartPose) -> Self {
        self.initial_pose = pose;
        self.load_pose(pose);
        self
    }

    /// Resets rotation, position, and scale to initial rest pose
    pub fn reset_pose(&mut self) {
        self.load_pose(self.initial_pose);
        for child in self.children.values_mut() {
            child.reset_pose();
        }
    }

    /// Loads a specific PartPose into active transforms
    pub fn load_pose(&mut self, pose: PartPose) {
        self.pivot = Vec3::new(pose.x, pose.y, pose.z);
        self.rotation = Vec3::new(pose.x_rot, pose.y_rot, pose.z_rot);
        self.scale = Vec3::ONE;
    }

    /// Sets the rotation of this limb (pitch, yaw, roll in radians)
    pub fn set_rotation(&mut self, x: f32, y: f32, z: f32) {
        self.rotation = Vec3::new(x, y, z);
    }

    /// Sets the translation pivot in pixels (1/16 block)
    pub fn set_pivot(&mut self, x: f32, y: f32, z: f32) {
        self.pivot = Vec3::new(x, y, z);
    }

    /// Look up child part by name
    pub fn get_child(&self, name: &str) -> Option<&ModelPart> {
        self.children.get(name)
    }

    /// Look up mutable child part by name
    pub fn get_child_mut(&mut self, name: &str) -> Option<&mut ModelPart> {
        self.children.get_mut(name)
    }

    /// Calculates local 4x4 transformation matrix:
    /// T(pivot/16) * R_zyx(roll, yaw, pitch) * S(scale)
    pub fn compute_local_matrix(&self) -> Mat4 {
        // Pivot in meters (pixels / 16.0)
        let translation = Mat4::from_translation(self.pivot / 16.0);
        // Authentic Minecraft rotation order: ZYX (Roll, then Yaw, then Pitch)
        let rotation = Mat4::from_euler(
            glam::EulerRot::ZYX,
            self.rotation.z,
            self.rotation.y,
            self.rotation.x,
        );
        let scale = Mat4::from_scale(self.scale);

        translation * rotation * scale
    }

    /// Forward Kinematics traversal: compiles all cubes into transformed vertices and indices
    pub fn compile_mesh<V, F>(
        &self,
        parent_transform: Mat4,
        atlas_uv_rect: Option<[f32; 4]>,
        color: [f32; 4],
        hurt_tint: f32,
        vertex_builder: &F,
        vertices: &mut Vec<V>,
        indices: &mut Vec<u32>,
    ) where
        F: Fn([f32; 3], [f32; 2], [f32; 3], [f32; 4], f32) -> V,
    {
        if !self.visible {
            return;
        }

        let current_transform = parent_transform * self.compute_local_matrix();

        // 1. Render all cubes attached directly to this part
        if !self.skip_draw {
            for cube in &self.cubes {
                let faces = cube.build_faces(atlas_uv_rect);
                for face in faces {
                    let base_idx = vertices.len() as u32;

                    // Transform normal vector using the 3x3 orientation
                    let n4 = current_transform * Vec4::new(face.normal[0], face.normal[1], face.normal[2], 0.0);
                    let normal = Vec3::new(n4.x, n4.y, n4.z).normalize_or_zero().to_array();

                    // Transform 4 corner vertices
                    for i in 0..4 {
                        let c = face.corners[i];
                        let v4 = current_transform * Vec4::new(c[0], c[1], c[2], 1.0);
                        let pos = [v4.x, v4.y, v4.z];
                        let uv = face.uvs[i];

                        vertices.push(vertex_builder(pos, uv, normal, color, hurt_tint));
                    }

                    // 2 Triangles for quad: [0, 1, 2, 0, 2, 3]
                    indices.extend_from_slice(&[
                        base_idx,
                        base_idx + 1,
                        base_idx + 2,
                        base_idx,
                        base_idx + 2,
                        base_idx + 3,
                    ]);
                }
            }
        }

        // 2. Recursively compile all children down the forward kinematics tree
        for child in self.children.values() {
            child.compile_mesh(
                current_transform,
                atlas_uv_rect,
                color,
                hurt_tint,
                vertex_builder,
                vertices,
                indices,
            );
        }
    }
}

/// Hierarchical Model trait matching Minecraft's `HierarchicalModel`
pub trait HierarchicalModel {
    /// Returns reference to the root ModelPart
    fn root(&self) -> &ModelPart;

    /// Returns mutable reference to the root ModelPart
    fn root_mut(&mut self) -> &mut ModelPart;

    /// Resets all limbs in the model to their default pose
    fn reset_all_poses(&mut self) {
        self.root_mut().reset_pose();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube_faces_and_uv_generation() {
        let cube = CubeDefinition::new(0.0, 0.0, -2.0, -2.0, -2.0, 4.0, 4.0, 4.0, 64.0, 32.0);
        let faces = cube.build_faces(None);
        assert_eq!(faces.len(), 6);

        // Verify normal orientations
        for face in &faces {
            let n = face.face.normal();
            assert_eq!(face.normal, n);
            assert_eq!(face.corners.len(), 4);
            assert_eq!(face.uvs.len(), 4);
        }
    }

    #[test]
    fn test_hierarchical_forward_kinematics() {
        // Root part at origin
        let mut root = ModelPart::new(vec![], HashMap::new());
        root.set_pivot(0.0, 16.0, 0.0); // 1.0 meter elevation

        // Child arm attached at +8 pixels (0.5m)
        let mut arm = ModelPart::new(
            vec![CubeDefinition::new(0.0, 0.0, 0.0, 0.0, 0.0, 4.0, 12.0, 4.0, 64.0, 64.0)],
            HashMap::new(),
        );
        arm.set_pivot(8.0, 0.0, 0.0);
        arm.set_rotation(std::f32::consts::FRAC_PI_2, 0.0, 0.0); // 90 deg pitch forward

        root.children.insert("arm".to_string(), arm);

        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        root.compile_mesh(
            Mat4::IDENTITY,
            None,
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            &|pos, _uv, _norm, _color, _hurt| pos,
            &mut vertices,
            &mut indices,
        );

        // 1 cube = 6 faces * 4 vertices = 24 vertices
        assert_eq!(vertices.len(), 24);
        assert_eq!(indices.len(), 36);

        // Verify that root translation (1.0m on Y) and arm translation (0.5m on X) were composed
        let avg_x: f32 = vertices.iter().map(|v| v[0]).sum::<f32>() / 24.0;
        let avg_y: f32 = vertices.iter().map(|v| v[1]).sum::<f32>() / 24.0;
        let avg_z: f32 = vertices.iter().map(|v| v[2]).sum::<f32>() / 24.0;
        assert!((avg_x - 0.625).abs() < 1e-3, "Arm should be shifted to x ~ 0.625m");
        assert!((avg_y - 0.875).abs() < 1e-3, "Arm should be elevated and rotated to y ~ 0.875m");
        assert!((avg_z - 0.375).abs() < 1e-3, "Arm should be extended along z ~ 0.375m");
    }
}
