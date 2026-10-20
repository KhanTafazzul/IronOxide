// REA Verified: net.minecraft.world.entity.ai.goal.Goal (mapped: bth.class)
// REA Verified: net.minecraft.world.entity.ai.goal.Goal$Flag (mapped: bth$a.class)
// REA Verified: net.minecraft.world.entity.ai.goal.WrappedGoal (mapped: buv.class)
//
// Core Goal abstraction and Flag Mutual Exclusion specification.

use glam::Vec3;
use mc_items::ItemId;
use crate::components::{Aabb, AirSupply, EntityId, EntityType, Equipment, Health, Transform, Velocity};

/// Goal control flags representing subsystems of the mob (Goal$Flag).
/// Multiple goals that require overlapping flags cannot execute concurrently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GoalFlags(pub u8);

impl GoalFlags {
    pub const NONE: Self = Self(0);
    pub const MOVE: Self = Self(1 << 0);
    pub const LOOK: Self = Self(1 << 1);
    pub const JUMP: Self = Self(1 << 2);
    pub const TARGET: Self = Self(1 << 3);

    pub const fn empty() -> Self {
        Self::NONE
    }

    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    pub const fn bits(&self) -> u8 {
        self.0
    }

    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn intersects(&self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOr for GoalFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl std::ops::BitOrAssign for GoalFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.insert(rhs);
    }
}

/// Information about a target entity in combat.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TargetEntity {
    pub id: EntityId,
    pub entity_type: EntityType,
    pub position: Vec3,
    pub health: f32,
    pub is_alive: bool,
    pub has_line_of_sight: bool,
}

/// Information about a nearby player for temptation, targeting, and looking.
#[derive(Debug, Clone, PartialEq)]
pub struct NearbyPlayer {
    pub id: EntityId,
    pub position: Vec3,
    pub eye_position: Vec3,
    pub held_item: Option<ItemId>,
    pub is_creative_or_spectator: bool,
    pub has_line_of_sight: bool,
}

/// Nearby friendly/animal entity for breeding and following.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NearbyEntity {
    pub id: EntityId,
    pub entity_type: EntityType,
    pub position: Vec3,
    pub is_alive: bool,
    pub in_love_ticks: u32,
}

/// Attack event emitted when a mob strikes a target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttackEvent {
    pub attacker_id: EntityId,
    pub target_id: EntityId,
    pub damage: f32,
    pub is_ranged: bool,
}

/// Full execution context provided to goals during `can_use`, `tick`, and lifecycle methods.
#[derive(Debug, Clone)]
pub struct GoalContext {
    pub entity_id: EntityId,
    pub entity_type: EntityType,
    pub transform: Transform,
    pub velocity: Velocity,
    pub health: Health,
    pub air_supply: AirSupply,
    pub aabb: Aabb,
    pub equipment: Equipment,
    pub in_love_ticks: u32,
    pub last_health: f32,
    pub is_on_fire: bool,
    pub is_in_water: bool,
    pub is_in_lava: bool,
    pub target: Option<TargetEntity>,
    pub attack_cooldown: u32,
    pub nearby_players: Vec<NearbyPlayer>,
    pub nearby_entities: Vec<NearbyEntity>,

    // Action outputs written by goals during tick
    pub requested_velocity: Option<Vec3>,
    pub look_target: Option<Vec3>,
    pub jump_requested: bool,
    pub attack_event: Option<AttackEvent>,
    pub spawn_child_event: bool,
    pub panic_active: bool,
}

impl GoalContext {
    pub fn new(entity_id: EntityId, entity_type: EntityType, position: Vec3) -> Self {
        let (width, height) = entity_type.default_dimensions();
        let eye_height = entity_type.default_eye_height();
        let max_hp = entity_type.default_max_health();

        Self {
            entity_id,
            entity_type,
            transform: Transform::new(position, 0.0, 0.0, eye_height),
            velocity: Velocity::ZERO,
            health: Health::new(max_hp),
            air_supply: AirSupply::standard(),
            aabb: Aabb::from_feet_and_dimensions(position, width, height),
            equipment: Equipment::new(),
            in_love_ticks: 0,
            last_health: max_hp,
            is_on_fire: false,
            is_in_water: false,
            is_in_lava: false,
            target: None,
            attack_cooldown: 0,
            nearby_players: Vec::new(),
            nearby_entities: Vec::new(),
            requested_velocity: None,
            look_target: None,
            jump_requested: false,
            attack_event: None,
            spawn_child_event: false,
            panic_active: false,
        }
    }

    /// Clears transient action requests between ticks.
    pub fn clear_actions(&mut self) {
        self.requested_velocity = None;
        self.look_target = None;
        self.jump_requested = false;
        self.attack_event = None;
        self.spawn_child_event = false;
        self.panic_active = false;
    }
}

/// Trait defining a behavioral AI Goal in Minecraft.
/// Models `net.minecraft.world.entity.ai.goal.Goal`.
pub trait Goal: Send + Sync {
    /// Returns true if this goal is ready to activate.
    fn can_use(&mut self, ctx: &GoalContext) -> bool;

    /// Returns true if this running goal can continue active execution.
    fn can_continue_to_use(&mut self, ctx: &GoalContext) -> bool {
        self.can_use(ctx)
    }

    /// Returns true if this goal can be interrupted by a higher-priority goal.
    fn is_interruptable(&self) -> bool {
        true
    }

    /// Called when the goal activates and transitions to running.
    fn start(&mut self, _ctx: &mut GoalContext) {}

    /// Called when the goal stops executing (either completed or preempted).
    fn stop(&mut self, _ctx: &mut GoalContext) {}

    /// Periodic execution tick for running goals.
    fn tick(&mut self, _ctx: &mut GoalContext) {}

    /// Control flags required by this goal (e.g. MOVE | LOOK).
    fn flags(&self) -> GoalFlags {
        GoalFlags::empty()
    }

    /// Diagnostic name of the goal.
    fn name(&self) -> &'static str {
        "Goal"
    }
}

/// Wrapped goal pairing a concrete Goal instance with an execution priority integer.
/// Models `net.minecraft.world.entity.ai.goal.WrappedGoal` (`buv.class`).
pub struct WrappedGoal {
    pub priority: i32,
    pub goal: Box<dyn Goal>,
    pub is_running: bool,
}

impl WrappedGoal {
    pub fn new(priority: i32, goal: Box<dyn Goal>) -> Self {
        Self {
            priority,
            goal,
            is_running: false,
        }
    }

    /// Tests if this running goal can be preempted/replaced by a candidate goal.
    /// In vanilla Minecraft: `other.priority < this.priority && this.is_interruptable()`.
    /// Lower priority integer = higher execution precedence!
    pub fn can_be_replaced_by(&self, candidate_priority: i32) -> bool {
        self.goal.is_interruptable() && candidate_priority < self.priority
    }

    pub fn flags(&self) -> GoalFlags {
        self.goal.flags()
    }

    pub fn name(&self) -> &'static str {
        self.goal.name()
    }
}
