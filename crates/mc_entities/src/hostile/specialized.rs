// REA Verified: net.minecraft.world.entity.monster.Guardian (mapped: ccl.class)
// REA Verified: net.minecraft.world.entity.monster.ElderGuardian (mapped: cce.class)
// REA Verified: net.minecraft.world.entity.monster.warden.Warden (mapped: cem.class)
// REA Verified: net.minecraft.world.entity.monster.breeze.Breeze (mapped: cdl.class)
//
// Deep Dark, Ocean Monument, and Trial Chamber Monsters:
// Guardian laser beam locking & thorns, Warden blindness & sonic boom,
// Breeze projectile reflection & wind charges, and Creaking eye-contact freezing.

use super::projectiles::WindChargeProjectile;
use crate::components::EntityId;
use glam::Vec3;
use mc_items::ItemId;

/// Guardian mob (`ccl.class`) and Elder Guardian (`cce.class`) with laser beam locking.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GuardianMob {
    pub health: f32,
    pub max_health: f32,
    pub is_elder: bool,
    pub beam_target: Option<EntityId>,
    pub beam_lock_ticks: u32,
    pub spikes_extended: bool,
}

impl Default for GuardianMob {
    fn default() -> Self {
        Self {
            health: 30.0,
            max_health: 30.0,
            is_elder: false,
            beam_target: None,
            beam_lock_ticks: 0,
            spikes_extended: true,
        }
    }
}

impl GuardianMob {
    pub const BEAM_LOCK_DURATION: u32 = 60; // 3.0s lock-on before beam strike
    pub const THORNS_DAMAGE: f32 = 2.0;

    pub fn guardian() -> Self {
        Self::default()
    }

    pub fn elder_guardian() -> Self {
        Self {
            health: 80.0,
            max_health: 80.0,
            is_elder: true,
            ..Default::default()
        }
    }

    /// Locks beam on target.
    pub fn start_beam_lock(&mut self, target: EntityId) {
        self.beam_target = Some(target);
        self.beam_lock_ticks = 0;
    }

    /// Ticks laser beam charge. When 60 ticks are reached, discharges magic damage:
    /// 9.0 damage for Guardian, 12.0 damage for Elder Guardian.
    pub fn tick_beam(&mut self) -> Option<f32> {
        if self.beam_target.is_some() {
            self.beam_lock_ticks += 1;
            if self.beam_lock_ticks >= Self::BEAM_LOCK_DURATION {
                self.beam_lock_ticks = 0;
                let dmg = if self.is_elder { 12.0 } else { 9.0 };
                return Some(dmg);
            }
        }
        None
    }

    /// Thorns contact damage when attacker hits extended spikes.
    pub fn on_melee_attacked(&self) -> f32 {
        if self.spikes_extended {
            Self::THORNS_DAMAGE
        } else {
            0.0
        }
    }

    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::PrismarineShard]
    }
}

/// Attack result from Warden Sonic Boom beam.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SonicBoomAttack {
    pub damage: f32,
    pub knockback: Vec3,
}

/// Warden mob (`cem.class`) with blindness, vibration hearing, and sonic boom attack.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WardenMob {
    pub health: f32,
    pub max_health: f32,
    pub is_blind: bool,
    pub anger_level: u32,
    pub suspect: Option<EntityId>,
    pub sonic_boom_charge_ticks: u32,
    pub sniff_cooldown: u32,
}

impl Default for WardenMob {
    fn default() -> Self {
        Self {
            health: 500.0, // 250 hearts
            max_health: 500.0,
            is_blind: true,
            anger_level: 0,
            suspect: None,
            sonic_boom_charge_ticks: 0,
            sniff_cooldown: 0,
        }
    }
}

impl WardenMob {
    pub const ANGER_ROAR_THRESHOLD: u32 = 80;
    pub const SONIC_BOOM_CHARGE_TICKS: u32 = 34; // 1.7s ribcage charging
    pub const SONIC_BOOM_DAMAGE: f32 = 10.0; // 5 hearts penetrating armor/shields

    pub fn new() -> Self {
        Self::default()
    }

    /// Hears a vibration in the world, increasing anger towards suspect.
    pub fn on_vibration(&mut self, entity: EntityId, strength: u32) {
        self.suspect = Some(entity);
        self.anger_level = (self.anger_level + strength * 10).min(150);
    }

    /// Sniffing detection: detects non-sneaking player within 6 blocks.
    pub fn try_sniff(&mut self, dist_to_player: f32, is_sneaking: bool) -> bool {
        if self.sniff_cooldown == 0 {
            self.sniff_cooldown = 80; // Sniff every 4.0s
            if dist_to_player <= 6.0 && !is_sneaking {
                self.anger_level = (self.anger_level + 35).min(150);
                return true;
            }
        }
        false
    }

    /// Charges and releases Sonic Boom penetrating beam attack.
    pub fn tick_sonic_boom(&mut self, current_pos: Vec3, target_pos: Vec3) -> Option<SonicBoomAttack> {
        let diff = target_pos - current_pos;
        let dist = diff.length();

        if dist > 16.0 {
            self.sonic_boom_charge_ticks = 0;
            return None;
        }

        self.sonic_boom_charge_ticks += 1;
        if self.sonic_boom_charge_ticks >= Self::SONIC_BOOM_CHARGE_TICKS {
            self.sonic_boom_charge_ticks = 0;
            let dir = diff.normalize_or_zero();
            Some(SonicBoomAttack {
                damage: Self::SONIC_BOOM_DAMAGE,
                knockback: dir * 1.8 + Vec3::new(0.0, 0.5, 0.0),
            })
        } else {
            None
        }
    }

    pub fn tick(&mut self) {
        if self.sniff_cooldown > 0 {
            self.sniff_cooldown -= 1;
        }
    }
}

/// Breeze mob (`cdl.class`) from Trial Chambers with wind leaping and projectile deflection.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BreezeMob {
    pub health: f32,
    pub max_health: f32,
    pub attack_cooldown: u32,
    pub leap_cooldown: u32,
}

impl Default for BreezeMob {
    fn default() -> Self {
        Self {
            health: 30.0,
            max_health: 30.0,
            attack_cooldown: 0,
            leap_cooldown: 0,
        }
    }
}

impl BreezeMob {
    pub fn new() -> Self {
        Self::default()
    }

    /// Projectile reflection: Breeze deflects all incoming arrows and projectiles!
    pub fn should_reflect_projectile() -> bool {
        true
    }

    /// Executes high wind leap in an arc around the target.
    pub fn try_wind_leap(&mut self, current_pos: Vec3, target_pos: Vec3) -> Option<Vec3> {
        if self.leap_cooldown == 0 {
            self.leap_cooldown = 60; // Leap every 3s
            let diff = target_pos - current_pos;
            let tangent = Vec3::new(-diff.z, 0.0, diff.x).normalize_or_zero() * 0.8;
            return Some(tangent + Vec3::new(0.0, 0.6, 0.0));
        }
        None
    }

    /// Shoots concussive Wind Charge projectile at target.
    pub fn shoot_wind_charge(&mut self, current_pos: Vec3, target_pos: Vec3) -> Option<WindChargeProjectile> {
        if self.attack_cooldown == 0 {
            self.attack_cooldown = 40;
            let diff = target_pos - current_pos;
            Some(WindChargeProjectile::new(current_pos, diff))
        } else {
            None
        }
    }

    pub fn tick(&mut self) {
        if self.attack_cooldown > 0 {
            self.attack_cooldown -= 1;
        }
        if self.leap_cooldown > 0 {
            self.leap_cooldown -= 1;
        }
    }

    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::BreezeRod]
    }
}

/// Creaking mob linked to a Creaking Heart block in Pale Garden.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreakingMob {
    pub health: f32,
    pub max_health: f32,
    pub heart_pos: Option<Vec3>,
    /// Whether the creaking is frozen still from direct player gaze.
    pub is_frozen: bool,
}

impl Default for CreakingMob {
    fn default() -> Self {
        Self {
            health: 1.0,
            max_health: 1.0,
            heart_pos: None,
            is_frozen: false,
        }
    }
}

impl CreakingMob {
    pub fn new(heart_pos: Vec3) -> Self {
        Self {
            heart_pos: Some(heart_pos),
            ..Default::default()
        }
    }

    /// Absolute invulnerability: direct attacks against the Creaking are absorbed!
    pub fn takes_damage_directly() -> bool {
        false
    }

    /// Eye contact freezing check:
    /// When a player is looking towards the creaking (dot product > 0.5 within view distance),
    /// the Creaking freezes completely in place!
    pub fn update_player_gaze(&mut self, creaking_pos: Vec3, player_pos: Vec3, player_look_dir: Vec3) -> bool {
        let to_creaking = (creaking_pos - player_pos).normalize_or_zero();
        let dot = player_look_dir.normalize_or_zero().dot(to_creaking);

        self.is_frozen = dot > 0.5;
        self.is_frozen
    }
}
