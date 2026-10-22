//! REA Verified: net.minecraft.world.level.NaturalSpawner (mapped: cua.class)
//! REA Verified: net.minecraft.world.level.NaturalSpawner$SpawnState (mapped: cua$d.class)
//!
//! Master chunk-based natural mob spawner engine, pack cluster instantiation, and despawn sweeps.

use super::conditions::{can_spawn_at, is_valid_spawn_distance, should_despawn, SpawnEnvironment};
use super::mob_category::{MobCategory, MobCategoryTracker, MAGIC_NUMBER};
use super::spawn_settings::{BiomeSpawnRegistry, SpawnerData};
use glam::Vec3;
use mc_entities::{Aabb, EntityId, EntityType, Health, NavigationState, Transform, Velocity};
use std::collections::HashSet;

/// Fully instantiated spawned mob representation ready for world ECS insertion.
#[derive(Debug, Clone, PartialEq)]
pub struct SpawnedMob {
    pub id: EntityId,
    pub entity_type: EntityType,
    pub category: MobCategory,
    pub position: Vec3,
    pub transform: Transform,
    pub velocity: Velocity,
    pub aabb: Aabb,
    pub health: Health,
    pub navigation: NavigationState,
    pub age_ticks: u32,
    pub is_persistent: bool,
}

impl SpawnedMob {
    /// Creates a new spawned mob entity with authentic vanilla dimensions and health.
    pub fn new(id: EntityId, entity_type: EntityType, category: MobCategory, position: Vec3) -> Self {
        let (width, height) = entity_type.default_dimensions();
        let aabb = Aabb::from_feet_and_dimensions(position, width, height);
        let max_hp = entity_type.default_max_health();

        Self {
            id,
            entity_type,
            category,
            position,
            transform: Transform::from_pos(position),
            velocity: Velocity::ZERO,
            aabb,
            health: Health::new(max_hp),
            navigation: NavigationState::new(),
            age_ticks: 0,
            is_persistent: category.is_persistent(),
        }
    }
}

/// Chunk coordinates pair (ChunkX, ChunkZ).
pub type ChunkPos = (i32, i32);

/// Master natural mob spawner running tick cycles across all loaded chunks.
#[derive(Debug, Clone)]
pub struct NaturalSpawner {
    pub tick_counter: u64,
    pub next_entity_id: u64,
    pub tracker: MobCategoryTracker,
    pub registry: BiomeSpawnRegistry,
    pub chunk_radius: i32,
}

impl Default for NaturalSpawner {
    fn default() -> Self {
        Self::new()
    }
}

impl NaturalSpawner {
    /// Creates a new spawner initialized with default vanilla spawn registries and 8-chunk radius.
    pub fn new() -> Self {
        let registry = BiomeSpawnRegistry::load_from_client_jar()
            .unwrap_or_else(|| BiomeSpawnRegistry::fallback_registry());

        Self {
            tick_counter: 0,
            next_entity_id: 1,
            tracker: MobCategoryTracker::new(),
            registry,
            chunk_radius: 8, // 8 chunks = 128 blocks radius
        }
    }

    /// Creates a spawner with a specific custom spawn registry.
    pub fn with_registry(registry: BiomeSpawnRegistry) -> Self {
        Self {
            tick_counter: 0,
            next_entity_id: 1,
            tracker: MobCategoryTracker::new(),
            registry,
            chunk_radius: 8,
        }
    }

    /// Allocates the next unique entity ID.
    pub fn next_id(&mut self) -> EntityId {
        let id = EntityId(self.next_entity_id);
        self.next_entity_id += 1;
        id
    }

    /// Collects all unique chunk coordinates within the active chunk radius of all players.
    pub fn collect_spawnable_chunks(&self, player_positions: &[Vec3]) -> Vec<ChunkPos> {
        let mut chunks = HashSet::new();

        for player in player_positions {
            let pcx = (player.x / 16.0).floor() as i32;
            let pcz = (player.z / 16.0).floor() as i32;

            for cx in (pcx - self.chunk_radius)..=(pcx + self.chunk_radius) {
                for cz in (pcz - self.chunk_radius)..=(pcz + self.chunk_radius) {
                    // Check if chunk is within 128-block horizontal Euclidean radius
                    let center_x = (cx * 16 + 8) as f32;
                    let center_z = (cz * 16 + 8) as f32;
                    let dx = center_x - player.x;
                    let dz = center_z - player.z;
                    if (dx * dx + dz * dz) <= (128.0 * 128.0) {
                        chunks.insert((cx, cz));
                    }
                }
            }
        }

        chunks.into_iter().collect()
    }

    /// Executes a full tick spawning cycle:
    /// 1. Advances tick counter.
    /// 2. Determines active categories for this tick:
    ///    - Monsters: every tick.
    ///    - Creatures: every 400 ticks (20s).
    ///    - Ambient & Water: every tick.
    /// 3. Computes loaded spawnable chunk count.
    /// 4. Evaluates category mob caps and performs pack spawning in eligible chunks.
    pub fn run_spawn_cycle<E, B, R>(
        &mut self,
        env: &E,
        biome_provider: &B,
        player_positions: &[Vec3],
        mut rng: R,
    ) -> Vec<SpawnedMob>
    where
        E: SpawnEnvironment,
        B: Fn(i32, i32) -> &'static str,
        R: FnMut() -> f32,
    {
        self.tick_counter += 1;

        if player_positions.is_empty() {
            return Vec::new();
        }

        let spawnable_chunks = self.collect_spawnable_chunks(player_positions);
        let total_chunks = spawnable_chunks.len().max(MAGIC_NUMBER);

        let mut spawned = Vec::new();

        // All spawnable categories in vanilla order
        let categories = [
            MobCategory::Monster,
            MobCategory::Creature,
            MobCategory::Ambient,
            MobCategory::WaterCreature,
            MobCategory::WaterAmbient,
            MobCategory::UndergroundWaterCreature,
            MobCategory::Axolotls,
        ];

        for &category in &categories {
            if !category.should_spawn_on_tick(self.tick_counter) {
                continue;
            }

            if !self.tracker.can_spawn(category, total_chunks) {
                continue;
            }

            // Iterate over spawnable chunks
            for &(cx, cz) in &spawnable_chunks {
                if !self.tracker.can_spawn(category, total_chunks) {
                    break;
                }

                // Sample candidate block coordinate within the chunk
                let local_x = (rng() * 16.0).floor() as i32;
                let local_z = (rng() * 16.0).floor() as i32;
                let bx = cx * 16 + local_x;
                let bz = cz * 16 + local_z;
                let by = env.get_top_motion_blocking_y(bx, bz) + 1;

                let candidate_pos = Vec3::new(bx as f32 + 0.5, by as f32, bz as f32 + 0.5);

                // Distance checks: >= 24 blocks from all players and <= 128 from at least one
                if !is_valid_spawn_distance(candidate_pos, player_positions) {
                    continue;
                }

                // Query biome and spawn settings
                let biome_key = biome_provider(bx, bz);
                let settings = self.registry.get_or_default(biome_key);

                // For creatures, roll creature generation probability (default 0.1)
                if category == MobCategory::Creature {
                    if rng() > settings.creature_probability {
                        continue;
                    }
                }

                // Weighted random mob selection from biome settings
                let roll = rng();
                let Some(spawner) = settings.sample_mob(category, roll) else {
                    continue;
                };

                let entity_type = spawner.entity_type;

                // Validate environmental conditions at base position
                if !can_spawn_at(env, category, bx, by, bz, entity_type) {
                    continue;
                }

                // Execute pack spawning cluster
                let pack_mobs = self.spawn_pack(
                    env,
                    category,
                    spawner,
                    bx,
                    by,
                    bz,
                    player_positions,
                    total_chunks,
                    &mut rng,
                );

                spawned.extend(pack_mobs);
            }
        }

        spawned
    }

    /// Executes cluster pack spawning around a center coordinate `(center_x, center_y, center_z)`.
    /// Spawns between `min_count` and `max_count` mobs distributed in a 9x9 horizontal area (+/- 4 blocks).
    fn spawn_pack<E, R>(
        &mut self,
        env: &E,
        category: MobCategory,
        spawner: &SpawnerData,
        center_x: i32,
        center_y: i32,
        center_z: i32,
        player_positions: &[Vec3],
        total_chunks: usize,
        rng: &mut R,
    ) -> Vec<SpawnedMob>
    where
        E: SpawnEnvironment,
        R: FnMut() -> f32,
    {
        let mut pack = Vec::new();

        // Calculate pack size in [min_count, max_count]
        let range = (spawner.max_count - spawner.min_count + 1) as f32;
        let pack_size = spawner.min_count + (rng() * range).floor() as u32;

        let mut spawned_in_pack = 0u32;
        let attempts = (pack_size * 4).max(12);

        for _ in 0..attempts {
            if spawned_in_pack >= pack_size {
                break;
            }

            if !self.tracker.can_spawn(category, total_chunks) {
                break;
            }

            // Random offset within [-4, 4] blocks
            let dx = (rng() * 9.0).floor() as i32 - 4;
            let dz = (rng() * 9.0).floor() as i32 - 4;
            let px = center_x + dx;
            let pz = center_z + dz;
            let py = env.get_top_motion_blocking_y(px, pz) + 1;

            // Only allow vertical difference up to 2 blocks from pack origin
            if (py - center_y).abs() > 2 {
                continue;
            }

            let pos = Vec3::new(px as f32 + 0.5, py as f32, pz as f32 + 0.5);

            if !is_valid_spawn_distance(pos, player_positions) {
                continue;
            }

            if !can_spawn_at(env, category, px, py, pz, spawner.entity_type) {
                continue;
            }

            // Valid pack member spawn
            let id = self.next_id();
            let mob = SpawnedMob::new(id, spawner.entity_type, category, pos);
            self.tracker.register_spawn(spawner.entity_type);
            pack.push(mob);
            spawned_in_pack += 1;
        }

        pack
    }

    /// Evaluates all active mobs in the world against player distances and lifetimes:
    /// - Mobs > 128 blocks away despawn immediately.
    /// - Mobs between 32 and 128 blocks away older than 600 ticks have a 1/800 roll to despawn.
    /// - Persistent (nametagged, tamed) mobs are preserved.
    ///
    /// Updates the active category counts and returns the IDs of all despawned mobs.
    pub fn run_despawn_sweep<R>(
        &mut self,
        active_mobs: &mut Vec<SpawnedMob>,
        player_positions: &[Vec3],
        mut rng: R,
    ) -> Vec<EntityId>
    where
        R: FnMut() -> f32,
    {
        let mut despawned_ids = Vec::new();

        active_mobs.retain_mut(|mob| {
            mob.age_ticks += 1;

            let roll = rng();
            let should_remove = should_despawn(
                mob.entity_type,
                mob.position,
                player_positions,
                mob.age_ticks,
                mob.is_persistent,
                roll,
            );

            if should_remove {
                despawned_ids.push(mob.id);
                self.tracker.register_despawn(mob.entity_type);
                false
            } else {
                true
            }
        });

        despawned_ids
    }
}
