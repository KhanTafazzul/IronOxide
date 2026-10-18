//! Tool and Armor tiers with exact attributes reverse-engineered from client.jar.
//! REA References:
//! - net.minecraft.world.item.Tiers (coo.class)
//! - net.minecraft.world.item.ArmorMaterials (ckk.class)

use crate::item_id::ItemId;

/// Tool material tiers matching net.minecraft.world.item.Tiers (coo.class).
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum ToolTier {
    Wood = 0,
    Stone = 1,
    Iron = 2,
    Diamond = 3,
    Gold = 4,
    Netherite = 5,
}

impl ToolTier {
    /// Maximum durability uses before the tool breaks.
    /// Bytecode verified from coo.class:
    /// - WOOD: 59
    /// - STONE: 131
    /// - IRON: 250
    /// - DIAMOND: 1561
    /// - GOLD: 32
    /// - NETHERITE: 2031
    pub const fn durability(&self) -> u32 {
        match self {
            ToolTier::Wood => 59,
            ToolTier::Stone => 131,
            ToolTier::Iron => 250,
            ToolTier::Diamond => 1561,
            ToolTier::Gold => 32,
            ToolTier::Netherite => 2031,
        }
    }

    /// Mining speed multiplier applied to break time when harvesting preferred blocks.
    /// Bytecode verified from coo.class:
    /// - WOOD: 2.0
    /// - STONE: 4.0
    /// - IRON: 6.0
    /// - DIAMOND: 8.0
    /// - GOLD: 12.0
    /// - NETHERITE: 9.0
    pub const fn mining_speed_multiplier(&self) -> f32 {
        match self {
            ToolTier::Wood => 2.0,
            ToolTier::Stone => 4.0,
            ToolTier::Iron => 6.0,
            ToolTier::Diamond => 8.0,
            ToolTier::Gold => 12.0,
            ToolTier::Netherite => 9.0,
        }
    }

    /// Base attack damage bonus imparted by the tool material.
    /// Bytecode verified from coo.class:
    /// - WOOD: 0.0
    /// - STONE: 1.0
    /// - IRON: 2.0
    /// - DIAMOND: 3.0
    /// - GOLD: 0.0
    /// - NETHERITE: 4.0
    pub const fn attack_damage_bonus(&self) -> f32 {
        match self {
            ToolTier::Wood => 0.0,
            ToolTier::Stone => 1.0,
            ToolTier::Iron => 2.0,
            ToolTier::Diamond => 3.0,
            ToolTier::Gold => 0.0,
            ToolTier::Netherite => 4.0,
        }
    }

    /// Mining level required to harvest specific block drops (e.g. Iron requires level 1 stone pick).
    /// - WOOD: 0
    /// - STONE: 1
    /// - IRON: 2
    /// - DIAMOND: 3
    /// - GOLD: 0
    /// - NETHERITE: 4
    pub const fn harvest_level(&self) -> u32 {
        match self {
            ToolTier::Wood => 0,
            ToolTier::Stone => 1,
            ToolTier::Iron => 2,
            ToolTier::Diamond => 3,
            ToolTier::Gold => 0,
            ToolTier::Netherite => 4,
        }
    }

    /// Enchantability rating determining higher-level enchantments.
    pub const fn enchantability(&self) -> u32 {
        match self {
            ToolTier::Wood => 15,
            ToolTier::Stone => 5,
            ToolTier::Iron => 14,
            ToolTier::Diamond => 10,
            ToolTier::Gold => 22,
            ToolTier::Netherite => 15,
        }
    }
}

/// Tool categories with intrinsic attack damages and attack speeds.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ToolType {
    Sword,
    Pickaxe,
    Axe,
    Shovel,
    Hoe,
}

impl ToolType {
    /// Computes total attack damage for this tool type with the given material tier.
    pub const fn attack_damage(&self, tier: ToolTier) -> f32 {
        match self {
            // Player base damage (1.0) + Sword bonus (3.0) + Tier bonus
            ToolType::Sword => 4.0 + tier.attack_damage_bonus(),
            // Player base damage (1.0) + Pickaxe bonus (1.0) + Tier bonus
            ToolType::Pickaxe => 2.0 + tier.attack_damage_bonus(),
            // Vanilla specific Axe attack damages
            ToolType::Axe => match tier {
                ToolTier::Wood | ToolTier::Gold => 7.0,
                ToolTier::Stone | ToolTier::Iron | ToolTier::Diamond => 9.0,
                ToolTier::Netherite => 10.0,
            },
            // Player base damage (1.0) + Shovel bonus (1.5) + Tier bonus
            ToolType::Shovel => 2.5 + tier.attack_damage_bonus(),
            // Hoe damage is fixed at 1.0 in vanilla
            ToolType::Hoe => 1.0,
        }
    }

    /// Attack speed (swings per second) for this tool type.
    pub const fn attack_speed(&self, tier: ToolTier) -> f32 {
        match self {
            ToolType::Sword => 1.6,
            ToolType::Pickaxe => 1.2,
            ToolType::Axe => match tier {
                ToolTier::Wood | ToolTier::Stone | ToolTier::Gold => 0.8,
                ToolTier::Iron => 0.9,
                ToolTier::Diamond | ToolTier::Netherite => 1.0,
            },
            ToolType::Shovel => 1.0,
            ToolType::Hoe => match tier {
                ToolTier::Wood | ToolTier::Gold => 1.0,
                ToolTier::Stone => 2.0,
                ToolTier::Iron => 3.0,
                ToolTier::Diamond | ToolTier::Netherite => 4.0,
            },
        }
    }
}

/// Armor equipment slot.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ArmorSlot {
    Helmet = 0,
    Chestplate = 1,
    Leggings = 2,
    Boots = 3,
}

/// Armor material tier matching net.minecraft.world.item.ArmorMaterials (ckk.class).
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ArmorTier {
    Leather,
    Chainmail,
    Iron,
    Gold,
    Diamond,
    Turtle,
    Netherite,
}

impl ArmorTier {
    /// Armor defense points provided for a specific armor slot.
    /// Bytecode verified from ckk.class:
    /// - LEATHER: [1, 3, 2, 1]
    /// - CHAIN:   [2, 5, 4, 1]
    /// - IRON:    [2, 6, 5, 2]
    /// - GOLD:    [2, 5, 3, 1]
    /// - DIAMOND: [3, 8, 6, 3]
    /// - TURTLE:  [2, 0, 0, 0]
    /// - NETHERITE: [3, 8, 6, 3]
    pub const fn defense_points(&self, slot: ArmorSlot) -> u32 {
        match (self, slot) {
            (ArmorTier::Leather, ArmorSlot::Helmet) => 1,
            (ArmorTier::Leather, ArmorSlot::Chestplate) => 3,
            (ArmorTier::Leather, ArmorSlot::Leggings) => 2,
            (ArmorTier::Leather, ArmorSlot::Boots) => 1,

            (ArmorTier::Chainmail, ArmorSlot::Helmet) => 2,
            (ArmorTier::Chainmail, ArmorSlot::Chestplate) => 5,
            (ArmorTier::Chainmail, ArmorSlot::Leggings) => 4,
            (ArmorTier::Chainmail, ArmorSlot::Boots) => 1,

            (ArmorTier::Iron, ArmorSlot::Helmet) => 2,
            (ArmorTier::Iron, ArmorSlot::Chestplate) => 6,
            (ArmorTier::Iron, ArmorSlot::Leggings) => 5,
            (ArmorTier::Iron, ArmorSlot::Boots) => 2,

            (ArmorTier::Gold, ArmorSlot::Helmet) => 2,
            (ArmorTier::Gold, ArmorSlot::Chestplate) => 5,
            (ArmorTier::Gold, ArmorSlot::Leggings) => 3,
            (ArmorTier::Gold, ArmorSlot::Boots) => 1,

            (ArmorTier::Diamond, ArmorSlot::Helmet) => 3,
            (ArmorTier::Diamond, ArmorSlot::Chestplate) => 8,
            (ArmorTier::Diamond, ArmorSlot::Leggings) => 6,
            (ArmorTier::Diamond, ArmorSlot::Boots) => 3,

            (ArmorTier::Turtle, ArmorSlot::Helmet) => 2,
            (ArmorTier::Turtle, _) => 0,

            (ArmorTier::Netherite, ArmorSlot::Helmet) => 3,
            (ArmorTier::Netherite, ArmorSlot::Chestplate) => 8,
            (ArmorTier::Netherite, ArmorSlot::Leggings) => 6,
            (ArmorTier::Netherite, ArmorSlot::Boots) => 3,
        }
    }

    /// Armor durability for the given slot.
    /// Slot base durability factors: Helmet: 11, Chestplate: 16, Leggings: 15, Boots: 13
    pub const fn durability(&self, slot: ArmorSlot) -> u32 {
        if matches!(self, ArmorTier::Turtle) {
            return if matches!(slot, ArmorSlot::Helmet) { 275 } else { 0 };
        }
        let factor = match slot {
            ArmorSlot::Helmet => 11,
            ArmorSlot::Chestplate => 16,
            ArmorSlot::Leggings => 15,
            ArmorSlot::Boots => 13,
        };
        let multiplier = match self {
            ArmorTier::Leather => 5,
            ArmorTier::Chainmail => 15,
            ArmorTier::Iron => 15,
            ArmorTier::Gold => 7,
            ArmorTier::Diamond => 33,
            ArmorTier::Netherite => 37,
            ArmorTier::Turtle => 0,
        };
        factor * multiplier
    }

    /// Armor toughness attribute.
    pub const fn toughness(&self) -> f32 {
        match self {
            ArmorTier::Diamond => 2.0,
            ArmorTier::Netherite => 3.0,
            _ => 0.0,
        }
    }

    /// Knockback resistance bonus (Netherite provides 0.1 per piece).
    pub const fn knockback_resistance(&self) -> f32 {
        match self {
            ArmorTier::Netherite => 0.1,
            _ => 0.0,
        }
    }

    /// Enchantability rating.
    pub const fn enchantability(&self) -> u32 {
        match self {
            ArmorTier::Leather => 15,
            ArmorTier::Chainmail => 12,
            ArmorTier::Iron => 9,
            ArmorTier::Gold => 25,
            ArmorTier::Diamond => 10,
            ArmorTier::Turtle => 9,
            ArmorTier::Netherite => 15,
        }
    }
}

/// Helper mapping from ItemId to Tool/Armor attributes.
impl ItemId {
    pub const fn tool_info(&self) -> Option<(ToolTier, ToolType)> {
        match self {
            ItemId::WoodenSword => Some((ToolTier::Wood, ToolType::Sword)),
            ItemId::WoodenPickaxe => Some((ToolTier::Wood, ToolType::Pickaxe)),
            ItemId::WoodenAxe => Some((ToolTier::Wood, ToolType::Axe)),
            ItemId::WoodenShovel => Some((ToolTier::Wood, ToolType::Shovel)),
            ItemId::WoodenHoe => Some((ToolTier::Wood, ToolType::Hoe)),

            ItemId::StoneSword => Some((ToolTier::Stone, ToolType::Sword)),
            ItemId::StonePickaxe => Some((ToolTier::Stone, ToolType::Pickaxe)),
            ItemId::StoneAxe => Some((ToolTier::Stone, ToolType::Axe)),
            ItemId::StoneShovel => Some((ToolTier::Stone, ToolType::Shovel)),
            ItemId::StoneHoe => Some((ToolTier::Stone, ToolType::Hoe)),

            ItemId::IronSword => Some((ToolTier::Iron, ToolType::Sword)),
            ItemId::IronPickaxe => Some((ToolTier::Iron, ToolType::Pickaxe)),
            ItemId::IronAxe => Some((ToolTier::Iron, ToolType::Axe)),
            ItemId::IronShovel => Some((ToolTier::Iron, ToolType::Shovel)),
            ItemId::IronHoe => Some((ToolTier::Iron, ToolType::Hoe)),

            ItemId::GoldenSword => Some((ToolTier::Gold, ToolType::Sword)),
            ItemId::GoldenPickaxe => Some((ToolTier::Gold, ToolType::Pickaxe)),
            ItemId::GoldenAxe => Some((ToolTier::Gold, ToolType::Axe)),
            ItemId::GoldenShovel => Some((ToolTier::Gold, ToolType::Shovel)),
            ItemId::GoldenHoe => Some((ToolTier::Gold, ToolType::Hoe)),

            ItemId::DiamondSword => Some((ToolTier::Diamond, ToolType::Sword)),
            ItemId::DiamondPickaxe => Some((ToolTier::Diamond, ToolType::Pickaxe)),
            ItemId::DiamondAxe => Some((ToolTier::Diamond, ToolType::Axe)),
            ItemId::DiamondShovel => Some((ToolTier::Diamond, ToolType::Shovel)),
            ItemId::DiamondHoe => Some((ToolTier::Diamond, ToolType::Hoe)),

            ItemId::NetheriteSword => Some((ToolTier::Netherite, ToolType::Sword)),
            ItemId::NetheritePickaxe => Some((ToolTier::Netherite, ToolType::Pickaxe)),
            ItemId::NetheriteAxe => Some((ToolTier::Netherite, ToolType::Axe)),
            ItemId::NetheriteShovel => Some((ToolTier::Netherite, ToolType::Shovel)),
            ItemId::NetheriteHoe => Some((ToolTier::Netherite, ToolType::Hoe)),

            _ => None,
        }
    }

    pub const fn armor_info(&self) -> Option<(ArmorTier, ArmorSlot)> {
        match self {
            ItemId::LeatherHelmet => Some((ArmorTier::Leather, ArmorSlot::Helmet)),
            ItemId::LeatherChestplate => Some((ArmorTier::Leather, ArmorSlot::Chestplate)),
            ItemId::LeatherLeggings => Some((ArmorTier::Leather, ArmorSlot::Leggings)),
            ItemId::LeatherBoots => Some((ArmorTier::Leather, ArmorSlot::Boots)),

            ItemId::ChainmailHelmet => Some((ArmorTier::Chainmail, ArmorSlot::Helmet)),
            ItemId::ChainmailChestplate => Some((ArmorTier::Chainmail, ArmorSlot::Chestplate)),
            ItemId::ChainmailLeggings => Some((ArmorTier::Chainmail, ArmorSlot::Leggings)),
            ItemId::ChainmailBoots => Some((ArmorTier::Chainmail, ArmorSlot::Boots)),

            ItemId::IronHelmet => Some((ArmorTier::Iron, ArmorSlot::Helmet)),
            ItemId::IronChestplate => Some((ArmorTier::Iron, ArmorSlot::Chestplate)),
            ItemId::IronLeggings => Some((ArmorTier::Iron, ArmorSlot::Leggings)),
            ItemId::IronBoots => Some((ArmorTier::Iron, ArmorSlot::Boots)),

            ItemId::GoldenHelmet => Some((ArmorTier::Gold, ArmorSlot::Helmet)),
            ItemId::GoldenChestplate => Some((ArmorTier::Gold, ArmorSlot::Chestplate)),
            ItemId::GoldenLeggings => Some((ArmorTier::Gold, ArmorSlot::Leggings)),
            ItemId::GoldenBoots => Some((ArmorTier::Gold, ArmorSlot::Boots)),

            ItemId::DiamondHelmet => Some((ArmorTier::Diamond, ArmorSlot::Helmet)),
            ItemId::DiamondChestplate => Some((ArmorTier::Diamond, ArmorSlot::Chestplate)),
            ItemId::DiamondLeggings => Some((ArmorTier::Diamond, ArmorSlot::Leggings)),
            ItemId::DiamondBoots => Some((ArmorTier::Diamond, ArmorSlot::Boots)),

            ItemId::TurtleHelmet => Some((ArmorTier::Turtle, ArmorSlot::Helmet)),

            ItemId::NetheriteHelmet => Some((ArmorTier::Netherite, ArmorSlot::Helmet)),
            ItemId::NetheriteChestplate => Some((ArmorTier::Netherite, ArmorSlot::Chestplate)),
            ItemId::NetheriteLeggings => Some((ArmorTier::Netherite, ArmorSlot::Leggings)),
            ItemId::NetheriteBoots => Some((ArmorTier::Netherite, ArmorSlot::Boots)),

            _ => None,
        }
    }
}
