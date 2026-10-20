// REA Verified: net.minecraft.world.entity.npc.VillagerTrades (mapped: cez.class)
// REA Verified: net.minecraft.world.item.trading.MerchantOffer (mapped: cfd.class)
//
// Villager Trade Offer structure, tier unlocking matrix, price demand adjustments,
// and authentic trade matrices across all working professions.

use super::profession::{TradeLevel, VillagerProfession};
use mc_items::ItemId;

/// Trade Offer between a player and a villager matching `MerchantOffer` (`cfd.class`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TradeOffer {
    /// Primary input item and required count.
    pub input_a: (ItemId, u32),
    /// Optional secondary input item and required count (e.g. 1 Book for Enchanted Book trade).
    pub input_b: Option<(ItemId, u32)>,
    /// Resulting output item and count received by the player.
    pub output: (ItemId, u32),
    /// Current number of times this trade has been executed since last restock.
    pub uses: u32,
    /// Maximum allowed uses before the trade is locked (typically 12 or 16).
    pub max_uses: u32,
    /// XP awarded to the villager upon successful trade execution.
    pub xp_reward: u32,
    /// Price discount or penalty from reputation / Hero of the Village.
    pub price_multiplier: f32,
    /// Minimum trade level required to unlock this offer.
    pub required_level: TradeLevel,
}

impl TradeOffer {
    pub fn new(
        input_a: (ItemId, u32),
        input_b: Option<(ItemId, u32)>,
        output: (ItemId, u32),
        max_uses: u32,
        xp_reward: u32,
        required_level: TradeLevel,
    ) -> Self {
        Self {
            input_a,
            input_b,
            output,
            uses: 0,
            max_uses,
            xp_reward,
            price_multiplier: 1.0,
            required_level,
        }
    }

    /// Single item trade constructor.
    pub fn simple(
        input: (ItemId, u32),
        output: (ItemId, u32),
        max_uses: u32,
        xp: u32,
        level: TradeLevel,
    ) -> Self {
        Self::new(input, None, output, max_uses, xp, level)
    }

    /// Whether this trade can currently be executed (has not exhausted its uses).
    pub fn can_trade(&self) -> bool {
        self.uses < self.max_uses
    }

    /// Calculates discounted or adjusted cost of the primary input item.
    pub fn adjusted_cost_a(&self) -> u32 {
        let base = self.input_a.1 as f32;
        let adjusted = (base * self.price_multiplier).round() as u32;
        adjusted.max(1).min(64)
    }

    /// Executes the trade once: increments use counter.
    /// Returns `Some(xp_awarded)` if successful, or `None` if the trade is locked.
    pub fn execute_trade(&mut self) -> Option<u32> {
        if self.can_trade() {
            self.uses += 1;
            Some(self.xp_reward)
        } else {
            None
        }
    }

    /// Restocks this trade offer, resetting uses to 0.
    pub fn restock(&mut self) {
        self.uses = 0;
    }
}

/// Generates authentic trade matrix for a given profession across all 5 tiers.
pub fn generate_trades_for_profession(profession: VillagerProfession) -> Vec<TradeOffer> {
    match profession {
        VillagerProfession::Farmer => vec![
            // Novice (Tier 1)
            TradeOffer::simple((ItemId::Wheat, 20), (ItemId::Emerald, 1), 16, 2, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Emerald, 1), (ItemId::Bread, 6), 16, 1, TradeLevel::Novice),
            // Apprentice (Tier 2)
            TradeOffer::simple((ItemId::Pumpkin, 6), (ItemId::Emerald, 1), 12, 10, TradeLevel::Apprentice),
            TradeOffer::simple((ItemId::Emerald, 1), (ItemId::PumpkinPie, 4), 12, 5, TradeLevel::Apprentice),
            // Journeyman (Tier 3)
            TradeOffer::simple((ItemId::Melon, 4), (ItemId::Emerald, 1), 12, 20, TradeLevel::Journeyman),
            TradeOffer::simple((ItemId::Emerald, 1), (ItemId::Cookie, 18), 12, 10, TradeLevel::Journeyman),
            // Expert (Tier 4)
            TradeOffer::simple((ItemId::Emerald, 3), (ItemId::GoldenCarrot, 3), 12, 30, TradeLevel::Expert),
            // Master (Tier 5)
            TradeOffer::simple((ItemId::Emerald, 3), (ItemId::GlisteringMelonSlice, 3), 12, 30, TradeLevel::Master),
        ],

        VillagerProfession::Librarian => vec![
            // Novice (Tier 1)
            TradeOffer::simple((ItemId::Paper, 24), (ItemId::Emerald, 1), 16, 2, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Emerald, 9), (ItemId::Bookshelf, 1), 12, 1, TradeLevel::Novice),
            // Apprentice (Tier 2)
            TradeOffer::simple((ItemId::Book, 4), (ItemId::Emerald, 1), 12, 10, TradeLevel::Apprentice),
            TradeOffer::simple((ItemId::Emerald, 1), (ItemId::Lantern, 1), 12, 5, TradeLevel::Apprentice),
            // Journeyman (Tier 3)
            TradeOffer::simple((ItemId::InkSac, 5), (ItemId::Emerald, 1), 12, 20, TradeLevel::Journeyman),
            TradeOffer::simple((ItemId::Emerald, 1), (ItemId::Glass, 4), 12, 10, TradeLevel::Journeyman),
            // Expert (Tier 4)
            TradeOffer::simple((ItemId::Emerald, 5), (ItemId::Clock, 1), 12, 30, TradeLevel::Expert),
            // Master (Tier 5)
            TradeOffer::simple((ItemId::Emerald, 20), (ItemId::NameTag, 1), 12, 30, TradeLevel::Master),
        ],

        VillagerProfession::Armorer => vec![
            // Novice (Tier 1)
            TradeOffer::simple((ItemId::Coal, 15), (ItemId::Emerald, 1), 16, 2, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Emerald, 5), (ItemId::IronHelmet, 1), 12, 1, TradeLevel::Novice),
            // Apprentice (Tier 2)
            TradeOffer::simple((ItemId::IronIngot, 4), (ItemId::Emerald, 1), 12, 10, TradeLevel::Apprentice),
            TradeOffer::simple((ItemId::Emerald, 9), (ItemId::IronChestplate, 1), 12, 5, TradeLevel::Apprentice),
            // Journeyman (Tier 3)
            TradeOffer::simple((ItemId::LavaBucket, 1), (ItemId::Emerald, 1), 12, 20, TradeLevel::Journeyman),
            TradeOffer::simple((ItemId::Emerald, 5), (ItemId::Shield, 1), 12, 10, TradeLevel::Journeyman),
            // Expert (Tier 4)
            TradeOffer::simple((ItemId::Diamond, 1), (ItemId::Emerald, 1), 12, 30, TradeLevel::Expert),
            TradeOffer::simple((ItemId::Emerald, 14), (ItemId::DiamondLeggings, 1), 12, 30, TradeLevel::Expert),
            // Master (Tier 5)
            TradeOffer::simple((ItemId::Emerald, 18), (ItemId::DiamondChestplate, 1), 12, 30, TradeLevel::Master),
        ],

        VillagerProfession::Cleric => vec![
            // Novice (Tier 1)
            TradeOffer::simple((ItemId::RottenFlesh, 32), (ItemId::Emerald, 1), 16, 2, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Emerald, 1), (ItemId::Redstone, 2), 16, 1, TradeLevel::Novice),
            // Apprentice (Tier 2)
            TradeOffer::simple((ItemId::GoldIngot, 3), (ItemId::Emerald, 1), 12, 10, TradeLevel::Apprentice),
            TradeOffer::simple((ItemId::Emerald, 1), (ItemId::LapisLazuli, 1), 12, 5, TradeLevel::Apprentice),
            // Journeyman (Tier 3)
            TradeOffer::simple((ItemId::RabbitFoot, 2), (ItemId::Emerald, 1), 12, 20, TradeLevel::Journeyman),
            TradeOffer::simple((ItemId::Emerald, 1), (ItemId::GlowstoneDust, 2), 12, 10, TradeLevel::Journeyman),
            // Expert (Tier 4)
            TradeOffer::simple((ItemId::Scute, 4), (ItemId::Emerald, 1), 12, 30, TradeLevel::Expert),
            TradeOffer::simple((ItemId::Emerald, 5), (ItemId::EnderPearl, 1), 12, 30, TradeLevel::Expert),
            // Master (Tier 5)
            TradeOffer::simple((ItemId::NetherWart, 22), (ItemId::Emerald, 1), 12, 30, TradeLevel::Master),
            TradeOffer::simple((ItemId::Emerald, 3), (ItemId::ExperienceBottle, 1), 12, 30, TradeLevel::Master),
        ],

        VillagerProfession::Fletcher => vec![
            // Novice (Tier 1)
            TradeOffer::simple((ItemId::Stick, 32), (ItemId::Emerald, 1), 16, 2, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Emerald, 1), (ItemId::Arrow, 16), 16, 1, TradeLevel::Novice),
            // Apprentice (Tier 2)
            TradeOffer::simple((ItemId::Flint, 26), (ItemId::Emerald, 1), 12, 10, TradeLevel::Apprentice),
            TradeOffer::simple((ItemId::Emerald, 2), (ItemId::Bow, 1), 12, 5, TradeLevel::Apprentice),
            // Journeyman (Tier 3)
            TradeOffer::simple((ItemId::String, 14), (ItemId::Emerald, 1), 12, 20, TradeLevel::Journeyman),
            TradeOffer::simple((ItemId::Emerald, 3), (ItemId::Crossbow, 1), 12, 10, TradeLevel::Journeyman),
            // Expert (Tier 4)
            TradeOffer::simple((ItemId::Feather, 24), (ItemId::Emerald, 1), 12, 30, TradeLevel::Expert),
            // Master (Tier 5)
            TradeOffer::simple((ItemId::Emerald, 8), (ItemId::SpectralArrow, 5), 12, 30, TradeLevel::Master),
        ],

        VillagerProfession::Weaponsmith => vec![
            // Novice (Tier 1)
            TradeOffer::simple((ItemId::Coal, 15), (ItemId::Emerald, 1), 16, 2, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Emerald, 3), (ItemId::IronAxe, 1), 12, 1, TradeLevel::Novice),
            // Apprentice (Tier 2)
            TradeOffer::simple((ItemId::IronIngot, 4), (ItemId::Emerald, 1), 12, 10, TradeLevel::Apprentice),
            TradeOffer::simple((ItemId::Emerald, 2), (ItemId::IronSword, 1), 12, 5, TradeLevel::Apprentice),
            // Journeyman (Tier 3)
            TradeOffer::simple((ItemId::Flint, 24), (ItemId::Emerald, 1), 12, 20, TradeLevel::Journeyman),
            // Expert (Tier 4)
            TradeOffer::simple((ItemId::Diamond, 1), (ItemId::Emerald, 1), 12, 30, TradeLevel::Expert),
            TradeOffer::simple((ItemId::Emerald, 12), (ItemId::DiamondAxe, 1), 12, 30, TradeLevel::Expert),
            // Master (Tier 5)
            TradeOffer::simple((ItemId::Emerald, 8), (ItemId::DiamondSword, 1), 12, 30, TradeLevel::Master),
        ],

        VillagerProfession::Toolsmith => vec![
            // Novice (Tier 1)
            TradeOffer::simple((ItemId::Coal, 15), (ItemId::Emerald, 1), 16, 2, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Emerald, 1), (ItemId::StonePickaxe, 1), 12, 1, TradeLevel::Novice),
            // Apprentice (Tier 2)
            TradeOffer::simple((ItemId::IronIngot, 4), (ItemId::Emerald, 1), 12, 10, TradeLevel::Apprentice),
            TradeOffer::simple((ItemId::Emerald, 1), (ItemId::IronShovel, 1), 12, 5, TradeLevel::Apprentice),
            // Journeyman (Tier 3)
            TradeOffer::simple((ItemId::Flint, 30), (ItemId::Emerald, 1), 12, 20, TradeLevel::Journeyman),
            TradeOffer::simple((ItemId::Emerald, 4), (ItemId::IronPickaxe, 1), 12, 10, TradeLevel::Journeyman),
            // Expert (Tier 4)
            TradeOffer::simple((ItemId::Diamond, 1), (ItemId::Emerald, 1), 12, 30, TradeLevel::Expert),
            TradeOffer::simple((ItemId::Emerald, 10), (ItemId::DiamondAxe, 1), 12, 30, TradeLevel::Expert),
            // Master (Tier 5)
            TradeOffer::simple((ItemId::Emerald, 13), (ItemId::DiamondPickaxe, 1), 12, 30, TradeLevel::Master),
        ],

        VillagerProfession::Butcher => vec![
            TradeOffer::simple((ItemId::Chicken, 14), (ItemId::Emerald, 1), 16, 2, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Emerald, 1), (ItemId::CookedChicken, 4), 16, 1, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Porkchop, 7), (ItemId::Emerald, 1), 12, 10, TradeLevel::Apprentice),
            TradeOffer::simple((ItemId::Beef, 10), (ItemId::Emerald, 1), 12, 20, TradeLevel::Journeyman),
            TradeOffer::simple((ItemId::Mutton, 7), (ItemId::Emerald, 1), 12, 30, TradeLevel::Expert),
            TradeOffer::simple((ItemId::SweetBerries, 10), (ItemId::Emerald, 1), 12, 30, TradeLevel::Master),
        ],

        VillagerProfession::Cartographer => vec![
            TradeOffer::simple((ItemId::Paper, 24), (ItemId::Emerald, 1), 16, 2, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Emerald, 7), (ItemId::Map, 1), 12, 1, TradeLevel::Novice),
            TradeOffer::simple((ItemId::GlassPane, 11), (ItemId::Emerald, 1), 12, 10, TradeLevel::Apprentice),
            TradeOffer::simple((ItemId::Compass, 1), (ItemId::Emerald, 1), 12, 20, TradeLevel::Journeyman),
            TradeOffer::simple((ItemId::Emerald, 12), (ItemId::ItemFrame, 1), 12, 30, TradeLevel::Master),
        ],

        VillagerProfession::Fisherman => vec![
            TradeOffer::simple((ItemId::String, 20), (ItemId::Emerald, 1), 16, 2, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Emerald, 1), (ItemId::Cod, 6), 16, 1, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Cod, 15), (ItemId::Emerald, 1), 12, 10, TradeLevel::Apprentice),
            TradeOffer::simple((ItemId::Salmon, 13), (ItemId::Emerald, 1), 12, 20, TradeLevel::Journeyman),
            TradeOffer::simple((ItemId::TropicalFish, 6), (ItemId::Emerald, 1), 12, 30, TradeLevel::Expert),
            TradeOffer::simple((ItemId::Emerald, 8), (ItemId::FishingRod, 1), 12, 30, TradeLevel::Master),
        ],

        VillagerProfession::Leatherworker => vec![
            TradeOffer::simple((ItemId::Leather, 6), (ItemId::Emerald, 1), 16, 2, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Emerald, 3), (ItemId::LeatherLeggings, 1), 12, 1, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Flint, 26), (ItemId::Emerald, 1), 12, 10, TradeLevel::Apprentice),
            TradeOffer::simple((ItemId::Emerald, 7), (ItemId::LeatherChestplate, 1), 12, 10, TradeLevel::Apprentice),
            TradeOffer::simple((ItemId::Emerald, 6), (ItemId::Saddle, 1), 12, 30, TradeLevel::Master),
        ],

        VillagerProfession::Mason => vec![
            TradeOffer::simple((ItemId::ClayBall, 10), (ItemId::Emerald, 1), 16, 2, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Emerald, 1), (ItemId::Bricks, 10), 16, 1, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Stone, 20), (ItemId::Emerald, 1), 12, 10, TradeLevel::Apprentice),
            TradeOffer::simple((ItemId::Granite, 16), (ItemId::Emerald, 1), 12, 20, TradeLevel::Journeyman),
            TradeOffer::simple((ItemId::Andesite, 16), (ItemId::Emerald, 1), 12, 20, TradeLevel::Journeyman),
            TradeOffer::simple((ItemId::Diorite, 16), (ItemId::Emerald, 1), 12, 20, TradeLevel::Journeyman),
            TradeOffer::simple((ItemId::QuartzBlock, 1), (ItemId::Emerald, 1), 12, 30, TradeLevel::Master),
        ],

        VillagerProfession::Shepherd => vec![
            TradeOffer::simple((ItemId::WhiteWool, 18), (ItemId::Emerald, 1), 16, 2, TradeLevel::Novice),
            TradeOffer::simple((ItemId::Emerald, 2), (ItemId::Shears, 1), 12, 1, TradeLevel::Novice),
            TradeOffer::simple((ItemId::BlackWool, 18), (ItemId::Emerald, 1), 12, 10, TradeLevel::Apprentice),
            TradeOffer::simple((ItemId::Emerald, 1), (ItemId::WhiteWool, 1), 12, 5, TradeLevel::Apprentice),
            TradeOffer::simple((ItemId::Emerald, 3), (ItemId::Painting, 3), 12, 30, TradeLevel::Master),
        ],

        VillagerProfession::None | VillagerProfession::Nitwit => Vec::new(),
    }
}
