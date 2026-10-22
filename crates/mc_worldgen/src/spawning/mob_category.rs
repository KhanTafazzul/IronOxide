//! REA Verified: net.minecraft.world.entity.MobCategory (mapped: bmo.class)
//! REA Verified: net.minecraft.world.level.NaturalSpawner (mapped: cua.class)
//!
//! Vanilla Minecraft 1.20+ mob classification categories, global caps, and lifecycle rules.

use mc_entities::EntityType;
use std::collections::HashMap;

/// Minecraft 1.20+ / 1.21 canonical mob spawning categories.
/// REA Reference: `bmo.class` (lines 7-18)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MobCategory {
    /// Hostile mobs (Zombies, Skeletons, Creepers, etc.)
    Monster,
    /// Passive land animals (Cows, Sheep, Pigs, Chickens, etc.)
    Creature,
    /// Flying ambient mobs (Bats)
    Ambient,
    /// Axolotls (specialized cave water fauna)
    Axolotls,
    /// Underground aquatic life (Glow Squids)
    UndergroundWaterCreature,
    /// Swimming aquatic creatures (Squids, Dolphins)
    WaterCreature,
    /// Small ambient fish (Cod, Salmon, Pufferfish, Tropical Fish, Tadpoles)
    WaterAmbient,
    /// Special un-spawned or custom entities (Iron Golems, Villagers, Items)
    Misc,
}

/// Vanilla constant: Number of chunks in a (2 * 8 + 1) = 17x17 chunk square around player.
/// REA Reference: `cua.e` = `Math.pow(17.0, 2.0)` = 289.
pub const MAGIC_NUMBER: usize = 289;

/// Minimum distance in blocks between any player and a candidate mob spawn position.
/// REA Reference: `cua.d` = 24.
pub const MIN_SPAWN_DISTANCE: f32 = 24.0;
pub const MIN_SPAWN_DISTANCE_SQ: f32 = MIN_SPAWN_DISTANCE * MIN_SPAWN_DISTANCE; // 576.0

/// Radius in blocks within which mobs never naturally despawn based on age.
/// REA Reference: `bmo.g()` = 32.
pub const NO_DESPAWN_DISTANCE: f32 = 32.0;
pub const NO_DESPAWN_DISTANCE_SQ: f32 = NO_DESPAWN_DISTANCE * NO_DESPAWN_DISTANCE; // 1024.0

/// Maximum distance in blocks beyond which mobs instantly despawn.
/// REA Reference: `bmo.f()` = 128 (64 for WaterAmbient).
pub const DEFAULT_DESPAWN_DISTANCE: f32 = 128.0;
pub const DEFAULT_DESPAWN_DISTANCE_SQ: f32 = DEFAULT_DESPAWN_DISTANCE * DEFAULT_DESPAWN_DISTANCE; // 16384.0

/// Minimum lifetime ticks before random 1/800 despawn rolls activate (30 seconds = 600 ticks).
pub const DESPAWN_AGE_THRESHOLD: u32 = 600;

/// Odds denominator for despawn roll when mob is between 32 and 128 blocks away.
pub const DESPAWN_CHANCE_1_IN: u32 = 800;

impl MobCategory {
    /// Canonical category string identifier matching vanilla JSON schema.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Monster => "monster",
            Self::Creature => "creature",
            Self::Ambient => "ambient",
            Self::Axolotls => "axolotls",
            Self::UndergroundWaterCreature => "underground_water_creature",
            Self::WaterCreature => "water_creature",
            Self::WaterAmbient => "water_ambient",
            Self::Misc => "misc",
        }
    }

    /// Resolves category from vanilla JSON key name.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "monster" => Some(Self::Monster),
            "creature" => Some(Self::Creature),
            "ambient" => Some(Self::Ambient),
            "axolotls" => Some(Self::Axolotls),
            "underground_water_creature" => Some(Self::UndergroundWaterCreature),
            "water_creature" => Some(Self::WaterCreature),
            "water_ambient" => Some(Self::WaterAmbient),
            "misc" => Some(Self::Misc),
            _ => None,
        }
    }

    /// Standard per-player base mob cap constant.
    /// REA Reference: `bmo.<init>` parameters:
    /// - Monster: 70
    /// - Creature: 10
    /// - Ambient: 15
    /// - Axolotls: 5
    /// - UndergroundWaterCreature: 5
    /// - WaterCreature: 5
    /// - WaterAmbient: 20
    /// - Misc: -1 (uncapped / manual)
    pub const fn base_cap(&self) -> i32 {
        match self {
            Self::Monster => 70,
            Self::Creature => 10,
            Self::Ambient => 15,
            Self::Axolotls => 5,
            Self::UndergroundWaterCreature => 5,
            Self::WaterCreature => 5,
            Self::WaterAmbient => 20,
            Self::Misc => -1,
        }
    }

    /// Whether this category represents peaceful / friendly mobs.
    /// REA Reference: `bmo.d()`
    pub const fn is_friendly(&self) -> bool {
        match self {
            Self::Monster => false,
            _ => true,
        }
    }

    /// Whether mobs in this category are persistent by default.
    /// REA Reference: `bmo.e()`
    pub const fn is_persistent(&self) -> bool {
        match self {
            Self::Creature | Self::Misc => true,
            _ => false,
        }
    }

    /// Distance in blocks at which mob instantly despawns.
    /// REA Reference: `bmo.f()`
    pub const fn despawn_distance(&self) -> f32 {
        match self {
            Self::WaterAmbient => 64.0,
            _ => 128.0,
        }
    }

    /// Squared despawn distance.
    pub const fn despawn_distance_sq(&self) -> f32 {
        let d = self.despawn_distance();
        d * d
    }

    /// Safe distance from player where mobs never naturally despawn.
    pub const fn no_despawn_distance(&self) -> f32 {
        NO_DESPAWN_DISTANCE
    }

    /// Spawning cycle interval in ticks:
    /// - Monsters: every 1 tick
    /// - Creatures: every 400 ticks (20 seconds)
    /// - Ambient & Water: every 1 tick
    pub const fn spawn_period_ticks(&self) -> u64 {
        match self {
            Self::Creature => 400,
            Self::Misc => 0,
            _ => 1,
        }
    }

    /// Checks if this category is scheduled to execute a spawning attempt on the given tick.
    pub fn should_spawn_on_tick(&self, tick: u64) -> bool {
        let period = self.spawn_period_ticks();
        if period == 0 {
            false
        } else {
            tick % period == 0
        }
    }
}

/// Classifies any vanilla `EntityType` into its official `MobCategory`.
pub fn category_for_entity(entity_type: EntityType) -> MobCategory {
    match entity_type {
        // Ambient
        EntityType::Bat => MobCategory::Ambient,

        // Water Ambient (fish & tadpoles)
        EntityType::Cod
        | EntityType::Salmon
        | EntityType::Pufferfish
        | EntityType::TropicalFish
        | EntityType::Tadpole => MobCategory::WaterAmbient,

        // Underground Water Creatures
        EntityType::GlowSquid => MobCategory::UndergroundWaterCreature,

        // Axolotls
        EntityType::Axolotl => MobCategory::Axolotls,

        // Water Creatures
        EntityType::Squid | EntityType::Dolphin => MobCategory::WaterCreature,

        // Passive Creatures / Farm & Wild Animals
        EntityType::Cow
        | EntityType::Sheep
        | EntityType::Pig
        | EntityType::Chicken
        | EntityType::Horse
        | EntityType::Donkey
        | EntityType::Mule
        | EntityType::Llama
        | EntityType::TraderLlama
        | EntityType::Cat
        | EntityType::Ocelot
        | EntityType::Wolf
        | EntityType::Fox
        | EntityType::Rabbit
        | EntityType::Frog
        | EntityType::Turtle
        | EntityType::Parrot
        | EntityType::Panda
        | EntityType::Bee
        | EntityType::Strider
        | EntityType::Camel
        | EntityType::Sniffer
        | EntityType::Armadillo
        | EntityType::Mooshroom
        | EntityType::Goat
        | EntityType::PolarBear => MobCategory::Creature,

        // Hostile & Neutral Monsters
        EntityType::Zombie
        | EntityType::ZombieVillager
        | EntityType::Husk
        | EntityType::Drowned
        | EntityType::Skeleton
        | EntityType::Stray
        | EntityType::Bogged
        | EntityType::WitherSkeleton
        | EntityType::Creeper
        | EntityType::Spider
        | EntityType::CaveSpider
        | EntityType::Enderman
        | EntityType::Witch
        | EntityType::Slime
        | EntityType::MagmaCube
        | EntityType::Ghast
        | EntityType::Blaze
        | EntityType::Piglin
        | EntityType::PiglinBrute
        | EntityType::ZombifiedPiglin
        | EntityType::Hoglin
        | EntityType::Zoglin
        | EntityType::Phantom
        | EntityType::Guardian
        | EntityType::ElderGuardian
        | EntityType::Warden
        | EntityType::Breeze
        | EntityType::Creaking
        | EntityType::Vindicator
        | EntityType::Evoker
        | EntityType::Pillager
        | EntityType::Ravager
        | EntityType::Vex
        | EntityType::Silverfish
        | EntityType::Endermite
        | EntityType::Shulker
        | EntityType::Wither
        | EntityType::EnderDragon => MobCategory::Monster,

        // Utility, Villagers & Golems
        EntityType::Villager
        | EntityType::WanderingTrader
        | EntityType::IronGolem
        | EntityType::SnowGolem
        | EntityType::Allay
        | EntityType::Player => MobCategory::Misc,
    }
}

/// Tracks the active entity population per `MobCategory` and calculates dynamic mob caps.
/// REA Reference: `cua$d.class` (SpawnState)
#[derive(Debug, Clone, Default)]
pub struct MobCategoryTracker {
    counts: HashMap<MobCategory, usize>,
}

impl MobCategoryTracker {
    /// Creates a new tracker initialized with zero counts for all categories.
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    /// Returns the active mob count for the specified category.
    pub fn get_count(&self, category: MobCategory) -> usize {
        *self.counts.get(&category).unwrap_or(&0)
    }

    /// Sets the active mob count for a category.
    pub fn set_count(&mut self, category: MobCategory, count: usize) {
        self.counts.insert(category, count);
    }

    /// Increments the active mob count for a category.
    pub fn increment(&mut self, category: MobCategory) {
        *self.counts.entry(category).or_insert(0) += 1;
    }

    /// Decrements the active mob count for a category (saturating at 0).
    pub fn decrement(&mut self, category: MobCategory) {
        let entry = self.counts.entry(category).or_insert(0);
        *entry = entry.saturating_sub(1);
    }

    /// Increments by entity type.
    pub fn register_spawn(&mut self, entity_type: EntityType) {
        let cat = category_for_entity(entity_type);
        self.increment(cat);
    }

    /// Decrements by entity type.
    pub fn register_despawn(&mut self, entity_type: EntityType) {
        let cat = category_for_entity(entity_type);
        self.decrement(cat);
    }

    /// Clears all counts.
    pub fn clear(&mut self) {
        self.counts.clear();
    }

    /// Computes the dynamic mob cap for a category given the number of loaded spawnable chunks.
    /// Vanilla Formula: `floor(base_cap * spawnable_chunks / 289)`
    /// In a standard 8-chunk singleplayer radius, spawnable_chunks = 289, yielding exact base_cap!
    pub fn calculate_mob_cap(category: MobCategory, spawnable_chunks: usize) -> usize {
        let base = category.base_cap();
        if base <= 0 {
            return usize::MAX;
        }
        let cap = (base as f64 * spawnable_chunks as f64 / MAGIC_NUMBER as f64).floor() as usize;
        cap.max(1)
    }

    /// Returns true if the active mob population for this category is strictly beneath the mob cap.
    pub fn can_spawn(&self, category: MobCategory, spawnable_chunks: usize) -> bool {
        let base = category.base_cap();
        if base <= 0 {
            return false;
        }
        let cap = Self::calculate_mob_cap(category, spawnable_chunks);
        self.get_count(category) < cap
    }
}
