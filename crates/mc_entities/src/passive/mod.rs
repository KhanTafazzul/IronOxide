// REA Verified: net.minecraft.world.entity.animal.Animal (mapped: bxx.class)
// REA Verified: net.minecraft.world.entity.TamableAnimal (mapped: bya.class)
// REA Verified: net.minecraft.world.item.DyeColor (mapped: bay.class)
//
// Core Passive Land Mammals & Birds data structures, breeding mechanics,
// dye coloring systems, and shared tameable animal traits.

pub mod farm;
pub mod tameable;
pub mod wild;

pub use farm::*;
pub use tameable::*;
pub use wild::*;

use crate::components::EntityId;
use mc_items::ItemId;

/// Vanilla 16 Dye Colors matching `net.minecraft.world.item.DyeColor` (`bay.class`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum DyeColor {
    #[default]
    White = 0,
    Orange = 1,
    Magenta = 2,
    LightBlue = 3,
    Yellow = 4,
    Lime = 5,
    Pink = 6,
    Gray = 7,
    LightGray = 8,
    Cyan = 9,
    Purple = 10,
    Blue = 11,
    Brown = 12,
    Green = 13,
    Red = 14,
    Black = 15,
}

impl DyeColor {
    /// Array of all 16 dye colors in canonical ID order.
    pub const ALL: [DyeColor; 16] = [
        DyeColor::White,
        DyeColor::Orange,
        DyeColor::Magenta,
        DyeColor::LightBlue,
        DyeColor::Yellow,
        DyeColor::Lime,
        DyeColor::Pink,
        DyeColor::Gray,
        DyeColor::LightGray,
        DyeColor::Cyan,
        DyeColor::Purple,
        DyeColor::Blue,
        DyeColor::Brown,
        DyeColor::Green,
        DyeColor::Red,
        DyeColor::Black,
    ];

    /// Returns the corresponding Wool item ID for this sheep/wool color.
    pub fn to_wool_item_id(&self) -> ItemId {
        match self {
            Self::White => ItemId::WhiteWool,
            Self::Orange => ItemId::OrangeWool,
            Self::Magenta => ItemId::MagentaWool,
            Self::LightBlue => ItemId::LightBlueWool,
            Self::Yellow => ItemId::YellowWool,
            Self::Lime => ItemId::LimeWool,
            Self::Pink => ItemId::PinkWool,
            Self::Gray => ItemId::GrayWool,
            Self::LightGray => ItemId::LightGrayWool,
            Self::Cyan => ItemId::CyanWool,
            Self::Purple => ItemId::PurpleWool,
            Self::Blue => ItemId::BlueWool,
            Self::Brown => ItemId::BrownWool,
            Self::Green => ItemId::GreenWool,
            Self::Red => ItemId::RedWool,
            Self::Black => ItemId::BlackWool,
        }
    }

    /// Resolves DyeColor from a Wool ItemId.
    pub fn from_wool_item_id(item: ItemId) -> Option<Self> {
        match item {
            ItemId::WhiteWool => Some(Self::White),
            ItemId::OrangeWool => Some(Self::Orange),
            ItemId::MagentaWool => Some(Self::Magenta),
            ItemId::LightBlueWool => Some(Self::LightBlue),
            ItemId::YellowWool => Some(Self::Yellow),
            ItemId::LimeWool => Some(Self::Lime),
            ItemId::PinkWool => Some(Self::Pink),
            ItemId::GrayWool => Some(Self::Gray),
            ItemId::LightGrayWool => Some(Self::LightGray),
            ItemId::CyanWool => Some(Self::Cyan),
            ItemId::PurpleWool => Some(Self::Purple),
            ItemId::BlueWool => Some(Self::Blue),
            ItemId::BrownWool => Some(Self::Brown),
            ItemId::GreenWool => Some(Self::Green),
            ItemId::RedWool => Some(Self::Red),
            ItemId::BlackWool => Some(Self::Black),
            _ => None,
        }
    }

    /// Normalized RGB floating-point color tuple (r, g, b) matching vanilla texture palettes.
    pub fn rgb(&self) -> (f32, f32, f32) {
        match self {
            Self::White => (0.9019, 0.9019, 0.9019),
            Self::Orange => (0.8509, 0.4666, 0.1764),
            Self::Magenta => (0.7019, 0.2862, 0.6901),
            Self::LightBlue => (0.4, 0.6, 0.8509),
            Self::Yellow => (0.8705, 0.7725, 0.2549),
            Self::Lime => (0.5019, 0.7019, 0.1568),
            Self::Pink => (0.8745, 0.5411, 0.6431),
            Self::Gray => (0.2941, 0.3137, 0.3372),
            Self::LightGray => (0.6156, 0.6156, 0.5725),
            Self::Cyan => (0.1764, 0.5411, 0.6117),
            Self::Purple => (0.5098, 0.2313, 0.7058),
            Self::Blue => (0.2431, 0.2980, 0.6784),
            Self::Brown => (0.4509, 0.2901, 0.1882),
            Self::Green => (0.3450, 0.4784, 0.1686),
            Self::Red => (0.6431, 0.1764, 0.1764),
            Self::Black => (0.1176, 0.1176, 0.1372),
        }
    }

    /// Blends two sheep parent dye colors following vanilla crafting recipe blending.
    pub fn blend(color1: DyeColor, color2: DyeColor) -> DyeColor {
        if color1 == color2 {
            return color1;
        }
        let pair = if (color1 as u8) < (color2 as u8) {
            (color1, color2)
        } else {
            (color2, color1)
        };
        match pair {
            (DyeColor::White, DyeColor::Gray) => DyeColor::LightGray,
            (DyeColor::White, DyeColor::Black) => DyeColor::Gray,
            (DyeColor::White, DyeColor::Blue) => DyeColor::LightBlue,
            (DyeColor::White, DyeColor::Green) => DyeColor::Lime,
            (DyeColor::White, DyeColor::Red) => DyeColor::Pink,
            (DyeColor::Yellow, DyeColor::Blue) => DyeColor::Green,
            (DyeColor::Yellow, DyeColor::Red) => DyeColor::Orange,
            (DyeColor::Blue, DyeColor::Green) => DyeColor::Cyan,
            (DyeColor::Blue, DyeColor::Red) => DyeColor::Purple,
            (DyeColor::Pink, DyeColor::Purple) => DyeColor::Magenta,
            _ => {
                // Default fallback: 50% probability of either parent
                color1
            }
        }
    }
}

/// Standard Animal breeding and age component matching `AgeableMob` / `Animal`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnimalData {
    /// Age in ticks:
    /// - Negative values (-24000 to -1): Baby growing up (20 minutes).
    /// - Zero (0): Mature adult ready to breed.
    /// - Positive values (> 0): Breeding cooldown (e.g. 6000 ticks = 5 minutes).
    pub age: i32,
    /// In-love timer: counts down from 600 ticks (30 seconds) while emitting heart particles.
    pub in_love_ticks: u32,
    /// Player entity ID who fed this animal into love mode.
    pub love_cause: Option<EntityId>,
    /// Breeding cooldown timer applied after successfully producing offspring (typically 6,000 ticks).
    pub breed_cooldown_ticks: u32,
}

impl Default for AnimalData {
    fn default() -> Self {
        Self {
            age: 0,
            in_love_ticks: 0,
            love_cause: None,
            breed_cooldown_ticks: 0,
        }
    }
}

impl AnimalData {
    pub const BABY_GROWTH_TICKS: i32 = -24000;
    pub const BREEDING_COOLDOWN_TICKS: u32 = 6000;
    pub const IN_LOVE_DURATION_TICKS: u32 = 600;

    /// Creates an adult animal ready to breed.
    pub fn adult() -> Self {
        Self::default()
    }

    /// Creates a newly born baby animal (-24,000 ticks = 20 minutes to adulthood).
    pub fn baby() -> Self {
        Self {
            age: Self::BABY_GROWTH_TICKS,
            in_love_ticks: 0,
            love_cause: None,
            breed_cooldown_ticks: 0,
        }
    }

    /// Returns true if this animal is a baby.
    pub fn is_baby(&self) -> bool {
        self.age < 0
    }

    /// Returns true if this animal is ready to enter love mode / breed.
    pub fn can_breed(&self) -> bool {
        self.age == 0 && self.in_love_ticks == 0 && self.breed_cooldown_ticks == 0
    }

    /// Sets this animal into love mode when fed appropriate food.
    pub fn set_in_love(&mut self, player: Option<EntityId>) {
        if self.can_breed() {
            self.in_love_ticks = Self::IN_LOVE_DURATION_TICKS;
            self.love_cause = player;
        }
    }

    /// Resets love mode and applies standard 5-minute breeding cooldown.
    pub fn on_bred(&mut self) {
        self.in_love_ticks = 0;
        self.love_cause = None;
        self.breed_cooldown_ticks = Self::BREEDING_COOLDOWN_TICKS;
        self.age = Self::BREEDING_COOLDOWN_TICKS as i32;
    }

    /// Accelerated baby growth (e.g. feeding food advances growth by 10%).
    pub fn advance_baby_growth(&mut self, ticks: i32) {
        if self.is_baby() {
            self.age = (self.age + ticks).min(0);
        }
    }

    /// Ticks the animal data state once per simulation tick (20 Hz).
    pub fn tick(&mut self) {
        if self.age < 0 {
            self.age += 1;
        } else if self.age > 0 {
            self.age -= 1;
        }
        if self.in_love_ticks > 0 {
            self.in_love_ticks -= 1;
            if self.in_love_ticks == 0 {
                self.love_cause = None;
            }
        }
        if self.breed_cooldown_ticks > 0 {
            self.breed_cooldown_ticks -= 1;
        }
    }
}

/// Shared Tameable Animal Component matching `TamableAnimal` (`bya.class`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TameableData {
    pub is_tamed: bool,
    pub owner_id: Option<EntityId>,
    pub is_sitting: bool,
    pub collar_color: Option<DyeColor>,
}

impl Default for TameableData {
    fn default() -> Self {
        Self {
            is_tamed: false,
            owner_id: None,
            is_sitting: false,
            collar_color: None,
        }
    }
}

impl TameableData {
    /// Creates an untamed entity state.
    pub fn untamed() -> Self {
        Self::default()
    }

    /// Tames the animal to a specific player owner.
    pub fn tame(&mut self, owner: EntityId, default_collar: Option<DyeColor>) {
        self.is_tamed = true;
        self.owner_id = Some(owner);
        self.is_sitting = false;
        self.collar_color = default_collar;
    }

    /// Toggles the sitting / stay state of a tamed mob.
    pub fn toggle_sitting(&mut self) {
        if self.is_tamed {
            self.is_sitting = !self.is_sitting;
        }
    }
}
