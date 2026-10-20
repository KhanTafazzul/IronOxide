// REA Verified: net.minecraft.world.entity.npc.Villager (mapped: cev.class)
// REA Verified: net.minecraft.world.entity.ai.gossip.GossipContainer (mapped: cfs.class)
//
// Complete Villager Entity combining daily schedule clock, profession career,
// trading offers, gossip mechanics, and breeding with food.

use super::profession::{TradeLevel, VillagerCareer, VillagerProfession};
use super::schedule::{VillagerActivity, VillagerSchedule};
use super::trades::{generate_trades_for_profession, TradeOffer};
use crate::passive::AnimalData;
use glam::Vec3;
use mc_items::ItemId;

/// Village gossip categories affecting player reputation and golem summoning.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VillagerGossip {
    /// Reputation bonus for trading (+1 per trade, decays slowly).
    pub trading_reputation: i32,
    /// Major positive reputation for curing zombie villager (+20).
    pub major_positive: i32,
    /// Negative reputation for hurting villagers (-10).
    pub minor_negative: i32,
    /// Golem summoning talk tally (shared during Gather at village bell).
    pub golem_talk_count: u32,
}

impl VillagerGossip {
    pub fn new() -> Self {
        Self::default()
    }

    /// Total reputation score towards players.
    pub fn net_reputation(&self) -> i32 {
        self.trading_reputation + self.major_positive - self.minor_negative
    }

    /// Records a successful trade.
    pub fn on_trade(&mut self) {
        self.trading_reputation = (self.trading_reputation + 2).min(50);
    }

    /// Records damage from player.
    pub fn on_hurt_by_player(&mut self) {
        self.minor_negative = (self.minor_negative + 10).min(100);
    }

    /// Talks about iron golems during Gather activity.
    pub fn add_golem_talk(&mut self) {
        self.golem_talk_count += 1;
    }
}

/// Villager NPC entity (`cev.class`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Villager {
    pub schedule: VillagerSchedule,
    pub career: VillagerCareer,
    pub trades: Vec<TradeOffer>,
    pub gossip: VillagerGossip,
    pub animal_data: AnimalData,
    pub health: f32,
    pub max_health: f32,
    /// Food inventory for breeding (Wheat, Carrots, Potatoes, Bread, Beetroots).
    pub food_points: u32,
}

impl Default for Villager {
    fn default() -> Self {
        Self {
            schedule: VillagerSchedule::default(),
            career: VillagerCareer::default(),
            trades: Vec::new(),
            gossip: VillagerGossip::default(),
            animal_data: AnimalData::adult(),
            health: 20.0, // 10 hearts
            max_health: 20.0,
            food_points: 0,
        }
    }
}

impl Villager {
    pub const BREED_FOOD_THRESHOLD: u32 = 12; // 12 food points (e.g. 3 bread or 12 carrots)

    pub fn new(profession: VillagerProfession) -> Self {
        let mut trades = generate_trades_for_profession(profession);
        // Only unlock Novice trades initially
        trades.retain(|t| t.required_level == TradeLevel::Novice);

        Self {
            schedule: VillagerSchedule::default(),
            career: VillagerCareer::new(profession),
            trades,
            gossip: VillagerGossip::default(),
            animal_data: AnimalData::adult(),
            health: 20.0,
            max_health: 20.0,
            food_points: 0,
        }
    }

    /// Creates an unemployed villager.
    pub fn unemployed() -> Self {
        Self::new(VillagerProfession::None)
    }

    /// Creates a Nitwit villager.
    pub fn nitwit() -> Self {
        Self::new(VillagerProfession::Nitwit)
    }

    /// Binds an unemployed villager to a newly claimed workstation block.
    pub fn claim_workstation(&mut self, pos: Vec3, workstation: ItemId) -> bool {
        if self.career.try_claim_workstation(workstation) {
            self.schedule.claim_workstation(pos);
            let mut all_trades = generate_trades_for_profession(self.career.profession);
            all_trades.retain(|t| t.required_level == TradeLevel::Novice);
            self.trades = all_trades;
            true
        } else {
            false
        }
    }

    /// Unbinds workstation if broken. If profession is not locked by prior trading, reverts to Unemployed.
    pub fn lose_workstation(&mut self) -> bool {
        self.schedule.release_workstation();
        if self.career.try_lose_workstation() {
            self.trades.clear();
            true
        } else {
            false
        }
    }

    /// Updates daily schedule clock and manages workstation restocks.
    pub fn tick(&mut self, day_time_ticks: u32) {
        self.animal_data.tick();

        // Dawn resets daily restocks (tick 0..2000)
        if day_time_ticks % 24000 < 50 {
            self.career.reset_daily_restocks();
        }

        let is_nitwit = self.career.profession == VillagerProfession::Nitwit;
        let activity = self.schedule.update_schedule(day_time_ticks, is_nitwit);

        // Working at workstation allows restocking trades (up to twice daily)
        if activity == VillagerActivity::Work && self.schedule.workstation_pos.is_some() {
            if self.career.restock(day_time_ticks) {
                for trade in &mut self.trades {
                    trade.restock();
                }
            }
        }

        // Gather at bell allows gossiping
        if activity == VillagerActivity::Gather && self.schedule.bell_pos.is_some() {
            self.gossip.add_golem_talk();
        }
    }

    /// Executes a trade offer at the specified index.
    pub fn trade(&mut self, trade_index: usize) -> bool {
        if let Some(trade) = self.trades.get_mut(trade_index) {
            if let Some(xp) = trade.execute_trade() {
                self.gossip.on_trade();
                let leveled_up = self.career.add_xp(xp);
                if leveled_up {
                    // Unlock next tier trades
                    let all_trades = generate_trades_for_profession(self.career.profession);
                    for t in all_trades {
                        if t.required_level == self.career.level && !self.trades.contains(&t) {
                            self.trades.push(t);
                        }
                    }
                }
                return true;
            }
        }
        false
    }

    /// Feeds food items (Bread, Carrot, Potato, Beetroot) to villager for breeding.
    pub fn feed_food(&mut self, item: ItemId, count: u32) -> bool {
        let pts_per_item = match item {
            ItemId::Bread => 4,
            ItemId::Carrot | ItemId::Potato | ItemId::Beetroot => 1,
            _ => 0,
        };
        if pts_per_item > 0 {
            self.food_points += pts_per_item * count;
            if self.food_points >= Self::BREED_FOOD_THRESHOLD && self.animal_data.can_breed() {
                self.food_points -= Self::BREED_FOOD_THRESHOLD;
                self.animal_data.set_in_love(None);
            }
            true
        } else {
            false
        }
    }
}
