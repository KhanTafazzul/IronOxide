//! Minecraft 1.21.4 Pale Garden Feature Decoration & Creaking Heart State Machine.
//! REA Reference: net.minecraft.world.level.block.CreakingHeartBlock
//! REA Reference: net.minecraft.world.level.block.entity.CreakingHeartBlockEntity

use crate::biomes::pale_garden::generate_pale_oak_tree;
use crate::biomes::{SurfaceBlock, MIN_Y, SEA_LEVEL};
use crate::pipeline::{ChunkColumnBuffer, CHUNK_HEIGHT};
use crate::seed_pipeline::random::Xoroshiro128PlusPlus;
use crate::structures::LegacyRandom;

// ----------------------------------------------------------------------------
// 1. CREAKING HEART BLOCK STATE & DORMANCY ENUMS
// ----------------------------------------------------------------------------

/// Axis alignment for Creaking Heart block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CreakingHeartAxis {
    X,
    Y,
    Z,
}

impl Default for CreakingHeartAxis {
    fn default() -> Self {
        Self::Y
    }
}

/// Operational dormancy state of the Creaking Heart tile entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CreakingHeartDormancy {
    /// Inactive during day (daylight > 0) or when not properly encased in Pale Oak logs.
    Dormant,
    /// Actively pulsing at night (daylight == 0), binding and controlling a Creaking puppet.
    Awake,
    /// Block destroyed or mined; puppet is instantly eradicated.
    Destroyed,
}

/// Actions dispatched by the Creaking Heart state machine during server / world tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreakingHeartAction {
    /// No change this tick.
    None,
    /// Creaking heart awakens at night and requests spawning a new bound Creaking puppet entity.
    SpawnCreaking {
        heart_pos: (i32, i32, i32),
    },
    /// Creaking heart enters dormant state during day or was destroyed; bound puppet must despawn.
    DespawnCreaking {
        creaking_uuid: u128,
    },
    /// Bound Creaking was attacked; damage redirected to heart with resin particle burst.
    ResinBurst {
        heart_pos: (i32, i32, i32),
        resin_level: u32,
    },
}

/// Result of an incoming attack on either the Creaking puppet or the heart.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CreakingDamageResult {
    /// Amount of damage absorbed and applied to the heart.
    pub heart_damage: f32,
    /// Whether the puppet itself negated the direct damage.
    pub puppet_damage_negated: bool,
    /// Current resin accumulation level (0..=4).
    pub new_resin_level: u32,
    /// Whether this attack destroyed the heart.
    pub heart_destroyed: bool,
}

// ----------------------------------------------------------------------------
// 2. CREAKING HEART TILE ENTITY STATE MACHINE
// ----------------------------------------------------------------------------

/// Tile entity state machine for Minecraft 1.21.4 Creaking Heart.
///
/// Behavior & Invariants:
/// - Naturally generates embedded inside ~15% of Pale Oak tree trunks.
/// - Requires being flanked on its axis by Pale Oak logs (top and bottom for Y axis).
/// - Dormant during day (`daylight_level > 0`): `is_active = false`. Bound Creaking despawns.
/// - Awakens at night (`daylight_level == 0`): `is_active = true`. Spawns and commands bound Creaking.
/// - Redirects all direct damage taken by the Creaking puppet directly into itself, emitting resin.
/// - If destroyed, the bound Creaking entity immediately crumbles and vanishes.
#[derive(Debug, Clone, PartialEq)]
pub struct CreakingHeartTileEntity {
    pub position: (i32, i32, i32),
    pub axis: CreakingHeartAxis,
    pub bound_creaking_uuid: Option<u128>,
    pub is_active: bool,
    pub is_natural: bool,
    pub dormancy_state: CreakingHeartDormancy,
    pub health: f32,
    pub max_health: f32,
    pub resin_level: u32,
    pub ticks_alive: u64,
}

impl CreakingHeartTileEntity {
    /// Creates a new Creaking Heart tile entity at given coordinates.
    pub fn new(position: (i32, i32, i32), is_natural: bool) -> Self {
        Self {
            position,
            axis: CreakingHeartAxis::Y,
            bound_creaking_uuid: None,
            is_active: false,
            is_natural,
            dormancy_state: CreakingHeartDormancy::Dormant,
            health: 20.0,
            max_health: 20.0,
            resin_level: 0,
            ticks_alive: 0,
        }
    }

    /// Primary server tick state machine execution.
    ///
    /// Parameters:
    /// - `daylight_level`: ambient sky light level (0 = full darkness/night, >0 = daylight).
    /// - `time_of_day`: Minecraft world game tick (day: 0..12999, night: 13000..23000).
    /// - `is_flanked_by_pale_oak`: true if flanked along its axis by Pale Oak logs.
    pub fn tick(
        &mut self,
        daylight_level: u8,
        time_of_day: u64,
        is_flanked_by_pale_oak: bool,
    ) -> CreakingHeartAction {
        self.ticks_alive += 1;

        if self.dormancy_state == CreakingHeartDormancy::Destroyed {
            return CreakingHeartAction::None;
        }

        let is_night = daylight_level == 0 || (time_of_day >= 13000 && time_of_day <= 23000);
        let can_activate = self.is_natural && is_night && is_flanked_by_pale_oak;

        if can_activate {
            // Night time awakening
            if !self.is_active || self.dormancy_state != CreakingHeartDormancy::Awake {
                self.is_active = true;
                self.dormancy_state = CreakingHeartDormancy::Awake;
            }

            // Spawn bound Creaking puppet if none is currently active
            if self.bound_creaking_uuid.is_none() {
                return CreakingHeartAction::SpawnCreaking {
                    heart_pos: self.position,
                };
            }

            CreakingHeartAction::None
        } else {
            // Daylight or missing log encasement -> enter dormant state
            let was_awake = self.is_active || self.dormancy_state == CreakingHeartDormancy::Awake;
            self.is_active = false;
            self.dormancy_state = CreakingHeartDormancy::Dormant;

            if was_awake {
                if let Some(uuid) = self.bound_creaking_uuid.take() {
                    return CreakingHeartAction::DespawnCreaking {
                        creaking_uuid: uuid,
                    };
                }
            }

            CreakingHeartAction::None
        }
    }

    /// Binds an existing or freshly spawned Creaking entity UUID to this heart.
    pub fn bind_creaking(&mut self, uuid: u128) {
        self.bound_creaking_uuid = Some(uuid);
    }

    /// Unbinds any active Creaking entity.
    pub fn unbind_creaking(&mut self) -> Option<u128> {
        self.bound_creaking_uuid.take()
    }

    /// Processes damage directed at the bound Creaking puppet.
    /// The puppet is invulnerable to direct physical hits; damage is redirected
    /// to the heart, incrementing resin accumulation and triggering visual resin bursts.
    pub fn on_creaking_attacked(&mut self, incoming_damage: f32) -> CreakingDamageResult {
        if self.dormancy_state == CreakingHeartDormancy::Destroyed {
            return CreakingDamageResult {
                heart_damage: 0.0,
                puppet_damage_negated: true,
                new_resin_level: self.resin_level,
                heart_destroyed: true,
            };
        }

        // Apply redirected damage to heart
        self.health = (self.health - incoming_damage).max(0.0);
        if self.resin_level < 4 {
            self.resin_level += 1;
        }

        let heart_destroyed = self.health <= 0.0;
        if heart_destroyed {
            self.dormancy_state = CreakingHeartDormancy::Destroyed;
            self.is_active = false;
        }

        CreakingDamageResult {
            heart_damage: incoming_damage,
            puppet_damage_negated: true,
            new_resin_level: self.resin_level,
            heart_destroyed,
        }
    }

    /// Triggered when the Creaking Heart block itself is broken by a player.
    /// Instantly eradicates and unbinds the Creaking entity puppet.
    pub fn on_heart_broken(&mut self) -> Option<u128> {
        self.dormancy_state = CreakingHeartDormancy::Destroyed;
        self.is_active = false;
        self.bound_creaking_uuid.take()
    }
}

// ----------------------------------------------------------------------------
// 3. CHUNK DECORATION FOR PALE GARDEN
// ----------------------------------------------------------------------------

/// Natural probability of embedding a Creaking Heart in a Pale Oak tree trunk (~15%).
pub const CREAKING_HEART_CHANCE_PERCENT: i32 = 15;

/// Decorates a chunk column buffer with Pale Garden vegetation:
/// - Pale Oak trees with 2x2 trunks and flat sprawling canopies.
/// - ~15% chance of embedding a natural `CreakingHeart` inside each Pale Oak tree trunk.
/// - Pale Moss ground cover (`PaleMossBlock` patches and `PaleMossCarpet`).
pub fn decorate_pale_garden_vegetation(
    buffer: &mut ChunkColumnBuffer,
    chunk_x: i32,
    chunk_z: i32,
    rng: &mut Xoroshiro128PlusPlus,
) -> usize {
    let mut trees_placed = 0;
    let tree_count = 3 + rng.next_int_bounded(3); // 3 to 5 trees per chunk

    for _ in 0..tree_count {
        let lx = 2 + rng.next_int_bounded(12);
        let lz = 2 + rng.next_int_bounded(12);
        let wx = chunk_x * 16 + lx;
        let wz = chunk_z * 16 + lz;

        // Find surface Y level
        let mut surface_y = None;
        for y in (SEA_LEVEL..=200).rev() {
            let b = buffer.get_local(lx as usize, (y - MIN_Y) as usize, lz as usize);
            if b != SurfaceBlock::Air && b != SurfaceBlock::Water && b != SurfaceBlock::PaleHangingMoss {
                surface_y = Some(y);
                break;
            }
        }

        if let Some(sy) = surface_y {
            let soil = buffer.get_local(lx as usize, (sy - MIN_Y) as usize, lz as usize);
            if matches!(soil, SurfaceBlock::GrassBlock | SurfaceBlock::Dirt | SurfaceBlock::PaleMossBlock) {
                // ~15% chance of placing Creaking Heart inside this tree
                let place_creaking_heart = rng.next_int_bounded(100) < CREAKING_HEART_CHANCE_PERCENT;
                let mut legacy_rng = LegacyRandom::new(rng.next_long());

                let result = generate_pale_oak_tree(
                    buffer,
                    (wx, sy + 1, wz),
                    &mut legacy_rng,
                    place_creaking_heart,
                );

                if result.logs_placed > 0 {
                    trees_placed += 1;

                    // Scatter Pale Moss block and carpet patches around tree base
                    let patch_radius = 2 + rng.next_int_bounded(3);
                    for px in -patch_radius..=patch_radius {
                        for pz in -patch_radius..=patch_radius {
                            let plx = (lx + px).clamp(0, 15) as usize;
                            let plz = (lz + pz).clamp(0, 15) as usize;
                            let py = (sy - MIN_Y) as usize;

                            if py < CHUNK_HEIGHT {
                                let cur = buffer.get_local(plx, py, plz);
                                if cur == SurfaceBlock::GrassBlock && rng.next_int_bounded(3) == 0 {
                                    buffer.set_local(plx, py, plz, SurfaceBlock::PaleMossBlock);

                                    // Option to place PaleMossCarpet on top if air
                                    if py + 1 < CHUNK_HEIGHT
                                        && buffer.get_local(plx, py + 1, plz) == SurfaceBlock::Air
                                        && rng.next_int_bounded(2) == 0
                                    {
                                        buffer.set_local(plx, py + 1, plz, SurfaceBlock::PaleMossCarpet);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    trees_placed
}
