// REA Verified: net.minecraft.client.model.AnimationUtils (mapped: fiw.class)
// REA Verified: net.minecraft.client.model.HumanoidModel (mapped: fkk.class)
// REA Verified: net.minecraft.client.model.QuadrupedModel (mapped: flh.class)
// REA Verified: net.minecraft.client.model.BreezeModel (mapped: fjg.class)
// REA Verified: net.minecraft.client.model.BatModel (mapped: fjb.class)
// REA Verified: net.minecraft.client.model.ParrotModel (mapped: fkx.class)
// REA Verified: net.minecraft.client.model.GhastModel (mapped: fkb.class)
//
// Procedural Animation Engine:
// - Walking: Authentic sinusoidal pendulum swing on legs and arms:
//   theta = cos(limb_pos * 0.6662) * 1.4 * limb_speed
// - Head tracking: Pitch and yaw interpolate to face player camera direction within biomechanical bounds.
// - Wing flapping: High-frequency aerial oscillations for Bat, Parrot, Ghast tentacles, Breeze wind rings.

use std::f32::consts::PI;
use glam::Vec3;

/// State inputs for driving procedural entity animations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntityAnimationState {
    /// Accumulated walk distance (drives cyclic leg/arm swinging)
    pub limb_pos: f32,
    /// Current horizontal locomotion speed in [0.0, 1.0]
    pub limb_speed: f32,
    /// Entity age in elapsed game ticks (drives idle/breathing/flapping)
    pub age_in_ticks: f32,
    /// Current head yaw in radians relative to mob body direction
    pub head_yaw: f32,
    /// Current head pitch in radians
    pub head_pitch: f32,
    /// Remaining hurt damage flash duration in seconds
    pub hurt_time: f32,
    /// Whether the entity is actively airborne/flying
    pub is_flying: bool,
    /// Whether the entity is perched/sleeping/resting
    pub is_resting: bool,
}

impl Default for EntityAnimationState {
    fn default() -> Self {
        Self {
            limb_pos: 0.0,
            limb_speed: 0.0,
            age_in_ticks: 0.0,
            head_yaw: 0.0,
            head_pitch: 0.0,
            hurt_time: 0.0,
            is_flying: false,
            is_resting: false,
        }
    }
}

/// Procedural Animation Engine implementing vanilla Minecraft kinematics
pub struct ProceduralAnimationEngine;

impl ProceduralAnimationEngine {
    /// Vanilla Minecraft limb swing frequency factor
    pub const WALK_FREQUENCY: f32 = 0.6662;
    /// Vanilla Minecraft leg pendulum swing amplitude multiplier
    pub const LEG_SWING_AMPLITUDE: f32 = 1.4;
    /// Degrees to radians factor (0.017453292)
    pub const DEG_TO_RAD: f32 = PI / 180.0;

    /// Calculates humanoid walking pendulum swing for right leg:
    /// theta = cos(limb_pos * 0.6662) * 1.4 * limb_speed
    #[inline]
    pub fn calculate_leg_swing(limb_pos: f32, limb_speed: f32) -> f32 {
        (limb_pos * Self::WALK_FREQUENCY).cos() * Self::LEG_SWING_AMPLITUDE * limb_speed
    }

    /// Calculates arm pendulum swing:
    /// In Minecraft, arms swing opposite to the corresponding leg (right arm swings with left leg).
    /// amplitude is 1.0 (2.0 * limb_speed * 0.5)
    #[inline]
    pub fn calculate_arm_swing(limb_pos: f32, limb_speed: f32) -> f32 {
        (limb_pos * Self::WALK_FREQUENCY).cos() * 1.0 * limb_speed
    }

    /// Procedural Head Tracking:
    /// Interpolates entity head pitch and yaw to gaze toward the target (e.g. player camera).
    /// Yaw is clamped to +/- 75 degrees (+/- 1.309 rad), Pitch clamped to +/- 45 degrees (+/- 0.785 rad).
    pub fn update_head_tracking(
        current_yaw: f32,
        current_pitch: f32,
        mob_pos: Vec3,
        mob_body_yaw: f32,
        target_pos: Vec3,
        smoothing: f32,
    ) -> (f32, f32) {
        let diff = target_pos - mob_pos;
        let dist_xz = (diff.x * diff.x + diff.z * diff.z).sqrt();

        if dist_xz < 1e-4 {
            return (current_yaw, current_pitch);
        }

        // World angle to target
        let world_target_yaw = diff.x.atan2(diff.z);
        // Relative yaw = target angle minus mob orientation
        let mut rel_yaw = world_target_yaw - mob_body_yaw;

        // Wrap to [-PI, PI]
        while rel_yaw > PI {
            rel_yaw -= 2.0 * PI;
        }
        while rel_yaw < -PI {
            rel_yaw += 2.0 * PI;
        }

        // Relative pitch (positive = looking down, negative = looking up in MC coords)
        let rel_pitch = -diff.y.atan2(dist_xz);

        // Biomechanical clamping
        let max_yaw = 75.0 * Self::DEG_TO_RAD; // 75 deg
        let max_pitch = 45.0 * Self::DEG_TO_RAD; // 45 deg

        let clamped_yaw = rel_yaw.clamp(-max_yaw, max_yaw);
        let clamped_pitch = rel_pitch.clamp(-max_pitch, max_pitch);

        // Smooth exponential interpolation
        let t = smoothing.clamp(0.0, 1.0);
        let next_yaw = current_yaw + (clamped_yaw - current_yaw) * t;
        let next_pitch = current_pitch + (clamped_pitch - current_pitch) * t;

        (next_yaw, next_pitch)
    }

    /// Bat Wing Flapping:
    /// High-frequency cyclic wing beats during flight:
    /// wing = cos(age * 1.3) * PI * 0.325 (~0.98 rad max deflection)
    /// When resting, wings fold tightly against body.
    pub fn calculate_bat_wings(age_in_ticks: f32, is_resting: bool) -> (f32, f32) {
        if is_resting {
            // Folded against body
            (-1.0, 1.0)
        } else {
            let flap = (age_in_ticks * 1.3).cos() * PI * 0.325;
            (flap, -flap)
        }
    }

    /// Parrot Wing Flapping:
    /// Flaps wings when airborne:
    /// roll_offset = 0.0873 + cos(age * 0.6) * 0.3 rad
    pub fn calculate_parrot_wings(age_in_ticks: f32, is_flying: bool) -> (f32, f32) {
        if is_flying {
            let flap = (age_in_ticks * 0.6).cos() * 0.3;
            (-0.0873 - flap, 0.0873 + flap)
        } else {
            (-0.0873, 0.0873)
        }
    }

    /// Ghast Tentacle Wave Oscillation:
    /// Authentic sinusoidal wave propagating across 9 tentacles (fkb.class):
    /// xRot = 0.2 * sin(age * 0.3 + i) + 0.4
    pub fn calculate_ghast_tentacle(age_in_ticks: f32, tentacle_idx: usize) -> f32 {
        0.2 * (age_in_ticks * 0.3 + tentacle_idx as f32).sin() + 0.4
    }

    /// Breeze Elemental Wind Rings:
    /// Atmospheric vortex orbiting around vertical body axis (fjg.class):
    /// theta = -age * PI * 0.1
    /// Top ring: [cos(theta) * 0.6, sin(theta) * 0.6]
    /// Mid ring: [sin(theta) * 0.4, cos(theta) * 0.8]
    /// Bottom ring: [cos(theta) * -0.25, sin(theta) * -0.25]
    pub fn calculate_breeze_rings(age_in_ticks: f32) -> ([f32; 2], [f32; 2], [f32; 2]) {
        let theta = -age_in_ticks * PI * 0.1;
        let top = [theta.cos() * 0.6, theta.sin() * 0.6];
        let mid = [theta.sin() * 0.4, theta.cos() * 0.8];
        let bottom = [theta.cos() * -0.25, theta.sin() * -0.25];
        (top, mid, bottom)
    }

    /// Hurt red tint flash multiplier:
    /// Returns 0.0 .. 1.0 factor based on remaining hurt_time
    #[inline]
    pub fn calculate_hurt_flash(hurt_time: f32) -> f32 {
        if hurt_time > 0.0 {
            // High visibility red pulse decaying smoothly
            (hurt_time * 2.5).min(1.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walking_pendulum_swing_formula() {
        let limb_pos = 0.0;
        let limb_speed = 1.0;

        // cos(0) * 1.4 * 1.0 = 1.4
        let swing = ProceduralAnimationEngine::calculate_leg_swing(limb_pos, limb_speed);
        assert!((swing - 1.4).abs() < 1e-4);

        // At quarter cycle: limb_pos = PI / (2.0 * 0.6662)
        let quarter_pos = PI / (2.0 * ProceduralAnimationEngine::WALK_FREQUENCY);
        let swing_zero = ProceduralAnimationEngine::calculate_leg_swing(quarter_pos, limb_speed);
        assert!(swing_zero.abs() < 1e-4);
    }

    #[test]
    fn test_head_tracking_clamping() {
        let mob_pos = Vec3::new(0.0, 0.0, 0.0);
        let mob_yaw = 0.0;
        // Target is far to the right (90 deg)
        let target_pos = Vec3::new(10.0, 0.0, 0.0);

        let (yaw, _pitch) = ProceduralAnimationEngine::update_head_tracking(
            0.0,
            0.0,
            mob_pos,
            mob_yaw,
            target_pos,
            1.0, // instant
        );

        // Yaw must be clamped to max 75 deg (1.309 rad)
        let max_yaw = 75.0 * ProceduralAnimationEngine::DEG_TO_RAD;
        assert!(yaw <= max_yaw + 1e-4, "Head yaw should clamp to 75 degrees");
    }

    #[test]
    fn test_wing_flapping_and_tentacles() {
        let (bat_r, bat_l) = ProceduralAnimationEngine::calculate_bat_wings(10.0, false);
        assert_eq!(bat_r, -bat_l);

        let t0 = ProceduralAnimationEngine::calculate_ghast_tentacle(0.0, 0);
        let t1 = ProceduralAnimationEngine::calculate_ghast_tentacle(0.0, 1);
        assert_ne!(t0, t1, "Adjacent tentacles should have phase shift");

        let (top, mid, bot) = ProceduralAnimationEngine::calculate_breeze_rings(5.0);
        assert_ne!(top, mid);
        assert_ne!(mid, bot);
    }
}
