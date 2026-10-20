// REA Verified: net.minecraft.world.entity.monster.Pillager (mapped: ccs.class)
// REA Verified: net.minecraft.world.entity.monster.Vindicator (mapped: cde.class)
// REA Verified: net.minecraft.world.entity.monster.Evoker (mapped: cci.class)
// REA Verified: net.minecraft.world.entity.monster.Vex (mapped: cdd.class)
// REA Verified: net.minecraft.world.entity.monster.Ravager (mapped: ccu.class)
//
// Illagers, Raid monsters, Evoker spells, Vex summoning, and Ravager concussive roars.

use super::projectiles::{ArrowEffect, ArrowProjectile, EvokerFangs};
use glam::Vec3;
use mc_items::ItemId;

/// Pillager mob (`ccs.class`) wielding a crossbow with reload countdowns.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PillagerMob {
    pub health: f32,
    pub max_health: f32,
    pub reload_ticks: u32,
    pub is_charged: bool,
}

impl Default for PillagerMob {
    fn default() -> Self {
        Self {
            health: 24.0,
            max_health: 24.0,
            reload_ticks: 0,
            is_charged: false,
        }
    }
}

impl PillagerMob {
    pub const CROSSBOW_RELOAD_TICKS: u32 = 25; // 1.25s reload

    pub fn new() -> Self {
        Self::default()
    }

    /// Ticks crossbow loading and firing.
    /// When loaded and target is in range, fires an ArrowProjectile.
    pub fn tick_attack(&mut self, current_pos: Vec3, target_pos: Vec3) -> Option<ArrowProjectile> {
        let diff = target_pos - current_pos;
        let dist = diff.length();

        if !self.is_charged {
            self.reload_ticks += 1;
            if self.reload_ticks >= Self::CROSSBOW_RELOAD_TICKS {
                self.is_charged = true;
                self.reload_ticks = 0;
            }
            return None;
        }

        // Fire when charged and target in range
        if dist <= 16.0 {
            self.is_charged = false;
            let arrow = ArrowProjectile::new(
                current_pos + Vec3::new(0.0, 1.4, 0.0),
                diff,
                2.2, // Fast crossbow bolt speed
                ArrowEffect::Normal,
                4.0,
            );
            return Some(arrow);
        }
        None
    }

    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::Arrow, ItemId::Arrow]
    }
}

/// Vindicator mob (`cde.class`) sprinting with iron axe.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VindicatorMob {
    pub health: f32,
    pub max_health: f32,
    pub is_johnny: bool, // Attacks all living mobs if named Johnny
    pub attack_cooldown: u32,
}

impl Default for VindicatorMob {
    fn default() -> Self {
        Self {
            health: 24.0,
            max_health: 24.0,
            is_johnny: false,
            attack_cooldown: 0,
        }
    }
}

impl VindicatorMob {
    pub const MELEE_DAMAGE: f32 = 13.0; // 6.5 hearts

    pub fn new() -> Self {
        Self::default()
    }

    pub fn johnny() -> Self {
        Self {
            is_johnny: true,
            ..Default::default()
        }
    }

    pub fn try_strike(&mut self, dist: f32) -> Option<f32> {
        if self.attack_cooldown == 0 && dist < 2.0 {
            self.attack_cooldown = 20;
            Some(Self::MELEE_DAMAGE)
        } else {
            None
        }
    }

    pub fn tick(&mut self) {
        if self.attack_cooldown > 0 {
            self.attack_cooldown -= 1;
        }
    }

    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::Emerald]
    }
}

/// Active spell casting mode for Evoker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EvokerSpell {
    #[default]
    None,
    Fangs,
    SummonVexes,
    Wololo,
}

/// Evoker mob (`cci.class`) casting fangs, vex summoning, and wololo sheep conversion.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EvokerMob {
    pub health: f32,
    pub max_health: f32,
    pub active_spell: EvokerSpell,
    pub spell_ticks: u32,
    pub spell_cooldown: u32,
}

impl Default for EvokerMob {
    fn default() -> Self {
        Self {
            health: 24.0,
            max_health: 24.0,
            active_spell: EvokerSpell::None,
            spell_ticks: 0,
            spell_cooldown: 0,
        }
    }
}

impl EvokerMob {
    pub const SPELL_CAST_DURATION: u32 = 40; // 2.0s casting time

    pub fn new() -> Self {
        Self::default()
    }

    /// Casts an Evoker Fangs line attack towards target.
    /// Creates 16 EvokerFangs surging forward in a straight line with staggered warmup.
    pub fn cast_fangs_line(&mut self, current_pos: Vec3, target_pos: Vec3) -> Vec<EvokerFangs> {
        self.spell_cooldown = 100;
        let diff = target_pos - current_pos;
        let dir = Vec3::new(diff.x, 0.0, diff.z).normalize_or_zero();

        let mut fangs = Vec::with_capacity(16);
        for i in 1..=16 {
            let pos = current_pos + dir * (i as f32 * 1.25);
            let warmup = (i as u32) * 2; // Staggered ripple effect
            fangs.push(EvokerFangs::new(pos, warmup));
        }
        fangs
    }

    /// Ticks spell cooldowns.
    pub fn tick(&mut self) {
        if self.spell_cooldown > 0 {
            self.spell_cooldown -= 1;
        }
    }

    /// Guaranteed drop: Totem of Undying, plus emeralds.
    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::TotemOfUndying, ItemId::Emerald]
    }
}

/// Vex mob (`cdd.class`) phasing through solid terrain and charging at targets.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VexMob {
    pub health: f32,
    pub max_health: f32,
    pub is_charging: bool,
    pub life_ticks: u32,
}

impl Default for VexMob {
    fn default() -> Self {
        Self {
            health: 14.0,
            max_health: 14.0,
            is_charging: false,
            life_ticks: 0,
        }
    }
}

impl VexMob {
    pub const MAX_LIFESPAN_TICKS: u32 = 1200; // 60s before decay
    pub const MELEE_DAMAGE: f32 = 9.0;

    pub fn new() -> Self {
        Self::default()
    }

    /// Computes flying velocity towards target, passing freely through walls (no-clip).
    pub fn compute_flying_velocity(&mut self, current_pos: Vec3, target_pos: Vec3) -> Vec3 {
        let diff = target_pos - current_pos;
        let dist = diff.length();

        if dist < 6.0 {
            self.is_charging = true;
            diff.normalize_or_zero() * 0.4 // Boosted charge velocity
        } else {
            self.is_charging = false;
            diff.normalize_or_zero() * 0.25
        }
    }

    /// Ticks life decay. Returns true when the vex should take decay damage.
    pub fn tick(&mut self) -> bool {
        self.life_ticks += 1;
        self.life_ticks >= Self::MAX_LIFESPAN_TICKS
    }
}

/// Ravager mob (`ccu.class`) with ram attack, shield stun, and concussive roar knockback.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RavagerMob {
    pub health: f32,
    pub max_health: f32,
    pub is_stunned: bool,
    pub stun_ticks_remaining: u32,
    pub is_roaring: bool,
    pub attack_cooldown: u32,
}

impl Default for RavagerMob {
    fn default() -> Self {
        Self {
            health: 100.0, // 50 hearts
            max_health: 100.0,
            is_stunned: false,
            stun_ticks_remaining: 0,
            is_roaring: false,
            attack_cooldown: 0,
        }
    }
}

/// Attack result from a Ravager concussive roar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RavagerRoarAttack {
    pub damage: f32,
    pub knockback_radius: f32,
    pub knockback_strength: f32,
}

impl RavagerMob {
    pub const RAM_DAMAGE: f32 = 12.0;
    pub const STUN_DURATION_TICKS: u32 = 40; // 2.0s stunned by shield

    pub fn new() -> Self {
        Self::default()
    }

    /// Triggered when an attack is blocked by a player shield.
    /// Stuns the Ravager for 40 ticks, priming the roar counter-attack!
    pub fn on_blocked_by_shield(&mut self) {
        self.is_stunned = true;
        self.stun_ticks_remaining = Self::STUN_DURATION_TICKS;
    }

    /// Ticks stun and roar timers. When waking from shield stun, unleashes a concussive roar!
    pub fn tick(&mut self) -> Option<RavagerRoarAttack> {
        if self.is_stunned && self.stun_ticks_remaining > 0 {
            self.stun_ticks_remaining -= 1;
            if self.stun_ticks_remaining == 0 {
                self.is_stunned = false;
                self.is_roaring = true;
                return Some(RavagerRoarAttack {
                    damage: 6.0,
                    knockback_radius: 4.0,
                    knockback_strength: 1.5,
                });
            }
        }
        if self.attack_cooldown > 0 {
            self.attack_cooldown -= 1;
        }
        None
    }

    /// Delivers direct melee ram attack.
    pub fn ram_attack(&mut self) -> Option<f32> {
        if !self.is_stunned && self.attack_cooldown == 0 {
            self.attack_cooldown = 20;
            Some(Self::RAM_DAMAGE)
        } else {
            None
        }
    }

    /// Drops Saddle upon death.
    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::Saddle]
    }
}
