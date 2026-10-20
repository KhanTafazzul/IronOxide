// REA Verified: net.minecraft.world.entity.npc.WanderingTrader (mapped: cfb.class)
// REA Verified: net.minecraft.world.entity.npc.WanderingTraderSpawner (mapped: cfc.class)
//
// Wandering Trader spawner schedule (every 10-12 days), 2 tethered Trader Llamas,
// invisibility potion drinking at dusk, milk drinking at dawn, and despawn timers.

use super::trades::TradeOffer;
use crate::components::EntityId;
use crate::villagers::profession::TradeLevel;
use mc_items::ItemId;

/// Wandering Trader mob (`cfb.class`) offering exotic biomes items.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WanderingTrader {
    /// Active trade offers.
    pub trades: Vec<TradeOffer>,
    /// Whether the trader is currently invisible (drunk Potion of Invisibility).
    pub is_invisible: bool,
    /// Despawn timer counting down from 48000..72000 ticks (2..3 in-game days).
    pub despawn_ticks: u32,
    /// Entity IDs of the 2 tethered Trader Llamas.
    pub llamas: [Option<EntityId>; 2],
    /// Ticks drink animation takes (32 ticks = 1.6s).
    pub drink_potion_ticks: u32,
}

impl Default for WanderingTrader {
    fn default() -> Self {
        Self {
            trades: Self::generate_default_trades(),
            is_invisible: false,
            despawn_ticks: 48000, // 2 days default
            llamas: [None, None],
            drink_potion_ticks: 0,
        }
    }
}

impl WanderingTrader {
    pub const DEFAULT_DESPAWN_TICKS: u32 = 48000;
    pub const DRINK_DURATION_TICKS: u32 = 32;

    pub fn new() -> Self {
        Self::default()
    }

    /// Generates pool of authentic Wandering Trader trades (exotic saplings, sea pickles, blue ice, etc.).
    pub fn generate_default_trades() -> Vec<TradeOffer> {
        vec![
            TradeOffer::simple((ItemId::Emerald, 5), (ItemId::PackedIce, 1), 6, 1, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Emerald, 6), (ItemId::BlueIce, 1), 6, 1, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Emerald, 1), (ItemId::Gunpowder, 1), 8, 1, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Emerald, 4), (ItemId::SlimeBall, 1), 5, 1, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Emerald, 2), (ItemId::SeaPickle, 1), 5, 1, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Emerald, 5), (ItemId::NautilusShell, 1), 5, 1, TradeLevel::Novice),
        ]
    }

    /// Attaches the 2 Trader Llamas spawned alongside this trader.
    pub fn attach_llamas(&mut self, llama1: EntityId, llama2: EntityId) {
        self.llamas = [Some(llama1), Some(llama2)];
    }

    /// Ticks daytime behavior:
    /// - Drinks Potion of Invisibility at dusk (ticks >= 12000) or under hostile threat.
    /// - Drinks Milk Bucket at dawn (ticks 0..2000) to clear invisibility.
    /// - Decrements despawn timer (returns true if time to despawn).
    pub fn tick(&mut self, day_time_ticks: u32, is_threatened: bool) -> bool {
        let time_of_day = day_time_ticks % 24000;

        if is_threatened || time_of_day >= 12000 {
            // Drink invisibility at night or under threat
            if !self.is_invisible && self.drink_potion_ticks == 0 {
                self.drink_potion_ticks = Self::DRINK_DURATION_TICKS;
            }
        } else if time_of_day < 2000 && self.is_invisible {
            // Clear invisibility with milk at dawn
            self.is_invisible = false;
        }

        if self.drink_potion_ticks > 0 {
            self.drink_potion_ticks -= 1;
            if self.drink_potion_ticks == 0 {
                self.is_invisible = true;
            }
        }

        if self.despawn_ticks > 0 {
            self.despawn_ticks -= 1;
        }

        self.despawn_ticks == 0
    }
}

/// Spawner tracking game ticks to summon a Wandering Trader near players every 10-12 days.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WanderingTraderSpawner {
    /// Total ticks elapsed since last spawn attempt.
    pub ticks_since_last_spawn: u32,
    /// Number of in-game days elapsed without a spawn.
    pub days_without_spawn: u32,
}

impl Default for WanderingTraderSpawner {
    fn default() -> Self {
        Self {
            ticks_since_last_spawn: 0,
            days_without_spawn: 0,
        }
    }
}

impl WanderingTraderSpawner {
    pub const DAY_TICKS: u32 = 24000;
    pub const MIN_DAYS_FOR_SPAWN: u32 = 10;
    pub const MAX_DAYS_FOR_SPAWN: u32 = 12;

    pub fn new() -> Self {
        Self::default()
    }

    /// Ticks the spawner. Every 24,000 ticks (1 day), evaluates spawn probability:
    /// - Days < 10: 0% chance
    /// - Day 10: 25% chance
    /// - Day 11: 50% chance
    /// - Day 12+: 100% guaranteed spawn!
    pub fn tick(&mut self, rng_roll: f32) -> bool {
        self.ticks_since_last_spawn += 1;
        if self.ticks_since_last_spawn >= Self::DAY_TICKS {
            self.ticks_since_last_spawn = 0;
            self.days_without_spawn += 1;

            if self.days_without_spawn >= Self::MAX_DAYS_FOR_SPAWN {
                self.days_without_spawn = 0;
                return true;
            } else if self.days_without_spawn >= Self::MIN_DAYS_FOR_SPAWN {
                let chance = if self.days_without_spawn == 10 { 0.25 } else { 0.50 };
                if rng_roll < chance {
                    self.days_without_spawn = 0;
                    return true;
                }
            }
        }
        false
    }
}
