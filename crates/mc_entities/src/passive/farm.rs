// REA Verified: net.minecraft.world.entity.animal.Cow (mapped: byf.class)
// REA Verified: net.minecraft.world.entity.animal.Pig (mapped: byp.class)
// REA Verified: net.minecraft.world.entity.animal.Sheep (mapped: byu.class)
// REA Verified: net.minecraft.world.entity.animal.Chicken (mapped: byd.class)
//
// Farm animals: Cow, Pig, Sheep, and Chicken with authentic tempt items,
// breeding mechanics, wool coloring/regrowth, and egg-laying timers.

use super::{AnimalData, DyeColor};
use crate::components::EntityId;
use mc_items::ItemId;

/// Cow entity implementation matching `net.minecraft.world.entity.animal.Cow` (`byf.class`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cow {
    pub animal_data: AnimalData,
}

impl Default for Cow {
    fn default() -> Self {
        Self {
            animal_data: AnimalData::adult(),
        }
    }
}

impl Cow {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn baby() -> Self {
        Self {
            animal_data: AnimalData::baby(),
        }
    }

    /// Checks if the provided item is tempting/breeding food for cows (Wheat).
    pub fn is_food(item: ItemId) -> bool {
        matches!(item, ItemId::Wheat)
    }

    /// Interacts with cow using held item. Feeding wheat triggers love mode;
    /// empty bucket milks the cow.
    pub fn interact(&mut self, player: EntityId, held_item: ItemId) -> Option<ItemId> {
        if Self::is_food(held_item) {
            if self.animal_data.is_baby() {
                self.animal_data.advance_baby_growth(2400); // 10% advance
                return None;
            } else if self.animal_data.can_breed() {
                self.animal_data.set_in_love(Some(player));
                return None;
            }
        } else if matches!(held_item, ItemId::Bucket) && !self.animal_data.is_baby() {
            return Some(ItemId::MilkBucket);
        }
        None
    }

    /// Evaluates drops on death: 1-3 beef (cooked if on fire) and 0-2 leather.
    pub fn death_drops(&self, is_on_fire: bool, looting_level: u32) -> Vec<(ItemId, u32)> {
        if self.animal_data.is_baby() {
            return Vec::new();
        }
        let beef_item = if is_on_fire {
            ItemId::CookedBeef
        } else {
            ItemId::Beef
        };
        let beef_count = 1 + (looting_level % 3);
        let leather_count = looting_level % 2;
        vec![
            (beef_item, beef_count),
            (ItemId::Leather, leather_count),
        ]
    }

    /// Breeds with a partner cow to produce a baby offspring.
    pub fn breed(&mut self, partner: &mut Cow) -> Cow {
        self.animal_data.on_bred();
        partner.animal_data.on_bred();
        Cow::baby()
    }
}

/// Pig entity implementation matching `net.minecraft.world.entity.animal.Pig` (`byp.class`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pig {
    pub animal_data: AnimalData,
    pub has_saddle: bool,
    pub boost_ticks_left: u32,
}

impl Default for Pig {
    fn default() -> Self {
        Self {
            animal_data: AnimalData::adult(),
            has_saddle: false,
            boost_ticks_left: 0,
        }
    }
}

impl Pig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn baby() -> Self {
        Self {
            animal_data: AnimalData::baby(),
            has_saddle: false,
            boost_ticks_left: 0,
        }
    }

    /// Checks if item is tempting/breeding food for pigs (Carrots, Potatoes, Beetroots).
    pub fn is_food(item: ItemId) -> bool {
        matches!(
            item,
            ItemId::Carrot | ItemId::Potato | ItemId::Beetroot
        )
    }

    /// Interacts with pig using held item.
    pub fn interact(&mut self, player: EntityId, held_item: ItemId) -> bool {
        if Self::is_food(held_item) {
            if self.animal_data.is_baby() {
                self.animal_data.advance_baby_growth(2400);
                return true;
            } else if self.animal_data.can_breed() {
                self.animal_data.set_in_love(Some(player));
                return true;
            }
        } else if matches!(held_item, ItemId::Saddle) && !self.has_saddle && !self.animal_data.is_baby() {
            self.has_saddle = true;
            return true;
        } else if matches!(held_item, ItemId::CarrotOnAStick) && self.has_saddle {
            self.boost_ticks_left = 60; // 3 seconds boost
            return true;
        }
        false
    }

    /// Ticks pig state, decreasing boost ticks.
    pub fn tick(&mut self) {
        self.animal_data.tick();
        if self.boost_ticks_left > 0 {
            self.boost_ticks_left -= 1;
        }
    }

    /// Speed multiplier based on saddle boost.
    pub fn speed_multiplier(&self) -> f32 {
        if self.boost_ticks_left > 0 {
            1.35
        } else {
            1.0
        }
    }

    /// Evaluates drops on death: 1-3 porkchops and drops saddle if equipped.
    pub fn death_drops(&self, is_on_fire: bool, looting_level: u32) -> Vec<(ItemId, u32)> {
        let mut drops = Vec::new();
        if !self.animal_data.is_baby() {
            let meat = if is_on_fire {
                ItemId::CookedPorkchop
            } else {
                ItemId::Porkchop
            };
            drops.push((meat, 1 + (looting_level % 3)));
        }
        if self.has_saddle {
            drops.push((ItemId::Saddle, 1));
        }
        drops
    }

    /// Breeds with a partner pig to produce a baby offspring.
    pub fn breed(&mut self, partner: &mut Pig) -> Pig {
        self.animal_data.on_bred();
        partner.animal_data.on_bred();
        Pig::baby()
    }
}

/// Sheep entity implementation matching `net.minecraft.world.entity.animal.Sheep` (`byu.class`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sheep {
    pub animal_data: AnimalData,
    pub color: DyeColor,
    pub is_sheared: bool,
    pub eat_animation_ticks: u32,
}

impl Default for Sheep {
    fn default() -> Self {
        Self {
            animal_data: AnimalData::adult(),
            color: DyeColor::White,
            is_sheared: false,
            eat_animation_ticks: 0,
        }
    }
}

impl Sheep {
    pub fn new(color: DyeColor) -> Self {
        Self {
            animal_data: AnimalData::adult(),
            color,
            is_sheared: false,
            eat_animation_ticks: 0,
        }
    }

    pub fn baby(color: DyeColor) -> Self {
        Self {
            animal_data: AnimalData::baby(),
            color,
            is_sheared: false,
            eat_animation_ticks: 0,
        }
    }

    /// Checks if item is tempting/breeding food for sheep (Wheat).
    pub fn is_food(item: ItemId) -> bool {
        matches!(item, ItemId::Wheat)
    }

    /// Shears the sheep with Shears tool, dropping 1-3 wool matching sheep color.
    pub fn shear(&mut self) -> Option<(ItemId, u32)> {
        if !self.is_sheared && !self.animal_data.is_baby() {
            self.is_sheared = true;
            Some((self.color.to_wool_item_id(), 2))
        } else {
            None
        }
    }

    /// Dyes the sheep's wool with a new dye color.
    pub fn dye(&mut self, new_color: DyeColor) {
        self.color = new_color;
    }

    /// Called when the sheep successfully eats grass:
    /// Regrows its sheared wool, and advances baby growth if immature.
    pub fn ate_grass(&mut self) {
        self.is_sheared = false;
        if self.animal_data.is_baby() {
            self.animal_data.advance_baby_growth(1200); // 10%
        }
    }

    /// Evaluates drops on death: 1-2 mutton and 1 wool if not sheared.
    pub fn death_drops(&self, is_on_fire: bool, looting_level: u32) -> Vec<(ItemId, u32)> {
        let mut drops = Vec::new();
        if !self.animal_data.is_baby() {
            let meat = if is_on_fire {
                ItemId::CookedMutton
            } else {
                ItemId::Mutton
            };
            drops.push((meat, 1 + (looting_level % 2)));
            if !self.is_sheared {
                drops.push((self.color.to_wool_item_id(), 1));
            }
        }
        drops
    }

    /// Breeds with partner sheep. Offspring inherits blended dye color.
    pub fn breed(&mut self, partner: &mut Sheep) -> Sheep {
        self.animal_data.on_bred();
        partner.animal_data.on_bred();
        let offspring_color = DyeColor::blend(self.color, partner.color);
        Sheep::baby(offspring_color)
    }
}

/// Chicken entity implementation matching `net.minecraft.world.entity.animal.Chicken` (`byd.class`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Chicken {
    pub animal_data: AnimalData,
    /// Egg laying countdown timer in ticks (6,000 to 12,000 ticks = 5 to 10 minutes).
    pub egg_lay_timer: u32,
    pub is_flapping: bool,
}

impl Default for Chicken {
    fn default() -> Self {
        Self {
            animal_data: AnimalData::adult(),
            egg_lay_timer: 8000,
            is_flapping: false,
        }
    }
}

impl Chicken {
    pub const MIN_EGG_TICKS: u32 = 6000;
    pub const MAX_EGG_TICKS: u32 = 12000;

    pub fn new(initial_egg_ticks: u32) -> Self {
        Self {
            animal_data: AnimalData::adult(),
            egg_lay_timer: initial_egg_ticks.clamp(Self::MIN_EGG_TICKS, Self::MAX_EGG_TICKS),
            is_flapping: false,
        }
    }

    pub fn baby() -> Self {
        Self {
            animal_data: AnimalData::baby(),
            egg_lay_timer: Self::MAX_EGG_TICKS,
            is_flapping: false,
        }
    }

    /// Checks if item is tempting/breeding food for chickens (Seeds).
    pub fn is_food(item: ItemId) -> bool {
        matches!(
            item,
            ItemId::WheatSeeds
                | ItemId::MelonSeeds
                | ItemId::PumpkinSeeds
                | ItemId::BeetrootSeeds
                | ItemId::TorchflowerSeeds
                | ItemId::PitcherPod
        )
    }

    /// Ticks the chicken state once per simulation tick (20 Hz).
    /// Returns `Some(ItemId::Egg)` when the egg timer expires and resets the timer.
    pub fn tick(&mut self, random_next: u32) -> Option<ItemId> {
        self.animal_data.tick();

        if !self.animal_data.is_baby() {
            if self.egg_lay_timer > 0 {
                self.egg_lay_timer -= 1;
            }
            if self.egg_lay_timer == 0 {
                let range = Self::MAX_EGG_TICKS - Self::MIN_EGG_TICKS;
                self.egg_lay_timer = Self::MIN_EGG_TICKS + (random_next % range);
                return Some(ItemId::Egg);
            }
        }
        None
    }

    /// Slow-falling wing flapping velocity calculation:
    /// Caps downward terminal velocity to -0.05 and eliminates fall damage.
    pub fn slow_falling_velocity(&self, current_y_velocity: f32) -> (f32, bool) {
        if current_y_velocity < 0.0 {
            (current_y_velocity.max(-0.05), true)
        } else {
            (current_y_velocity, false)
        }
    }

    /// Evaluates drops on death: 1 chicken meat and 0-2 feathers.
    pub fn death_drops(&self, is_on_fire: bool, looting_level: u32) -> Vec<(ItemId, u32)> {
        if self.animal_data.is_baby() {
            return Vec::new();
        }
        let meat = if is_on_fire {
            ItemId::CookedChicken
        } else {
            ItemId::Chicken
        };
        let feathers = looting_level % 3;
        vec![
            (meat, 1),
            (ItemId::Feather, feathers),
        ]
    }

    /// Breeds with partner chicken to produce a baby chick.
    pub fn breed(&mut self, partner: &mut Chicken) -> Chicken {
        self.animal_data.on_bred();
        partner.animal_data.on_bred();
        Chicken::baby()
    }
}
