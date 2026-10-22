// REA Verified: net.minecraft.client.model.HierarchicalModel (mapped: fkg.class)
// REA Verified: net.minecraft.client.model.ZombieModel (mapped: fmp.class)
// REA Verified: net.minecraft.client.model.QuadrupedModel (mapped: flh.class)
// REA Verified: net.minecraft.client.model.BreezeModel (mapped: fjg.class)
// REA Verified: net.minecraft.client.model.BatModel (mapped: fjb.class)
// REA Verified: net.minecraft.client.model.ParrotModel (mapped: fkx.class)
// REA Verified: net.minecraft.client.model.GhastModel (mapped: fkb.class)
//
// Mob Model Definitions:
// Hierarchical Box Model Tree definitions and runtime procedural animation handlers.

use std::collections::HashMap;
use super::animation::{EntityAnimationState, ProceduralAnimationEngine};
use super::box_model::{CubeDefinition, HierarchicalModel, ModelPart, PartPose};

// =========================================================================
// 1. ZOMBIE MODEL (fmp.class)
// =========================================================================

pub struct ZombieModel {
    pub root: ModelPart,
}

impl ZombieModel {
    pub fn new() -> Self {
        let tw = 64.0;
        let th = 64.0;

        // Head: 8x8x8 pixels at [-4, -8, -4], tex: [0, 0]
        let head = ModelPart::new(
            vec![CubeDefinition::new(0.0, 0.0, -4.0, -8.0, -4.0, 8.0, 8.0, 8.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(0.0, 0.0, 0.0));

        // Right Arm: 4x12x4 pixels at [-3, -2, -2], tex: [40, 16]
        let right_arm = ModelPart::new(
            vec![CubeDefinition::new(40.0, 16.0, -3.0, -2.0, -2.0, 4.0, 12.0, 4.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(-5.0, 2.0, 0.0));

        // Left Arm: 4x12x4 pixels at [-1, -2, -2], tex: [32, 48]
        let left_arm = ModelPart::new(
            vec![CubeDefinition::new(32.0, 48.0, -1.0, -2.0, -2.0, 4.0, 12.0, 4.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(5.0, 2.0, 0.0));

        // Body: 8x12x4 pixels at [-4, 0, -2], tex: [16, 16]
        let mut body_children = HashMap::new();
        body_children.insert("head".to_string(), head);
        body_children.insert("right_arm".to_string(), right_arm);
        body_children.insert("left_arm".to_string(), left_arm);

        let body = ModelPart::new(
            vec![CubeDefinition::new(16.0, 16.0, -4.0, 0.0, -2.0, 8.0, 12.0, 4.0, tw, th)],
            body_children,
        )
        .with_pose(PartPose::offset(0.0, 0.0, 0.0));

        // Right Leg: 4x12x4 pixels at [-2, 0, -2], tex: [0, 16]
        let right_leg = ModelPart::new(
            vec![CubeDefinition::new(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 12.0, 4.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(-1.9, 12.0, 0.0));

        // Left Leg: 4x12x4 pixels at [-2, 0, -2], tex: [16, 48]
        let left_leg = ModelPart::new(
            vec![CubeDefinition::new(16.0, 48.0, -2.0, 0.0, -2.0, 4.0, 12.0, 4.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(1.9, 12.0, 0.0));

        // Root attaches Body, Right Leg, Left Leg
        let mut root_children = HashMap::new();
        root_children.insert("body".to_string(), body);
        root_children.insert("right_leg".to_string(), right_leg);
        root_children.insert("left_leg".to_string(), left_leg);

        let root = ModelPart::new(vec![], root_children);

        Self { root }
    }

    pub fn setup_anim(&mut self, state: &EntityAnimationState) {
        self.reset_all_poses();

        let leg_swing = ProceduralAnimationEngine::calculate_leg_swing(state.limb_pos, state.limb_speed);

        // Legs: pendulum swinging in opposite phases
        if let Some(right_leg) = self.root.get_child_mut("right_leg") {
            right_leg.rotation.x = leg_swing;
        }
        if let Some(left_leg) = self.root.get_child_mut("left_leg") {
            left_leg.rotation.x = -leg_swing;
        }

        if let Some(body) = self.root.get_child_mut("body") {
            // Head tracking
            if let Some(head) = body.get_child_mut("head") {
                head.rotation.x = state.head_pitch;
                head.rotation.y = state.head_yaw;
            }

            // Zombie Arms: Outstretched forward (-1.35 rad) with sinusoidal walking oscillation
            let arm_sway = (state.limb_pos * ProceduralAnimationEngine::WALK_FREQUENCY).sin() * 0.15 * state.limb_speed;
            let zombie_pitch = -1.35;

            if let Some(right_arm) = body.get_child_mut("right_arm") {
                right_arm.rotation.x = zombie_pitch + arm_sway;
                right_arm.rotation.y = -0.1;
            }
            if let Some(left_arm) = body.get_child_mut("left_arm") {
                left_arm.rotation.x = zombie_pitch - arm_sway;
                left_arm.rotation.y = 0.1;
            }
        }
    }
}

impl HierarchicalModel for ZombieModel {
    fn root(&self) -> &ModelPart {
        &self.root
    }

    fn root_mut(&mut self) -> &mut ModelPart {
        &mut self.root
    }
}

// =========================================================================
// 2. QUADRUPED MODEL (flh.class - Pig, Cow, Sheep, Creeper)
// =========================================================================

pub struct QuadrupedModel {
    pub root: ModelPart,
}

impl QuadrupedModel {
    pub fn new() -> Self {
        let tw = 64.0;
        let th = 32.0;

        // Head: 8x8x8 at [-4, -4, -8], tex: [0, 0]
        let head = ModelPart::new(
            vec![CubeDefinition::new(0.0, 0.0, -4.0, -4.0, -8.0, 8.0, 8.0, 8.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(0.0, 12.0, -6.0));

        // Horizontal Body: 10x8x16 at [-5, -4, -8], tex: [28, 8]
        let body = ModelPart::new(
            vec![CubeDefinition::new(28.0, 8.0, -5.0, -4.0, -8.0, 10.0, 8.0, 16.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(0.0, 12.0, 0.0));

        // 4 Legs: 4x12x4 each, tex: [0, 16]
        let right_hind_leg = ModelPart::new(
            vec![CubeDefinition::new(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 12.0, 4.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(-3.0, 12.0, 7.0));

        let left_hind_leg = ModelPart::new(
            vec![CubeDefinition::new(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 12.0, 4.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(3.0, 12.0, 7.0));

        let right_front_leg = ModelPart::new(
            vec![CubeDefinition::new(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 12.0, 4.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(-3.0, 12.0, -5.0));

        let left_front_leg = ModelPart::new(
            vec![CubeDefinition::new(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 12.0, 4.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(3.0, 12.0, -5.0));

        let mut root_children = HashMap::new();
        root_children.insert("head".to_string(), head);
        root_children.insert("body".to_string(), body);
        root_children.insert("right_hind_leg".to_string(), right_hind_leg);
        root_children.insert("left_hind_leg".to_string(), left_hind_leg);
        root_children.insert("right_front_leg".to_string(), right_front_leg);
        root_children.insert("left_front_leg".to_string(), left_front_leg);

        let root = ModelPart::new(vec![], root_children);
        Self { root }
    }

    pub fn setup_anim(&mut self, state: &EntityAnimationState) {
        self.reset_all_poses();

        // Head tracking
        if let Some(head) = self.root.get_child_mut("head") {
            head.rotation.x = state.head_pitch;
            head.rotation.y = state.head_yaw;
        }

        // Diagonally synchronized quadruped walking cycle (flh.class):
        let swing = ProceduralAnimationEngine::calculate_leg_swing(state.limb_pos, state.limb_speed);

        if let Some(r_hind) = self.root.get_child_mut("right_hind_leg") {
            r_hind.rotation.x = swing;
        }
        if let Some(l_hind) = self.root.get_child_mut("left_hind_leg") {
            l_hind.rotation.x = -swing;
        }
        if let Some(r_front) = self.root.get_child_mut("right_front_leg") {
            r_front.rotation.x = -swing;
        }
        if let Some(l_front) = self.root.get_child_mut("left_front_leg") {
            l_front.rotation.x = swing;
        }
    }
}

impl HierarchicalModel for QuadrupedModel {
    fn root(&self) -> &ModelPart {
        &self.root
    }

    fn root_mut(&mut self) -> &mut ModelPart {
        &mut self.root
    }
}

// =========================================================================
// 3. BREEZE MODEL (fjg.class)
// =========================================================================

pub struct BreezeModel {
    pub root: ModelPart,
}

impl BreezeModel {
    pub fn new() -> Self {
        let tw = 32.0;
        let th = 32.0;

        // Head: floating cubic core [8x8x8]
        let head = ModelPart::new(
            vec![CubeDefinition::new(0.0, 0.0, -4.0, -8.0, -4.0, 8.0, 8.0, 8.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(0.0, 4.0, 0.0));

        // Core Rods/Body
        let rods = ModelPart::new(
            vec![CubeDefinition::new(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 8.0, 4.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(0.0, 4.0, 0.0));

        // 3 Elemental Wind Rings: Top, Mid, Bottom
        let wind_top = ModelPart::new(
            vec![CubeDefinition::new(0.0, 0.0, -5.0, 0.0, -5.0, 10.0, 2.0, 10.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(0.0, 12.0, 0.0));

        let wind_mid = ModelPart::new(
            vec![CubeDefinition::new(0.0, 0.0, -4.0, 0.0, -4.0, 8.0, 2.0, 8.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(0.0, 16.0, 0.0));

        let wind_bottom = ModelPart::new(
            vec![CubeDefinition::new(0.0, 0.0, -3.0, 0.0, -3.0, 6.0, 2.0, 6.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(0.0, 20.0, 0.0));

        let mut root_children = HashMap::new();
        root_children.insert("head".to_string(), head);
        root_children.insert("rods".to_string(), rods);
        root_children.insert("wind_top".to_string(), wind_top);
        root_children.insert("wind_mid".to_string(), wind_mid);
        root_children.insert("wind_bottom".to_string(), wind_bottom);

        let root = ModelPart::new(vec![], root_children);
        Self { root }
    }

    pub fn setup_anim(&mut self, state: &EntityAnimationState) {
        self.reset_all_poses();

        // Head tracking
        if let Some(head) = self.root.get_child_mut("head") {
            head.rotation.x = state.head_pitch;
            head.rotation.y = state.head_yaw;
        }

        // Rods rotate continuously
        if let Some(rods) = self.root.get_child_mut("rods") {
            rods.rotation.y = state.age_in_ticks * 0.1;
        }

        // Swirling wind vortex rings
        let (top, mid, bottom) = ProceduralAnimationEngine::calculate_breeze_rings(state.age_in_ticks);

        if let Some(ring) = self.root.get_child_mut("wind_top") {
            ring.pivot.x = top[0];
            ring.pivot.z = top[1];
        }
        if let Some(ring) = self.root.get_child_mut("wind_mid") {
            ring.pivot.x = mid[0];
            ring.pivot.z = mid[1];
        }
        if let Some(ring) = self.root.get_child_mut("wind_bottom") {
            ring.pivot.x = bottom[0];
            ring.pivot.z = bottom[1];
        }
    }
}

impl HierarchicalModel for BreezeModel {
    fn root(&self) -> &ModelPart {
        &self.root
    }

    fn root_mut(&mut self) -> &mut ModelPart {
        &mut self.root
    }
}

// =========================================================================
// 4. CREAKING MODEL (Pale Garden Hostile Mob)
// =========================================================================

pub struct CreakingModel {
    pub root: ModelPart,
}

impl CreakingModel {
    pub fn new() -> Self {
        let tw = 64.0;
        let th = 64.0;

        // Pale Oak Wood Head with jagged glowing eyes: 6x6x6
        let head = ModelPart::new(
            vec![CubeDefinition::new(0.0, 0.0, -3.0, -6.0, -3.0, 6.0, 6.0, 6.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(0.0, -2.0, 0.0));

        // Slender twisted wooden torso: 6x14x4
        let mut body_children = HashMap::new();
        body_children.insert("head".to_string(), head);

        // Long asymmetric branch arms: 3x16x3
        let right_arm = ModelPart::new(
            vec![CubeDefinition::new(40.0, 0.0, -2.0, 0.0, -1.5, 3.0, 16.0, 3.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(-4.0, 0.0, 0.0));

        let left_arm = ModelPart::new(
            vec![CubeDefinition::new(40.0, 0.0, -1.0, 0.0, -1.5, 3.0, 18.0, 3.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(4.0, 0.0, 0.0));

        body_children.insert("right_arm".to_string(), right_arm);
        body_children.insert("left_arm".to_string(), left_arm);

        let body = ModelPart::new(
            vec![CubeDefinition::new(16.0, 16.0, -3.0, 0.0, -2.0, 6.0, 14.0, 4.0, tw, th)],
            body_children,
        )
        .with_pose(PartPose::offset(0.0, -2.0, 0.0));

        // Root legs: 3x14x3
        let right_leg = ModelPart::new(
            vec![CubeDefinition::new(0.0, 16.0, -1.5, 0.0, -1.5, 3.0, 14.0, 3.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(-2.0, 12.0, 0.0));

        let left_leg = ModelPart::new(
            vec![CubeDefinition::new(0.0, 16.0, -1.5, 0.0, -1.5, 3.0, 14.0, 3.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(2.0, 12.0, 0.0));

        let mut root_children = HashMap::new();
        root_children.insert("body".to_string(), body);
        root_children.insert("right_leg".to_string(), right_leg);
        root_children.insert("left_leg".to_string(), left_leg);

        let root = ModelPart::new(vec![], root_children);
        Self { root }
    }

    pub fn setup_anim(&mut self, state: &EntityAnimationState) {
        self.reset_all_poses();

        // If resting / gaze-locked: Creaking freezes completely in rigid silence
        if state.is_resting {
            return;
        }

        // Jerky, eerie stalk animation:
        let leg_swing = ProceduralAnimationEngine::calculate_leg_swing(state.limb_pos, state.limb_speed);

        if let Some(right_leg) = self.root.get_child_mut("right_leg") {
            right_leg.rotation.x = leg_swing * 0.8;
        }
        if let Some(left_leg) = self.root.get_child_mut("left_leg") {
            left_leg.rotation.x = -leg_swing * 0.8;
        }

        if let Some(body) = self.root.get_child_mut("body") {
            // Eerie subtle torso twist
            body.rotation.y = (state.limb_pos * 0.333).sin() * 0.1 * state.limb_speed;

            if let Some(head) = body.get_child_mut("head") {
                head.rotation.x = state.head_pitch;
                head.rotation.y = state.head_yaw;
            }

            // Swaying dangling gnarled arms
            if let Some(right_arm) = body.get_child_mut("right_arm") {
                right_arm.rotation.x = -leg_swing * 0.6;
                right_arm.rotation.z = 0.1;
            }
            if let Some(left_arm) = body.get_child_mut("left_arm") {
                left_arm.rotation.x = leg_swing * 0.6;
                left_arm.rotation.z = -0.1;
            }
        }
    }
}

impl HierarchicalModel for CreakingModel {
    fn root(&self) -> &ModelPart {
        &self.root
    }

    fn root_mut(&mut self) -> &mut ModelPart {
        &mut self.root
    }
}

// =========================================================================
// 5. GHAST MODEL (fkb.class)
// =========================================================================

pub struct GhastModel {
    pub root: ModelPart,
}

impl GhastModel {
    pub fn new() -> Self {
        let tw = 64.0;
        let th = 64.0;

        // Massive floating cube: 16x16x16 at [-8, -16, -8], tex: [0, 0]
        let body = ModelPart::new(
            vec![CubeDefinition::new(0.0, 0.0, -8.0, -16.0, -8.0, 16.0, 16.0, 16.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(0.0, 16.0, 0.0));

        let mut root_children = HashMap::new();
        root_children.insert("body".to_string(), body);

        // 9 Hanging Tentacles in a 3x3 grid
        for i in 0..9 {
            let x = (((i % 3) as f32) - 1.0) * 5.0;
            let z = (((i / 3) as f32) - 1.0) * 5.0;
            let tentacle = ModelPart::new(
                vec![CubeDefinition::new(0.0, 0.0, -1.0, 0.0, -1.0, 2.0, 10.0, 2.0, tw, th)],
                HashMap::new(),
            )
            .with_pose(PartPose::offset(x, 16.0, z));

            root_children.insert(format!("tentacle_{i}"), tentacle);
        }

        let root = ModelPart::new(vec![], root_children);
        Self { root }
    }

    pub fn setup_anim(&mut self, state: &EntityAnimationState) {
        self.reset_all_poses();

        // 9 Tentacles wave in phase-shifted sinusoidal harmony
        for i in 0..9 {
            let name = format!("tentacle_{i}");
            if let Some(tentacle) = self.root.get_child_mut(&name) {
                tentacle.rotation.x = ProceduralAnimationEngine::calculate_ghast_tentacle(state.age_in_ticks, i);
            }
        }
    }
}

impl HierarchicalModel for GhastModel {
    fn root(&self) -> &ModelPart {
        &self.root
    }

    fn root_mut(&mut self) -> &mut ModelPart {
        &mut self.root
    }
}

// =========================================================================
// 6. BAT MODEL (fjb.class)
// =========================================================================

pub struct BatModel {
    pub root: ModelPart,
}

impl BatModel {
    pub fn new() -> Self {
        let tw = 64.0;
        let th = 64.0;

        let head = ModelPart::new(
            vec![CubeDefinition::new(0.0, 0.0, -3.0, -3.0, -3.0, 6.0, 6.0, 6.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(0.0, 16.0, 0.0));

        let body = ModelPart::new(
            vec![CubeDefinition::new(0.0, 34.0, -5.0, 0.0, -3.0, 10.0, 16.0, 6.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(0.0, 16.0, 0.0));

        let right_wing = ModelPart::new(
            vec![CubeDefinition::new(42.0, 0.0, -12.0, 1.0, 0.0, 10.0, 16.0, 1.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(0.0, 16.0, 0.0));

        let left_wing = ModelPart::new(
            vec![CubeDefinition::new(42.0, 0.0, 2.0, 1.0, 0.0, 10.0, 16.0, 1.0, tw, th).with_mirror(true)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(0.0, 16.0, 0.0));

        let mut root_children = HashMap::new();
        root_children.insert("head".to_string(), head);
        root_children.insert("body".to_string(), body);
        root_children.insert("right_wing".to_string(), right_wing);
        root_children.insert("left_wing".to_string(), left_wing);

        let root = ModelPart::new(vec![], root_children);
        Self { root }
    }

    pub fn setup_anim(&mut self, state: &EntityAnimationState) {
        self.reset_all_poses();

        if let Some(head) = self.root.get_child_mut("head") {
            head.rotation.x = state.head_pitch;
            head.rotation.y = state.head_yaw;
        }

        let (right_flap, left_flap) = ProceduralAnimationEngine::calculate_bat_wings(state.age_in_ticks, state.is_resting);
        if let Some(r_wing) = self.root.get_child_mut("right_wing") {
            r_wing.rotation.y = right_flap;
        }
        if let Some(l_wing) = self.root.get_child_mut("left_wing") {
            l_wing.rotation.y = left_flap;
        }
    }
}

impl HierarchicalModel for BatModel {
    fn root(&self) -> &ModelPart {
        &self.root
    }

    fn root_mut(&mut self) -> &mut ModelPart {
        &mut self.root
    }
}

// =========================================================================
// 7. PARROT MODEL (fkx.class)
// =========================================================================

pub struct ParrotModel {
    pub root: ModelPart,
}

impl ParrotModel {
    pub fn new() -> Self {
        let tw = 32.0;
        let th = 32.0;

        let head = ModelPart::new(
            vec![CubeDefinition::new(0.0, 0.0, -1.0, -1.5, -1.0, 2.0, 3.0, 2.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(0.0, 15.69, -2.76));

        let body = ModelPart::new(
            vec![CubeDefinition::new(2.0, 8.0, -1.5, 0.0, -1.5, 3.0, 6.0, 3.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(0.0, 16.5, -3.0));

        let right_wing = ModelPart::new(
            vec![CubeDefinition::new(19.0, 8.0, -0.5, 0.0, -1.5, 1.0, 5.0, 3.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(-1.5, 16.94, -2.76));

        let left_wing = ModelPart::new(
            vec![CubeDefinition::new(19.0, 8.0, -0.5, 0.0, -1.5, 1.0, 5.0, 3.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(1.5, 16.94, -2.76));

        let tail = ModelPart::new(
            vec![CubeDefinition::new(22.0, 1.0, -1.0, 0.0, -1.0, 2.0, 4.0, 1.0, tw, th)],
            HashMap::new(),
        )
        .with_pose(PartPose::offset(0.0, 21.07, 1.16));

        let mut root_children = HashMap::new();
        root_children.insert("head".to_string(), head);
        root_children.insert("body".to_string(), body);
        root_children.insert("right_wing".to_string(), right_wing);
        root_children.insert("left_wing".to_string(), left_wing);
        root_children.insert("tail".to_string(), tail);

        let root = ModelPart::new(vec![], root_children);
        Self { root }
    }

    pub fn setup_anim(&mut self, state: &EntityAnimationState) {
        self.reset_all_poses();

        if let Some(head) = self.root.get_child_mut("head") {
            head.rotation.x = state.head_pitch;
            head.rotation.y = state.head_yaw;
        }

        let (r_roll, l_roll) = ProceduralAnimationEngine::calculate_parrot_wings(state.age_in_ticks, state.is_flying);
        if let Some(r_wing) = self.root.get_child_mut("right_wing") {
            r_wing.rotation.z = r_roll;
        }
        if let Some(l_wing) = self.root.get_child_mut("left_wing") {
            l_wing.rotation.z = l_roll;
        }

        if let Some(tail) = self.root.get_child_mut("tail") {
            tail.rotation.x = 1.015 + (state.limb_pos * 0.6662).cos() * 0.3 * state.limb_speed;
        }
    }
}

impl HierarchicalModel for ParrotModel {
    fn root(&self) -> &ModelPart {
        &self.root
    }

    fn root_mut(&mut self) -> &mut ModelPart {
        &mut self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zombie_model_construction_and_limb_swing() {
        let mut model = ZombieModel::new();
        let state = EntityAnimationState {
            limb_pos: 1.0,
            limb_speed: 0.5,
            age_in_ticks: 20.0,
            head_yaw: 0.2,
            head_pitch: 0.1,
            ..Default::default()
        };

        model.setup_anim(&state);

        let r_leg = model.root.get_child("right_leg").unwrap();
        let l_leg = model.root.get_child("left_leg").unwrap();
        assert_eq!(r_leg.rotation.x, -l_leg.rotation.x, "Legs should swing in opposite directions");

        let body = model.root.get_child("body").unwrap();
        let head = body.get_child("head").unwrap();
        assert_eq!(head.rotation.y, 0.2);
        assert_eq!(head.rotation.x, 0.1);
    }

    #[test]
    fn test_quadruped_model_walking_diagonals() {
        let mut model = QuadrupedModel::new();
        let state = EntityAnimationState {
            limb_pos: 2.0,
            limb_speed: 1.0,
            ..Default::default()
        };

        model.setup_anim(&state);

        let r_hind = model.root.get_child("right_hind_leg").unwrap();
        let l_hind = model.root.get_child("left_hind_leg").unwrap();
        let r_front = model.root.get_child("right_front_leg").unwrap();
        let l_front = model.root.get_child("left_front_leg").unwrap();

        assert_eq!(r_hind.rotation.x, l_front.rotation.x, "Right hind and left front swing together");
        assert_eq!(l_hind.rotation.x, r_front.rotation.x, "Left hind and right front swing together");
    }

    #[test]
    fn test_ghast_tentacles_oscillation() {
        let mut model = GhastModel::new();
        let state = EntityAnimationState {
            age_in_ticks: 15.0,
            ..Default::default()
        };

        model.setup_anim(&state);

        let t0 = model.root.get_child("tentacle_0").unwrap();
        let t1 = model.root.get_child("tentacle_1").unwrap();
        assert_ne!(t0.rotation.x, t1.rotation.x);
    }
}
