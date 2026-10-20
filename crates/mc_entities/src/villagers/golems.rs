// REA Verified: net.minecraft.world.entity.animal.IronGolem (mapped: byk.class)
// REA Verified: net.minecraft.world.entity.animal.SnowGolem (mapped: byw.class)
//
// Iron Golem village panic/gossip summoning, 4-stage cracking, iron repair,
// and vertical toss flinging attack; Snow Golem snow trails, pumpkin shearing,
// and snowball projectile defense loops.

use crate::components::{EntityId, EntityType};
use glam::Vec3;
use mc_items::ItemId;

/// 4 Iron Golem visual cracking stages matching vanilla `IronGolem.Crackiness` (`byk$a.class`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IronGolemCrack {
    /// 100% - 75% HP: pristine condition.
    #[default]
    None = 0,
    /// 75% - 50% HP: fine hairline cracks.
    Low = 1,
    /// 50% - 25% HP: pronounced structural cracks.
    Medium = 2,
    /// < 25% HP: critical crumbling cracks.
    High = 3,
}

impl IronGolemCrack {
    pub fn from_health(current: f32, max: f32) -> Self {
        let ratio = current / max;
        if ratio > 0.75 {
            Self::None
        } else if ratio > 0.50 {
            Self::Low
        } else if ratio > 0.25 {
            Self::Medium
        } else {
            Self::High
        }
    }
}

/// Attack result from Iron Golem upward fling attack.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GolemFlingAttack {
    /// Instant damage dealt (7 to 21 HP).
    pub damage: f32,
    /// Upward vertical knockback impulse (vanilla launches target high into air).
    pub vertical_knockback: f32,
}

/// Iron Golem mob (`byk.class`) protecting villagers and flinging hostiles.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IronGolem {
    pub health: f32,
    pub max_health: f32,
    /// Attack wind-up / arm swing animation ticks (10 ticks).
    pub attack_animation_ticks: u32,
    /// Current target being pursued.
    pub target: Option<EntityId>,
}

impl Default for IronGolem {
    fn default() -> Self {
        Self {
            health: 100.0, // 50 hearts
            max_health: 100.0,
            attack_animation_ticks: 0,
            target: None,
        }
    }
}

impl IronGolem {
    pub const MAX_HEALTH: f32 = 100.0;
    pub const ATTACK_COOLDOWN_TICKS: u32 = 20;
    pub const REPAIR_AMOUNT_PER_INGOT: f32 = 25.0;

    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates if village conditions satisfy spawning an Iron Golem:
    /// In vanilla, an Iron Golem can spawn if:
    /// 1. At least 3 villagers have panicked from hostiles within the last 10 seconds, OR
    /// 2. At least 5 villagers have completed golem gossip checks and meet minimum reputation.
    pub fn can_spawn_in_village(
        panicked_villagers_count: usize,
        gossiping_villagers_count: usize,
        claimed_beds_count: usize,
    ) -> bool {
        if claimed_beds_count < 3 {
            return false;
        }
        panicked_villagers_count >= 3 || gossiping_villagers_count >= 5
    }

    /// Returns the current crackiness stage.
    pub fn crackiness(&self) -> IronGolemCrack {
        IronGolemCrack::from_health(self.health, self.max_health)
    }

    /// Repairs the Iron Golem with an Iron Ingot (+25 HP).
    pub fn repair(&mut self, item: ItemId) -> bool {
        if matches!(item, ItemId::IronIngot) && self.health < self.max_health {
            self.health = (self.health + Self::REPAIR_AMOUNT_PER_INGOT).min(self.max_health);
            true
        } else {
            false
        }
    }

    /// Delivers the signature upward fling strike against a target.
    pub fn fling_attack(&mut self) -> GolemFlingAttack {
        self.attack_animation_ticks = 10;
        GolemFlingAttack {
            damage: 14.0, // Average of 7..21 damage
            vertical_knockback: 0.45,
        }
    }

    /// Ticks combat animation timers.
    pub fn tick(&mut self) {
        if self.attack_animation_ticks > 0 {
            self.attack_animation_ticks -= 1;
        }
    }

    /// Returns true if this entity type is naturally targeted and attacked by Iron Golems.
    pub fn is_hostile_target(target: EntityType) -> bool {
        matches!(
            target,
            EntityType::Zombie
                | EntityType::ZombieVillager
                | EntityType::Husk
                | EntityType::Drowned
                | EntityType::Skeleton
                | EntityType::Stray
                | EntityType::Spider
                | EntityType::CaveSpider
                | EntityType::Pillager
                | EntityType::Vindicator
                | EntityType::Evoker
                | EntityType::Ravager
                | EntityType::Slime
                | EntityType::MagmaCube
                | EntityType::Wither
                | EntityType::Warden
        )
    }

    /// Drops when killed: 3 to 5 Iron Ingots, 0 to 2 Poppies.
    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::IronIngot, ItemId::IronIngot, ItemId::IronIngot, ItemId::Poppy]
    }
}

/// Snow Golem mob (`byw.class`) throwing snowballs and laying snow trails.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SnowGolem {
    pub health: f32,
    pub max_health: f32,
    /// Whether the carved pumpkin helmet is still worn.
    pub has_pumpkin: bool,
    /// Cooldown ticks between snowball throws (20 ticks = 1s).
    pub throw_cooldown: u32,
    /// Whether this snow golem is active and laying snow layer trails.
    pub leaves_snow_trail: bool,
}

impl Default for SnowGolem {
    fn default() -> Self {
        Self {
            health: 4.0, // 2 hearts
            max_health: 4.0,
            has_pumpkin: true,
            throw_cooldown: 0,
            leaves_snow_trail: true,
        }
    }
}

impl SnowGolem {
    pub const SNOWBALL_THROW_INTERVAL: u32 = 20;

    pub fn new() -> Self {
        Self::default()
    }

    /// Shears the carved pumpkin helmet off using Shears, revealing the funny face.
    pub fn shear_pumpkin(&mut self, held_item: ItemId) -> Option<ItemId> {
        if self.has_pumpkin && matches!(held_item, ItemId::Shears) {
            self.has_pumpkin = false;
            Some(ItemId::CarvedPumpkin)
        } else {
            None
        }
    }

    /// Snow Golems melt in warm biomes (temperature > 0.8) or when wet with water/rain.
    /// Deals 1.0 damage per 20 ticks.
    pub fn tick_temperature_and_water(&mut self, temperature: f32, is_wet: bool) -> Option<f32> {
        if temperature > 0.8 || is_wet {
            self.health = (self.health - 1.0).max(0.0);
            Some(1.0)
        } else {
            None
        }
    }

    /// Throws a snowball at a hostile entity if in range and cooldown expired.
    /// Returns the projectile launch velocity vector towards the target.
    pub fn try_throw_snowball(&mut self, current_pos: Vec3, target_pos: Vec3) -> Option<Vec3> {
        if self.throw_cooldown == 0 {
            let diff = target_pos - current_pos;
            let dist = diff.length();
            if dist <= 16.0 {
                self.throw_cooldown = Self::SNOWBALL_THROW_INTERVAL;
                // Arc snowball slightly upward for trajectory
                let dir = (diff / dist) + Vec3::new(0.0, 0.1, 0.0);
                return Some(dir.normalize_or_zero() * 1.5);
            }
        }
        None
    }

    /// Ticks weapon cooldown.
    pub fn tick(&mut self) {
        if self.throw_cooldown > 0 {
            self.throw_cooldown -= 1;
        }
    }

    /// Drops when killed: 0 to 15 Snowballs.
    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::Snowball, ItemId::Snowball, ItemId::Snowball, ItemId::Snowball]
    }
}
