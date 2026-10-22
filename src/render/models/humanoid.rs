// REA Verified: net.minecraft.client.model.HumanoidModel (mapped: fkk.class)
// REA Verified: net.minecraft.client.model.ZombieModel (mapped: fmp.class)
//
// Hierarchical Humanoid Box Model & Limb Pendulum Animation:
// - 6 Body Parts: Head, Torso, Left Arm, Right Arm, Left Leg, Right Leg.
// - Authentic pivot hierarchy:
//   * Legs pivot at hips (y = 0.75m).
//   * Arms pivot at shoulders (y = 1.45m, x = +/- 0.375m).
//   * Head pivots at neck (y = 1.50m).
// - Simple limb pendulum animation based on entity walk speed:
//   * Right Leg: cos(limb_swing) * 1.2 * swing_amount
//   * Left Leg: -cos(limb_swing) * 1.2 * swing_amount
//   * Zombie Arms: Outstretched forward (-1.4 rad) + pendulum oscillation.

use glam::Vec3;
use crate::game::mobs::{MobEntity, MobType};
use crate::render::atlas::{Direction, TextureAtlas};
use crate::render::effects::EffectVertex;

/// Hierarchical Humanoid Model Generator
pub struct HumanoidModel;

impl HumanoidModel {
    /// Renders a single 3D box oriented with arbitrary pitch and yaw around a local pivot
    fn build_box(
        // Box half-extents: [half_x, half_y, half_z]
        extents: [f32; 3],
        // Center offset from limb pivot
        center_offset: Vec3,
        // Pivot point in mob local space (before entity yaw)
        pivot: Vec3,
        // Limb rotation: [pitch, yaw, roll]
        limb_rot: Vec3,
        // Entity root position in world space
        root_pos: Vec3,
        // Entity yaw rotation (around world Y axis)
        entity_yaw: f32,
        // Base color for the limb
        base_color: [f32; 4],
        white_uv: [f32; 4],
        vertices: &mut Vec<EffectVertex>,
        indices: &mut Vec<u32>,
    ) {
        let hx = extents[0];
        let hy = extents[1];
        let hz = extents[2];

        let cos_yaw = entity_yaw.cos();
        let sin_yaw = entity_yaw.sin();

        let cos_pitch = limb_rot.x.cos();
        let sin_pitch = limb_rot.x.sin();

        // Local 3D vertex transformer:
        // 1. Position corner relative to box center
        // 2. Rotate around limb pitch (X-axis) relative to pivot
        // 3. Rotate around entity yaw (Y-axis) in world space
        // 4. Translate by root position
        let transform_vertex = |lx: f32, ly: f32, lz: f32| -> [f32; 3] {
            // Box local pos relative to limb pivot
            let rel_x = lx + center_offset.x;
            let rel_y = ly + center_offset.y;
            let rel_z = lz + center_offset.z;

            // Pitch rotation around limb pivot (X axis)
            let pitched_y = rel_y * cos_pitch - rel_z * sin_pitch;
            let pitched_z = rel_y * sin_pitch + rel_z * cos_pitch;
            let pitched_x = rel_x;

            // Add pivot offset in mob local space
            let mob_local_x = pivot.x + pitched_x;
            let mob_local_y = pivot.y + pitched_y;
            let mob_local_z = pivot.z + pitched_z;

            // Yaw rotation around vertical axis
            let world_x = root_pos.x + (mob_local_x * cos_yaw - mob_local_z * sin_yaw);
            let world_y = root_pos.y + mob_local_y;
            let world_z = root_pos.z + (mob_local_x * sin_yaw + mob_local_z * cos_yaw);

            [world_x, world_y, world_z]
        };

        // 6 faces of the rectangular box with directional shading
        let faces = [
            // Direction, local corners [c0, c1, c2, c3], directional shade multiplier
            (
                Direction::Up,
                [[-hx, hy, hz], [hx, hy, hz], [hx, hy, -hz], [-hx, hy, -hz]],
                1.0f32,
            ),
            (
                Direction::Down,
                [[-hx, -hy, -hz], [hx, -hy, -hz], [hx, -hy, hz], [-hx, -hy, hz]],
                0.5f32,
            ),
            (
                Direction::North,
                [[hx, -hy, -hz], [-hx, -hy, -hz], [-hx, hy, -hz], [hx, hy, -hz]],
                0.8f32,
            ),
            (
                Direction::South,
                [[-hx, -hy, hz], [hx, -hy, hz], [hx, hy, hz], [-hx, hy, hz]],
                0.8f32,
            ),
            (
                Direction::West,
                [[-hx, -hy, -hz], [-hx, -hy, hz], [-hx, hy, hz], [-hx, hy, -hz]],
                0.6f32,
            ),
            (
                Direction::East,
                [[hx, -hy, hz], [hx, -hy, -hz], [hx, hy, -hz], [hx, hy, hz]],
                0.6f32,
            ),
        ];

        for (_dir, corners, shade) in faces {
            let color = [
                (base_color[0] * shade).clamp(0.0, 1.0),
                (base_color[1] * shade).clamp(0.0, 1.0),
                (base_color[2] * shade).clamp(0.0, 1.0),
                base_color[3],
            ];

            let p0 = transform_vertex(corners[0][0], corners[0][1], corners[0][2]);
            let p1 = transform_vertex(corners[1][0], corners[1][1], corners[1][2]);
            let p2 = transform_vertex(corners[2][0], corners[2][1], corners[2][2]);
            let p3 = transform_vertex(corners[3][0], corners[3][1], corners[3][2]);

            let base = vertices.len() as u32;
            vertices.push(EffectVertex::new(p0, [white_uv[0], white_uv[3]], color));
            vertices.push(EffectVertex::new(p1, [white_uv[2], white_uv[3]], color));
            vertices.push(EffectVertex::new(p2, [white_uv[2], white_uv[1]], color));
            vertices.push(EffectVertex::new(p3, [white_uv[0], white_uv[1]], color));

            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }

    /// Builds the complete hierarchical humanoid model for an entity
    pub fn build_mob_model(
        mob: &MobEntity,
        white_uv: [f32; 4],
        vertices: &mut Vec<EffectVertex>,
        indices: &mut Vec<u32>,
    ) {
        let root = mob.transform.position;
        let yaw = mob.transform.rotation.x;

        // Limb pendulum animation parameters:
        // Calculate walk cycle from accumulated walk distance
        let limb_swing = mob.ai.walk_distance * 4.0;
        let horiz_speed = (mob.velocity.value.x * mob.velocity.value.x + mob.velocity.value.z * mob.velocity.value.z).sqrt();
        let swing_amount = (horiz_speed * 5.0).clamp(0.0, 1.0);

        // Legs: pendulum swinging in opposite directions
        let right_leg_pitch = (limb_swing).cos() * 1.2 * swing_amount;
        let left_leg_pitch = -(limb_swing).cos() * 1.2 * swing_amount;

        // Zombie Arms: held outstretched forward (-1.4 rad) with pendulum sway
        let (right_arm_pitch, left_arm_pitch) = match mob.mob_type {
            MobType::Zombie => {
                let base_pitch = -1.35;
                let sway = (limb_swing).sin() * 0.15 * swing_amount;
                (base_pitch + sway, base_pitch - sway)
            }
            _ => (0.0, 0.0),
        };

        // Authentic Zombie palette
        let skin_color = [0.31, 0.53, 0.24, 1.0]; // Rotten Zombie Green
        let shirt_color = [0.18, 0.47, 0.53, 1.0]; // Cyan Shirt
        let pants_color = [0.24, 0.23, 0.45, 1.0]; // Indigo Pants

        // 1. Head: 0.5m x 0.5m x 0.5m (extents: [0.25, 0.25, 0.25])
        // Pivot at neck: y = 1.50m
        Self::build_box(
            [0.25, 0.25, 0.25],
            Vec3::new(0.0, 0.25, 0.0), // center offset
            Vec3::new(0.0, 1.50, 0.0), // pivot
            Vec3::new(mob.transform.rotation.y, 0.0, 0.0), // pitch
            root,
            yaw,
            skin_color,
            white_uv,
            vertices,
            indices,
        );

        // 2. Torso: 0.5m x 0.75m x 0.25m (extents: [0.25, 0.375, 0.125])
        // Positioned at y = 0.75m .. 1.50m
        Self::build_box(
            [0.25, 0.375, 0.125],
            Vec3::new(0.0, 0.375, 0.0),
            Vec3::new(0.0, 0.75, 0.0),
            Vec3::ZERO,
            root,
            yaw,
            shirt_color,
            white_uv,
            vertices,
            indices,
        );

        // 3. Right Arm: 0.25m x 0.75m x 0.25m (extents: [0.125, 0.375, 0.125])
        // Pivot at right shoulder: x = -0.375m, y = 1.45m
        Self::build_box(
            [0.125, 0.375, 0.125],
            Vec3::new(0.0, -0.375, 0.0),
            Vec3::new(-0.375, 1.45, 0.0),
            Vec3::new(right_arm_pitch, 0.0, 0.0),
            root,
            yaw,
            skin_color,
            white_uv,
            vertices,
            indices,
        );

        // 4. Left Arm: 0.25m x 0.75m x 0.25m
        // Pivot at left shoulder: x = +0.375m, y = 1.45m
        Self::build_box(
            [0.125, 0.375, 0.125],
            Vec3::new(0.0, -0.375, 0.0),
            Vec3::new(0.375, 1.45, 0.0),
            Vec3::new(left_arm_pitch, 0.0, 0.0),
            root,
            yaw,
            skin_color,
            white_uv,
            vertices,
            indices,
        );

        // 5. Right Leg: 0.25m x 0.75m x 0.25m
        // Pivot at right hip: x = -0.125m, y = 0.75m
        Self::build_box(
            [0.125, 0.375, 0.125],
            Vec3::new(0.0, -0.375, 0.0),
            Vec3::new(-0.125, 0.75, 0.0),
            Vec3::new(right_leg_pitch, 0.0, 0.0),
            root,
            yaw,
            pants_color,
            white_uv,
            vertices,
            indices,
        );

        // 6. Left Leg: 0.25m x 0.75m x 0.25m
        // Pivot at left hip: x = +0.125m, y = 0.75m
        Self::build_box(
            [0.125, 0.375, 0.125],
            Vec3::new(0.0, -0.375, 0.0),
            Vec3::new(0.125, 0.75, 0.0),
            Vec3::new(left_leg_pitch, 0.0, 0.0),
            root,
            yaw,
            pants_color,
            white_uv,
            vertices,
            indices,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanoid_model_part_quad_counts() {
        let mob = MobEntity::new_zombie(1, Vec3::new(0.0, 0.0, 0.0));
        let white_uv = [0.0, 0.0, 1.0, 1.0];
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        HumanoidModel::build_mob_model(&mob, white_uv, &mut vertices, &mut indices);

        // 6 body parts * 6 faces * 4 vertices = 144 vertices
        assert_eq!(vertices.len(), 144);
        // 6 body parts * 6 faces * 6 indices = 216 indices
        assert_eq!(indices.len(), 216);
    }

    #[test]
    fn test_limb_pendulum_animation_changes_with_walk() {
        let mut mob1 = MobEntity::new_zombie(1, Vec3::ZERO);
        mob1.velocity.value = Vec3::new(0.2, 0.0, 0.0);
        mob1.ai.walk_distance = 0.0;

        let mut mob2 = MobEntity::new_zombie(2, Vec3::ZERO);
        mob2.velocity.value = Vec3::new(0.2, 0.0, 0.0);
        mob2.ai.walk_distance = 1.0; // Walked forward

        let white_uv = [0.0, 0.0, 1.0, 1.0];
        let mut v1 = Vec::new();
        let mut i1 = Vec::new();
        HumanoidModel::build_mob_model(&mob1, white_uv, &mut v1, &mut i1);

        let mut v2 = Vec::new();
        let mut i2 = Vec::new();
        HumanoidModel::build_mob_model(&mob2, white_uv, &mut v2, &mut i2);

        // Right leg vertices (limb 5, starting at index 4 * 24 = 96) should have shifted due to pendulum swing
        assert_ne!(v1[96].pos, v2[96].pos, "Leg vertices should swing as walk distance advances");
    }
}
