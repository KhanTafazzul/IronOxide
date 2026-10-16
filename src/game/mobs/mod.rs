// REA Verified: net.minecraft.world.entity.monster.Zombie (mapped: cdi.class)
// REA Verified: net.minecraft.world.entity.ai.goal.MeleeAttackGoal (mapped: btr.class)
//
// Entity Component System (ECS) & Hostile Mob AI (Zombie):
// - Components: Transform, Velocity, Aabb, Health, MobType.
// - Authentic State Machine:
//   * IDLE: Wanders randomly every 60-100 ticks.
//   * TARGETING: Detects player within 16-block line-of-sight radius.
//   * ATTACK: Moves toward player; deals 3 damage on collision with 20-tick cooldown.

pub mod pathfinding;

use glam::{IVec3, Vec3};
use crate::game::physics::Aabb;
use crate::game::stats::PlayerStats;
use crate::render::mesher::BlockReader;
use pathfinding::{find_path, has_line_of_sight};

/// ECS Component: 3D spatial transform (position and rotation in radians)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3, // yaw, pitch, roll
}

impl Transform {
    pub fn new(position: Vec3, rotation: Vec3) -> Self {
        Self { position, rotation }
    }
}

/// ECS Component: 3D velocity vector
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Velocity {
    pub value: Vec3,
}

impl Velocity {
    pub fn new(value: Vec3) -> Self {
        Self { value }
    }

    pub fn zero() -> Self {
        Self { value: Vec3::ZERO }
    }
}

/// ECS Component: Living entity health
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }
}

/// ECS Component: Mob Classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobType {
    Zombie,
}

/// Mob AI State Machine states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobAiState {
    Idle,
    Targeting,
    Attack,
}

/// ECS Component: Artificial Intelligence & Navigation
#[derive(Debug, Clone)]
pub struct MobAi {
    pub state: MobAiState,
    pub idle_timer: u32,
    pub attack_cooldown: u32,
    pub path: Vec<IVec3>,
    pub path_index: usize,
    pub recalculate_timer: u32,
    pub walk_distance: f32,
}

impl MobAi {
    pub const ATTACK_COOLDOWN_TICKS: u32 = 20; // 20 ticks = 1 second
    pub const ZOMBIE_ATTACK_DAMAGE: f32 = 3.0; // 3 health damage (1.5 hearts)
    pub const TARGET_DETECTION_RANGE: f32 = 16.0; // 16-block LOS radius
    pub const MELEE_ATTACK_RANGE: f32 = 1.6; // 1.6 blocks melee reach

    pub fn new() -> Self {
        Self {
            state: MobAiState::Idle,
            idle_timer: 80,
            attack_cooldown: 0,
            path: Vec::new(),
            path_index: 0,
            recalculate_timer: 0,
            walk_distance: 0.0,
        }
    }
}

/// ECS Entity uniting all components
#[derive(Debug, Clone)]
pub struct MobEntity {
    pub id: u32,
    pub transform: Transform,
    pub velocity: Velocity,
    pub aabb: Aabb,
    pub health: Health,
    pub mob_type: MobType,
    pub ai: MobAi,
    pub on_ground: bool,
}

impl MobEntity {
    /// Creates a Zombie entity matching vanilla dimensions and stats
    /// Width: 0.6m, Height: 1.95m, Health: 20.0
    pub fn new_zombie(id: u32, pos: Vec3) -> Self {
        let half_w = 0.3; // 0.6m total width
        let height = 1.95;
        let aabb = Aabb::new(
            Vec3::new(pos.x - half_w, pos.y, pos.z - half_w),
            Vec3::new(pos.x + half_w, pos.y + height, pos.z + half_w),
        );

        Self {
            id,
            transform: Transform::new(pos, Vec3::ZERO),
            velocity: Velocity::zero(),
            aabb,
            health: Health::new(20.0),
            mob_type: MobType::Zombie,
            ai: MobAi::new(),
            on_ground: false,
        }
    }

    /// Update entity bounding box to match position
    pub fn update_aabb(&mut self) {
        let half_w = 0.3;
        let height = 1.95;
        let pos = self.transform.position;
        self.aabb = Aabb::new(
            Vec3::new(pos.x - half_w, pos.y, pos.z - half_w),
            Vec3::new(pos.x + half_w, pos.y + height, pos.z + half_w),
        );
    }
}

/// Game events emitted by mob AI
#[derive(Debug, Clone, PartialEq)]
pub enum MobEvent {
    ZombieAttack { mob_id: u32, pos: Vec3, damage: f32 },
}

/// Entity Component System Mob Manager
#[derive(Debug, Default)]
pub struct MobEcs {
    pub mobs: Vec<MobEntity>,
    next_id: u32,
    rng_state: u64,
}

impl MobEcs {
    pub fn new() -> Self {
        Self {
            mobs: Vec::new(),
            next_id: 1,
            rng_state: 135792468,
        }
    }

    fn next_rand(&mut self) -> f32 {
        self.rng_state = self.rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((self.rng_state >> 33) as f32) / (u32::MAX as f32)
    }

    /// Spawns a Zombie at the given world position
    pub fn spawn_zombie(&mut self, pos: Vec3) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.mobs.push(MobEntity::new_zombie(id, pos));
        id
    }

    /// Main ECS simulation tick (runs at 20Hz / 0.05s)
    pub fn tick(
        &mut self,
        player_pos: Vec3,
        player_stats: &mut PlayerStats,
        world: &dyn BlockReader,
    ) -> Vec<MobEvent> {
        let mut events = Vec::new();

        for mob in &mut self.mobs {
            if !mob.health.is_alive() {
                continue;
            }

            let mob_pos = mob.transform.position;
            let dist_to_player = (player_pos - mob_pos).length();

            // Line-of-sight check from mob eye to player eye
            let mob_eye = mob_pos + Vec3::new(0.0, 1.74, 0.0);
            let player_eye = player_pos + Vec3::new(0.0, 1.62, 0.0);
            let can_see_player = dist_to_player <= MobAi::TARGET_DETECTION_RANGE
                && has_line_of_sight(mob_eye, player_eye, world);

            // 1. State Transitions
            if dist_to_player <= MobAi::MELEE_ATTACK_RANGE {
                mob.ai.state = MobAiState::Attack;
            } else if can_see_player {
                mob.ai.state = MobAiState::Targeting;
            } else {
                mob.ai.state = MobAiState::Idle;
            }

            // Decrement attack cooldown
            if mob.ai.attack_cooldown > 0 {
                mob.ai.attack_cooldown -= 1;
            }

            // 2. State Actions
            match mob.ai.state {
                MobAiState::Idle => {
                    if mob.ai.idle_timer > 0 {
                        mob.ai.idle_timer -= 1;
                    } else {
                        // Wander randomly every 60-100 ticks
                        self.rng_state = self.rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
                        let rand_val = (self.rng_state >> 33) as u32;
                        mob.ai.idle_timer = 60 + (rand_val % 41); // 60 to 100 ticks

                        // Pick random wander target within 4..7 blocks
                        let dx = ((rand_val % 15) as i32) - 7;
                        let dz = (((rand_val >> 8) % 15) as i32) - 7;
                        let curr_block = mob_pos.floor().as_ivec3();
                        let target_block = IVec3::new(curr_block.x + dx, curr_block.y, curr_block.z + dz);

                        if let Some(path) = find_path(curr_block, target_block, 64, world) {
                            mob.ai.path = path;
                            mob.ai.path_index = 0;
                        }
                    }
                }
                MobAiState::Targeting => {
                    // Recalculate path toward player every 15 ticks
                    if mob.ai.recalculate_timer == 0 || mob.ai.path.is_empty() {
                        let curr_block = mob_pos.floor().as_ivec3();
                        let target_block = player_pos.floor().as_ivec3();
                        if let Some(path) = find_path(curr_block, target_block, 200, world) {
                            mob.ai.path = path;
                            mob.ai.path_index = 0;
                        }
                        mob.ai.recalculate_timer = 15;
                    } else {
                        mob.ai.recalculate_timer -= 1;
                    }
                }
                MobAiState::Attack => {
                    // Turn to face player directly
                    let diff = player_pos - mob_pos;
                    let yaw = diff.z.atan2(diff.x) - std::f32::consts::FRAC_PI_2;
                    mob.transform.rotation.x = yaw;

                    // Melee attack: apply 3 damage on collision with 20-tick cooldown
                    if mob.ai.attack_cooldown == 0 {
                        player_stats.damage(MobAi::ZOMBIE_ATTACK_DAMAGE);
                        mob.ai.attack_cooldown = MobAi::ATTACK_COOLDOWN_TICKS;
                        events.push(MobEvent::ZombieAttack {
                            mob_id: mob.id,
                            pos: mob_pos,
                            damage: MobAi::ZOMBIE_ATTACK_DAMAGE,
                        });
                    }
                }
            }

            // 3. Follow path movement toward active waypoint
            let move_speed = match mob.ai.state {
                MobAiState::Idle => 0.08,
                MobAiState::Targeting | MobAiState::Attack => 0.18, // 0.23 max speed
            };

            if mob.ai.path_index < mob.ai.path.len() {
                let wp = mob.ai.path[mob.ai.path_index];
                let wp_center = wp.as_vec3() + Vec3::new(0.5, 0.0, 0.5);
                let to_wp = wp_center - mob.transform.position;
                let horizontal_dist = (to_wp.x * to_wp.x + to_wp.z * to_wp.z).sqrt();

                if horizontal_dist < 0.25 {
                    mob.ai.path_index += 1;
                } else {
                    let dir_x = to_wp.x / horizontal_dist;
                    let dir_z = to_wp.z / horizontal_dist;

                    mob.velocity.value.x = dir_x * move_speed;
                    mob.velocity.value.z = dir_z * move_speed;

                    // Rotate toward waypoint
                    let target_yaw = dir_z.atan2(dir_x) - std::f32::consts::FRAC_PI_2;
                    mob.transform.rotation.x = target_yaw;

                    // Jump 1-block step if waypoint is higher
                    if wp.y as f32 > mob.transform.position.y + 0.1 && mob.on_ground {
                        mob.velocity.value.y = 0.38; // Jump impulse
                    }

                    mob.ai.walk_distance += move_speed;
                }
            } else {
                mob.velocity.value.x *= 0.5;
                mob.velocity.value.z *= 0.5;
            }

            // 4. Gravity & Voxel Collision Resolution
            mob.velocity.value.y -= 0.04; // Gravity

            let cur_box = mob.aabb;
            let query_box = cur_box.expand(mob.velocity.value);
            let min_x = (query_box.min.x - 0.001).floor() as i32;
            let max_x = (query_box.max.x + 0.001).floor() as i32;
            let min_y = (query_box.min.y - 0.001).floor() as i32;
            let max_y = (query_box.max.y + 0.001).floor() as i32;
            let min_z = (query_box.min.z - 0.001).floor() as i32;
            let max_z = (query_box.max.z + 0.001).floor() as i32;

            let mut boxes = Vec::new();
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    for x in min_x..=max_x {
                        if world.get_block_at(x, y, z).is_opaque() {
                            boxes.push(Aabb::from_block(x, y, z));
                        }
                    }
                }
            }

            // Y resolution
            let mut mov_y = mob.velocity.value.y;
            for b in &boxes {
                mov_y = b.calculate_max_offset_y(&cur_box, mov_y);
            }
            mob.transform.position.y += mov_y;
            mob.on_ground = mob.velocity.value.y < 0.0 && mov_y > mob.velocity.value.y;
            if mob.on_ground || (mob.velocity.value.y > 0.0 && mov_y < mob.velocity.value.y) {
                mob.velocity.value.y = 0.0;
            }

            let after_y = cur_box.offset(Vec3::new(0.0, mov_y, 0.0));

            // X resolution
            let mut mov_x = mob.velocity.value.x;
            for b in &boxes {
                mov_x = b.calculate_max_offset_x(&after_y, mov_x);
            }
            mob.transform.position.x += mov_x;
            let after_x = after_y.offset(Vec3::new(mov_x, 0.0, 0.0));

            // Z resolution
            let mut mov_z = mob.velocity.value.z;
            for b in &boxes {
                mov_z = b.calculate_max_offset_z(&after_x, mov_z);
            }
            mob.transform.position.z += mov_z;

            mob.update_aabb();
        }

        events
    }

    pub fn mobs(&self) -> &[MobEntity] {
        &self.mobs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::render::atlas::BlockType;

    struct DummyWorld {
        blocks: HashMap<IVec3, BlockType>,
    }

    impl BlockReader for DummyWorld {
        fn get_block_at(&self, x: i32, y: i32, z: i32) -> BlockType {
            self.blocks
                .get(&IVec3::new(x, y, z))
                .copied()
                .unwrap_or(BlockType::Air)
        }
    }

    #[test]
    fn test_ecs_components_initialization() {
        let zombie = MobEntity::new_zombie(1, Vec3::new(0.0, 1.0, 0.0));

        assert_eq!(zombie.id, 1);
        assert_eq!(zombie.transform.position, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(zombie.velocity.value, Vec3::ZERO);
        assert_eq!(zombie.health.current, 20.0);
        assert_eq!(zombie.health.max, 20.0);
        assert_eq!(zombie.mob_type, MobType::Zombie);
        assert_eq!(zombie.ai.state, MobAiState::Idle);

        // AABB dimensions: width 0.6, height 1.95
        let bb = zombie.aabb;
        assert!(((bb.max.x - bb.min.x) - 0.6).abs() < 1e-4);
        assert!(((bb.max.y - bb.min.y) - 1.95).abs() < 1e-4);
        assert!(((bb.max.z - bb.min.z) - 0.6).abs() < 1e-4);
    }

    #[test]
    fn test_targeting_within_16_blocks_line_of_sight() {
        let mut ecs = MobEcs::new();
        ecs.spawn_zombie(Vec3::new(0.0, 1.0, 0.0));

        let mut world = DummyWorld {
            blocks: HashMap::new(),
        };
        // Solid ground floor at y=0
        for x in -20..=20 {
            for z in -20..=20 {
                world.blocks.insert(IVec3::new(x, 0, z), BlockType::Stone);
            }
        }

        let mut stats = PlayerStats::new();

        // 1. Player is 10 blocks away with clear line of sight
        let player_pos = Vec3::new(10.0, 1.0, 0.0);
        ecs.tick(player_pos, &mut stats, &world);

        assert_eq!(ecs.mobs[0].ai.state, MobAiState::Targeting);

        // 2. Player moves 25 blocks away (beyond 16-block detection radius)
        let player_far = Vec3::new(25.0, 1.0, 0.0);
        ecs.tick(player_far, &mut stats, &world);

        assert_eq!(ecs.mobs[0].ai.state, MobAiState::Idle);
    }

    #[test]
    fn test_attack_state_and_20_tick_cooldown() {
        let mut ecs = MobEcs::new();
        ecs.spawn_zombie(Vec3::new(0.0, 1.0, 0.0));

        let mut world = DummyWorld {
            blocks: HashMap::new(),
        };
        for x in -5..=5 {
            for z in -5..=5 {
                world.blocks.insert(IVec3::new(x, 0, z), BlockType::Stone);
            }
        }

        let mut stats = PlayerStats::new();
        let initial_health = stats.health;

        // Player within melee range (0.8 blocks)
        let player_pos = Vec3::new(0.8, 1.0, 0.0);

        // Tick 1: should attack and deal 3 damage
        let events = ecs.tick(player_pos, &mut stats, &world);
        assert_eq!(ecs.mobs[0].ai.state, MobAiState::Attack);
        assert_eq!(events.len(), 1);
        assert_eq!(stats.health, initial_health - 3.0);
        assert_eq!(ecs.mobs[0].ai.attack_cooldown, MobAi::ATTACK_COOLDOWN_TICKS);

        // Tick 2: immediately next tick, cooldown is 19, should not deal damage again
        let events_tick2 = ecs.tick(player_pos, &mut stats, &world);
        assert_eq!(events_tick2.len(), 0);
        assert_eq!(stats.health, initial_health - 3.0);
        assert_eq!(ecs.mobs[0].ai.attack_cooldown, 19);
    }
}
