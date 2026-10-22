//! REA Verified: net.minecraft.world.level.NaturalSpawner (mapped: cua.class)
//! REA Verified: net.minecraft.world.level.NaturalSpawner$isValidEmptySpawnBlock (cua.a(dld, hx, djh, dmc, blz))
//!
//! Spawn condition validation, light-level verification, floor checks, and despawning logic.

use super::mob_category::{
    category_for_entity, MobCategory, DESPAWN_AGE_THRESHOLD, DESPAWN_CHANCE_1_IN,
    MIN_SPAWN_DISTANCE_SQ, NO_DESPAWN_DISTANCE,
};
use crate::biomes::SurfaceBlock;
use glam::Vec3;
use mc_entities::{Aabb, EntityType};

/// Trait abstracting world voxel blocks and dual-nibble lighting for spawn validation.
pub trait SpawnEnvironment {
    /// Returns the block at voxel coordinate `(x, y, z)`.
    fn get_block(&self, x: i32, y: i32, z: i32) -> SurfaceBlock;

    /// Returns the artificial block light level (0..15).
    fn get_block_light(&self, x: i32, y: i32, z: i32) -> u8;

    /// Returns the filtered sky light level (0..15).
    fn get_sky_light(&self, x: i32, y: i32, z: i32) -> u8;

    /// Returns true if the world is currently in daytime (sun illuminated).
    fn is_daytime(&self) -> bool;

    /// Returns the highest non-air/motion-blocking Y coordinate at `(x, z)`.
    fn get_top_motion_blocking_y(&self, x: i32, z: i32) -> i32;
}

/// Checks whether a block type provides a solid top surface for mobs to spawn upon.
pub fn is_solid_floor(block: SurfaceBlock) -> bool {
    match block {
        SurfaceBlock::Stone
        | SurfaceBlock::Deepslate
        | SurfaceBlock::Dirt
        | SurfaceBlock::GrassBlock
        | SurfaceBlock::Podzol
        | SurfaceBlock::CoarseDirt
        | SurfaceBlock::Mud
        | SurfaceBlock::Sand
        | SurfaceBlock::Sandstone
        | SurfaceBlock::RedSand
        | SurfaceBlock::RedSandstone
        | SurfaceBlock::Gravel
        | SurfaceBlock::Cobblestone
        | SurfaceBlock::Terracotta
        | SurfaceBlock::WhiteTerracotta
        | SurfaceBlock::OrangeTerracotta
        | SurfaceBlock::YellowTerracotta
        | SurfaceBlock::BrownTerracotta
        | SurfaceBlock::RedTerracotta
        | SurfaceBlock::LightGrayTerracotta
        | SurfaceBlock::Ice
        | SurfaceBlock::PackedIce
        | SurfaceBlock::SnowBlock
        | SurfaceBlock::Netherrack
        | SurfaceBlock::NetherBricks
        | SurfaceBlock::Blackstone
        | SurfaceBlock::PolishedBlackstone
        | SurfaceBlock::PolishedBlackstoneBricks
        | SurfaceBlock::EndStone
        | SurfaceBlock::Obsidian
        | SurfaceBlock::Prismarine
        | SurfaceBlock::PrismarineBricks
        | SurfaceBlock::DarkPrismarine
        | SurfaceBlock::OakPlanks
        | SurfaceBlock::DarkOakPlanks
        | SurfaceBlock::PaleOakPlanks
        | SurfaceBlock::PaleMossBlock => true,
        _ => false,
    }
}

/// Checks whether a block is passable/non-colliding space that mobs can occupy.
pub fn is_passable_space(block: SurfaceBlock) -> bool {
    match block {
        SurfaceBlock::Air
        | SurfaceBlock::ShortGrass
        | SurfaceBlock::TallGrass
        | SurfaceBlock::Fern
        | SurfaceBlock::Poppy
        | SurfaceBlock::Dandelion
        | SurfaceBlock::Sunflower
        | SurfaceBlock::SnowLayer
        | SurfaceBlock::PaleMossCarpet
        | SurfaceBlock::PaleHangingMoss => true,
        _ => false,
    }
}

/// Checks whether a block is liquid water or underwater vegetation.
pub fn is_water(block: SurfaceBlock) -> bool {
    matches!(
        block,
        SurfaceBlock::Water | SurfaceBlock::Seagrass | SurfaceBlock::Kelp
    )
}

/// Checks whether a block is a grass block.
pub fn is_grass_block(block: SurfaceBlock) -> bool {
    matches!(block, SurfaceBlock::GrassBlock)
}

/// Validates that a candidate 3D spawn position obeys distance bounds to all active players:
/// - Must be >= 24 blocks away from every player.
/// - Must be <= 128 blocks away from at least one player.
pub fn is_valid_spawn_distance(candidate_pos: Vec3, player_positions: &[Vec3]) -> bool {
    if player_positions.is_empty() {
        return false;
    }

    let mut within_max_range = false;

    for player_pos in player_positions {
        let dist_sq = candidate_pos.distance_squared(*player_pos);
        if dist_sq < MIN_SPAWN_DISTANCE_SQ {
            // Too close to this player!
            return false;
        }

        let cat_max_sq = 128.0 * 128.0;
        if dist_sq <= cat_max_sq {
            within_max_range = true;
        }
    }

    within_max_range
}

/// Evaluates whether an unobstructed AABB can fit at the given voxel coordinate without intersecting solid blocks.
pub fn has_unobstructed_aabb(
    env: &impl SpawnEnvironment,
    x: i32,
    y: i32,
    z: i32,
    entity_type: EntityType,
) -> bool {
    let (width, height) = entity_type.default_dimensions();
    let feet_pos = Vec3::new(x as f32 + 0.5, y as f32, z as f32 + 0.5);
    let aabb = Aabb::from_feet_and_dimensions(feet_pos, width, height);

    let min_bx = aabb.min.x.floor() as i32;
    let max_bx = aabb.max.x.floor() as i32;
    let min_by = aabb.min.y.floor() as i32;
    let max_by = aabb.max.y.floor() as i32;
    let min_bz = aabb.min.z.floor() as i32;
    let max_bz = aabb.max.z.floor() as i32;

    for bx in min_bx..=max_bx {
        for by in min_by..=max_by {
            for bz in min_bz..=max_bz {
                let block = env.get_block(bx, by, bz);
                if is_solid_floor(block) {
                    return false;
                }
            }
        }
    }

    true
}

/// Verifies natural spawn criteria for hostile / monster mobs.
/// - Modern 1.18+ rule: Block light MUST be exactly 0.
/// - Solid floor beneath feet.
/// - Unobstructed head and body space.
pub fn verify_monster_conditions(
    env: &impl SpawnEnvironment,
    x: i32,
    y: i32,
    z: i32,
    entity_type: EntityType,
) -> bool {
    // 1. Block light check (Modern rule: Block Light == 0)
    let block_light = env.get_block_light(x, y, z);
    if block_light > 0 {
        return false;
    }

    // 2. Daytime sky light check for overworld undead
    if env.is_daytime() && entity_type.is_undead() {
        let sky_light = env.get_sky_light(x, y, z);
        if sky_light > 7 {
            return false;
        }
    }

    // 3. Solid floor beneath
    let floor_block = env.get_block(x, y - 1, z);
    if !is_solid_floor(floor_block) {
        return false;
    }

    // 4. Passable body and head space
    let body_block = env.get_block(x, y, z);
    let head_block = env.get_block(x, y + 1, z);
    if !is_passable_space(body_block) || !is_passable_space(head_block) {
        return false;
    }

    // 5. Unobstructed AABB
    has_unobstructed_aabb(env, x, y, z, entity_type)
}

/// Verifies natural spawn criteria for passive land animals / creatures.
/// - Must spawn on GrassBlock.
/// - Combined light level (sky or block light) >= 9.
/// - Unobstructed body and head space.
pub fn verify_creature_conditions(
    env: &impl SpawnEnvironment,
    x: i32,
    y: i32,
    z: i32,
    entity_type: EntityType,
) -> bool {
    // 1. Solid Grass Block floor beneath
    let floor_block = env.get_block(x, y - 1, z);
    if !is_grass_block(floor_block) {
        return false;
    }

    // 2. Light level >= 9
    let block_light = env.get_block_light(x, y, z);
    let sky_light = if env.is_daytime() {
        env.get_sky_light(x, y, z)
    } else {
        0
    };
    let effective_light = block_light.max(sky_light);
    if effective_light < 9 {
        return false;
    }

    // 3. Passable space
    let body_block = env.get_block(x, y, z);
    let head_block = env.get_block(x, y + 1, z);
    if !is_passable_space(body_block) || !is_passable_space(head_block) {
        return false;
    }

    // 4. Unobstructed AABB
    has_unobstructed_aabb(env, x, y, z, entity_type)
}

/// Verifies natural spawn criteria for aquatic mobs (Fish, Squids, Dolphins, etc.).
/// - Target position must be water.
/// - Block beneath must be water or solid sea floor.
pub fn verify_aquatic_conditions(
    env: &impl SpawnEnvironment,
    x: i32,
    y: i32,
    z: i32,
    _entity_type: EntityType,
) -> bool {
    let body_block = env.get_block(x, y, z);
    if !is_water(body_block) {
        return false;
    }

    let floor_block = env.get_block(x, y - 1, z);
    if !is_water(floor_block) && !is_solid_floor(floor_block) {
        return false;
    }

    true
}

/// Verifies natural spawn criteria for ambient mobs (Bats).
/// - Must be dark (block light <= 3 or underground below Y=63).
/// - Passable air space and solid block adjacent or ceiling.
pub fn verify_ambient_conditions(
    env: &impl SpawnEnvironment,
    x: i32,
    y: i32,
    z: i32,
    entity_type: EntityType,
) -> bool {
    let body_block = env.get_block(x, y, z);
    if !is_passable_space(body_block) {
        return false;
    }

    let block_light = env.get_block_light(x, y, z);
    if block_light > 3 && y >= 63 {
        return false;
    }

    has_unobstructed_aabb(env, x, y, z, entity_type)
}

/// Master spawn predicate dispatcher based on mob category.
pub fn can_spawn_at(
    env: &impl SpawnEnvironment,
    category: MobCategory,
    x: i32,
    y: i32,
    z: i32,
    entity_type: EntityType,
) -> bool {
    match category {
        MobCategory::Monster => verify_monster_conditions(env, x, y, z, entity_type),
        MobCategory::Creature => verify_creature_conditions(env, x, y, z, entity_type),
        MobCategory::WaterCreature
        | MobCategory::WaterAmbient
        | MobCategory::UndergroundWaterCreature
        | MobCategory::Axolotls => verify_aquatic_conditions(env, x, y, z, entity_type),
        MobCategory::Ambient => verify_ambient_conditions(env, x, y, z, entity_type),
        MobCategory::Misc => false,
    }
}

/// Evaluates whether an active mob in the world should be despawned:
/// 1. Persistent mobs (tamed, nametagged) never despawn.
/// 2. Beyond category despawn distance (128 blocks, 64 for fish): instant despawn.
/// 3. Between 32 and 128 blocks: if age >= 600 ticks, 1/800 roll per tick triggers despawn.
/// 4. Within 32 blocks: mob never naturally despawns based on distance/age.
pub fn should_despawn(
    entity_type: EntityType,
    mob_pos: Vec3,
    player_positions: &[Vec3],
    age_ticks: u32,
    is_persistent: bool,
    rng_roll: f32,
) -> bool {
    if is_persistent {
        return false;
    }

    if player_positions.is_empty() {
        return false;
    }

    let mut min_dist_sq = f32::MAX;
    for player_pos in player_positions {
        let d2 = mob_pos.distance_squared(*player_pos);
        if d2 < min_dist_sq {
            min_dist_sq = d2;
        }
    }

    let category = category_for_entity(entity_type);
    let despawn_dist_sq = category.despawn_distance_sq();

    // 1. Instant despawn beyond max despawn distance
    if min_dist_sq > despawn_dist_sq {
        return true;
    }

    // 2. Safe zone within 32 blocks
    let min_dist = min_dist_sq.sqrt();
    if min_dist <= NO_DESPAWN_DISTANCE {
        return false;
    }

    // 3. Random roll between 32 and 128 blocks for mobs older than 600 ticks (30s)
    if age_ticks >= DESPAWN_AGE_THRESHOLD {
        let threshold = 1.0 / (DESPAWN_CHANCE_1_IN as f32);
        if rng_roll < threshold {
            return true;
        }
    }

    false
}
