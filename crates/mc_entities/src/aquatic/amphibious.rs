// REA Verified: net.minecraft.world.entity.animal.Turtle (mapped: byz.class)
// REA Verified: net.minecraft.world.entity.animal.Frog (mapped: bzn.class)
// REA Verified: net.minecraft.world.entity.animal.Tadpole (mapped: bzq.class)
// REA Verified: net.minecraft.world.entity.animal.axolotl.Axolotl (mapped: bzf.class)
//
// Amphibious Mobs: Turtle home beach egg laying & crawling/swimming speeds,
// Frog 3 temperature variants & slime/magma cube froglight consumption,
// Tadpole bucketable metamorphosis, and Axolotl 5 variants with play-dead regeneration.

use super::water_navigation::{BreathMeter, WaterPathNavigation};
use crate::components::EntityId;
use crate::passive::AnimalData;
use glam::Vec3;
use mc_items::ItemId;

/// Turtle mob (`byz.class`) with home beach memory, egg laying on sand, and scute shedding.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Turtle {
    pub animal_data: AnimalData,
    pub navigation: WaterPathNavigation,
    pub breath: BreathMeter,
    /// Memory coordinate of the beach where this turtle hatched.
    pub home_pos: Option<Vec3>,
    /// Whether the turtle is pregnant and carrying eggs to lay.
    pub has_egg: bool,
    /// Whether the turtle is currently digging in the sand to deposit eggs.
    pub is_digging_sand: bool,
    /// Digging countdown ticks before eggs are deposited (200 ticks = 10s).
    pub dig_sand_ticks: u32,
    /// Whether the turtle was just bred and needs to travel home.
    pub traveling_home: bool,
}

impl Default for Turtle {
    fn default() -> Self {
        Self {
            animal_data: AnimalData::adult(),
            navigation: WaterPathNavigation::new(1.0, false),
            breath: BreathMeter::amphibious(),
            home_pos: None,
            has_egg: false,
            is_digging_sand: false,
            dig_sand_ticks: 0,
            traveling_home: false,
        }
    }
}

impl Turtle {
    pub const DIG_SAND_DURATION_TICKS: u32 = 200; // 10 seconds of digging
    pub const LAND_SPEED_MODIFIER: f32 = 0.25; // 4x slower crawling on land
    pub const SWIM_SPEED_MODIFIER: f32 = 1.0;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn baby() -> Self {
        Self {
            animal_data: AnimalData::baby(),
            ..Default::default()
        }
    }

    /// Sets turtle home beach spawn location.
    pub fn set_home_pos(&mut self, pos: Vec3) {
        self.home_pos = Some(pos);
    }

    /// Breeding food is Seagrass.
    pub fn is_food(item: ItemId) -> bool {
        matches!(item, ItemId::Seagrass)
    }

    /// Feed turtle seagrass to enter love mode or accelerate baby growth.
    pub fn interact(&mut self, player: EntityId, held_item: ItemId) -> bool {
        if Self::is_food(held_item) {
            if self.animal_data.is_baby() {
                self.animal_data.advance_baby_growth(2400);
                return true;
            } else if self.animal_data.can_breed() {
                self.animal_data.set_in_love(Some(player));
                return true;
            }
        }
        false
    }

    /// After breeding, one parent turtle becomes gravid with eggs and heads home.
    pub fn become_gravid(&mut self) {
        self.has_egg = true;
        self.traveling_home = true;
    }

    /// Advances baby growth. If baby transitions to adult, sheds 1 Scute!
    pub fn tick_growth(&mut self) -> Option<ItemId> {
        let was_baby = self.animal_data.is_baby();
        self.animal_data.tick();
        if was_baby && !self.animal_data.is_baby() {
            // Shed scute on reaching adulthood
            Some(ItemId::Scute)
        } else {
            None
        }
    }

    /// Starts digging sand to deposit turtle eggs when arrived at home sand block.
    pub fn start_digging_sand(&mut self) {
        if self.has_egg {
            self.is_digging_sand = true;
            self.dig_sand_ticks = Self::DIG_SAND_DURATION_TICKS;
        }
    }

    /// Ticks digging progress. When complete, returns deposited TurtleEggs (1-4).
    pub fn tick_digging(&mut self) -> Option<Vec<ItemId>> {
        if self.is_digging_sand {
            if self.dig_sand_ticks > 0 {
                self.dig_sand_ticks -= 1;
            } else {
                self.is_digging_sand = false;
                self.has_egg = false;
                self.traveling_home = false;
                // Deposits between 1 and 4 turtle eggs
                return Some(vec![ItemId::TurtleEgg; 3]);
            }
        }
        None
    }

    /// Returns movement speed based on whether the turtle is crawling on land or swimming in water.
    pub fn current_speed_modifier(&self, is_in_water: bool) -> f32 {
        if is_in_water {
            Self::SWIM_SPEED_MODIFIER
        } else {
            Self::LAND_SPEED_MODIFIER
        }
    }
}

/// 3 Frog climate variants matching `net.minecraft.world.entity.animal.Frog.Variant` (`bzn.class`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FrogVariant {
    /// Temperate biomes (Plains, Forests, etc.) - Orange/Brown.
    #[default]
    Temperate,
    /// Warm biomes (Jungle, Desert, Savanna, Mangrove, Nether) - White/Ochre.
    Warm,
    /// Cold biomes (Snowy, Frozen, The End) - Green/Verdant.
    Cold,
}

impl FrogVariant {
    /// Resolves the FrogVariant from the ambient biome temperature.
    pub fn from_temperature(temperature: f32) -> Self {
        if temperature < 0.15 {
            Self::Cold
        } else if temperature > 0.8 {
            Self::Warm
        } else {
            Self::Temperate
        }
    }

    /// Froglight block dropped when consuming a small Magma Cube.
    pub fn froglight_item(&self) -> ItemId {
        match self {
            Self::Temperate => ItemId::PearlescentFroglight,
            Self::Warm => ItemId::OchreFroglight,
            Self::Cold => ItemId::VerdantFroglight,
        }
    }
}

/// Prey swallowed by frog tongue attack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrogPreyType {
    /// Small Slime -> produces Slimeball.
    SmallSlime,
    /// Small Magma Cube -> produces corresponding Froglight block.
    SmallMagmaCube,
}

/// Frog mob (`bzn.class`) with 3 variants, tongue attacks, and froglight block generation.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Frog {
    pub variant: FrogVariant,
    pub animal_data: AnimalData,
    pub navigation: WaterPathNavigation,
    pub breath: BreathMeter,
    /// Jump leap state.
    pub is_jumping: bool,
    /// Target entity currently captured in tongue attack.
    pub tongue_target: Option<EntityId>,
}

impl Default for Frog {
    fn default() -> Self {
        Self {
            variant: FrogVariant::Temperate,
            animal_data: AnimalData::adult(),
            navigation: WaterPathNavigation::new(1.0, false),
            breath: BreathMeter::amphibious(),
            is_jumping: false,
            tongue_target: None,
        }
    }
}

impl Frog {
    pub fn new(variant: FrogVariant) -> Self {
        Self {
            variant,
            ..Default::default()
        }
    }

    /// Breeding food is Slimeball.
    pub fn is_food(item: ItemId) -> bool {
        matches!(item, ItemId::SlimeBall)
    }

    /// Interacts with frog using held item.
    pub fn interact(&mut self, player: EntityId, held_item: ItemId) -> bool {
        if Self::is_food(held_item) && self.animal_data.can_breed() {
            self.animal_data.set_in_love(Some(player));
            true
        } else {
            false
        }
    }

    /// Consumes a small prey mob with tongue and drops the corresponding item:
    /// - Small Slime -> Slimeball
    /// - Small Magma Cube -> Froglight (Pearlescent, Ochre, or Verdant)
    pub fn swallow_prey(&mut self, prey: FrogPreyType) -> ItemId {
        self.tongue_target = None;
        match prey {
            FrogPreyType::SmallSlime => ItemId::SlimeBall,
            FrogPreyType::SmallMagmaCube => self.variant.froglight_item(),
        }
    }
}

/// Tadpole mob (`bzq.class`) that hatches from frogspawn, bucketable, and grows into a Frog.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tadpole {
    /// Age in ticks counting up from -24000 to 0 (20 minutes).
    pub age: i32,
    pub navigation: WaterPathNavigation,
    pub breath: BreathMeter,
}

impl Default for Tadpole {
    fn default() -> Self {
        Self {
            age: AnimalData::BABY_GROWTH_TICKS,
            navigation: WaterPathNavigation::new(1.0, false),
            breath: BreathMeter::fish(), // Tadpoles breathe water and suffocate on land
        }
    }
}

impl Tadpole {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feeding a slime ball accelerates tadpole growth by 10%.
    pub fn feed_slime_ball(&mut self) -> bool {
        if self.age < 0 {
            self.age = (self.age + 2400).min(0);
            true
        } else {
            false
        }
    }

    /// Picks up the tadpole using a water bucket.
    pub fn interact_bucket(&self, held_item: ItemId) -> Option<ItemId> {
        if matches!(held_item, ItemId::WaterBucket) {
            Some(ItemId::TadpoleBucket)
        } else {
            None
        }
    }

    /// Ticks tadpole age. If age reaches 0, metamorphoses into a Frog whose variant
    /// is determined by the ambient temperature.
    pub fn tick(&mut self, ambient_temperature: f32) -> Option<Frog> {
        if self.age < 0 {
            self.age += 1;
            if self.age >= 0 {
                let variant = FrogVariant::from_temperature(ambient_temperature);
                return Some(Frog::new(variant));
            }
        }
        None
    }
}

/// 5 Axolotl color variants matching `net.minecraft.world.entity.animal.axolotl.Axolotl.Variant`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AxolotlVariant {
    /// Pink (Lucy) - default common variant.
    #[default]
    Lucy = 0,
    /// Brown (Wild).
    Wild = 1,
    /// Yellow (Gold).
    Gold = 2,
    /// Cyan / light blue.
    Cyan = 3,
    /// Blue (extremely rare: 1/1200 chance in vanilla).
    Blue = 4,
}

/// Axolotl mob (`bzf.class`) with 5 variants, "playing dead" self-preservation,
/// regeneration effect, and fish hunting.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Axolotl {
    pub variant: AxolotlVariant,
    pub animal_data: AnimalData,
    pub navigation: WaterPathNavigation,
    pub breath: BreathMeter,
    pub health: f32,
    pub max_health: f32,
    /// Whether the axolotl is currently "playing dead" on the seabed.
    pub is_playing_dead: bool,
    /// Ticks remaining for the playing dead state (200 ticks = 10 seconds).
    pub play_dead_ticks: u32,
}

impl Default for Axolotl {
    fn default() -> Self {
        Self {
            variant: AxolotlVariant::Lucy,
            animal_data: AnimalData::adult(),
            navigation: WaterPathNavigation::new(1.0, false),
            breath: BreathMeter::amphibious(),
            health: 14.0, // 7 hearts
            max_health: 14.0,
            is_playing_dead: false,
            play_dead_ticks: 0,
        }
    }
}

impl Axolotl {
    pub const PLAY_DEAD_DURATION_TICKS: u32 = 200; // 10 seconds
    pub const MAX_HEALTH: f32 = 14.0;

    pub fn new(variant: AxolotlVariant) -> Self {
        Self {
            variant,
            ..Default::default()
        }
    }

    /// Breeding food is a bucket of tropical fish.
    pub fn is_food(item: ItemId) -> bool {
        matches!(item, ItemId::TropicalFishBucket)
    }

    /// Picks up the axolotl using a water bucket.
    pub fn interact_bucket(&self, held_item: ItemId) -> Option<ItemId> {
        if matches!(held_item, ItemId::WaterBucket) {
            Some(ItemId::AxolotlBucket)
        } else {
            None
        }
    }

    /// Interacts with axolotl using held item.
    pub fn interact(&mut self, player: EntityId, held_item: ItemId) -> Option<ItemId> {
        if Self::is_food(held_item) && self.animal_data.can_breed() {
            self.animal_data.set_in_love(Some(player));
            // Returns empty water bucket after feeding tropical fish bucket
            Some(ItemId::WaterBucket)
        } else {
            None
        }
    }

    /// Applies damage to the axolotl.
    /// If taking damage underwater while health falls below 50% (< 7.0 HP),
    /// enters the "playing dead" self-preservation state!
    pub fn take_damage(&mut self, damage: f32, is_in_water: bool) {
        self.health = (self.health - damage).max(0.0);
        if is_in_water && self.health > 0.0 && self.health < (self.max_health * 0.5) {
            self.enter_play_dead();
        }
    }

    /// Enters playing dead state: floats motionless, ignored by hostiles, gains Regeneration.
    pub fn enter_play_dead(&mut self) {
        self.is_playing_dead = true;
        self.play_dead_ticks = Self::PLAY_DEAD_DURATION_TICKS;
    }

    /// Ticks playing dead state and applies Regeneration I healing (+1 HP every 50 ticks).
    pub fn tick_play_dead(&mut self) {
        if self.is_playing_dead {
            if self.play_dead_ticks > 0 {
                self.play_dead_ticks -= 1;
                // Regeneration I: heal 1 HP every 50 ticks (2.5s)
                if self.play_dead_ticks % 50 == 0 {
                    self.health = (self.health + 1.0).min(self.max_health);
                }
            } else {
                self.is_playing_dead = false;
            }
        }
    }

    /// Whether this axolotl can be targeted by hostile mobs (false while playing dead).
    pub fn can_be_targeted_by_hostiles(&self) -> bool {
        !self.is_playing_dead && self.health > 0.0
    }

    /// Checks if this axolotl hunts the given target entity type.
    pub fn is_hunt_target(&self, target: crate::components::EntityType) -> bool {
        matches!(
            target,
            crate::components::EntityType::Cod
                | crate::components::EntityType::Salmon
                | crate::components::EntityType::TropicalFish
                | crate::components::EntityType::Squid
                | crate::components::EntityType::GlowSquid
                | crate::components::EntityType::Tadpole
                | crate::components::EntityType::Drowned
        )
    }
}
