//! REA Verified: net.minecraft.world.level.NaturalSpawner (mapped: cua.class)
//! REA Verified: net.minecraft.world.entity.MobCategory (mapped: bmo.class)
//! REA Verified: net.minecraft.world.level.biome.MobSpawnSettings (mapped: cva.class)
//!
//! Minecraft 1.20+ / 1.21 tick-based natural chunk spawner, mob caps, biome spawn tables,
//! environmental validation, pack clustering, and distance-based despawning.

pub mod conditions;
pub mod mob_category;
pub mod natural_spawner;
pub mod spawn_settings;

pub use conditions::{
    can_spawn_at, has_unobstructed_aabb, is_grass_block, is_passable_space, is_solid_floor,
    is_valid_spawn_distance, is_water, should_despawn, verify_ambient_conditions,
    verify_aquatic_conditions, verify_creature_conditions, verify_monster_conditions,
    SpawnEnvironment,
};

pub use mob_category::{
    category_for_entity, MobCategory, MobCategoryTracker, DEFAULT_DESPAWN_DISTANCE,
    DEFAULT_DESPAWN_DISTANCE_SQ, DESPAWN_AGE_THRESHOLD, DESPAWN_CHANCE_1_IN, MAGIC_NUMBER,
    MIN_SPAWN_DISTANCE, MIN_SPAWN_DISTANCE_SQ, NO_DESPAWN_DISTANCE, NO_DESPAWN_DISTANCE_SQ,
};

pub use natural_spawner::{ChunkPos, NaturalSpawner, SpawnedMob};

pub use spawn_settings::{
    parse_entity_type, BiomeSpawnRegistry, BiomeSpawnSettings, SpawnerData,
};
