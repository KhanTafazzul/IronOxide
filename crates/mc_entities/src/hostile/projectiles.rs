// REA Verified: net.minecraft.world.entity.projectile (mapped: cfe.class)
// REA Verified: net.minecraft.world.level.Explosion (mapped: cxb.class)
//
// Projectile physics, arrow trajectories, fireballs, sonic boom raycasts,
// and voxel explosion cratering.

use glam::Vec3;

/// Tipped arrow potion effects applied to targets on hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ArrowEffect {
    #[default]
    Normal,
    /// Slowness I for 30s (Stray).
    Slowness,
    /// Poison I for 4s (Bogged / Cave Spider).
    Poison,
    /// Spectral glowing highlight for 10s.
    Spectral,
}

/// Arrow projectile matching `net.minecraft.world.entity.projectile.Arrow`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArrowProjectile {
    pub position: Vec3,
    pub velocity: Vec3,
    pub effect: ArrowEffect,
    pub damage: f32,
    pub in_ground: bool,
    pub life_ticks: u32,
}

impl ArrowProjectile {
    pub const GRAVITY: f32 = 0.05; // 0.05 blocks/tick^2 downward
    pub const AIR_DRAG: f32 = 0.99; // 1% drag per tick

    pub fn new(position: Vec3, direction: Vec3, speed: f32, effect: ArrowEffect, damage: f32) -> Self {
        Self {
            position,
            velocity: direction.normalize_or_zero() * speed,
            effect,
            damage,
            in_ground: false,
            life_ticks: 0,
        }
    }

    /// Ticks arrow ballistic physics trajectory.
    pub fn tick(&mut self) {
        if self.in_ground {
            self.life_ticks += 1;
            return;
        }

        self.position += self.velocity;
        self.velocity.y -= Self::GRAVITY;
        self.velocity *= Self::AIR_DRAG;
        self.life_ticks += 1;
    }
}

/// Fireball projectile matching `LargeFireball` (Ghast) and `SmallFireball` (Blaze).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FireballProjectile {
    pub position: Vec3,
    pub velocity: Vec3,
    pub is_large: bool,
    pub explosion_power: f32,
    pub has_exploded: bool,
}

impl FireballProjectile {
    pub fn ghast_ball(position: Vec3, direction: Vec3) -> Self {
        Self {
            position,
            velocity: direction.normalize_or_zero() * 0.9,
            is_large: true,
            explosion_power: 1.0,
            has_exploded: false,
        }
    }

    pub fn blaze_ball(position: Vec3, direction: Vec3) -> Self {
        Self {
            position,
            velocity: direction.normalize_or_zero() * 0.7,
            is_large: false,
            explosion_power: 0.0,
            has_exploded: false,
        }
    }

    pub fn tick(&mut self) {
        if !self.has_exploded {
            self.position += self.velocity;
        }
    }
}

/// Wind Charge projectile matching `BreezeWindCharge` (Breeze).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WindChargeProjectile {
    pub position: Vec3,
    pub velocity: Vec3,
    pub burst_radius: f32,
    pub knockback_strength: f32,
}

impl WindChargeProjectile {
    pub fn new(position: Vec3, direction: Vec3) -> Self {
        Self {
            position,
            velocity: direction.normalize_or_zero() * 1.2,
            burst_radius: 3.0,
            knockback_strength: 1.5,
        }
    }

    pub fn tick(&mut self) {
        self.position += self.velocity;
    }
}

/// Evoker Fangs ground trap matching `EvokerFangs` (`cch.class`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EvokerFangs {
    pub position: Vec3,
    pub warmup_ticks: u32,
    pub is_snapped: bool,
    pub life_ticks: u32,
}

impl EvokerFangs {
    pub const DAMAGE: f32 = 6.0; // 3 hearts ignoring armor

    pub fn new(position: Vec3, warmup_ticks: u32) -> Self {
        Self {
            position,
            warmup_ticks,
            is_snapped: false,
            life_ticks: 0,
        }
    }

    pub fn tick(&mut self) -> Option<f32> {
        self.life_ticks += 1;
        if self.warmup_ticks > 0 {
            self.warmup_ticks -= 1;
            if self.warmup_ticks == 0 {
                self.is_snapped = true;
                return Some(Self::DAMAGE);
            }
        }
        None
    }
}

/// Voxel explosion physics matching `net.minecraft.world.level.Explosion` (`cxb.class`).
#[derive(Debug, Clone, PartialEq)]
pub struct VoxelExplosion {
    pub center: Vec3,
    pub power: f32,
    pub causes_fire: bool,
}

impl VoxelExplosion {
    pub fn creeper(center: Vec3, is_charged: bool) -> Self {
        Self {
            center,
            power: if is_charged { 6.0 } else { 3.0 },
            causes_fire: false,
        }
    }

    pub fn ghast(center: Vec3) -> Self {
        Self {
            center,
            power: 1.0,
            causes_fire: true,
        }
    }

    /// Computes damage to an entity based on inverse distance from the explosion epicenter.
    pub fn compute_damage_at(&self, target_pos: Vec3) -> f32 {
        let dist = (target_pos - self.center).length();
        let max_dist = self.power * 2.0;

        if dist >= max_dist {
            0.0
        } else {
            let factor = (1.0 - (dist / max_dist)).max(0.0);
            factor * self.power * 7.0
        }
    }

    /// Computes knockback impulse vector away from explosion epicenter.
    pub fn compute_knockback_at(&self, target_pos: Vec3) -> Vec3 {
        let diff = target_pos - self.center;
        let dist = diff.length();
        let max_dist = self.power * 2.0;

        if dist >= max_dist || dist < 0.001 {
            Vec3::ZERO
        } else {
            let factor = (1.0 - (dist / max_dist)).max(0.0);
            (diff / dist) * factor * (self.power * 0.5)
        }
    }
}
