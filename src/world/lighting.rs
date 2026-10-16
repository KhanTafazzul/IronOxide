// REA Verified: net.minecraft.world.level.lighting.BlockLightEngine (mapped: edz.class)
// REA Verified: net.minecraft.world.level.lighting.SkyLightEngine (mapped: eek.class)
// REA Verified: net.minecraft.world.level.lighting.LightEngine (mapped: eei.class)
//
// Dual 4-bit nibble light storage (Block Light 0-15 & Sky Light 0-15 per voxel).
// Breadth-First Search (BFS) flood fill for point light sources (Torch = 14, Lava = 15).
// - Power decreases by 1 for every step through transparent voxels (Air, Glass, Water).
// - Light immediately drops to 0 when entering opaque blocks.
// BFS removal propagation: retraction of obsolete light followed by re-propagation of remaining light.

use std::collections::{HashMap, VecDeque};
use glam::IVec3;
use crate::render::atlas::BlockType;
use crate::render::mesher::{BlockReader, SECTION_SIZE, SECTION_VOLUME};

pub const LIGHT_SECTION_BYTES: usize = SECTION_VOLUME / 2; // 2048 bytes for 4096 4-bit nibbles

/// 4-bit Nibble Array storing 4096 values (0 to 15) in a 2048-byte buffer.
/// REA Verified: net.minecraft.world.level.chunk.DataLayer (mapped: dzn.class)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NibbleArray {
    pub data: [u8; LIGHT_SECTION_BYTES],
}

impl Default for NibbleArray {
    fn default() -> Self {
        Self::new()
    }
}

impl NibbleArray {
    pub fn new() -> Self {
        Self {
            data: [0u8; LIGHT_SECTION_BYTES],
        }
    }

    pub fn with_fill(val: u8) -> Self {
        let nib = val & 0x0F;
        let byte = (nib << 4) | nib;
        Self {
            data: [byte; LIGHT_SECTION_BYTES],
        }
    }

    #[inline(always)]
    pub fn index(x: usize, y: usize, z: usize) -> usize {
        x + z * SECTION_SIZE + y * (SECTION_SIZE * SECTION_SIZE)
    }

    #[inline(always)]
    pub fn get(&self, x: usize, y: usize, z: usize) -> u8 {
        if x < SECTION_SIZE && y < SECTION_SIZE && z < SECTION_SIZE {
            self.get_index(Self::index(x, y, z))
        } else {
            0
        }
    }

    #[inline(always)]
    pub fn get_index(&self, idx: usize) -> u8 {
        if idx >= SECTION_VOLUME {
            return 0;
        }
        let byte = self.data[idx >> 1];
        if (idx & 1) == 0 {
            byte & 0x0F
        } else {
            (byte >> 4) & 0x0F
        }
    }

    #[inline(always)]
    pub fn set(&mut self, x: usize, y: usize, z: usize, val: u8) {
        if x < SECTION_SIZE && y < SECTION_SIZE && z < SECTION_SIZE {
            self.set_index(Self::index(x, y, z), val);
        }
    }

    #[inline(always)]
    pub fn set_index(&mut self, idx: usize, val: u8) {
        if idx >= SECTION_VOLUME {
            return;
        }
        let byte_idx = idx >> 1;
        let v = val & 0x0F;
        if (idx & 1) == 0 {
            self.data[byte_idx] = (self.data[byte_idx] & 0xF0) | v;
        } else {
            self.data[byte_idx] = (self.data[byte_idx] & 0x0F) | (v << 4);
        }
    }

    #[inline(always)]
    pub fn raw_data(&self) -> &[u8] {
        &self.data
    }

    #[inline(always)]
    pub fn raw_data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

/// Dual 4-bit nibble light storage per voxel (Block Light 0-15 and Sky Light 0-15)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionLight {
    pub block_light: NibbleArray,
    pub sky_light: NibbleArray,
}

pub type DualLightStorage = SectionLight;

impl Default for SectionLight {
    fn default() -> Self {
        Self::new()
    }
}

impl SectionLight {
    pub fn new() -> Self {
        Self {
            block_light: NibbleArray::new(),
            sky_light: NibbleArray::new(),
        }
    }

    pub fn with_full_sky() -> Self {
        Self {
            block_light: NibbleArray::new(),
            sky_light: NibbleArray::with_fill(15),
        }
    }

    #[inline(always)]
    pub fn get_block_light(&self, x: usize, y: usize, z: usize) -> u8 {
        self.block_light.get(x, y, z)
    }

    #[inline(always)]
    pub fn set_block_light(&mut self, x: usize, y: usize, z: usize, val: u8) {
        self.block_light.set(x, y, z, val);
    }

    #[inline(always)]
    pub fn get_sky_light(&self, x: usize, y: usize, z: usize) -> u8 {
        self.sky_light.get(x, y, z)
    }

    #[inline(always)]
    pub fn set_sky_light(&mut self, x: usize, y: usize, z: usize, val: u8) {
        self.sky_light.set(x, y, z, val);
    }

    #[inline(always)]
    pub fn get_packed(&self, x: usize, y: usize, z: usize) -> u8 {
        let b = self.get_block_light(x, y, z);
        let s = self.get_sky_light(x, y, z);
        (s << 4) | (b & 0x0F)
    }

    #[inline(always)]
    pub fn get_combined_light(&self, x: usize, y: usize, z: usize) -> u8 {
        self.get_block_light(x, y, z).max(self.get_sky_light(x, y, z))
    }
}

/// Six cardinal 3D neighbor offsets
pub const NEIGHBOR_DIRECTIONS: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];

/// Breadth-First Search Voxel Light Engine managing multi-section 3D lighting
#[derive(Clone, Debug, Default)]
pub struct LightEngine {
    pub sections: HashMap<IVec3, SectionLight>,
}

impl LightEngine {
    pub fn new() -> Self {
        Self {
            sections: HashMap::new(),
        }
    }

    /// Convert world position to section position and local section coordinates
    #[inline(always)]
    pub fn world_to_section_and_local(pos: IVec3) -> (IVec3, usize, usize, usize) {
        let size = SECTION_SIZE as i32;
        let sec_pos = IVec3::new(
            pos.x.div_euclid(size),
            pos.y.div_euclid(size),
            pos.z.div_euclid(size),
        );
        let lx = pos.x.rem_euclid(size) as usize;
        let ly = pos.y.rem_euclid(size) as usize;
        let lz = pos.z.rem_euclid(size) as usize;
        (sec_pos, lx, ly, lz)
    }

    /// Retrieve or initialize section light storage
    pub fn get_or_create_section(&mut self, sec_pos: IVec3) -> &mut SectionLight {
        self.sections.entry(sec_pos).or_insert_with(SectionLight::new)
    }

    /// Get block light level (0-15) at world coordinate
    pub fn get_block_light(&self, pos: IVec3) -> u8 {
        let (sec_pos, lx, ly, lz) = Self::world_to_section_and_local(pos);
        self.sections
            .get(&sec_pos)
            .map(|s| s.get_block_light(lx, ly, lz))
            .unwrap_or(0)
    }

    /// Set block light level (0-15) at world coordinate
    pub fn set_block_light(&mut self, pos: IVec3, val: u8) {
        let (sec_pos, lx, ly, lz) = Self::world_to_section_and_local(pos);
        let section = self.get_or_create_section(sec_pos);
        section.set_block_light(lx, ly, lz, val);
    }

    /// Get sky light level (0-15) at world coordinate
    pub fn get_sky_light(&self, pos: IVec3) -> u8 {
        let (sec_pos, lx, ly, lz) = Self::world_to_section_and_local(pos);
        self.sections
            .get(&sec_pos)
            .map(|s| s.get_sky_light(lx, ly, lz))
            .unwrap_or(0)
    }

    /// Set sky light level (0-15) at world coordinate
    pub fn set_sky_light(&mut self, pos: IVec3, val: u8) {
        let (sec_pos, lx, ly, lz) = Self::world_to_section_and_local(pos);
        let section = self.get_or_create_section(sec_pos);
        section.set_sky_light(lx, ly, lz, val);
    }

    /// Combined light level = max(Block Light, Sky Light)
    pub fn get_combined_light(&self, pos: IVec3) -> u8 {
        self.get_block_light(pos).max(self.get_sky_light(pos))
    }

    /// Packed dual 4-bit nibbles: `(sky_light << 4) | (block_light & 0x0F)`
    pub fn get_packed_light(&self, pos: IVec3) -> u8 {
        let b = self.get_block_light(pos);
        let s = self.get_sky_light(pos);
        (s << 4) | (b & 0x0F)
    }

    /// REA Verified: net.minecraft.world.level.lighting.BlockLightEngine (edz.a(long, long, int))
    /// Breadth-First Search (BFS) flood fill for point light sources (Torch = 14, Lava = 15).
    /// - Power decreases by 1 for every step through transparent voxels (Air, Glass, Water).
    /// - Light immediately drops to 0 when entering opaque blocks.
    /// Returns the list of affected section coordinates for re-meshing.
    pub fn add_light_source(
        &mut self,
        pos: IVec3,
        emission: u8,
        world: &dyn BlockReader,
    ) -> Vec<IVec3> {
        let mut affected_sections = Vec::new();
        let mut increase_queue = VecDeque::new();

        self.set_block_light(pos, emission);
        let (sec_pos, _, _, _) = Self::world_to_section_and_local(pos);
        affected_sections.push(sec_pos);
        increase_queue.push_back((pos, emission));

        while let Some((curr_pos, curr_level)) = increase_queue.pop_front() {
            if curr_level <= 1 {
                continue;
            }
            let next_level = curr_level - 1;

            for &dir in &NEIGHBOR_DIRECTIONS {
                let neighbor = curr_pos + dir;
                let block = world.get_block_at(neighbor.x, neighbor.y, neighbor.z);

                // Light immediately drops to 0 when entering opaque blocks
                if block.is_opaque() {
                    continue;
                }

                // Transparent voxels (Air, Glass, Water, etc.)
                let current_val = self.get_block_light(neighbor);
                if current_val < next_level {
                    self.set_block_light(neighbor, next_level);
                    let (sec, _, _, _) = Self::world_to_section_and_local(neighbor);
                    affected_sections.push(sec);
                    increase_queue.push_back((neighbor, next_level));
                }
            }
        }

        affected_sections.sort_by_key(|p| (p.x, p.y, p.z));
        affected_sections.dedup();
        affected_sections
    }

    /// REA Verified: net.minecraft.world.level.lighting.BlockLightEngine (edz.a(long, long))
    /// Removal propagation: when a light source is destroyed, queue BFS retraction and re-propagate neighboring light.
    /// Returns the list of affected section coordinates for re-meshing.
    pub fn remove_light_source(
        &mut self,
        pos: IVec3,
        old_level: u8,
        world: &dyn BlockReader,
    ) -> Vec<IVec3> {
        let mut affected_sections = Vec::new();
        let mut decrease_queue = VecDeque::new();
        let mut increase_queue = VecDeque::new();

        let current_val = self.get_block_light(pos);
        let removal_level = if old_level > 0 { old_level } else { current_val };
        if removal_level == 0 {
            return affected_sections;
        }

        self.set_block_light(pos, 0);
        let (sec_pos, _, _, _) = Self::world_to_section_and_local(pos);
        affected_sections.push(sec_pos);
        decrease_queue.push_back((pos, removal_level));

        // 1. BFS Decrease propagation: retract light that originated from this source
        while let Some((curr_pos, curr_val)) = decrease_queue.pop_front() {
            for &dir in &NEIGHBOR_DIRECTIONS {
                let neighbor = curr_pos + dir;
                let neighbor_val = self.get_block_light(neighbor);

                if neighbor_val != 0 && neighbor_val < curr_val {
                    // This neighbor received its light from curr_pos; reset to 0 and propagate
                    self.set_block_light(neighbor, 0);
                    let (sec, _, _, _) = Self::world_to_section_and_local(neighbor);
                    affected_sections.push(sec);
                    decrease_queue.push_back((neighbor, neighbor_val));
                } else if neighbor_val >= curr_val {
                    // Neighbor has equal or higher light from an alternate light source; queue for re-propagation
                    increase_queue.push_back((neighbor, neighbor_val));
                }
            }
        }

        // 2. BFS Re-propagation: fill back into retracted empty areas from neighboring light sources
        while let Some((curr_pos, curr_val)) = increase_queue.pop_front() {
            if curr_val <= 1 {
                continue;
            }
            let next_val = curr_val - 1;

            for &dir in &NEIGHBOR_DIRECTIONS {
                let neighbor = curr_pos + dir;
                let block = world.get_block_at(neighbor.x, neighbor.y, neighbor.z);
                if block.is_opaque() {
                    continue;
                }

                let neighbor_val = self.get_block_light(neighbor);
                if neighbor_val < next_val {
                    self.set_block_light(neighbor, next_val);
                    let (sec, _, _, _) = Self::world_to_section_and_local(neighbor);
                    affected_sections.push(sec);
                    increase_queue.push_back((neighbor, next_val));
                }
            }
        }

        affected_sections.sort_by_key(|p| (p.x, p.y, p.z));
        affected_sections.dedup();
        affected_sections
    }

    /// REA Verified: net.minecraft.world.level.lighting.SkyLightEngine (eek.b(int, int))
    /// Direct column sky light propagation down to the highest opaque block,
    /// followed by horizontal flood fill through transparent blocks.
    pub fn update_skylight_column(
        &mut self,
        x: i32,
        z: i32,
        max_y: i32,
        min_y: i32,
        world: &dyn BlockReader,
    ) -> Vec<IVec3> {
        let mut affected = Vec::new();
        let mut horizontal_queue = VecDeque::new();

        // 1. Direct vertical downward propagation (power = 15 through transparent voxels)
        let mut y = max_y;
        while y >= min_y {
            let pos = IVec3::new(x, y, z);
            let block = world.get_block_at(x, y, z);

            if block.is_opaque() {
                self.set_sky_light(pos, 0);
                break;
            }

            self.set_sky_light(pos, 15);
            let (sec, _, _, _) = Self::world_to_section_and_local(pos);
            affected.push(sec);
            horizontal_queue.push_back((pos, 15));
            y -= 1;
        }

        // 2. Horizontal & vertical spread through transparent neighbors
        while let Some((curr_pos, curr_val)) = horizontal_queue.pop_front() {
            if curr_val <= 1 {
                continue;
            }
            let next_val = curr_val - 1;

            for &dir in &NEIGHBOR_DIRECTIONS {
                let neighbor = curr_pos + dir;
                let block = world.get_block_at(neighbor.x, neighbor.y, neighbor.z);
                if block.is_opaque() {
                    continue;
                }

                let current_sky = self.get_sky_light(neighbor);
                if current_sky < next_val {
                    self.set_sky_light(neighbor, next_val);
                    let (sec, _, _, _) = Self::world_to_section_and_local(neighbor);
                    affected.push(sec);
                    horizontal_queue.push_back((neighbor, next_val));
                }
            }
        }

        affected.sort_by_key(|p| (p.x, p.y, p.z));
        affected.dedup();
        affected
    }

    /// React to a block placed in the world (handles emitters, occlusion of light, and skylight blockage)
    pub fn on_block_placed(
        &mut self,
        pos: IVec3,
        block: BlockType,
        world: &dyn BlockReader,
    ) -> Vec<IVec3> {
        let mut affected = Vec::new();
        let emission = block.light_emission();

        if emission > 0 {
            let mut aff = self.add_light_source(pos, emission, world);
            affected.append(&mut aff);
        } else if block.is_opaque() {
            let old_block_light = self.get_block_light(pos);
            if old_block_light > 0 {
                let mut aff = self.remove_light_source(pos, old_block_light, world);
                affected.append(&mut aff);
            }
            let old_sky_light = self.get_sky_light(pos);
            if old_sky_light > 0 {
                self.set_sky_light(pos, 0);
                let (sec, _, _, _) = Self::world_to_section_and_local(pos);
                affected.push(sec);
            }
        }

        affected.sort_by_key(|p| (p.x, p.y, p.z));
        affected.dedup();
        affected
    }

    /// React to a block broken in the world (handles removing emitters and unblocking light paths)
    pub fn on_block_broken(
        &mut self,
        pos: IVec3,
        old_block: BlockType,
        world: &dyn BlockReader,
    ) -> Vec<IVec3> {
        let mut affected = Vec::new();
        let emission = old_block.light_emission();

        if emission > 0 {
            let mut aff = self.remove_light_source(pos, emission, world);
            affected.append(&mut aff);
        } else if old_block.is_opaque() {
            // Block is now air/transparent; re-propagate incoming light from neighboring light sources
            let mut max_neighbor_light = 0u8;
            let mut best_neighbor = pos;

            for &dir in &NEIGHBOR_DIRECTIONS {
                let neighbor = pos + dir;
                let nl = self.get_block_light(neighbor);
                if nl > max_neighbor_light {
                    max_neighbor_light = nl;
                    best_neighbor = neighbor;
                }
            }

            if max_neighbor_light > 1 {
                let mut aff = self.add_light_source(best_neighbor, max_neighbor_light, world);
                affected.append(&mut aff);
            }
        }

        affected.sort_by_key(|p| (p.x, p.y, p.z));
        affected.dedup();
        affected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestWorld {
        blocks: HashMap<IVec3, BlockType>,
    }

    impl TestWorld {
        fn new() -> Self {
            Self {
                blocks: HashMap::new(),
            }
        }

        fn set(&mut self, pos: IVec3, block: BlockType) {
            self.blocks.insert(pos, block);
        }
    }

    impl BlockReader for TestWorld {
        fn get_block_at(&self, x: i32, y: i32, z: i32) -> BlockType {
            self.blocks.get(&IVec3::new(x, y, z)).copied().unwrap_or(BlockType::Air)
        }
    }

    #[test]
    fn test_nibble_array_get_set() {
        let mut nibbles = NibbleArray::new();
        assert_eq!(nibbles.get(0, 0, 0), 0);
        assert_eq!(nibbles.get(15, 15, 15), 0);

        nibbles.set(0, 0, 0, 14);
        nibbles.set(1, 0, 0, 15);
        nibbles.set(5, 7, 9, 7);

        assert_eq!(nibbles.get(0, 0, 0), 14);
        assert_eq!(nibbles.get(1, 0, 0), 15);
        assert_eq!(nibbles.get(5, 7, 9), 7);
        // Untouched should remain 0
        assert_eq!(nibbles.get(2, 0, 0), 0);
    }

    #[test]
    fn test_dual_nibble_storage_packed() {
        let mut section = SectionLight::new();
        section.set_block_light(3, 4, 5, 14);
        section.set_sky_light(3, 4, 5, 15);

        assert_eq!(section.get_block_light(3, 4, 5), 14);
        assert_eq!(section.get_sky_light(3, 4, 5), 15);
        assert_eq!(section.get_combined_light(3, 4, 5), 15);
        // Packed: (15 << 4) | 14 = 0xFE = 254
        assert_eq!(section.get_packed(3, 4, 5), (15 << 4) | 14);
    }

    #[test]
    fn test_torch_bfs_flood_fill() {
        let mut engine = LightEngine::new();
        let world = TestWorld::new();

        let torch_pos = IVec3::new(5, 5, 5);
        engine.add_light_source(torch_pos, 14, &world);

        // Torch pos emits 14
        assert_eq!(engine.get_block_light(torch_pos), 14);
        // Distance 1: 13
        assert_eq!(engine.get_block_light(IVec3::new(6, 5, 5)), 13);
        assert_eq!(engine.get_block_light(IVec3::new(4, 5, 5)), 13);
        assert_eq!(engine.get_block_light(IVec3::new(5, 6, 5)), 13);
        // Distance 2: 12
        assert_eq!(engine.get_block_light(IVec3::new(7, 5, 5)), 12);
        // Distance 13: 1
        assert_eq!(engine.get_block_light(IVec3::new(18, 5, 5)), 1);
        // Distance 14: 0
        assert_eq!(engine.get_block_light(IVec3::new(19, 5, 5)), 0);
    }

    #[test]
    fn test_lava_bfs_flood_fill() {
        let mut engine = LightEngine::new();
        let world = TestWorld::new();

        let lava_pos = IVec3::new(10, 10, 10);
        engine.add_light_source(lava_pos, 15, &world);

        assert_eq!(engine.get_block_light(lava_pos), 15);
        assert_eq!(engine.get_block_light(IVec3::new(11, 10, 10)), 14);
        assert_eq!(engine.get_block_light(IVec3::new(12, 10, 10)), 13);
        // Distance 14: 1
        assert_eq!(engine.get_block_light(IVec3::new(24, 10, 10)), 1);
        // Distance 15: 0
        assert_eq!(engine.get_block_light(IVec3::new(25, 10, 10)), 0);
    }

    #[test]
    fn test_light_drops_to_zero_at_opaque_blocks() {
        let mut engine = LightEngine::new();
        let mut world = TestWorld::new();

        let torch_pos = IVec3::new(0, 0, 0);
        // Place stone wall at x = 2 spanning y and z so light cannot wrap around
        for y in -20..=20 {
            for z in -20..=20 {
                world.set(IVec3::new(2, y, z), BlockType::Stone);
            }
        }

        engine.add_light_source(torch_pos, 14, &world);

        assert_eq!(engine.get_block_light(IVec3::new(0, 0, 0)), 14);
        assert_eq!(engine.get_block_light(IVec3::new(1, 0, 0)), 13);
        // Opaque block: light does not penetrate
        assert_eq!(engine.get_block_light(IVec3::new(2, 0, 0)), 0);
        // Behind the opaque wall: light is completely blocked (0)
        assert_eq!(engine.get_block_light(IVec3::new(3, 0, 0)), 0);
    }

    #[test]
    fn test_removal_propagation_and_repropagation() {
        let mut engine = LightEngine::new();
        let world = TestWorld::new();

        let torch1_pos = IVec3::new(0, 0, 0);
        let torch2_pos = IVec3::new(4, 0, 0);

        engine.add_light_source(torch1_pos, 14, &world);
        engine.add_light_source(torch2_pos, 14, &world);

        // Midpoint at x = 2 has light from both (14 - 2 = 12)
        assert_eq!(engine.get_block_light(IVec3::new(2, 0, 0)), 12);

        // Remove torch 1
        engine.remove_light_source(torch1_pos, 14, &world);

        // Torch 1 position should be at 10 from torch 2 (14 - 4 = 10) due to re-propagation!
        assert_eq!(engine.get_block_light(torch1_pos), 10);
        assert_eq!(engine.get_block_light(IVec3::new(1, 0, 0)), 11);
        assert_eq!(engine.get_block_light(IVec3::new(2, 0, 0)), 12);
        assert_eq!(engine.get_block_light(torch2_pos), 14);

        // Now remove torch 2
        engine.remove_light_source(torch2_pos, 14, &world);
        // All light completely gone
        assert_eq!(engine.get_block_light(torch1_pos), 0);
        assert_eq!(engine.get_block_light(torch2_pos), 0);
        assert_eq!(engine.get_block_light(IVec3::new(2, 0, 0)), 0);
    }

    #[test]
    fn test_skylight_column_and_attenuation() {
        let mut engine = LightEngine::new();
        let mut world = TestWorld::new();

        // Roof at y = 10 spanning x and z so skylight cannot wrap underneath
        for x in -20..=20 {
            for z in -20..=20 {
                world.set(IVec3::new(x, 10, z), BlockType::Stone);
            }
        }

        engine.update_skylight_column(0, 0, 20, 0, &world);

        // Direct sunlight from 20 down to 11 is 15
        assert_eq!(engine.get_sky_light(IVec3::new(0, 20, 0)), 15);
        assert_eq!(engine.get_sky_light(IVec3::new(0, 15, 0)), 15);
        assert_eq!(engine.get_sky_light(IVec3::new(0, 11, 0)), 15);
        // Stone roof at y = 10 blocks sky light
        assert_eq!(engine.get_sky_light(IVec3::new(0, 10, 0)), 0);
        assert_eq!(engine.get_sky_light(IVec3::new(0, 9, 0)), 0);

        // Horizontal attenuation at y = 15:
        // x = 0 is 15, x = 1 is 14, x = 2 is 13
        assert_eq!(engine.get_sky_light(IVec3::new(1, 15, 0)), 14);
        assert_eq!(engine.get_sky_light(IVec3::new(2, 15, 0)), 13);
    }
}
