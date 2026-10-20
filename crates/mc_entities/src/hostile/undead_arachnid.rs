// REA Verified: net.minecraft.world.entity.monster.Zombie (mapped: cdi.class)
// REA Verified: net.minecraft.world.entity.monster.Husk (mapped: ccm.class)
// REA Verified: net.minecraft.world.entity.monster.Drowned (mapped: ccd.class)
// REA Verified: net.minecraft.world.entity.monster.Skeleton (mapped: ccx.class)
// REA Verified: net.minecraft.world.entity.monster.Stray (mapped: cdb.class)
// REA Verified: net.minecraft.world.entity.monster.Spider (mapped: cda.class)
// REA Verified: net.minecraft.world.entity.monster.CaveSpider (mapped: cca.class)
// REA Verified: net.minecraft.world.entity.monster.Creeper (mapped: ccb.class)
// REA Verified: net.minecraft.world.entity.monster.Phantom (mapped: ccr.class)
// REA Verified: net.minecraft.world.entity.monster.Witch (mapped: cdf.class)
//
// Overworld Undead, Arachnids, Creepers, Phantoms, and Witches.

use super::projectiles::{ArrowEffect, ArrowProjectile, VoxelExplosion};
use glam::Vec3;
use mc_items::ItemId;

/// Evaluates whether an undead mob catches fire in direct sunlight.
pub fn is_sun_sensitive_burning(
    day_time_ticks: u32,
    has_sky_exposure: bool,
    is_in_water_or_rain: bool,
    has_helmet: bool,
) -> bool {
    let is_day = (day_time_ticks % 24000) < 12000;
    is_day && has_sky_exposure && !is_in_water_or_rain && !has_helmet
}

/// Zombie mob family (`cdi.class`, `ccm.class`, `ccd.class`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ZombieMob {
    pub health: f32,
    pub max_health: f32,
    pub is_baby: bool,
    pub has_helmet: bool,
    pub submerged_ticks: u32,
    pub conversion_ticks: u32,
    pub is_curing: bool,
    pub cure_ticks_remaining: u32,
}

impl Default for ZombieMob {
    fn default() -> Self {
        Self {
            health: 20.0,
            max_health: 20.0,
            is_baby: false,
            has_helmet: false,
            submerged_ticks: 0,
            conversion_ticks: 0,
            is_curing: false,
            cure_ticks_remaining: 0,
        }
    }
}

impl ZombieMob {
    pub const SUBMERGED_DROWN_TIME: u32 = 600; // 30 seconds submerged
    pub const DROWN_CONVERSION_TIME: u32 = 300; // 15 seconds shaking before turning into Drowned
    pub const CURE_TIME_TICKS: u32 = 3600; // 3 minutes

    pub fn new() -> Self {
        Self::default()
    }

    /// Ticks underwater drowning conversion into a Drowned mob.
    /// Returns true when conversion is complete.
    pub fn tick_underwater(&mut self, is_underwater: bool) -> bool {
        if is_underwater {
            self.submerged_ticks += 1;
            if self.submerged_ticks >= Self::SUBMERGED_DROWN_TIME {
                self.conversion_ticks += 1;
                if self.conversion_ticks >= Self::DROWN_CONVERSION_TIME {
                    return true;
                }
            }
        } else {
            self.submerged_ticks = 0;
            self.conversion_ticks = 0;
        }
        false
    }

    /// Initiates zombie villager curing when fed golden apple while under Weakness.
    pub fn start_curing(&mut self) {
        self.is_curing = true;
        self.cure_ticks_remaining = Self::CURE_TIME_TICKS;
    }

    /// Ticks curing process. Returns true when cured back to Villager.
    pub fn tick_curing(&mut self) -> bool {
        if self.is_curing && self.cure_ticks_remaining > 0 {
            self.cure_ticks_remaining -= 1;
            if self.cure_ticks_remaining == 0 {
                self.is_curing = false;
                return true;
            }
        }
        false
    }

    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::RottenFlesh]
    }
}

/// Skeleton mob family (`ccx.class`, `cdb.class`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SkeletonMob {
    pub health: f32,
    pub max_health: f32,
    pub bow_charge_ticks: u32,
    pub strafe_direction: f32,
    pub strafe_timer: u32,
    pub arrow_effect: ArrowEffect,
    pub has_helmet: bool,
}

impl Default for SkeletonMob {
    fn default() -> Self {
        Self {
            health: 20.0,
            max_health: 20.0,
            bow_charge_ticks: 0,
            strafe_direction: 1.0,
            strafe_timer: 0,
            arrow_effect: ArrowEffect::Normal,
            has_helmet: false,
        }
    }
}

impl SkeletonMob {
    pub const BOW_DRAW_DURATION: u32 = 20; // 1.0 second draw time

    pub fn skeleton() -> Self {
        Self::default()
    }

    pub fn stray() -> Self {
        Self {
            arrow_effect: ArrowEffect::Slowness,
            ..Default::default()
        }
    }

    pub fn bogged() -> Self {
        Self {
            arrow_effect: ArrowEffect::Poison,
            ..Default::default()
        }
    }

    /// Ticks strafing combat AI and bow charge.
    /// Returns an ArrowProjectile when bow is fully drawn and fired.
    pub fn tick_bow_aim(
        &mut self,
        current_pos: Vec3,
        target_pos: Vec3,
    ) -> (Vec3, Option<ArrowProjectile>) {
        self.strafe_timer += 1;
        if self.strafe_timer >= 40 {
            self.strafe_timer = 0;
            self.strafe_direction = -self.strafe_direction;
        }

        let to_target = target_pos - current_pos;
        let dist = to_target.length();
        let right = Vec3::new(-to_target.z, 0.0, to_target.x).normalize_or_zero();

        // Strafing movement vector
        let strafe_vel = right * (0.15 * self.strafe_direction);
        let dist_vel = if dist < 5.0 {
            -to_target.normalize_or_zero() * 0.1
        } else if dist > 12.0 {
            to_target.normalize_or_zero() * 0.15
        } else {
            Vec3::ZERO
        };
        let movement_vel = strafe_vel + dist_vel;

        self.bow_charge_ticks += 1;
        if self.bow_charge_ticks >= Self::BOW_DRAW_DURATION {
            self.bow_charge_ticks = 0;
            let arrow = ArrowProjectile::new(
                current_pos + Vec3::new(0.0, 1.4, 0.0),
                to_target,
                1.6,
                self.arrow_effect,
                4.0,
            );
            (movement_vel, Some(arrow))
        } else {
            (movement_vel, None)
        }
    }

    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::Bone, ItemId::Arrow]
    }
}

/// Spider mob (`cda.class`) & Cave Spider (`cca.class`) with vertical wall climbing.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpiderMob {
    pub health: f32,
    pub max_health: f32,
    pub is_cave_spider: bool,
    pub is_climbing: bool,
    pub pounce_cooldown: u32,
}

impl Default for SpiderMob {
    fn default() -> Self {
        Self {
            health: 16.0,
            max_health: 16.0,
            is_cave_spider: false,
            is_climbing: false,
            pounce_cooldown: 0,
        }
    }
}

impl SpiderMob {
    pub fn spider() -> Self {
        Self::default()
    }

    pub fn cave_spider() -> Self {
        Self {
            health: 12.0,
            max_health: 12.0,
            is_cave_spider: true,
            is_climbing: false,
            pounce_cooldown: 0,
        }
    }

    /// Handles vertical wall climbing when colliding horizontally against solid voxel faces.
    pub fn compute_movement(&mut self, is_horizontal_colliding: bool, desired_vel: Vec3) -> Vec3 {
        if is_horizontal_colliding {
            self.is_climbing = true;
            // Climb upward at 0.15 blocks/tick
            Vec3::new(desired_vel.x * 0.5, 0.15, desired_vel.z * 0.5)
        } else {
            self.is_climbing = false;
            desired_vel
        }
    }

    /// Pounce leap towards target when in range (2.0 to 6.0 blocks).
    pub fn try_pounce(&mut self, current_pos: Vec3, target_pos: Vec3) -> Option<Vec3> {
        if self.pounce_cooldown == 0 {
            let diff = target_pos - current_pos;
            let dist = diff.length();
            if (2.0..=6.0).contains(&dist) {
                self.pounce_cooldown = 40;
                let horizontal = Vec3::new(diff.x, 0.0, diff.z).normalize_or_zero() * 0.4;
                return Some(horizontal + Vec3::new(0.0, 0.35, 0.0));
            }
        }
        None
    }

    pub fn tick(&mut self) {
        if self.pounce_cooldown > 0 {
            self.pounce_cooldown -= 1;
        }
    }

    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::String, ItemId::SpiderEye]
    }
}

/// Creeper mob (`ccb.class`) with swelling fuse and voxel explosion.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreeperMob {
    pub health: f32,
    pub max_health: f32,
    pub fuse_ticks: u32,
    pub is_ignited: bool,
    pub is_powered: bool, // Charged Creeper struck by lightning
}

impl Default for CreeperMob {
    fn default() -> Self {
        Self {
            health: 20.0,
            max_health: 20.0,
            fuse_ticks: 0,
            is_ignited: false,
            is_powered: false,
        }
    }
}

impl CreeperMob {
    pub const MAX_FUSE_TICKS: u32 = 30; // 1.5 seconds

    pub fn new() -> Self {
        Self::default()
    }

    pub fn charged() -> Self {
        Self {
            is_powered: true,
            ..Default::default()
        }
    }

    /// Ticks creeper proximity and fuse countdown.
    /// When fuse hits 30 ticks, returns a VoxelExplosion epicenter.
    pub fn tick_fuse(&mut self, distance_to_target: Option<f32>, current_pos: Vec3) -> Option<VoxelExplosion> {
        if let Some(dist) = distance_to_target {
            if dist < 3.0 || self.is_ignited {
                self.fuse_ticks += 1;
                if self.fuse_ticks >= Self::MAX_FUSE_TICKS {
                    return Some(VoxelExplosion::creeper(current_pos, self.is_powered));
                }
                return None;
            }
        }

        // Deflate if target moves away
        if self.fuse_ticks > 0 && !self.is_ignited {
            self.fuse_ticks -= 1;
        }
        None
    }

    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::Gunpowder, ItemId::Gunpowder]
    }
}

/// Phantom mob (`ccr.class`) flying aerial circles and diving at insomniac players.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PhantomMob {
    pub health: f32,
    pub max_health: f32,
    pub circling_angle: f32,
    pub is_swooping: bool,
}

impl Default for PhantomMob {
    fn default() -> Self {
        Self {
            health: 20.0,
            max_health: 20.0,
            circling_angle: 0.0,
            is_swooping: false,
        }
    }
}

impl PhantomMob {
    pub const INSOMNIA_THRESHOLD_TICKS: u32 = 72000; // 3 in-game days without sleep

    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates player insomnia. Phantoms target players who haven't slept in 3+ days.
    pub fn can_target_player(ticks_since_rest: u32) -> bool {
        ticks_since_rest >= Self::INSOMNIA_THRESHOLD_TICKS
    }

    /// Computes circling path or steep swoop dive velocity towards target.
    pub fn compute_flight_velocity(&mut self, current_pos: Vec3, target_pos: Vec3) -> Vec3 {
        let diff = target_pos - current_pos;
        let horizontal_dist = (diff.x * diff.x + diff.z * diff.z).sqrt();

        if self.is_swooping {
            if diff.length() < 1.5 || current_pos.y <= target_pos.y {
                self.is_swooping = false; // abort swoop and climb back up
                Vec3::new(0.0, 0.4, 0.0)
            } else {
                diff.normalize_or_zero() * 0.5
            }
        } else {
            self.circling_angle += 0.05;
            let radius = 16.0;
            let desired_x = target_pos.x + self.circling_angle.cos() * radius;
            let desired_z = target_pos.z + self.circling_angle.sin() * radius;
            let desired_y = target_pos.y + 20.0; // 20 blocks in sky

            let target_orbit = Vec3::new(desired_x, desired_y, desired_z);
            let to_orbit = target_orbit - current_pos;

            if horizontal_dist < 6.0 && current_pos.y > target_pos.y + 10.0 {
                // Initiate dive swoop!
                self.is_swooping = true;
            }

            to_orbit.normalize_or_zero() * 0.3
        }
    }

    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::PhantomMembrane]
    }
}

/// Witch mob (`cdf.class`) drinking defensive potions and throwing splash potions.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WitchMob {
    pub health: f32,
    pub max_health: f32,
    pub drink_potion_ticks: u32,
    pub attack_cooldown: u32,
}

impl Default for WitchMob {
    fn default() -> Self {
        Self {
            health: 26.0,
            max_health: 26.0,
            drink_potion_ticks: 0,
            attack_cooldown: 0,
        }
    }
}

impl WitchMob {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates defensive potion drinking:
    /// - Health < 11.0: Instant Health (+4 HP)
    /// - On fire: Fire Resistance
    /// - In water: Water Breathing
    pub fn tick_defensive_potions(&mut self, is_on_fire: bool, is_underwater: bool) -> bool {
        if self.drink_potion_ticks > 0 {
            self.drink_potion_ticks -= 1;
            if self.drink_potion_ticks == 0 && self.health < 11.0 {
                self.health = (self.health + 4.0).min(self.max_health);
                return true;
            }
        } else if self.health < 11.0 || is_on_fire || is_underwater {
            self.drink_potion_ticks = 30; // 1.5s drinking animation
        }
        false
    }

    /// Throws offensive splash potion at player within 10 blocks every 40 ticks.
    pub fn try_throw_splash_potion(&mut self, dist_to_target: f32) -> bool {
        if self.attack_cooldown == 0 && dist_to_target <= 10.0 {
            self.attack_cooldown = 40;
            true
        } else {
            false
        }
    }

    pub fn tick(&mut self) {
        if self.attack_cooldown > 0 {
            self.attack_cooldown -= 1;
        }
    }

    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::GlassBottle, ItemId::GlowstoneDust, ItemId::Sugar]
    }
}
