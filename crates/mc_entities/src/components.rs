// REA Verified: net.minecraft.world.entity.Mob (mapped: bmn.class)
// REA Verified: net.minecraft.world.entity.LivingEntity (mapped: bmm.class)
// REA Verified: net.minecraft.world.entity.EntityType (mapped: bmf.class)
// REA Verified: net.minecraft.world.phys.AABB (mapped: ewj.class)
//
// Entity Component System (ECS) Architecture for Minecraft 1.20+ / 1.21 mobs.

use glam::Vec3;
use mc_items::ItemId;
use std::fmt;

/// Unique entity runtime identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EntityId(pub u64);

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({})", self.0)
    }
}

/// Vanilla Mob Classifications covering 65+ Minecraft mobs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EntityType {
    // Player
    Player,

    // Passive & Ambient Animals
    Cow,
    Sheep,
    Pig,
    Chicken,
    Horse,
    Donkey,
    Mule,
    Llama,
    TraderLlama,
    Bat,
    Villager,
    WanderingTrader,
    IronGolem,
    SnowGolem,
    Cat,
    Ocelot,
    Wolf,
    Fox,
    Rabbit,
    Frog,
    Turtle,
    Axolotl,
    Parrot,
    Panda,
    Bee,
    Strider,
    Camel,
    Sniffer,
    Armadillo,
    Mooshroom,
    Goat,
    Allay,
    PolarBear,
    Cod,
    Salmon,
    Pufferfish,
    TropicalFish,
    Dolphin,
    Squid,
    GlowSquid,
    Tadpole,

    // Hostile & Neutral Monsters
    Zombie,
    ZombieVillager,
    Husk,
    Drowned,
    Skeleton,
    Stray,
    WitherSkeleton,
    Creeper,
    Spider,
    CaveSpider,
    Enderman,
    Piglin,
    PiglinBrute,
    ZombifiedPiglin,
    Hoglin,
    Zoglin,
    Blaze,
    Ghast,
    MagmaCube,
    Slime,
    Witch,
    Wither,
    EnderDragon,
    Shulker,
    Vindicator,
    Evoker,
    Pillager,
    Ravager,
    Vex,
    Phantom,
    Guardian,
    ElderGuardian,
    Warden,
    Breeze,
    Bogged,
    Creaking,
    Silverfish,
    Endermite,
}

impl EntityType {
    /// Returns true if the entity is naturally passive/peaceful towards players.
    pub const fn is_passive(&self) -> bool {
        matches!(
            self,
            Self::Cow
                | Self::Sheep
                | Self::Pig
                | Self::Chicken
                | Self::Horse
                | Self::Donkey
                | Self::Mule
                | Self::Llama
                | Self::TraderLlama
                | Self::Bat
                | Self::Villager
                | Self::WanderingTrader
                | Self::SnowGolem
                | Self::Cat
                | Self::Ocelot
                | Self::Fox
                | Self::Rabbit
                | Self::Frog
                | Self::Turtle
                | Self::Axolotl
                | Self::Parrot
                | Self::Allay
                | Self::Cod
                | Self::Salmon
                | Self::Pufferfish
                | Self::TropicalFish
                | Self::Squid
                | Self::GlowSquid
                | Self::Tadpole
                | Self::Strider
                | Self::Camel
                | Self::Sniffer
                | Self::Armadillo
                | Self::Mooshroom
                | Self::Goat
                | Self::PolarBear
        )
    }

    /// Returns true if the entity is hostile towards players on sight.
    pub const fn is_hostile(&self) -> bool {
        matches!(
            self,
            Self::Zombie
                | Self::ZombieVillager
                | Self::Husk
                | Self::Drowned
                | Self::Skeleton
                | Self::Stray
                | Self::WitherSkeleton
                | Self::Creeper
                | Self::Witch
                | Self::Wither
                | Self::EnderDragon
                | Self::Shulker
                | Self::Vindicator
                | Self::Evoker
                | Self::Pillager
                | Self::Ravager
                | Self::Vex
                | Self::Phantom
                | Self::Guardian
                | Self::ElderGuardian
                | Self::Warden
                | Self::Breeze
                | Self::Bogged
                | Self::Creaking
                | Self::Silverfish
                | Self::Endermite
                | Self::PiglinBrute
                | Self::Zoglin
                | Self::Blaze
                | Self::Ghast
                | Self::MagmaCube
                | Self::Slime
        )
    }

    /// Returns true if the entity is neutral (attacks only if provoked or in dark).
    pub const fn is_neutral(&self) -> bool {
        matches!(
            self,
            Self::Enderman
                | Self::ZombifiedPiglin
                | Self::Piglin
                | Self::Wolf
                | Self::IronGolem
                | Self::Bee
                | Self::Goat
                | Self::Panda
                | Self::Dolphin
                | Self::Spider
                | Self::CaveSpider
                | Self::Hoglin
        )
    }

    /// Returns true if the mob takes damage from Instant Health and heals from Instant Damage.
    pub const fn is_undead(&self) -> bool {
        matches!(
            self,
            Self::Zombie
                | Self::ZombieVillager
                | Self::Husk
                | Self::Drowned
                | Self::Skeleton
                | Self::Stray
                | Self::WitherSkeleton
                | Self::ZombifiedPiglin
                | Self::Zoglin
                | Self::Phantom
                | Self::Wither
                | Self::Bogged
        )
    }

    /// Returns true if the mob is an arthropod (Bane of Arthropods weakness).
    pub const fn is_arthropod(&self) -> bool {
        matches!(
            self,
            Self::Spider
                | Self::CaveSpider
                | Self::Silverfish
                | Self::Endermite
                | Self::Bee
        )
    }

    /// Default width and height (bounding box dimensions) in blocks.
    pub const fn default_dimensions(&self) -> (f32, f32) {
        match self {
            Self::Zombie | Self::ZombieVillager | Self::Husk | Self::Drowned => (0.6, 1.95),
            Self::Skeleton | Self::Stray | Self::Bogged => (0.6, 1.99),
            Self::WitherSkeleton => (0.7, 2.4),
            Self::Creeper => (0.6, 1.7),
            Self::Spider => (1.4, 0.9),
            Self::CaveSpider => (0.7, 0.5),
            Self::Enderman => (0.6, 2.9),
            Self::Piglin | Self::PiglinBrute | Self::ZombifiedPiglin => (0.6, 1.95),
            Self::Cow | Self::Mooshroom => (0.9, 1.4),
            Self::Sheep => (0.9, 1.3),
            Self::Pig => (0.9, 0.9),
            Self::Chicken => (0.4, 0.7),
            Self::Horse | Self::Donkey | Self::Mule => (1.3964844, 1.6),
            Self::Wolf => (0.6, 0.85),
            Self::Cat | Self::Ocelot => (0.6, 0.7),
            Self::Fox => (0.6, 0.7),
            Self::Rabbit => (0.4, 0.5),
            Self::IronGolem => (1.4, 2.7),
            Self::Villager | Self::WanderingTrader | Self::Witch => (0.6, 1.95),
            Self::Vindicator | Self::Evoker | Self::Pillager => (0.6, 1.95),
            Self::Ravager => (1.95, 2.2),
            Self::Vex => (0.4, 0.8),
            Self::Bat => (0.5, 0.9),
            Self::Bee => (0.7, 0.6),
            Self::Blaze => (0.6, 1.8),
            Self::Ghast => (4.0, 4.0),
            Self::Slime | Self::MagmaCube => (2.04, 2.04), // Size 1: (0.51, 0.51), Size 2: (1.02, 1.02), Size 4: (2.04, 2.04)
            Self::Warden => (0.9, 2.9),
            Self::Breeze => (0.6, 1.77),
            Self::Creaking => (0.9, 2.7),
            Self::Silverfish | Self::Endermite => (0.4, 0.3),
            Self::Sniffer => (1.9, 1.75),
            Self::Camel => (1.7, 2.375),
            Self::Armadillo => (0.7, 0.65),
            _ => (0.6, 1.8),
        }
    }

    /// Default eye height relative to entity base feet position.
    pub const fn default_eye_height(&self) -> f32 {
        match self {
            Self::Zombie | Self::ZombieVillager | Self::Husk | Self::Drowned => 1.74,
            Self::Skeleton | Self::Stray | Self::Bogged => 1.74,
            Self::WitherSkeleton => 2.1,
            Self::Creeper => 1.5,
            Self::Spider => 0.65,
            Self::CaveSpider => 0.35,
            Self::Enderman => 2.55,
            Self::Cow | Self::Mooshroom => 1.3,
            Self::Sheep => 1.2,
            Self::Pig => 0.8,
            Self::Chicken => 0.65,
            Self::IronGolem => 2.4,
            Self::Villager | Self::WanderingTrader | Self::Witch => 1.62,
            Self::Wolf => 0.75,
            Self::Cat => 0.6,
            Self::Warden => 2.6,
            Self::Creaking => 2.4,
            _ => 1.5,
        }
    }

    /// Default maximum health points (2.0 = 1 heart).
    pub const fn default_max_health(&self) -> f32 {
        match self {
            Self::Zombie | Self::ZombieVillager | Self::Husk | Self::Drowned => 20.0,
            Self::Skeleton | Self::Stray | Self::Bogged => 20.0,
            Self::Creeper => 20.0,
            Self::Spider => 16.0,
            Self::CaveSpider => 12.0,
            Self::Enderman => 40.0,
            Self::Cow | Self::Mooshroom => 10.0,
            Self::Sheep => 8.0,
            Self::Pig => 10.0,
            Self::Chicken => 4.0,
            Self::Horse => 24.0,
            Self::IronGolem => 100.0,
            Self::Villager => 20.0,
            Self::Warden => 500.0,
            Self::Wither => 300.0,
            Self::EnderDragon => 200.0,
            Self::Ravager => 100.0,
            Self::Witch => 26.0,
            Self::Blaze => 20.0,
            Self::Ghast => 10.0,
            Self::Breeze => 30.0,
            Self::Creaking => 1.0, // Creaking is immune to direct damage unless heart destroyed
            _ => 20.0,
        }
    }

    /// Checks if a given item tempts this animal to follow the player and breed.
    pub fn is_tempting_item(&self, item: ItemId) -> bool {
        match self {
            Self::Cow | Self::Sheep | Self::Mooshroom => matches!(item, ItemId::Wheat),
            Self::Pig => matches!(item, ItemId::Carrot | ItemId::Potato | ItemId::Beetroot),
            Self::Chicken => matches!(
                item,
                ItemId::WheatSeeds | ItemId::MelonSeeds | ItemId::PumpkinSeeds | ItemId::BeetrootSeeds
            ),
            Self::Horse | Self::Donkey | Self::Mule => matches!(
                item,
                ItemId::GoldenApple | ItemId::EnchantedGoldenApple | ItemId::GoldenCarrot
            ),
            Self::Wolf => matches!(
                item,
                ItemId::Beef | ItemId::CookedBeef | ItemId::Porkchop | ItemId::CookedPorkchop | ItemId::Chicken | ItemId::CookedChicken | ItemId::Mutton | ItemId::CookedMutton
            ),
            Self::Cat | Self::Ocelot => matches!(item, ItemId::Cod | ItemId::Salmon),
            Self::Rabbit => matches!(item, ItemId::Carrot | ItemId::GoldenCarrot | ItemId::Dandelion),
            Self::Frog => matches!(item, ItemId::SlimeBall),
            Self::Turtle => matches!(item, ItemId::Seagrass),
            Self::Axolotl => matches!(item, ItemId::TropicalFishBucket),
            Self::Strider => matches!(item, ItemId::WarpedFungus),
            Self::Camel => matches!(item, ItemId::Cactus),
            Self::Sniffer => matches!(item, ItemId::TorchflowerSeeds),
            Self::Armadillo => matches!(item, ItemId::SpiderEye),
            _ => false,
        }
    }
}

/// 3D Axis-Aligned Bounding Box (AABB) for physics, raycasting, and hit detection.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub const ZERO: Self = Self {
        min: Vec3::ZERO,
        max: Vec3::ZERO,
    };

    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self {
            min: Vec3::new(min.x.min(max.x), min.y.min(max.y), min.z.min(max.z)),
            max: Vec3::new(min.x.max(max.x), min.y.max(max.y), min.z.max(max.z)),
        }
    }

    /// Builds an AABB centered horizontally around `base_pos` (feet) with width and height.
    pub fn from_feet_and_dimensions(feet_pos: Vec3, width: f32, height: f32) -> Self {
        let half_w = width * 0.5;
        Self {
            min: Vec3::new(feet_pos.x - half_w, feet_pos.y, feet_pos.z - half_w),
            max: Vec3::new(feet_pos.x + half_w, feet_pos.y + height, feet_pos.z + half_w),
        }
    }

    /// Builds an AABB centered around a 3D point.
    pub fn from_center_and_size(center: Vec3, size: Vec3) -> Self {
        let half = size * 0.5;
        Self {
            min: center - half,
            max: center + half,
        }
    }

    #[inline]
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    #[inline]
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
            && self.min.z < other.max.z
            && self.max.z > other.min.z
    }

    #[inline]
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Expands the AABB outwards by `delta` in all directions.
    #[inline]
    pub fn expand(&self, delta: Vec3) -> Self {
        Self {
            min: self.min - delta,
            max: self.max + delta,
        }
    }

    /// Deflates the AABB inwards by `delta`.
    #[inline]
    pub fn deflate(&self, delta: Vec3) -> Self {
        Self {
            min: self.min + delta,
            max: self.max - delta,
        }
    }

    /// Offsets the AABB position by `offset`.
    #[inline]
    pub fn offset(&self, offset: Vec3) -> Self {
        Self {
            min: self.min + offset,
            max: self.max + offset,
        }
    }

    /// Ray intersection test returning hit parameter `t` if ray hits this AABB.
    pub fn ray_intersects(&self, ray_origin: Vec3, ray_dir: Vec3, max_dist: f32) -> Option<f32> {
        let inv_dir = Vec3::new(1.0 / ray_dir.x, 1.0 / ray_dir.y, 1.0 / ray_dir.z);

        let t1 = (self.min.x - ray_origin.x) * inv_dir.x;
        let t2 = (self.max.x - ray_origin.x) * inv_dir.x;
        let t3 = (self.min.y - ray_origin.y) * inv_dir.y;
        let t4 = (self.max.y - ray_origin.y) * inv_dir.y;
        let t5 = (self.min.z - ray_origin.z) * inv_dir.z;
        let t6 = (self.max.z - ray_origin.z) * inv_dir.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax < 0.0 || tmin > tmax || tmin > max_dist {
            None
        } else {
            Some(tmin.max(0.0))
        }
    }
}

/// 3D Spatial Transform (position, rotation, eye height).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3, // yaw, pitch, roll in degrees
    pub eye_height: f32,
}

impl Transform {
    pub fn new(position: Vec3, yaw: f32, pitch: f32, eye_height: f32) -> Self {
        Self {
            position,
            rotation: Vec3::new(yaw, pitch, 0.0),
            eye_height,
        }
    }

    pub fn from_pos(position: Vec3) -> Self {
        Self {
            position,
            rotation: Vec3::ZERO,
            eye_height: 1.62,
        }
    }

    #[inline]
    pub fn eye_position(&self) -> Vec3 {
        Vec3::new(
            self.position.x,
            self.position.y + self.eye_height,
            self.position.z,
        )
    }

    #[inline]
    pub fn yaw(&self) -> f32 {
        self.rotation.x
    }

    #[inline]
    pub fn pitch(&self) -> f32 {
        self.rotation.y
    }

    /// Forward facing normalized unit vector based on yaw and pitch.
    pub fn forward(&self) -> Vec3 {
        let yaw_rad = self.rotation.x.to_radians();
        let pitch_rad = self.rotation.y.to_radians();

        let xz_len = pitch_rad.cos();
        let x = -yaw_rad.sin() * xz_len;
        let y = -pitch_rad.sin();
        let z = yaw_rad.cos() * xz_len;

        Vec3::new(x, y, z).normalize_or_zero()
    }

    /// Computes yaw and pitch angles in degrees to look directly towards target position.
    pub fn look_at_angles(&self, target: Vec3) -> (f32, f32) {
        let eye = self.eye_position();
        let dx = target.x - eye.x;
        let dy = target.y - eye.y;
        let dz = target.z - eye.z;

        let horizontal_dist = (dx * dx + dz * dz).sqrt();
        let yaw = (-dx).atan2(dz).to_degrees();
        let pitch = (-dy).atan2(horizontal_dist).to_degrees();

        (yaw, pitch)
    }
}

/// 3D Velocity vector and kinematic physics helpers.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Velocity {
    pub value: Vec3,
}

impl Velocity {
    pub const ZERO: Self = Self { value: Vec3::ZERO };

    pub fn new(value: Vec3) -> Self {
        Self { value }
    }

    pub fn apply_gravity(&mut self, gravity: f32) {
        self.value.y -= gravity;
    }

    pub fn apply_drag(&mut self, horizontal_drag: f32, vertical_drag: f32) {
        self.value.x *= horizontal_drag;
        self.value.z *= horizontal_drag;
        self.value.y *= vertical_drag;
    }
}

/// Living entity health, damage tracking, and invulnerability cooldowns.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Health {
    pub current: f32,
    pub max: f32,
    pub invulnerable_ticks: u32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self {
            current: max,
            max,
            invulnerable_ticks: 0,
        }
    }

    #[inline]
    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }

    /// Inflicts damage, obeying 10-tick damage immunity window like vanilla Minecraft.
    /// Returns true if damage was actually inflicted.
    pub fn take_damage(&mut self, amount: f32) -> bool {
        if self.invulnerable_ticks > 0 || amount <= 0.0 {
            return false;
        }
        self.current = (self.current - amount).max(0.0);
        self.invulnerable_ticks = 10; // 0.5s damage cooldown
        true
    }

    pub fn heal(&mut self, amount: f32) {
        if self.is_alive() && amount > 0.0 {
            self.current = (self.current + amount).min(self.max);
        }
    }

    pub fn tick(&mut self) {
        if self.invulnerable_ticks > 0 {
            self.invulnerable_ticks -= 1;
        }
    }
}

/// Underwater respiration and drowning mechanics (300 ticks = 15 seconds).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AirSupply {
    pub current: i32,
    pub max: i32,
    pub is_drowning: bool,
}

impl AirSupply {
    pub const DEFAULT_MAX_AIR: i32 = 300;

    pub fn new(max: i32) -> Self {
        Self {
            current: max,
            max,
            is_drowning: false,
        }
    }

    pub fn standard() -> Self {
        Self::new(Self::DEFAULT_MAX_AIR)
    }

    /// Ticks air when submerged in water. Returns true if drowning damage (2.0) should be applied.
    pub fn tick_underwater(&mut self) -> bool {
        if self.current > 0 {
            self.current -= 1;
            self.is_drowning = false;
            false
        } else {
            self.is_drowning = true;
            // Drowning damage occurs every 20 ticks (1 second) when air is <= -20
            self.current -= 1;
            if self.current <= -20 {
                self.current = 0;
                true
            } else {
                false
            }
        }
    }

    /// Replenishes air quickly when head emerges into air.
    pub fn replenish(&mut self) {
        self.current = self.max;
        self.is_drowning = false;
    }
}

/// Navigation and movement state component.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NavigationState {
    pub target_pos: Option<Vec3>,
    pub speed_modifier: f32,
    pub stuck_ticks: u32,
    pub last_pos: Vec3,
    pub is_stuck: bool,
    pub path_index: usize,
}

impl NavigationState {
    pub fn new() -> Self {
        Self {
            target_pos: None,
            speed_modifier: 1.0,
            stuck_ticks: 0,
            last_pos: Vec3::ZERO,
            is_stuck: false,
            path_index: 0,
        }
    }

    pub fn reset(&mut self) {
        self.target_pos = None;
        self.speed_modifier = 1.0;
        self.stuck_ticks = 0;
        self.is_stuck = false;
        self.path_index = 0;
    }
}

impl Default for NavigationState {
    fn default() -> Self {
        Self::new()
    }
}

/// Entity equipment / inventory slots for armor, tools, and held items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Equipment {
    pub main_hand: Option<ItemId>,
    pub off_hand: Option<ItemId>,
    pub head: Option<ItemId>,
    pub chest: Option<ItemId>,
    pub legs: Option<ItemId>,
    pub feet: Option<ItemId>,
}

impl Equipment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_main_hand(mut self, item: ItemId) -> Self {
        self.main_hand = Some(item);
        self
    }

    pub fn is_holding(&self, item: ItemId) -> bool {
        self.main_hand == Some(item) || self.off_hand == Some(item)
    }

    pub fn holds_tempting_item_for(&self, mob: EntityType) -> bool {
        if let Some(main) = self.main_hand {
            if mob.is_tempting_item(main) {
                return true;
            }
        }
        if let Some(off) = self.off_hand {
            if mob.is_tempting_item(off) {
                return true;
            }
        }
        false
    }
}
