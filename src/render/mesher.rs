// REA Verified: net.minecraft.client.renderer.chunk.SectionCompiler (mapped: ftr.class)
// REA Verified: net.minecraft.client.renderer.block.ModelBlockRenderer (mapped: ftx.class - Ambient Occlusion calc)
//
// 3D Greedy Meshing with Ambient Occlusion (0 to 3) for 16x16x16 Chunk Sections.
// Outputs interleaved vertex buffers: [x, y, z, normal_index, u, v, ao_value].
// Dispatches asynchronously via Rayon thread pool for zero-stutter background meshing.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use bytemuck::{Pod, Zeroable};
use glam::IVec3;
use rayon::prelude::*;

use crate::render::atlas::{BlockType, Direction, TextureAtlas};
use crate::render::colormap::{get_default_colormaps, BiomeColormaps};

pub const SECTION_SIZE: usize = 16;
pub const SECTION_VOLUME: usize = SECTION_SIZE * SECTION_SIZE * SECTION_SIZE; // 4096

/// Interleaved block vertex buffer representation: [x, y, z, normal_index, u, v, ao_value, r, g, b]
/// Guaranteed 4-byte alignment and 40-byte stride.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct BlockVertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub normal_idx: u32, // 0=Up, 1=Down, 2=North, 3=South, 4=West, 5=East
    pub u: f32,
    pub v: f32,
    pub ao: f32, // Ambient occlusion value: 0.0 (darkest), 1.0, 2.0, 3.0 (brightest)
    pub r: f32,  // Biome tint multiplier R (0.0 to 1.0)
    pub g: f32,  // Biome tint multiplier G (0.0 to 1.0)
    pub b: f32,  // Biome tint multiplier B (0.0 to 1.0)
}

impl BlockVertex {
    pub fn new(pos: [f32; 3], normal_idx: u32, uv: [f32; 2], ao: f32, color: [f32; 3]) -> Self {
        Self {
            x: pos[0],
            y: pos[1],
            z: pos[2],
            normal_idx,
            u: uv[0],
            v: uv[1],
            ao,
            r: color[0],
            g: color[1],
            b: color[2],
        }
    }

    pub fn new_untinted(pos: [f32; 3], normal_idx: u32, uv: [f32; 2], ao: f32) -> Self {
        Self::new(pos, normal_idx, uv, ao, [1.0, 1.0, 1.0])
    }

    /// WGPU Vertex Buffer Layout descriptor for standard pipeline binding
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BlockVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // x, y, z: Float32x3 (offset 0)
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // normal_index: Uint32 (offset 12)
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Uint32,
                },
                // u, v: Float32x2 (offset 16)
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // ao_value: Float32 (offset 24)
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
                // color r, g, b: Float32x3 (offset 28)
                wgpu::VertexAttribute {
                    offset: 28,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

/// Abstract block reader for neighbor querying across chunk borders
pub trait BlockReader: Send + Sync {
    fn get_block_at(&self, x: i32, y: i32, z: i32) -> BlockType;

    /// Read combined voxel light level (0-15) for dynamic vertex shading (defaults to full bright 15)
    fn get_light_at(&self, _x: i32, _y: i32, _z: i32) -> u8 {
        15
    }

    /// Read fluid level (1..=8) if fluid is present at this voxel coordinate
    fn get_fluid_level(&self, _x: i32, _y: i32, _z: i32) -> Option<u8> {
        None
    }

    /// Read biome climate (temperature, downfall/humidity) for colormap sampling.
    /// Defaults to Plains climate (0.8, 0.4).
    fn get_biome_climate(&self, _x: i32, _z: i32) -> (f32, f32) {
        (0.8, 0.4)
    }
}

/// 16x16x16 Chunk Section representing a single sub-chunk in Minecraft 1.20.4
#[derive(Clone, Debug)]
pub struct ChunkSection {
    pub position: IVec3,
    pub blocks: [BlockType; SECTION_VOLUME],
    pub tile_entities: std::collections::HashMap<IVec3, crate::world::tile_entity::TileEntity>,
}

impl ChunkSection {
    pub fn new(position: IVec3) -> Self {
        Self {
            position,
            blocks: [BlockType::Air; SECTION_VOLUME],
            tile_entities: std::collections::HashMap::new(),
        }
    }

    #[inline(always)]
    pub fn index(x: usize, y: usize, z: usize) -> usize {
        x + z * SECTION_SIZE + y * (SECTION_SIZE * SECTION_SIZE)
    }

    #[inline(always)]
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType {
        if x < SECTION_SIZE && y < SECTION_SIZE && z < SECTION_SIZE {
            self.blocks[Self::index(x, y, z)]
        } else {
            BlockType::Air
        }
    }

    #[inline(always)]
    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockType {
        self.get_block(x, y, z)
    }

    #[inline(always)]
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        if x < SECTION_SIZE && y < SECTION_SIZE && z < SECTION_SIZE {
            let idx = Self::index(x, y, z);
            self.blocks[idx] = block;
        }
    }

    #[inline(always)]
    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        self.set_block(x, y, z, block);
    }

    pub fn get_tile_entity(&self, local_pos: IVec3) -> Option<&crate::world::tile_entity::TileEntity> {
        self.tile_entities.get(&local_pos)
    }

    pub fn get_tile_entity_mut(
        &mut self,
        local_pos: IVec3,
    ) -> Option<&mut crate::world::tile_entity::TileEntity> {
        self.tile_entities.get_mut(&local_pos)
    }

    pub fn set_tile_entity(&mut self, local_pos: IVec3, entity: crate::world::tile_entity::TileEntity) {
        self.tile_entities.insert(local_pos, entity);
    }

    pub fn remove_tile_entity(&mut self, local_pos: IVec3) -> Option<crate::world::tile_entity::TileEntity> {
        self.tile_entities.remove(&local_pos)
    }
}

impl BlockReader for ChunkSection {
    #[inline(always)]
    fn get_block_at(&self, x: i32, y: i32, z: i32) -> BlockType {
        if x >= 0 && x < SECTION_SIZE as i32
            && y >= 0 && y < SECTION_SIZE as i32
            && z >= 0 && z < SECTION_SIZE as i32
        {
            self.blocks[Self::index(x as usize, y as usize, z as usize)]
        } else {
            BlockType::Air
        }
    }
}

/// Mesh output containing interleaved vertices and triangle index buffer
#[derive(Default, Clone, Debug)]
pub struct ChunkMesh {
    pub vertices: Vec<BlockVertex>,
    pub indices: Vec<u32>,
}

impl ChunkMesh {
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    pub fn quad_count(&self) -> usize {
        self.indices.len() / 6
    }
}

/// Direction index mapping matching standard shader normals
#[inline(always)]
pub fn direction_to_normal_idx(dir: Direction) -> u32 {
    match dir {
        Direction::Up => 0,    // +Y
        Direction::Down => 1,  // -Y
        Direction::North => 2, // -Z
        Direction::South => 3, // +Z
        Direction::West => 4,  // -X
        Direction::East => 5,  // +X
    }
}

/// Direction unit normal vector
#[inline(always)]
pub fn direction_to_normal(dir: Direction) -> [i32; 3] {
    match dir {
        Direction::Up => [0, 1, 0],
        Direction::Down => [0, -1, 0],
        Direction::North => [0, 0, -1],
        Direction::South => [0, 0, 1],
        Direction::West => [-1, 0, 0],
        Direction::East => [1, 0, 0],
    }
}

/// Ambient Occlusion calculator based on net.minecraft.client.renderer.block.ModelBlockRenderer (ftx.class)
/// Evaluates 3 neighboring blocks (side 1, side 2, corner) touching the face vertex.
/// Returns integer AO value: 0 (darkest), 1, 2, 3 (brightest).
#[inline]
pub fn calculate_vertex_ao(
    reader: &dyn BlockReader,
    block_pos: [i32; 3],
    normal: [i32; 3],
    offset_u: [i32; 3],
    offset_v: [i32; 3],
) -> u8 {
    let side1_pos = [
        block_pos[0] + normal[0] + offset_u[0],
        block_pos[1] + normal[1] + offset_u[1],
        block_pos[2] + normal[2] + offset_u[2],
    ];
    let side2_pos = [
        block_pos[0] + normal[0] + offset_v[0],
        block_pos[1] + normal[1] + offset_v[1],
        block_pos[2] + normal[2] + offset_v[2],
    ];
    let corner_pos = [
        block_pos[0] + normal[0] + offset_u[0] + offset_v[0],
        block_pos[1] + normal[1] + offset_u[1] + offset_v[1],
        block_pos[2] + normal[2] + offset_u[2] + offset_v[2],
    ];

    let s1 = reader.get_block_at(side1_pos[0], side1_pos[1], side1_pos[2]).is_opaque();
    let s2 = reader.get_block_at(side2_pos[0], side2_pos[1], side2_pos[2]).is_opaque();
    let c = reader.get_block_at(corner_pos[0], corner_pos[1], corner_pos[2]).is_opaque();

    if s1 && s2 {
        0
    } else {
        3 - (s1 as u8 + s2 as u8 + c as u8)
    }
}

/// Smooth vertex light calculation based on net.minecraft.client.renderer.block.ModelBlockRenderer (ftx.class).
/// Averages light levels across neighboring non-occluded voxels touching the vertex.
#[inline]
pub fn calculate_vertex_light(
    reader: &dyn BlockReader,
    block_pos: [i32; 3],
    normal: [i32; 3],
    offset_u: [i32; 3],
    offset_v: [i32; 3],
) -> u8 {
    let center = [
        block_pos[0] + normal[0],
        block_pos[1] + normal[1],
        block_pos[2] + normal[2],
    ];
    let s1 = [
        center[0] + offset_u[0],
        center[1] + offset_u[1],
        center[2] + offset_u[2],
    ];
    let s2 = [
        center[0] + offset_v[0],
        center[1] + offset_v[1],
        center[2] + offset_v[2],
    ];
    let c = [
        center[0] + offset_u[0] + offset_v[0],
        center[1] + offset_u[1] + offset_v[1],
        center[2] + offset_u[2] + offset_v[2],
    ];

    let l_center = reader.get_light_at(center[0], center[1], center[2]);
    let l_s1 = reader.get_light_at(s1[0], s1[1], s1[2]);
    let l_s2 = reader.get_light_at(s2[0], s2[1], s2[2]);
    let l_c = reader.get_light_at(c[0], c[1], c[2]);

    let max_val = l_center.max(l_s1).max(l_s2).max(l_c);
    let sum = l_center as u32 + l_s1 as u32 + l_s2 as u32 + l_c as u32;
    ((sum / 4) as u8).max(max_val.saturating_sub(1)).min(15)
}

/// Dynamic vertex shading combining ambient occlusion (0 to 3) with voxel light level (0 to 15).
/// Returns final vertex shade multiplier (0.0 to 3.0 scale).
#[inline]
pub fn compute_shaded_ao(ao: u8, light: u8) -> f32 {
    let light_mult = 0.10 + 0.90 * (light.min(15) as f32 / 15.0);
    (ao as f32) * light_mult
}

/// Internal cell representation on a 2D slice during greedy sweep
/// Internal cell representation on a 2D slice during greedy sweep
#[derive(Clone, Copy, PartialEq, Eq)]
struct FaceInfo {
    block: BlockType,
    direction: Direction,
    ao: [u8; 4], // [AO_bottom_left, AO_bottom_right, AO_top_right, AO_top_left]
    light: [u8; 4], // [Light_bottom_left, Light_bottom_right, Light_top_right, Light_top_left]
    tint: [u8; 3], // Quantized RGB tint multiplier for exact greedy face equality comparison
}

/// Determines if `from` block should render an exterior face against `to` block
#[inline]
pub fn face_is_visible(from: BlockType, to: BlockType) -> bool {
    if from.is_air() || from.is_fluid() {
        return false;
    }
    if to.is_air() || to.is_fluid() {
        return true;
    }
    if from == to {
        return false;
    }
    if !to.is_opaque() {
        return true;
    }
    false
}

/// 3D Greedy Meshing Algorithm with default global colormaps
pub fn greedy_mesh_section(
    section: &ChunkSection,
    atlas: &TextureAtlas,
    reader: &dyn BlockReader,
) -> ChunkMesh {
    greedy_mesh_section_with_colormaps(section, atlas, reader, get_default_colormaps())
}

/// 3D Greedy Meshing Algorithm with custom BiomeColormaps
/// Iterates across each axis (X, Y, Z), culls hidden faces, merges coplanar adjacent faces,
/// computes Ambient Occlusion per vertex, and applies 3x3 smoothed biome tinting.
pub fn greedy_mesh_section_with_colormaps(
    section: &ChunkSection,
    atlas: &TextureAtlas,
    reader: &dyn BlockReader,
    colormaps: &BiomeColormaps,
) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let world_offset = section.position * (SECTION_SIZE as i32);

    // Sweep across all 3 primary coordinate axes:
    // d = 0 -> X axis (faces point West / East)
    // d = 1 -> Y axis (faces point Down / Up)
    // d = 2 -> Z axis (faces point North / South)
    for d in 0..3 {
        let u = (d + 1) % 3;
        let v = (d + 2) % 3;

        let mut x = [0i32; 3];
        let mut q = [0i32; 3];
        q[d] = 1;

        // Mask for 2D slice faces: size (SECTION_SIZE + 1) along axis d, SECTION_SIZE along u & v
        // We evaluate slices from slice_d = 0 to slice_d = SECTION_SIZE
        for slice_d in 0..=SECTION_SIZE as i32 {
            let mut mask: [Option<FaceInfo>; SECTION_SIZE * SECTION_SIZE] =
                [None; SECTION_SIZE * SECTION_SIZE];

            // Build slice mask
            for slice_u in 0..SECTION_SIZE as i32 {
                for slice_v in 0..SECTION_SIZE as i32 {
                    x[d] = slice_d;
                    x[u] = slice_u;
                    x[v] = slice_v;

                    let pos_a = [x[0] - q[0], x[1] - q[1], x[2] - q[2]];
                    let pos_b = [x[0], x[1], x[2]];

                    let block_a = reader.get_block_at(pos_a[0], pos_a[1], pos_a[2]);
                    let block_b = reader.get_block_at(pos_b[0], pos_b[1], pos_b[2]);

                    let vis_a = face_is_visible(block_a, block_b);
                    let vis_b = face_is_visible(block_b, block_a);

                    let mask_idx = (slice_u as usize) * SECTION_SIZE + (slice_v as usize);

                    if vis_a || vis_b {
                        let (block, dir, face_block_pos) = if vis_a {
                            let dir = match d {
                                0 => Direction::East,
                                1 => Direction::Up,
                                _ => Direction::South,
                            };
                            (block_a, dir, pos_a)
                        } else {
                            let dir = match d {
                                0 => Direction::West,
                                1 => Direction::Down,
                                _ => Direction::North,
                            };
                            (block_b, dir, pos_b)
                        };

                        let normal = direction_to_normal(dir);

                        let mut u_dir = [0i32; 3];
                        u_dir[u] = 1;
                        let mut v_dir = [0i32; 3];
                        v_dir[v] = 1;

                        // Calculate AO for all 4 corners of this face cell
                        let mut neg_u = [0i32; 3];
                        neg_u[u] = -1;
                        let mut neg_v = [0i32; 3];
                        neg_v[v] = -1;

                        let ao0 = calculate_vertex_ao(reader, face_block_pos, normal, neg_u, neg_v);
                        let ao1 = calculate_vertex_ao(reader, face_block_pos, normal, u_dir, neg_v);
                        let ao2 = calculate_vertex_ao(reader, face_block_pos, normal, u_dir, v_dir);
                        let ao3 = calculate_vertex_ao(reader, face_block_pos, normal, neg_u, v_dir);

                        let light0 = calculate_vertex_light(reader, face_block_pos, normal, neg_u, neg_v);
                        let light1 = calculate_vertex_light(reader, face_block_pos, normal, u_dir, neg_v);
                        let light2 = calculate_vertex_light(reader, face_block_pos, normal, u_dir, v_dir);
                        let light3 = calculate_vertex_light(reader, face_block_pos, normal, neg_u, v_dir);

                        // Biome tint calculation:
                        // - Grass Blocks (Direction::Up)
                        // - Oak Leaves (all faces)
                        // - Vines (all faces)
                        let wx = world_offset.x + face_block_pos[0];
                        let wz = world_offset.z + face_block_pos[2];

                        let tint = if block == BlockType::GrassBlock && dir == Direction::Up {
                            let rgb = colormaps.get_smoothed_grass_color(reader, wx, wz);
                            [
                                (rgb[0] * 255.0).clamp(0.0, 255.0) as u8,
                                (rgb[1] * 255.0).clamp(0.0, 255.0) as u8,
                                (rgb[2] * 255.0).clamp(0.0, 255.0) as u8,
                            ]
                        } else if block == BlockType::OakLeaves || block == BlockType::Vine {
                            let rgb = colormaps.get_smoothed_foliage_color(reader, wx, wz);
                            [
                                (rgb[0] * 255.0).clamp(0.0, 255.0) as u8,
                                (rgb[1] * 255.0).clamp(0.0, 255.0) as u8,
                                (rgb[2] * 255.0).clamp(0.0, 255.0) as u8,
                            ]
                        } else {
                            [255, 255, 255]
                        };

                        mask[mask_idx] = Some(FaceInfo {
                            block,
                            direction: dir,
                            ao: [ao0, ao1, ao2, ao3],
                            light: [light0, light1, light2, light3],
                            tint,
                        });
                    }
                }
            }

            // Greedy merge coplanar adjacent faces with identical textures, AO, and tint
            for slice_u in 0..SECTION_SIZE {
                let mut slice_v = 0;
                while slice_v < SECTION_SIZE {
                    let idx = slice_u * SECTION_SIZE + slice_v;
                    if let Some(current_face) = mask[idx] {
                        // 1. Expand along v (width)
                        let mut width = 1;
                        while slice_v + width < SECTION_SIZE {
                            let next_idx = slice_u * SECTION_SIZE + (slice_v + width);
                            if mask[next_idx] == Some(current_face) {
                                width += 1;
                            } else {
                                break;
                            }
                        }

                        // 2. Expand along u (height)
                        let mut height = 1;
                        'height_loop: while slice_u + height < SECTION_SIZE {
                            for w in 0..width {
                                let check_idx = (slice_u + height) * SECTION_SIZE + (slice_v + w);
                                if mask[check_idx] != Some(current_face) {
                                    break 'height_loop;
                                }
                            }
                            height += 1;
                        }

                        // Clear the merged rectangle in the mask
                        for h in 0..height {
                            for w in 0..width {
                                mask[(slice_u + h) * SECTION_SIZE + (slice_v + w)] = None;
                            }
                        }

                        // Emit merged Quad
                        let u_start = slice_u as f32;
                        let u_end = (slice_u + height) as f32;
                        let v_start = slice_v as f32;
                        let v_end = (slice_v + width) as f32;
                        let depth = slice_d as f32;

                        let mut p0 = [0.0f32; 3];
                        let mut p1 = [0.0f32; 3];
                        let mut p2 = [0.0f32; 3];
                        let mut p3 = [0.0f32; 3];

                        p0[d] = depth;
                        p0[u] = u_start;
                        p0[v] = v_start;

                        p1[d] = depth;
                        p1[u] = u_end;
                        p1[v] = v_start;

                        p2[d] = depth;
                        p2[u] = u_end;
                        p2[v] = v_end;

                        p3[d] = depth;
                        p3[u] = u_start;
                        p3[v] = v_end;

                        // World position translation
                        let wp0 = [
                            p0[0] + world_offset.x as f32,
                            p0[1] + world_offset.y as f32,
                            p0[2] + world_offset.z as f32,
                        ];
                        let wp1 = [
                            p1[0] + world_offset.x as f32,
                            p1[1] + world_offset.y as f32,
                            p1[2] + world_offset.z as f32,
                        ];
                        let wp2 = [
                            p2[0] + world_offset.x as f32,
                            p2[1] + world_offset.y as f32,
                            p2[2] + world_offset.z as f32,
                        ];
                        let wp3 = [
                            p3[0] + world_offset.x as f32,
                            p3[1] + world_offset.y as f32,
                            p3[2] + world_offset.z as f32,
                        ];

                        let normal_idx = direction_to_normal_idx(current_face.direction);
                        let uv_bounds = atlas.get_face_uv(current_face.block, current_face.direction);
                        let u_min = uv_bounds[0];
                        let v_min = uv_bounds[1];
                        let u_max = uv_bounds[2];
                        let v_max = uv_bounds[3];

                        let shade0 = compute_shaded_ao(current_face.ao[0], current_face.light[0]);
                        let shade1 = compute_shaded_ao(current_face.ao[1], current_face.light[1]);
                        let shade2 = compute_shaded_ao(current_face.ao[2], current_face.light[2]);
                        let shade3 = compute_shaded_ao(current_face.ao[3], current_face.light[3]);

                        let vertex_tint = [
                            current_face.tint[0] as f32 / 255.0,
                            current_face.tint[1] as f32 / 255.0,
                            current_face.tint[2] as f32 / 255.0,
                        ];

                        let base_vertex_idx = mesh.vertices.len() as u32;

                        // Ensure counter-clockwise winding when viewed from exterior
                        let is_back_face = matches!(
                            current_face.direction,
                            Direction::Down | Direction::North | Direction::West
                        );

                        if !is_back_face {
                            mesh.vertices.push(BlockVertex::new(wp0, normal_idx, [u_min, v_min], shade0, vertex_tint));
                            mesh.vertices.push(BlockVertex::new(wp1, normal_idx, [u_max, v_min], shade1, vertex_tint));
                            mesh.vertices.push(BlockVertex::new(wp2, normal_idx, [u_max, v_max], shade2, vertex_tint));
                            mesh.vertices.push(BlockVertex::new(wp3, normal_idx, [u_min, v_max], shade3, vertex_tint));
                        } else {
                            mesh.vertices.push(BlockVertex::new(wp0, normal_idx, [u_min, v_min], shade0, vertex_tint));
                            mesh.vertices.push(BlockVertex::new(wp3, normal_idx, [u_min, v_max], shade3, vertex_tint));
                            mesh.vertices.push(BlockVertex::new(wp2, normal_idx, [u_max, v_max], shade2, vertex_tint));
                            mesh.vertices.push(BlockVertex::new(wp1, normal_idx, [u_max, v_min], shade1, vertex_tint));
                        }

                        // Minecraft AO/lighting diagonal flip rule to prevent interpolation creases
                        if (shade0 + shade2) > (shade1 + shade3) {
                            mesh.indices.extend_from_slice(&[
                                base_vertex_idx,
                                base_vertex_idx + 1,
                                base_vertex_idx + 2,
                                base_vertex_idx,
                                base_vertex_idx + 2,
                                base_vertex_idx + 3,
                            ]);
                        } else {
                            mesh.indices.extend_from_slice(&[
                                base_vertex_idx + 1,
                                base_vertex_idx + 2,
                                base_vertex_idx + 3,
                                base_vertex_idx + 1,
                                base_vertex_idx + 3,
                                base_vertex_idx,
                            ]);
                        }

                        slice_v += width;
                    } else {
                        slice_v += 1;
                    }
                }
            }
        }
    }

    // 2. Custom Fluid Pass: generate translucent/transparent quad faces
    // with non-full block heights proportional to fluid level (h = level / 8.0)
    for y in 0..SECTION_SIZE {
        for z in 0..SECTION_SIZE {
            for x in 0..SECTION_SIZE {
                let block = section.get_block(x, y, z);
                if block.is_fluid() {
                    mesh_fluid_voxel(x, y, z, block, world_offset, reader, atlas, &mut mesh);
                }
            }
        }
    }

    mesh
}

/// Generates custom proportional transparent/translucent quad faces for a fluid voxel
fn mesh_fluid_voxel(
    x: usize,
    y: usize,
    z: usize,
    block: BlockType,
    world_offset: IVec3,
    reader: &dyn BlockReader,
    atlas: &TextureAtlas,
    mesh: &mut ChunkMesh,
) {
    let wx = world_offset.x + x as i32;
    let wy = world_offset.y + y as i32;
    let wz = world_offset.z + z as i32;

    let fx = wx as f32;
    let fy = wy as f32;
    let fz = wz as f32;

    let level = reader.get_fluid_level(wx, wy, wz).unwrap_or(8);
    let h = (level as f32 / 8.0).clamp(0.125, 1.0);

    let light = reader.get_light_at(wx, wy, wz);

    // 1. Top Face (Direction::Up)
    let above = reader.get_block_at(wx, wy + 1, wz);
    if above != block {
        let normal_idx = 0; // Up
        let uv = atlas.get_face_uv(block, Direction::Up);
        let shade = compute_shaded_ao(3, reader.get_light_at(wx, wy + 1, wz).max(light));
        let base = mesh.vertices.len() as u32;

        mesh.vertices.push(BlockVertex::new_untinted([fx, fy + h, fz + 1.0], normal_idx, [uv[0], uv[3]], shade));
        mesh.vertices.push(BlockVertex::new_untinted([fx + 1.0, fy + h, fz + 1.0], normal_idx, [uv[2], uv[3]], shade));
        mesh.vertices.push(BlockVertex::new_untinted([fx + 1.0, fy + h, fz], normal_idx, [uv[2], uv[1]], shade));
        mesh.vertices.push(BlockVertex::new_untinted([fx, fy + h, fz], normal_idx, [uv[0], uv[1]], shade));

        mesh.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    // 2. Bottom Face (Direction::Down)
    let below = reader.get_block_at(wx, wy - 1, wz);
    if !below.is_opaque() && below != block {
        let normal_idx = 1; // Down
        let uv = atlas.get_face_uv(block, Direction::Down);
        let shade = compute_shaded_ao(3, reader.get_light_at(wx, wy - 1, wz).max(light)) * 0.5;
        let base = mesh.vertices.len() as u32;

        mesh.vertices.push(BlockVertex::new_untinted([fx, fy, fz], normal_idx, [uv[0], uv[3]], shade));
        mesh.vertices.push(BlockVertex::new_untinted([fx + 1.0, fy, fz], normal_idx, [uv[2], uv[3]], shade));
        mesh.vertices.push(BlockVertex::new_untinted([fx + 1.0, fy, fz + 1.0], normal_idx, [uv[2], uv[1]], shade));
        mesh.vertices.push(BlockVertex::new_untinted([fx, fy, fz + 1.0], normal_idx, [uv[0], uv[1]], shade));

        mesh.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    // 3. North Face (Direction::North, -Z)
    let north = reader.get_block_at(wx, wy, wz - 1);
    if !north.is_opaque() && north != block {
        let normal_idx = 2; // North
        let uv = atlas.get_face_uv(block, Direction::North);
        let shade = compute_shaded_ao(3, reader.get_light_at(wx, wy, wz - 1).max(light)) * 0.8;
        let base = mesh.vertices.len() as u32;
        let v_top = uv[3] - (uv[3] - uv[1]) * h;

        mesh.vertices.push(BlockVertex::new_untinted([fx + 1.0, fy, fz], normal_idx, [uv[2], uv[3]], shade));
        mesh.vertices.push(BlockVertex::new_untinted([fx, fy, fz], normal_idx, [uv[0], uv[3]], shade));
        mesh.vertices.push(BlockVertex::new_untinted([fx, fy + h, fz], normal_idx, [uv[0], v_top], shade));
        mesh.vertices.push(BlockVertex::new_untinted([fx + 1.0, fy + h, fz], normal_idx, [uv[2], v_top], shade));

        mesh.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    // 4. South Face (Direction::South, +Z)
    let south = reader.get_block_at(wx, wy, wz + 1);
    if !south.is_opaque() && south != block {
        let normal_idx = 3; // South
        let uv = atlas.get_face_uv(block, Direction::South);
        let shade = compute_shaded_ao(3, reader.get_light_at(wx, wy, wz + 1).max(light)) * 0.8;
        let base = mesh.vertices.len() as u32;
        let v_top = uv[3] - (uv[3] - uv[1]) * h;

        mesh.vertices.push(BlockVertex::new_untinted([fx, fy, fz + 1.0], normal_idx, [uv[0], uv[3]], shade));
        mesh.vertices.push(BlockVertex::new_untinted([fx + 1.0, fy, fz + 1.0], normal_idx, [uv[2], uv[3]], shade));
        mesh.vertices.push(BlockVertex::new_untinted([fx + 1.0, fy + h, fz + 1.0], normal_idx, [uv[2], v_top], shade));
        mesh.vertices.push(BlockVertex::new_untinted([fx, fy + h, fz + 1.0], normal_idx, [uv[0], v_top], shade));

        mesh.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    // 5. West Face (Direction::West, -X)
    let west = reader.get_block_at(wx - 1, wy, wz);
    if !west.is_opaque() && west != block {
        let normal_idx = 4; // West
        let uv = atlas.get_face_uv(block, Direction::West);
        let shade = compute_shaded_ao(3, reader.get_light_at(wx - 1, wy, wz).max(light)) * 0.6;
        let base = mesh.vertices.len() as u32;
        let v_top = uv[3] - (uv[3] - uv[1]) * h;

        mesh.vertices.push(BlockVertex::new_untinted([fx, fy, fz], normal_idx, [uv[0], uv[3]], shade));
        mesh.vertices.push(BlockVertex::new_untinted([fx, fy, fz + 1.0], normal_idx, [uv[2], uv[3]], shade));
        mesh.vertices.push(BlockVertex::new_untinted([fx, fy + h, fz + 1.0], normal_idx, [uv[2], v_top], shade));
        mesh.vertices.push(BlockVertex::new_untinted([fx, fy + h, fz], normal_idx, [uv[0], v_top], shade));

        mesh.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    // 6. East Face (Direction::East, +X)
    let east = reader.get_block_at(wx + 1, wy, wz);
    if !east.is_opaque() && east != block {
        let normal_idx = 5; // East
        let uv = atlas.get_face_uv(block, Direction::East);
        let shade = compute_shaded_ao(3, reader.get_light_at(wx + 1, wy, wz).max(light)) * 0.6;
        let base = mesh.vertices.len() as u32;
        let v_top = uv[3] - (uv[3] - uv[1]) * h;

        mesh.vertices.push(BlockVertex::new_untinted([fx + 1.0, fy, fz + 1.0], normal_idx, [uv[2], uv[3]], shade));
        mesh.vertices.push(BlockVertex::new_untinted([fx + 1.0, fy, fz], normal_idx, [uv[0], uv[3]], shade));
        mesh.vertices.push(BlockVertex::new_untinted([fx + 1.0, fy + h, fz], normal_idx, [uv[0], v_top], shade));
        mesh.vertices.push(BlockVertex::new_untinted([fx + 1.0, fy + h, fz + 1.0], normal_idx, [uv[2], v_top], shade));

        mesh.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

/// Mesh multiple chunk sections in parallel using Rayon thread pool
pub fn mesh_sections_parallel(
    sections: &[ChunkSection],
    atlas: &TextureAtlas,
) -> Vec<ChunkMesh> {
    sections
        .par_iter()
        .map(|section| greedy_mesh_section(section, atlas, section))
        .collect()
}

/// Asynchronous background chunk mesher running on Rayon worker threads
/// Delivers completed meshes to the render loop without dropping frames.
pub struct AsyncChunkMesher {
    tx: Sender<(IVec3, ChunkMesh)>,
    rx: Receiver<(IVec3, ChunkMesh)>,
}

impl Default for AsyncChunkMesher {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncChunkMesher {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self { tx, rx }
    }

    /// Dispatch a section to be meshed asynchronously in the background
    pub fn request_mesh(&self, section: ChunkSection, atlas: Arc<TextureAtlas>) {
        let tx = self.tx.clone();
        rayon::spawn(move || {
            let mesh = greedy_mesh_section(&section, &atlas, &section);
            let _ = tx.send((section.position, mesh));
        });
    }

    /// Retrieve all finished chunk meshes ready for GPU buffer upload
    pub fn poll_completed(&self) -> Vec<(IVec3, ChunkMesh)> {
        let mut completed = Vec::new();
        while let Ok(item) = self.rx.try_recv() {
            completed.push(item);
        }
        completed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_layout() {
        assert_eq!(std::mem::size_of::<BlockVertex>(), 40);
        let desc = BlockVertex::desc();
        assert_eq!(desc.array_stride, 40);
        assert_eq!(desc.attributes.len(), 5);
    }

    #[test]
    fn test_empty_section_mesh() {
        let section = ChunkSection::new(IVec3::ZERO);
        let atlas = TextureAtlas::load_or_create(None);
        let mesh = greedy_mesh_section(&section, &atlas, &section);
        assert!(mesh.is_empty());
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(mesh.index_count(), 0);
    }

    #[test]
    fn test_single_block_meshing() {
        let mut section = ChunkSection::new(IVec3::ZERO);
        section.set_block(8, 8, 8, BlockType::Stone);
        let atlas = TextureAtlas::load_or_create(None);
        let mesh = greedy_mesh_section(&section, &atlas, &section);

        // A single cube has 6 faces * 4 vertices = 24 vertices, 6 * 6 = 36 indices
        assert_eq!(mesh.quad_count(), 6);
        assert_eq!(mesh.vertex_count(), 24);
        assert_eq!(mesh.index_count(), 36);

        // Check normal indices covering all 6 directions
        let mut normal_indices: Vec<u32> = mesh.vertices.iter().map(|v| v.normal_idx).collect();
        normal_indices.sort_unstable();
        normal_indices.dedup();
        assert_eq!(normal_indices.len(), 6);
    }

    #[test]
    fn test_greedy_face_merging() {
        let mut section = ChunkSection::new(IVec3::ZERO);
        // Place a 2x2 flat slab of stone blocks on layer y = 0
        for x in 0..2 {
            for z in 0..2 {
                section.set_block(x, 0, z, BlockType::Stone);
            }
        }
        let atlas = TextureAtlas::load_or_create(None);
        let mesh = greedy_mesh_section(&section, &atlas, &section);

        // Without greedy meshing, the top face would be 4 separate quads.
        // With greedy meshing, the top face must be merged into 1 single quad!
        let top_quads = mesh
            .vertices
            .chunks_exact(4)
            .filter(|quad| quad[0].normal_idx == direction_to_normal_idx(Direction::Up))
            .count();

        assert_eq!(top_quads, 1, "2x2 top face should be merged into 1 greedy quad");
    }

    #[test]
    fn test_hidden_face_culling() {
        let mut section = ChunkSection::new(IVec3::ZERO);
        // Place 2 adjacent solid blocks of different types touching on X axis
        // (Stone and Dirt have different textures, so coplanar faces won't be greedy-merged)
        section.set_block(4, 4, 4, BlockType::Stone);
        section.set_block(5, 4, 4, BlockType::Dirt);

        let atlas = TextureAtlas::load_or_create(None);
        let mesh = greedy_mesh_section(&section, &atlas, &section);

        // 2 isolated blocks = 12 faces. With touching internal faces culled: 12 - 2 = 10 quads.
        assert_eq!(mesh.quad_count(), 10);

        // When identical blocks touch (e.g. 2 stone blocks), greedy meshing ALSO merges
        // the 4 coplanar face pairs (top, bottom, north, south) into 4 quads + 2 ends = 6 quads.
        let mut section_identical = ChunkSection::new(IVec3::ZERO);
        section_identical.set_block(4, 4, 4, BlockType::Stone);
        section_identical.set_block(5, 4, 4, BlockType::Stone);
        let mesh_identical = greedy_mesh_section(&section_identical, &atlas, &section_identical);
        assert_eq!(mesh_identical.quad_count(), 6);
    }

    #[test]
    fn test_async_meshing_dispatch() {
        let mesher = AsyncChunkMesher::new();
        let atlas = Arc::new(TextureAtlas::load_or_create(None));

        let mut section = ChunkSection::new(IVec3::new(1, 0, 1));
        section.set_block(0, 0, 0, BlockType::Dirt);

        mesher.request_mesh(section, atlas);

        // Wait up to 5 seconds for background worker thread to finish
        let start = std::time::Instant::now();
        let mut completed = Vec::new();
        while start.elapsed().as_millis() < 5000 {
            completed = mesher.poll_completed();
            if !completed.is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        assert_eq!(completed.len(), 1);
        let (pos, mesh) = &completed[0];
        assert_eq!(*pos, IVec3::new(1, 0, 1));
        assert_eq!(mesh.quad_count(), 6);
    }

    #[test]
    fn test_lighting_shading_and_face_separation() {
        struct LitReader {
            blocks: std::collections::HashMap<IVec3, BlockType>,
            lights: std::collections::HashMap<IVec3, u8>,
        }
        impl BlockReader for LitReader {
            fn get_block_at(&self, x: i32, y: i32, z: i32) -> BlockType {
                self.blocks.get(&IVec3::new(x, y, z)).copied().unwrap_or(BlockType::Air)
            }
            fn get_light_at(&self, x: i32, y: i32, z: i32) -> u8 {
                self.lights.get(&IVec3::new(x, y, z)).copied().unwrap_or(15)
            }
        }

        let mut section = ChunkSection::new(IVec3::ZERO);
        section.set_block(0, 0, 0, BlockType::Stone);
        section.set_block(1, 0, 0, BlockType::Stone);

        let mut reader = LitReader {
            blocks: std::collections::HashMap::new(),
            lights: std::collections::HashMap::new(),
        };
        reader.blocks.insert(IVec3::new(0, 0, 0), BlockType::Stone);
        reader.blocks.insert(IVec3::new(1, 0, 0), BlockType::Stone);

        // Different light levels above the two blocks:
        reader.lights.insert(IVec3::new(0, 1, 0), 15);
        reader.lights.insert(IVec3::new(1, 1, 0), 5);

        let atlas = TextureAtlas::load_or_create(None);
        let mesh = greedy_mesh_section(&section, &atlas, &reader);

        // Because light levels differ (15 vs 5), the top faces cannot be merged into 1 quad;
        // they must remain separate to represent the lighting gradient!
        let top_quads: Vec<_> = mesh
            .vertices
            .chunks_exact(4)
            .filter(|quad| quad[0].normal_idx == direction_to_normal_idx(Direction::Up))
            .collect();
        assert_eq!(top_quads.len(), 2, "Faces with different light levels should not merge");

        // The quad above block (0, 0, 0) has vertices with min x = 0.0
        // The quad above block (1, 0, 0) has vertices with max x = 2.0
        let quad_x0 = top_quads.iter().find(|q| q.iter().any(|v| v.x == 0.0)).unwrap();
        let quad_x1 = top_quads.iter().find(|q| q.iter().any(|v| v.x == 2.0)).unwrap();
        assert!(quad_x0[0].ao > quad_x1[0].ao, "Bright face should have higher shade value than dim face");
    }

    #[test]
    fn test_biome_tinting_grass_foliage_and_vines() {
        struct BiomeReader {
            blocks: std::collections::HashMap<IVec3, BlockType>,
            climate: (f32, f32),
        }
        impl BlockReader for BiomeReader {
            fn get_block_at(&self, x: i32, y: i32, z: i32) -> BlockType {
                self.blocks.get(&IVec3::new(x, y, z)).copied().unwrap_or(BlockType::Air)
            }
            fn get_biome_climate(&self, _x: i32, _z: i32) -> (f32, f32) {
                self.climate
            }
        }

        let mut section = ChunkSection::new(IVec3::ZERO);
        section.set_block(2, 2, 2, BlockType::GrassBlock);
        section.set_block(4, 4, 4, BlockType::OakLeaves);
        section.set_block(6, 6, 6, BlockType::Vine);
        section.set_block(8, 8, 8, BlockType::Stone);

        let mut reader = BiomeReader {
            blocks: std::collections::HashMap::new(),
            climate: (0.8, 0.4), // Plains
        };
        reader.blocks.insert(IVec3::new(2, 2, 2), BlockType::GrassBlock);
        reader.blocks.insert(IVec3::new(4, 4, 4), BlockType::OakLeaves);
        reader.blocks.insert(IVec3::new(6, 6, 6), BlockType::Vine);
        reader.blocks.insert(IVec3::new(8, 8, 8), BlockType::Stone);

        let atlas = TextureAtlas::load_or_create(None);
        let mesh = greedy_mesh_section(&section, &atlas, &reader);

        // Find Grass Block top vertices
        let grass_top_verts: Vec<_> = mesh.vertices.iter().filter(|v| {
            (v.y - 3.0).abs() < 1e-4 && (v.x >= 2.0 && v.x <= 3.0) && (v.z >= 2.0 && v.z <= 3.0)
                && v.normal_idx == direction_to_normal_idx(Direction::Up)
        }).collect();
        assert_eq!(grass_top_verts.len(), 4);
        // Grass top must have tinted green color (g > r and g > b, not 1.0 white)
        let gv = grass_top_verts[0];
        assert!(gv.g > gv.r && gv.g > gv.b, "Grass top must have green biome tint");
        assert!(gv.r < 0.95 || gv.b < 0.95, "Grass top should not be untinted white");

        // Find Oak Leaves vertices
        let leaf_verts: Vec<_> = mesh.vertices.iter().filter(|v| {
            (v.x >= 4.0 && v.x <= 5.0) && (v.y >= 4.0 && v.y <= 5.0) && (v.z >= 4.0 && v.z <= 5.0)
        }).collect();
        assert!(!leaf_verts.is_empty());
        let lv = leaf_verts[0];
        assert!(lv.g > lv.r && lv.g > lv.b, "Oak leaves must have foliage biome tint");

        // Find Vine vertices
        let vine_verts: Vec<_> = mesh.vertices.iter().filter(|v| {
            (v.x >= 6.0 && v.x <= 7.0) && (v.y >= 6.0 && v.y <= 7.0) && (v.z >= 6.0 && v.z <= 7.0)
        }).collect();
        assert!(!vine_verts.is_empty());
        let vv = vine_verts[0];
        assert!(vv.g > vv.r && vv.g > vv.b, "Vines must have foliage biome tint");

        // Stone vertices must remain untinted white [1.0, 1.0, 1.0]
        let stone_verts: Vec<_> = mesh.vertices.iter().filter(|v| {
            (v.x >= 8.0 && v.x <= 9.0) && (v.y >= 8.0 && v.y <= 9.0) && (v.z >= 8.0 && v.z <= 9.0)
        }).collect();
        assert!(!stone_verts.is_empty());
        for sv in stone_verts {
            assert!((sv.r - 1.0).abs() < 1e-4);
            assert!((sv.g - 1.0).abs() < 1e-4);
            assert!((sv.b - 1.0).abs() < 1e-4);
        }
    }
}
