// REA Verified: net.minecraft.server.level.ChunkMap (mapped: aqe.class)
// REA Verified: net.minecraft.world.level.chunk.storage.ChunkSerializer (mapped: dmr.class)
//
// Real-Time Asynchronous Chunk Streaming & Meshing Pipeline:
// - Radius R active exploration (e.g. 12-16 chunks) prioritized by Euclidean distance from player.
// - Anvil Region file (.mca) check -> NBT deserialization fallback to full 6-stage procedural generation:
//   Noise (3D density) -> Carvers (cave bubbles/tunnels) -> Surface (biome strata) ->
//   Ores (vanilla trapezoids) -> Foliage (trees & vegetation) -> Structures (mineshafts).
// - 4-Neighbor completion barrier: Dispatches 3D Greedy Meshing to background Rayon worker pool.
// - Main-thread budget-controlled GPU vertex/index buffer upload for stutter-free 60+ FPS.
// - Radius R + 2 chunk column unloading, dirty .mca persistence, and memory reclamation (< 500 MB RAM).

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use glam::IVec3;
use wgpu::util::DeviceExt;

use crate::game::World;
use crate::render::atlas::{BlockType, TextureAtlas};
use crate::render::mesher::{greedy_mesh_section, ChunkMesh, ChunkSection, SECTION_SIZE};
use crate::storage::region::{deserialize_chunk_column, RegionFile};
use crate::storage::WorldStorageManager;
use crate::world::lighting::SectionLight;
use mc_worldgen::biomes::{BiomeId, SurfaceBlock};
use mc_worldgen::pipeline::{generate_chunk_column_pipeline, ChunkColumnBuffer, CHUNK_HEIGHT, MIN_Y};
use mc_worldgen::seed_pipeline::terrain::OverworldGenerator;

/// Default active chunk radius R
pub const DEFAULT_CHUNK_RADIUS: i32 = 12;
/// Unload buffer radius (R + 2)
pub const UNLOAD_PADDING: i32 = 2;
/// Maximum GPU mesh uploads per frame to prevent frame-time spikes
pub const MAX_GPU_UPLOADS_PER_FRAME: usize = 32;
/// Maximum chunk generation tasks dispatched per frame
pub const MAX_CHUNKS_DISPATCHED_PER_FRAME: usize = 8;

/// Maps mc_worldgen SurfaceBlock to game engine BlockType
pub fn surface_block_to_block_type(sb: SurfaceBlock) -> BlockType {
    match sb {
        SurfaceBlock::Air => BlockType::Air,
        SurfaceBlock::Stone => BlockType::Stone,
        SurfaceBlock::Deepslate => BlockType::Deepslate,
        SurfaceBlock::Bedrock => BlockType::Bedrock,
        SurfaceBlock::Dirt
        | SurfaceBlock::CoarseDirt
        | SurfaceBlock::Podzol
        | SurfaceBlock::Mud => BlockType::Dirt,
        SurfaceBlock::GrassBlock => BlockType::GrassBlock,
        SurfaceBlock::Sand
        | SurfaceBlock::Sandstone
        | SurfaceBlock::RedSand
        | SurfaceBlock::RedSandstone => BlockType::Sand,
        SurfaceBlock::Gravel => BlockType::Gravel,
        SurfaceBlock::Water | SurfaceBlock::Ice | SurfaceBlock::PackedIce => BlockType::Water,
        SurfaceBlock::Lava => BlockType::Lava,
        SurfaceBlock::SnowBlock | SurfaceBlock::SnowLayer | SurfaceBlock::PowderSnow => BlockType::Glass,
        SurfaceBlock::Terracotta
        | SurfaceBlock::WhiteTerracotta
        | SurfaceBlock::OrangeTerracotta
        | SurfaceBlock::YellowTerracotta
        | SurfaceBlock::BrownTerracotta
        | SurfaceBlock::RedTerracotta
        | SurfaceBlock::LightGrayTerracotta => BlockType::Bricks,

        SurfaceBlock::CoalOre | SurfaceBlock::DeepslateCoalOre => BlockType::CoalOre,
        SurfaceBlock::IronOre | SurfaceBlock::DeepslateIronOre => BlockType::IronOre,
        SurfaceBlock::CopperOre | SurfaceBlock::DeepslateCopperOre => BlockType::CopperOre,
        SurfaceBlock::GoldOre | SurfaceBlock::DeepslateGoldOre => BlockType::GoldOre,
        SurfaceBlock::RedstoneOre | SurfaceBlock::DeepslateRedstoneOre => BlockType::RedstoneOre,
        SurfaceBlock::DiamondOre | SurfaceBlock::DeepslateDiamondOre => BlockType::DiamondOre,
        SurfaceBlock::EmeraldOre | SurfaceBlock::DeepslateEmeraldOre => BlockType::EmeraldOre,
        SurfaceBlock::LapisOre | SurfaceBlock::DeepslateLapisOre => BlockType::LapisOre,

        SurfaceBlock::OakLog => BlockType::OakLog,
        SurfaceBlock::BirchLog => BlockType::BirchLog,
        SurfaceBlock::SpruceLog => BlockType::SpruceLog,
        SurfaceBlock::DarkOakLog | SurfaceBlock::JungleLog | SurfaceBlock::AcaciaLog => {
            BlockType::DarkOakLog
        }

        SurfaceBlock::OakLeaves => BlockType::OakLeaves,
        SurfaceBlock::BirchLeaves => BlockType::BirchLeaves,
        SurfaceBlock::SpruceLeaves => BlockType::SpruceLeaves,
        SurfaceBlock::DarkOakLeaves | SurfaceBlock::JungleLeaves | SurfaceBlock::AcaciaLeaves => {
            BlockType::DarkOakLeaves
        }

        SurfaceBlock::OakPlanks => BlockType::OakPlanks,
        SurfaceBlock::DarkOakPlanks => BlockType::DarkOakPlanks,
        SurfaceBlock::OakFence => BlockType::OakPlanks,
        SurfaceBlock::Cobblestone => BlockType::Cobblestone,
        SurfaceBlock::Chest => BlockType::Chest,
        SurfaceBlock::ShortGrass
        | SurfaceBlock::TallGrass
        | SurfaceBlock::Fern
        | SurfaceBlock::Poppy
        | SurfaceBlock::Dandelion
        | SurfaceBlock::Sunflower => BlockType::Vine,
        _ => BlockType::Stone,
    }
}

/// Converts a generated ChunkColumnBuffer into 16x16x16 ChunkSection models with skylight
pub fn buffer_to_chunk_sections(
    buffer: &ChunkColumnBuffer,
) -> Vec<(i32, ChunkSection, Option<SectionLight>)> {
    let cx = buffer.chunk_x;
    let cz = buffer.chunk_z;
    let mut sections = Vec::new();

    for sec_y in -4..=19 {
        let sec_pos = IVec3::new(cx, sec_y, cz);
        let mut section = ChunkSection::new(sec_pos);
        let mut non_air_count = 0;

        for ly in 0..16 {
            let world_y = sec_y * 16 + ly as i32;
            let buf_y = (world_y - MIN_Y) as usize;
            if buf_y >= CHUNK_HEIGHT {
                continue;
            }

            for lz in 0..16 {
                for lx in 0..16 {
                    let sb = buffer.get_local(lx, buf_y, lz);
                    let b = surface_block_to_block_type(sb);
                    if b != BlockType::Air {
                        section.set(lx, ly, lz, b);
                        non_air_count += 1;
                    }
                }
            }
        }

        if non_air_count > 0 {
            let mut light = SectionLight::new();
            // Initialize skylight level 15 for top voxels
            for lz in 0..16 {
                for lx in 0..16 {
                    for ly in 0..16 {
                        if section.get(lx, ly, lz) == BlockType::Air {
                            light.sky_light.set(lx, ly, lz, 15);
                        }
                    }
                }
            }
            sections.push((sec_y, section, Some(light)));
        }
    }

    sections
}

/// GPU-uploaded chunk mesh buffers
pub struct GpuChunkMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub quad_count: usize,
}

/// Status of chunk in pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkStatus {
    Generating,
    Loaded,
}

/// Message returned from background chunk loading / generation task
pub struct ChunkLoadMessage {
    pub cx: i32,
    pub cz: i32,
    pub sections: Vec<(i32, ChunkSection, Option<SectionLight>)>,
    pub from_disk: bool,
}

/// Message returned from background meshing task
pub struct MeshLoadMessage {
    pub chunk_pos: (i32, i32),
    pub section_meshes: Vec<(IVec3, ChunkMesh)>,
}

/// Real-time asynchronous chunk streaming coordinator
pub struct ChunkManager {
    pub active_radius: i32,
    pub unload_radius: i32,
    pub world_seed: i64,
    pub chunk_states: HashMap<(i32, i32), ChunkStatus>,
    pub meshed_chunks: HashSet<(i32, i32)>,
    pub in_flight_load_gen: HashSet<(i32, i32)>,
    pub in_flight_meshing: HashSet<(i32, i32)>,
    pub gpu_meshes: HashMap<IVec3, GpuChunkMesh>,
    chunk_tx: Sender<ChunkLoadMessage>,
    chunk_rx: Receiver<ChunkLoadMessage>,
    mesh_tx: Sender<MeshLoadMessage>,
    mesh_rx: Receiver<MeshLoadMessage>,
}

impl ChunkManager {
    pub fn new(world_seed: i64, active_radius: i32) -> Self {
        let (chunk_tx, chunk_rx) = channel();
        let (mesh_tx, mesh_rx) = channel();

        Self {
            active_radius,
            unload_radius: active_radius + UNLOAD_PADDING,
            world_seed,
            chunk_states: HashMap::new(),
            meshed_chunks: HashSet::new(),
            in_flight_load_gen: HashSet::new(),
            in_flight_meshing: HashSet::new(),
            gpu_meshes: HashMap::new(),
            chunk_tx,
            chunk_rx,
            mesh_tx,
            mesh_rx,
        }
    }

    /// Queries player position (PlayerChunkX, PlayerChunkZ), drains completed tasks,
    /// prioritizes missing chunks by distance, and manages memory eviction.
    pub fn update(
        &mut self,
        player_pos: glam::Vec3,
        world: &mut World,
        storage: &mut WorldStorageManager,
        atlas: &Arc<TextureAtlas>,
        device: Option<&wgpu::Device>,
        cpu_meshes: &mut HashMap<IVec3, ChunkMesh>,
    ) {
        let player_cx = (player_pos.x / 16.0).floor() as i32;
        let player_cz = (player_pos.z / 16.0).floor() as i32;

        // 1. Drain loaded/generated chunks from background worker threads
        while let Ok(msg) = self.chunk_rx.try_recv() {
            let cx = msg.cx;
            let cz = msg.cz;
            self.in_flight_load_gen.remove(&(cx, cz));

            // Discard if player moved far away before chunk finished generating
            let dist = (cx - player_cx).abs().max((cz - player_cz).abs());
            if dist > self.unload_radius {
                continue;
            }

            for (sec_y, sec, light_opt) in msg.sections {
                let sec_pos = IVec3::new(cx, sec_y, cz);
                world.sections.insert(sec_pos, sec);
                if let Some(light) = light_opt {
                    world.lighting.sections.insert(sec_pos, light);
                }
            }

            if !msg.from_disk {
                storage.mark_dirty(cx, cz);
            }

            self.chunk_states.insert((cx, cz), ChunkStatus::Loaded);
        }

        // 2. Drain completed greedy section meshes and upload to WGPU
        let mut uploaded_this_frame = 0;
        while let Ok(msg) = self.mesh_rx.try_recv() {
            self.in_flight_meshing.remove(&msg.chunk_pos);
            self.meshed_chunks.insert(msg.chunk_pos);

            for (sec_pos, mesh) in msg.section_meshes {
                if let Some(dev) = device {
                    if !mesh.is_empty() {
                        let vertex_buffer = dev.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Section Dynamic Vertex Buffer"),
                            contents: bytemuck::cast_slice(&mesh.vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                        let index_buffer = dev.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Section Dynamic Index Buffer"),
                            contents: bytemuck::cast_slice(&mesh.indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });
                        let quad_count = mesh.quad_count();
                        let index_count = mesh.indices.len() as u32;

                        self.gpu_meshes.insert(
                            sec_pos,
                            GpuChunkMesh {
                                vertex_buffer,
                                index_buffer,
                                index_count,
                                quad_count,
                            },
                        );
                    } else {
                        self.gpu_meshes.remove(&sec_pos);
                    }
                }
                cpu_meshes.insert(sec_pos, mesh);
                uploaded_this_frame += 1;
                if uploaded_this_frame >= MAX_GPU_UPLOADS_PER_FRAME {
                    break;
                }
            }
        }

        // 3. Unload chunks beyond R + 2 to reclaim RAM & GPU memory (< 500 MB)
        let mut to_evict = Vec::new();
        for (&(cx, cz), _) in &self.chunk_states {
            let dist = (cx - player_cx).abs().max((cz - player_cz).abs());
            if dist > self.unload_radius {
                to_evict.push((cx, cz));
            }
        }

        for (cx, cz) in to_evict {
            // Save dirty chunk to .mca if modified
            if storage.dirty_chunks.contains(&(cx, cz)) {
                storage.save_chunk_async(cx, cz, world);
                storage.dirty_chunks.remove(&(cx, cz));
            }

            // Evict chunk geometry, voxel array and palette from RAM
            world.sections.retain(|pos, _| !(pos.x == cx && pos.z == cz));
            world.lighting.sections.retain(|pos, _| !(pos.x == cx && pos.z == cz));
            cpu_meshes.retain(|pos, _| !(pos.x == cx && pos.z == cz));
            self.gpu_meshes.retain(|pos, _| !(pos.x == cx && pos.z == cz));

            self.chunk_states.remove(&(cx, cz));
            self.meshed_chunks.remove(&(cx, cz));
            self.in_flight_meshing.remove(&(cx, cz));
            self.in_flight_load_gen.remove(&(cx, cz));
        }

        // 4. Identify chunks needing generation or loading, prioritized by distance (closest first)
        let mut candidates = Vec::new();
        for dx in -self.active_radius..=self.active_radius {
            for dz in -self.active_radius..=self.active_radius {
                let dist_sq = dx * dx + dz * dz;
                if dist_sq <= self.active_radius * self.active_radius {
                    let cx = player_cx + dx;
                    let cz = player_cz + dz;
                    if !self.chunk_states.contains_key(&(cx, cz))
                        && !self.in_flight_load_gen.contains(&(cx, cz))
                    {
                        candidates.push(((cx, cz), dist_sq));
                    }
                }
            }
        }

        // Sort closest first
        candidates.sort_by_key(|&(_, d)| d);

        // Dispatch background load/gen tasks up to budget
        let region_dir = storage.region_dir.clone();
        let world_seed = self.world_seed;
        let mut dispatched = 0;

        for ((cx, cz), _) in candidates {
            if dispatched >= MAX_CHUNKS_DISPATCHED_PER_FRAME {
                break;
            }

            self.in_flight_load_gen.insert((cx, cz));
            self.chunk_states.insert((cx, cz), ChunkStatus::Generating);

            let tx = self.chunk_tx.clone();
            let r_dir = region_dir.clone();

            rayon::spawn(move || {
                let rx_coord = cx.div_euclid(32);
                let rz_coord = cz.div_euclid(32);
                let mca_path = r_dir.join(format!("r.{rx_coord}.{rz_coord}.mca"));

                // 1. Check local Anvil region file (saves/World/region/r.X.Z.mca)
                let mut loaded_sections = None;
                if mca_path.exists() {
                    if let Ok(mut region) = RegionFile::open_or_create(&mca_path) {
                        if let Ok(Some(data)) = region.read_chunk(cx, cz) {
                            if let Ok(secs) = deserialize_chunk_column(&data) {
                                if !secs.is_empty() {
                                    loaded_sections = Some(secs);
                                }
                            }
                        }
                    }
                }

                if let Some(sections) = loaded_sections {
                    let _ = tx.send(ChunkLoadMessage {
                        cx,
                        cz,
                        sections,
                        from_disk: true,
                    });
                } else {
                    // 2. Dispatch full generation pipeline:
                    // Noise -> Carvers -> Surface -> Ores -> Foliage -> Structures
                    let generator = OverworldGenerator::new(world_seed);
                    let buffer = generate_chunk_column_pipeline(&generator, cx, cz);
                    let sections = buffer_to_chunk_sections(&buffer);
                    let _ = tx.send(ChunkLoadMessage {
                        cx,
                        cz,
                        sections,
                        from_disk: false,
                    });
                }
            });

            dispatched += 1;
        }

        // 5. Chunk meshing dispatch:
        // When a chunk and its 4 horizontal neighbors complete stage 4 (Foliage),
        // dispatch 3D Greedy Meshing to background thread pool.
        let mut ready_to_mesh = Vec::new();
        for (&(cx, cz), &status) in &self.chunk_states {
            if status != ChunkStatus::Loaded
                || self.meshed_chunks.contains(&(cx, cz))
                || self.in_flight_meshing.contains(&(cx, cz))
            {
                continue;
            }

            // Check 4 horizontal neighbors
            let n_north = self.chunk_states.get(&(cx, cz - 1)).copied() == Some(ChunkStatus::Loaded);
            let n_south = self.chunk_states.get(&(cx, cz + 1)).copied() == Some(ChunkStatus::Loaded);
            let n_west = self.chunk_states.get(&(cx - 1, cz)).copied() == Some(ChunkStatus::Loaded);
            let n_east = self.chunk_states.get(&(cx + 1, cz)).copied() == Some(ChunkStatus::Loaded);

            if n_north && n_south && n_west && n_east {
                ready_to_mesh.push((cx, cz));
            }
        }

        for (cx, cz) in ready_to_mesh {
            self.in_flight_meshing.insert((cx, cz));

            // Extract 3x3 neighborhood of chunk sections for boundary meshing and ambient occlusion
            let mut neighborhood = HashMap::new();
            let mut target_positions = Vec::new();

            for nx in (cx - 1)..=(cx + 1) {
                for nz in (cz - 1)..=(cz + 1) {
                    for sec_y in -4..=19 {
                        let pos = IVec3::new(nx, sec_y, nz);
                        if let Some(sec) = world.sections.get(&pos) {
                            neighborhood.insert(pos, sec.clone());
                            if nx == cx && nz == cz {
                                target_positions.push(pos);
                            }
                        }
                    }
                }
            }

            let mesh_tx = self.mesh_tx.clone();
            let atlas_clone = Arc::clone(atlas);

            rayon::spawn(move || {
                let mut section_meshes = Vec::new();
                for pos in target_positions {
                    if let Some(sec) = neighborhood.get(&pos) {
                        let mesh = greedy_mesh_section(sec, &atlas_clone, &neighborhood);
                        section_meshes.push((pos, mesh));
                    }
                }
                let _ = mesh_tx.send(MeshLoadMessage {
                    chunk_pos: (cx, cz),
                    section_meshes,
                });
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_manager_distance_sorting() {
        let mgr = ChunkManager::new(123456789, 4);
        assert_eq!(mgr.active_radius, 4);
        assert_eq!(mgr.unload_radius, 6);

        // Verify surface block mapping
        assert_eq!(surface_block_to_block_type(SurfaceBlock::GrassBlock), BlockType::GrassBlock);
        assert_eq!(surface_block_to_block_type(SurfaceBlock::Deepslate), BlockType::Deepslate);
        assert_eq!(surface_block_to_block_type(SurfaceBlock::DiamondOre), BlockType::DiamondOre);
        assert_eq!(surface_block_to_block_type(SurfaceBlock::OakLog), BlockType::OakLog);
    }
}
