// REA Verified: net.minecraft.world.entity.animal.AbstractSchoolingFish (mapped: byb.class)
// REA Verified: net.minecraft.world.entity.animal.Cod (mapped: bye.class)
// REA Verified: net.minecraft.world.entity.animal.Salmon (mapped: byt.class)
// REA Verified: net.minecraft.world.entity.animal.Pufferfish (mapped: byr.class)
// REA Verified: net.minecraft.world.entity.animal.TropicalFish (mapped: byy.class)
// REA Verified: net.minecraft.world.entity.animal.Squid (mapped: byx.class)
// REA Verified: net.minecraft.world.entity.GlowSquid (mapped: bmd.class)
//
// Fish & Cephalopod mobs: Schooling alignment, pufferfish multi-stage inflation,
// poison defense, tropical fish variants, and squid periodic jet thrust with ink clouds.

use super::water_navigation::{BreathMeter, WaterPathNavigation};
use crate::components::EntityId;
use crate::passive::DyeColor;
use glam::Vec3;
use mc_items::ItemId;

/// Maximum number of follower fish in a single school (vanilla: 8 followers + 1 leader = 9).
pub const MAX_SCHOOL_SIZE: usize = 9;

/// Schooling behavior for Cod and Salmon matching `AbstractSchoolingFish` (`byb.class`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FishSchool {
    /// Entity ID of the school leader fish if this fish is following.
    pub leader: Option<EntityId>,
    /// Number of fish currently following this fish if it is a leader.
    pub followers: Vec<EntityId>,
}

impl Default for FishSchool {
    fn default() -> Self {
        Self {
            leader: None,
            followers: Vec::new(),
        }
    }
}

impl FishSchool {
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether this fish is currently following another leader.
    pub fn has_leader(&self) -> bool {
        self.leader.is_some()
    }

    /// Whether this fish is currently leading a school.
    pub fn is_leader(&self) -> bool {
        !self.followers.is_empty()
    }

    /// Whether this fish can accept more followers into its school.
    pub fn can_have_more_followers(&self) -> bool {
        self.leader.is_none() && self.followers.len() + 1 < MAX_SCHOOL_SIZE
    }

    /// Joins a school led by `leader_id`.
    pub fn start_following(&mut self, leader_id: EntityId) {
        self.leave_school();
        self.leader = Some(leader_id);
    }

    /// Leaves any current school or dissolves followers.
    pub fn leave_school(&mut self) {
        self.leader = None;
    }

    /// Adds a follower to this leader's school.
    pub fn add_follower(&mut self, follower_id: EntityId) -> bool {
        if self.can_have_more_followers() && !self.followers.contains(&follower_id) {
            self.followers.push(follower_id);
            true
        } else {
            false
        }
    }

    /// Removes a follower from this leader's school.
    pub fn remove_follower(&mut self, follower_id: EntityId) {
        self.followers.retain(|id| *id != follower_id);
    }

    /// Computes Boids flocking alignment steering vector towards the leader or school centroid.
    pub fn compute_school_velocity(
        &self,
        current_pos: Vec3,
        leader_pos: Vec3,
        leader_vel: Vec3,
    ) -> Vec3 {
        let to_leader = leader_pos - current_pos;
        let dist = to_leader.length();

        if dist < 1.0 {
            // Separation: too close, ease off
            -to_leader.normalize_or_zero() * 0.05 + leader_vel * 0.8
        } else if dist > 8.0 {
            // Far away: swim quickly to rejoin school
            to_leader.normalize_or_zero() * 0.25
        } else {
            // Normal cohesion and velocity alignment
            to_leader.normalize_or_zero() * 0.15 + leader_vel * 0.85
        }
    }
}

/// Cod mob (`bye.class`) with schooling behavior and water bucket handling.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cod {
    pub school: FishSchool,
    pub navigation: WaterPathNavigation,
    pub breath: BreathMeter,
}

impl Default for Cod {
    fn default() -> Self {
        Self {
            school: FishSchool::new(),
            navigation: WaterPathNavigation::new(1.0, false),
            breath: BreathMeter::fish(),
        }
    }
}

impl Cod {
    pub fn new() -> Self {
        Self::default()
    }

    /// Picks up the cod using a water bucket.
    pub fn interact_bucket(&self, held_item: ItemId) -> Option<ItemId> {
        if matches!(held_item, ItemId::WaterBucket) {
            Some(ItemId::CodBucket)
        } else {
            None
        }
    }

    /// Drops when killed: 1 raw cod, rare bone meal.
    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::Cod]
    }
}

/// Salmon mob (`byt.class`) with schooling behavior, upstream leaping ability, and bucket handling.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Salmon {
    pub school: FishSchool,
    pub navigation: WaterPathNavigation,
    pub breath: BreathMeter,
    /// Salmon can breach and leap waterfalls upstream.
    pub can_leap_waterfalls: bool,
}

impl Default for Salmon {
    fn default() -> Self {
        Self {
            school: FishSchool::new(),
            navigation: WaterPathNavigation::new(1.2, true),
            breath: BreathMeter::fish(),
            can_leap_waterfalls: true,
        }
    }
}

impl Salmon {
    pub fn new() -> Self {
        Self::default()
    }

    /// Picks up the salmon using a water bucket.
    pub fn interact_bucket(&self, held_item: ItemId) -> Option<ItemId> {
        if matches!(held_item, ItemId::WaterBucket) {
            Some(ItemId::SalmonBucket)
        } else {
            None
        }
    }

    /// Drops when killed: 1 raw salmon, rare bone meal.
    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::Salmon]
    }
}

/// Multi-stage inflation levels for Pufferfish matching `net.minecraft.world.entity.animal.Pufferfish` (`byr.class`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum PuffState {
    /// Normal, relaxed state (hitbox: 0.35m).
    #[default]
    Small = 0,
    /// Semi-inflated warning state when threat is within 2.5m (hitbox: 0.5m).
    Mid = 1,
    /// Fully inflated defensive state when threat is within 1.5m (hitbox: 0.7m).
    Full = 2,
}

impl PuffState {
    pub fn hitbox_size(&self) -> (f32, f32) {
        match self {
            Self::Small => (0.35, 0.35),
            Self::Mid => (0.5, 0.5),
            Self::Full => (0.7, 0.7),
        }
    }
}

/// Pufferfish mob (`byr.class`) with 3 puff expansion stages poisoning nearby players.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pufferfish {
    pub puff_state: PuffState,
    pub navigation: WaterPathNavigation,
    pub breath: BreathMeter,
    /// Cooldown ticks before deflating after threat departs (default: 60 ticks = 3s).
    pub deflate_cooldown: u32,
}

impl Default for Pufferfish {
    fn default() -> Self {
        Self {
            puff_state: PuffState::Small,
            navigation: WaterPathNavigation::new(1.0, false),
            breath: BreathMeter::fish(),
            deflate_cooldown: 0,
        }
    }
}

/// Stinging poison effect delivered to touching entities.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PufferfishSting {
    /// Instant contact damage.
    pub damage: f32,
    /// Duration of Poison effect in ticks (20 ticks = 1s).
    pub poison_duration_ticks: u32,
    /// Poison amplifier level (0 = Poison I, 1 = Poison II, 3 = Poison IV).
    pub poison_amplifier: u8,
}

impl Pufferfish {
    pub const DEFLATE_TIME_TICKS: u32 = 60;

    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates proximity of potential threats and adjusts puff state:
    /// - Distance < 1.5 blocks -> Full puff
    /// - Distance < 2.5 blocks -> Mid puff
    /// - Distance >= 2.5 blocks -> Count down to Small
    pub fn update_threat_distance(&mut self, min_threat_distance: Option<f32>) {
        if let Some(dist) = min_threat_distance {
            if dist < 1.5 {
                self.puff_state = PuffState::Full;
                self.deflate_cooldown = Self::DEFLATE_TIME_TICKS;
                return;
            } else if dist < 2.5 {
                if self.puff_state < PuffState::Mid {
                    self.puff_state = PuffState::Mid;
                }
                self.deflate_cooldown = Self::DEFLATE_TIME_TICKS;
                return;
            }
        }

        // Threat left or is beyond 2.5m
        if self.deflate_cooldown > 0 {
            self.deflate_cooldown -= 1;
        } else if self.puff_state != PuffState::Small {
            self.puff_state = match self.puff_state {
                PuffState::Full => PuffState::Mid,
                PuffState::Mid => PuffState::Small,
                PuffState::Small => PuffState::Small,
            };
            self.deflate_cooldown = Self::DEFLATE_TIME_TICKS;
        }
    }

    /// Delivers stinging defensive strike when touched by another entity while inflated.
    pub fn touch_target(&self) -> Option<PufferfishSting> {
        match self.puff_state {
            PuffState::Small => None,
            PuffState::Mid => Some(PufferfishSting {
                damage: 1.0,
                poison_duration_ticks: 80, // 4 seconds of Poison I
                poison_amplifier: 0,
            }),
            PuffState::Full => Some(PufferfishSting {
                damage: 2.0,
                poison_duration_ticks: 140, // 7 seconds of Poison II
                poison_amplifier: 1,
            }),
        }
    }

    /// Picks up the pufferfish using a water bucket.
    pub fn interact_bucket(&self, held_item: ItemId) -> Option<ItemId> {
        if matches!(held_item, ItemId::WaterBucket) {
            Some(ItemId::PufferfishBucket)
        } else {
            None
        }
    }

    /// Drops when killed.
    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::Pufferfish]
    }
}

/// Tropical Fish body shape (vanilla: small vs large).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TropicalFishShape {
    #[default]
    Small,
    Large,
}

/// Tropical fish pattern variations matching vanilla `TropicalFish.Pattern`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TropicalFishPattern {
    #[default]
    Kob,
    Sunstreak,
    Snooper,
    Dasher,
    Brinely,
    Spotty,
    Flopper,
    Stripey,
    Glitter,
    Blockfish,
    Betty,
    Clayfish,
}

/// Tropical Fish mob (`byy.class`) with 3,584+ procedural color and pattern combinations.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TropicalFish {
    pub shape: TropicalFishShape,
    pub pattern: TropicalFishPattern,
    pub base_color: DyeColor,
    pub pattern_color: DyeColor,
    pub navigation: WaterPathNavigation,
    pub breath: BreathMeter,
}

impl Default for TropicalFish {
    fn default() -> Self {
        Self {
            shape: TropicalFishShape::Small,
            pattern: TropicalFishPattern::Flopper,
            base_color: DyeColor::Orange,
            pattern_color: DyeColor::White,
            navigation: WaterPathNavigation::new(1.0, false),
            breath: BreathMeter::fish(),
        }
    }
}

impl TropicalFish {
    /// Creates the iconic Clownfish preset (Orange-White Flopper).
    pub fn clownfish() -> Self {
        Self {
            shape: TropicalFishShape::Small,
            pattern: TropicalFishPattern::Flopper,
            base_color: DyeColor::Orange,
            pattern_color: DyeColor::White,
            ..Default::default()
        }
    }

    /// Creates the Blue Tang preset (Light Blue-Gray Flopper).
    pub fn blue_tang() -> Self {
        Self {
            shape: TropicalFishShape::Small,
            pattern: TropicalFishPattern::Flopper,
            base_color: DyeColor::LightBlue,
            pattern_color: DyeColor::Gray,
            ..Default::default()
        }
    }

    /// Picks up the tropical fish using a water bucket.
    pub fn interact_bucket(&self, held_item: ItemId) -> Option<ItemId> {
        if matches!(held_item, ItemId::WaterBucket) {
            Some(ItemId::TropicalFishBucket)
        } else {
            None
        }
    }

    /// Drops when killed.
    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::TropicalFish]
    }
}

/// Squid & Glow Squid cephalopod periodic jet propulsion and defensive ink discharge.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CephalopodMovement {
    /// Periodic thrust cycle counter (vanilla runs a 40-tick tentacle thrust loop).
    pub thrust_cycle_ticks: u32,
    /// Current tentacle spread angle in radians.
    pub tentacle_angle: f32,
    /// Thrust propulsion impulse speed.
    pub thrust_speed: f32,
}

impl Default for CephalopodMovement {
    fn default() -> Self {
        Self {
            thrust_cycle_ticks: 0,
            tentacle_angle: 0.0,
            thrust_speed: 0.18,
        }
    }
}

impl CephalopodMovement {
    pub const THRUST_INTERVAL: u32 = 40; // 2.0 seconds between tentacle jets

    /// Updates squid tentacle opening/closing cycle and returns propulsion impulse velocity
    /// vector when a jet thrust occurs.
    pub fn tick(&mut self, is_in_water: bool, heading: Vec3) -> Option<Vec3> {
        if !is_in_water {
            // Squids on land cannot thrust
            return None;
        }

        self.thrust_cycle_ticks += 1;
        if self.thrust_cycle_ticks >= Self::THRUST_INTERVAL {
            self.thrust_cycle_ticks = 0;
            // Impulse propulsion in the facing direction
            let impulse = heading.normalize_or_zero() * self.thrust_speed;
            Some(impulse)
        } else {
            None
        }
    }
}

/// Regular Squid mob (`byx.class`) with ink cloud emission and jet movement.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Squid {
    pub movement: CephalopodMovement,
    pub navigation: WaterPathNavigation,
    pub breath: BreathMeter,
}

impl Default for Squid {
    fn default() -> Self {
        Self {
            movement: CephalopodMovement::default(),
            navigation: WaterPathNavigation::new(0.8, false),
            breath: BreathMeter::fish(),
        }
    }
}

impl Squid {
    pub fn new() -> Self {
        Self::default()
    }

    /// When damaged, squids discharge a blinding ink cloud and initiate an emergency escape burst.
    pub fn on_damaged(&mut self) -> (ItemId, Vec3) {
        // Emergency escape thrust impulse
        let escape_impulse = Vec3::new(0.0, 0.2, 0.0);
        (ItemId::InkSac, escape_impulse)
    }

    /// Drops when killed: 1 to 3 Ink Sacs.
    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::InkSac]
    }
}

/// Glow Squid mob (`bmd.class`) with bioluminescent glow, dark ticks, and glow ink emission.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GlowSquid {
    pub movement: CephalopodMovement,
    pub navigation: WaterPathNavigation,
    pub breath: BreathMeter,
    /// When damaged, glow squids stop glowing for 100 ticks (5 seconds).
    pub dark_ticks_remaining: u32,
}

impl Default for GlowSquid {
    fn default() -> Self {
        Self {
            movement: CephalopodMovement::default(),
            navigation: WaterPathNavigation::new(0.8, false),
            breath: BreathMeter::fish(),
            dark_ticks_remaining: 0,
        }
    }
}

impl GlowSquid {
    pub const DARK_DURATION_TICKS: u32 = 100; // 5 seconds of darkness on hit

    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the glow squid is currently glowing.
    pub fn is_glowing(&self) -> bool {
        self.dark_ticks_remaining == 0
    }

    pub fn tick(&mut self) {
        if self.dark_ticks_remaining > 0 {
            self.dark_ticks_remaining -= 1;
        }
    }

    /// When damaged, glow squids discharge a glowing ink cloud, go dark, and escape.
    pub fn on_damaged(&mut self) -> (ItemId, Vec3) {
        self.dark_ticks_remaining = Self::DARK_DURATION_TICKS;
        let escape_impulse = Vec3::new(0.0, 0.2, 0.0);
        (ItemId::GlowInkSac, escape_impulse)
    }

    /// Drops when killed: 1 to 3 Glow Ink Sacs.
    pub fn get_death_drops(&self) -> Vec<ItemId> {
        vec![ItemId::GlowInkSac]
    }
}
