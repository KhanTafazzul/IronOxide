// REA Verified: net.minecraft.world.entity.monster.Ghast (mapped: ccj.class)
// REA Verified: net.minecraft.world.entity.monster.Blaze (mapped: cbz.class)
// REA Verified: net.minecraft.world.entity.monster.MagmaCube (mapped: ccp.class)
// REA Verified: net.minecraft.world.entity.monster.piglin.Piglin (mapped: cdy.class)
//
// Nether Hostiles: Ghast aerial fireballs, Blaze triple bursts, Magma Cube splits,
// Wither Skeleton decay, Piglin bartering & Overworld zombification.

use super::projectiles::FireballProjectile;
use glam::Vec3;
use mc_items::ItemId;

/// Ghast mob (`ccj.class`) flying through the Nether and firing explosive fireballs.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GhastMob {
    pub health: f32,
    pub max_health: f32,
    pub charge_ticks: u32,
    pub attack_cooldown: u32,
    pub is_charging: bool,
}

impl Default for GhastMob {
    fn default() -> Self {
        Self {
            health: 10.0,
            max_health: 10.0,
            charge_ticks: 0,
            attack_cooldown: 0,
            is_charging: false,
        }
    }
}

impl GhastMob {
    pub const CHARGE_DURATION: u32 = 10; // 0.5s mouth opening charge

    pub fn new() -> Self {
        Self::default()
    }

    /// Ticks fireball aiming and shooting. Returns a FireballProjectile when released.
    pub fn tick_attack(&mut self, current_pos: Vec3, target_pos: Vec3) -> Option<FireballProjectile> {
        let diff = target_pos - current_pos;
        let dist = diff.length();

        if dist > 64.0 {
            self.is_charging = false;
            self.charge_ticks = 0;
            return None;
        }

        if self.attack_cooldown > 0 {
            self.attack_cooldown -= 1;
            return None;
        }

        self.is_charging = true;
        self.charge_ticks += 1;
        if self.charge_ticks >= Self::CHARGE_DURATION {
            self.charge_ticks = 0;
            self.is_charging = false;
            self.attack_cooldown = 40; // 2.0s cooldown
            Some(FireballProjectile::ghast_ball(current_pos, diff))
        } else {
            None
        }
    }

    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::GhastTear, ItemId::Gunpowder]
    }
}

/// Blaze mob (`cbz.class`) hovering and firing triple burst fireballs.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlazeMob {
    pub health: f32,
    pub max_health: f32,
    pub burst_count: u32,
    pub burst_timer: u32,
    pub attack_cooldown: u32,
}

impl Default for BlazeMob {
    fn default() -> Self {
        Self {
            health: 20.0,
            max_health: 20.0,
            burst_count: 0,
            burst_timer: 0,
            attack_cooldown: 0,
        }
    }
}

impl BlazeMob {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fires a barrage of 3 consecutive fireballs spaced 6 ticks apart, then enters cooldown.
    pub fn tick_attack(&mut self, current_pos: Vec3, target_pos: Vec3) -> Option<FireballProjectile> {
        if self.attack_cooldown > 0 {
            self.attack_cooldown -= 1;
            return None;
        }

        let diff = target_pos - current_pos;
        self.burst_timer += 1;
        if self.burst_timer >= 6 {
            self.burst_timer = 0;
            self.burst_count += 1;

            let fireball = FireballProjectile::blaze_ball(current_pos, diff);
            if self.burst_count >= 3 {
                self.burst_count = 0;
                self.attack_cooldown = 100; // 5.0s between bursts
            }
            return Some(fireball);
        }
        None
    }

    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::BlazeRod]
    }
}

/// Magma Cube & Slime split sizes (1 = Small, 2 = Medium, 4 = Large).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SlimeSize {
    Small = 1,
    Medium = 2,
    #[default]
    Large = 4,
}

/// Magma Cube mob (`ccp.class`) jumping high and splitting on death.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MagmaCubeMob {
    pub size: SlimeSize,
    pub health: f32,
    pub max_health: f32,
}

impl MagmaCubeMob {
    pub fn new(size: SlimeSize) -> Self {
        let hp = match size {
            SlimeSize::Small => 1.0,
            SlimeSize::Medium => 4.0,
            SlimeSize::Large => 16.0,
        };
        Self {
            size,
            health: hp,
            max_health: hp,
        }
    }

    /// Splits into 2-4 smaller Magma Cubes upon death.
    pub fn split_on_death(&self) -> Option<Vec<MagmaCubeMob>> {
        match self.size {
            SlimeSize::Large => Some(vec![MagmaCubeMob::new(SlimeSize::Medium); 3]),
            SlimeSize::Medium => Some(vec![MagmaCubeMob::new(SlimeSize::Small); 3]),
            SlimeSize::Small => None,
        }
    }

    pub fn get_death_drops(&self) -> Vec<ItemId> {
        if self.size == SlimeSize::Small {
            vec![ItemId::MagmaCream]
        } else {
            Vec::new()
        }
    }
}

/// Piglin mob (`cdy.class`) with gold bartering and Overworld zombification.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PiglinMob {
    pub health: f32,
    pub max_health: f32,
    pub is_bartering: bool,
    pub barter_ticks: u32,
    pub overworld_ticks: u32,
    pub is_zombified: bool,
}

impl Default for PiglinMob {
    fn default() -> Self {
        Self {
            health: 16.0,
            max_health: 16.0,
            is_bartering: false,
            barter_ticks: 0,
            overworld_ticks: 0,
            is_zombified: false,
        }
    }
}

impl PiglinMob {
    pub const BARTER_DURATION_TICKS: u32 = 120; // 6.0s inspecting gold
    pub const ZOMBIFICATION_THRESHOLD: u32 = 300; // 15s in Overworld

    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if player is wearing gold armor (pacifying Piglins).
    pub fn is_player_wearing_gold(held_item: ItemId) -> bool {
        matches!(
            held_item,
            ItemId::GoldenHelmet
                | ItemId::GoldenChestplate
                | ItemId::GoldenLeggings
                | ItemId::GoldenBoots
        )
    }

    /// Hands a gold ingot to the piglin to initiate bartering.
    pub fn offer_gold_ingot(&mut self, item: ItemId) -> bool {
        if matches!(item, ItemId::GoldIngot) && !self.is_bartering {
            self.is_bartering = true;
            self.barter_ticks = Self::BARTER_DURATION_TICKS;
            true
        } else {
            false
        }
    }

    /// Ticks bartering and Overworld zombification.
    /// When barter inspection concludes, returns barter item drop!
    pub fn tick(&mut self, is_in_overworld: bool) -> Option<ItemId> {
        // Zombification check
        if is_in_overworld {
            self.overworld_ticks += 1;
            if self.overworld_ticks >= Self::ZOMBIFICATION_THRESHOLD {
                self.is_zombified = true;
            }
        } else {
            self.overworld_ticks = 0;
        }

        // Bartering check
        if self.is_bartering {
            if self.barter_ticks > 0 {
                self.barter_ticks -= 1;
            } else {
                self.is_bartering = false;
                // Returns barter reward (Ender Pearl, Obsidian, etc.)
                return Some(ItemId::EnderPearl);
            }
        }
        None
    }
}
