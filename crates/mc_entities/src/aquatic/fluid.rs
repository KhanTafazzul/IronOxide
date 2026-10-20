// REA Verified: net.minecraft.world.entity.monster.Strider (mapped: cdc.class)
//
// Nether Fluid Mob: Strider lava ocean walking, shivering/purple-gray cold state,
// saddling, and warped fungus on a stick speed boosting.

use crate::components::EntityId;
use crate::passive::AnimalData;
use glam::Vec3;
use mc_items::ItemId;

/// Strider mob (`cdc.class`) capable of walking across lava oceans.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Strider {
    pub animal_data: AnimalData,
    /// Whether the strider is currently suffering cold (out of lava/fire).
    pub is_cold: bool,
    /// Shivering animation tick cycle when cold.
    pub shiver_ticks: u32,
    /// Whether the strider has a saddle equipped.
    pub is_saddled: bool,
    /// Speed boost countdown ticks when stimulated by Warped Fungus on a Stick.
    pub boost_ticks: u32,
    /// Entity ID of the currently mounted player rider, if any.
    pub rider: Option<EntityId>,
    pub health: f32,
    pub max_health: f32,
}

impl Default for Strider {
    fn default() -> Self {
        Self {
            animal_data: AnimalData::adult(),
            is_cold: false,
            shiver_ticks: 0,
            is_saddled: false,
            boost_ticks: 0,
            rider: None,
            health: 20.0, // 10 hearts
            max_health: 20.0,
        }
    }
}

impl Strider {
    pub const BASE_LAVA_SPEED: f32 = 0.3;
    pub const COLD_SPEED_PENALTY: f32 = 0.35; // ~65% speed reduction out of lava
    pub const BOOST_SPEED_MULTIPLIER: f32 = 1.5; // Warped fungus on a stick boost
    pub const BOOST_DURATION_TICKS: u32 = 40; // 2.0 seconds

    pub fn new() -> Self {
        Self::default()
    }

    pub fn baby() -> Self {
        Self {
            animal_data: AnimalData::baby(),
            ..Default::default()
        }
    }

    /// Breeding food is Warped Fungus.
    pub fn is_food(item: ItemId) -> bool {
        matches!(item, ItemId::WarpedFungus)
    }

    /// Updates thermal state based on whether the strider is submerged/standing in lava.
    /// In lava: happy, warm reddish color, full speed.
    /// Out of lava: shivering, turns grayish-purple, -65% movement speed penalty.
    pub fn update_temperature(&mut self, is_in_lava: bool) {
        if is_in_lava {
            self.is_cold = false;
            self.shiver_ticks = 0;
        } else {
            self.is_cold = true;
            self.shiver_ticks = (self.shiver_ticks + 1) % 10;
        }
    }

    /// Ticks environmental damage: Striders are harmed by water and rain (1.0 damage per 20 ticks).
    pub fn tick_water_damage(&mut self, is_in_water_or_rain: bool) -> Option<f32> {
        if is_in_water_or_rain {
            self.health = (self.health - 1.0).max(0.0);
            Some(1.0)
        } else {
            None
        }
    }

    /// Computes current walking/swimming speed modifier taking into account
    /// cold shivering penalty and active fungus speed boosts.
    pub fn current_speed(&self) -> f32 {
        let mut speed = Self::BASE_LAVA_SPEED;
        if self.is_cold {
            speed *= Self::COLD_SPEED_PENALTY;
        }
        if self.boost_ticks > 0 {
            speed *= Self::BOOST_SPEED_MULTIPLIER;
        }
        speed
    }

    /// Equips a saddle on the strider.
    pub fn equip_saddle(&mut self) -> bool {
        if !self.is_saddled && !self.animal_data.is_baby() {
            self.is_saddled = true;
            true
        } else {
            false
        }
    }

    /// Activates a speed boost using Warped Fungus on a Stick.
    pub fn apply_boost(&mut self) -> bool {
        if self.is_saddled && self.rider.is_some() {
            self.boost_ticks = Self::BOOST_DURATION_TICKS;
            true
        } else {
            false
        }
    }

    /// Interacts with player using held item (breeding with fungus, saddling, or boosting).
    pub fn interact(&mut self, player: EntityId, held_item: ItemId) -> bool {
        if Self::is_food(held_item) {
            if self.animal_data.is_baby() {
                self.animal_data.advance_baby_growth(2400);
                return true;
            } else if self.animal_data.can_breed() {
                self.animal_data.set_in_love(Some(player));
                return true;
            }
        } else if matches!(held_item, ItemId::Saddle) && !self.is_saddled && !self.animal_data.is_baby() {
            self.is_saddled = true;
            return true;
        } else if matches!(held_item, ItemId::WarpedFungusOnAStick) && self.is_saddled {
            return self.apply_boost();
        }
        false
    }

    /// Ticks boost timer and animal data.
    pub fn tick(&mut self) {
        self.animal_data.tick();
        if self.boost_ticks > 0 {
            self.boost_ticks -= 1;
        }
    }

    /// Steers the strider in the rider's facing direction if mounted.
    pub fn compute_movement_vector(&self, look_direction: Vec3) -> Vec3 {
        let speed = self.current_speed();
        let horizontal = Vec3::new(look_direction.x, 0.0, look_direction.z).normalize_or_zero();
        horizontal * speed
    }

    /// Drops when killed: 2 to 5 String, plus equipped Saddle.
    pub fn get_death_drops(&self) -> Vec<ItemId> {
        let mut drops = vec![ItemId::String, ItemId::String];
        if self.is_saddled {
            drops.push(ItemId::Saddle);
        }
        drops
    }
}
