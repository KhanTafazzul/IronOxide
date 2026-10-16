// REA Verified: net.minecraft.world.level.Level (Multi-Section Chunk Level)
// Manages multi-section 16x16x16 chunk grid, boundary-aware block placement/break,
// and adjacent section re-meshing triggers.

use std::collections::HashMap;
use glam::IVec3;
use crate::render::atlas::BlockType;
use crate::render::mesher::{BlockReader, ChunkSection, SECTION_SIZE};
use crate::world::fluids::{FluidEngine, FluidState, FluidType, FluidWorldAccess};
use crate::world::lighting::LightEngine;

use crate::world::weather::WeatherManager;

/// Multi-section voxel world implementing global BlockReader and LightEngine
#[derive(Clone, Debug)]
pub struct World {
    pub sections: HashMap<IVec3, ChunkSection>,
    pub lighting: LightEngine,
    pub fluids: FluidEngine,
    pub weather: WeatherManager,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            sections: HashMap::new(),
            lighting: LightEngine::new(),
            fluids: FluidEngine::new(),
            weather: WeatherManager::new(),
        }
    }

    /// Advances weather simulation by 1 game tick
    pub fn tick_weather(&mut self) {
        self.weather.tick();
    }

    /// Ticks cellular automaton fluid simulation and returns affected section coordinates
    pub fn tick_fluids(&mut self) -> Vec<IVec3> {
        let mut fluids = std::mem::take(&mut self.fluids);
        let affected = fluids.tick(self);
        self.fluids = fluids;
        affected
    }

    /// Retrieve or create section at section grid coordinate
    pub fn get_or_create_section(&mut self, pos: IVec3) -> &mut ChunkSection {
        self.sections.entry(pos).or_insert_with(|| ChunkSection::new(pos))
    }

    /// Read block at world coordinates (x, y, z)
    pub fn get_block(&self, world_pos: IVec3) -> BlockType {
        self.get_block_at(world_pos.x, world_pos.y, world_pos.z)
    }

    /// Place or break block at world coordinate (x, y, z).
    /// Returns the list of affected section coordinates that must be re-meshed
    /// (including adjacent neighbor sections if the block sits on a 16x16x16 boundary,
    /// or if lighting propagation affected neighbor sections).
    pub fn set_block_at(&mut self, world_pos: IVec3, block: BlockType) -> Vec<IVec3> {
        let old_block = self.get_block(world_pos);

        let size = SECTION_SIZE as i32;
        let sec_x = world_pos.x.div_euclid(size);
        let sec_y = world_pos.y.div_euclid(size);
        let sec_z = world_pos.z.div_euclid(size);

        let local_x = world_pos.x.rem_euclid(size) as usize;
        let local_y = world_pos.y.rem_euclid(size) as usize;
        let local_z = world_pos.z.rem_euclid(size) as usize;

        let sec_pos = IVec3::new(sec_x, sec_y, sec_z);
        let section = self.get_or_create_section(sec_pos);
        section.set_block(local_x, local_y, local_z, block);

        let mut affected = vec![sec_pos];

        // Boundary checks: if the block sits on an edge (0 or 15),
        // it affects the hidden-face culling and ambient occlusion of the adjacent section!
        if local_x == 0 {
            affected.push(IVec3::new(sec_x - 1, sec_y, sec_z));
        } else if local_x == SECTION_SIZE - 1 {
            affected.push(IVec3::new(sec_x + 1, sec_y, sec_z));
        }

        if local_y == 0 {
            affected.push(IVec3::new(sec_x, sec_y - 1, sec_z));
        } else if local_y == SECTION_SIZE - 1 {
            affected.push(IVec3::new(sec_x, sec_y + 1, sec_z));
        }

        if local_z == 0 {
            affected.push(IVec3::new(sec_x, sec_y, sec_z - 1));
        } else if local_z == SECTION_SIZE - 1 {
            affected.push(IVec3::new(sec_x, sec_y, sec_z + 1));
        }

        // Fluid engine state tracking
        if let Some(ft) = FluidType::from_block(block) {
            self.fluids.set_fluid_state(world_pos, FluidState::new_source(ft));
            self.fluids.schedule_update(world_pos);
        } else if old_block.is_fluid() && !block.is_fluid() {
            self.fluids.remove_fluid_state(world_pos);
        }

        // Lighting BFS updates
        let mut light_affected = if block.is_air() && !old_block.is_air() {
            self.lighting.on_block_broken(world_pos, old_block, &self.sections)
        } else if !block.is_air() {
            self.lighting.on_block_placed(world_pos, block, &self.sections)
        } else {
            Vec::new()
        };
        affected.append(&mut light_affected);

        affected.sort_by_key(|p| (p.x, p.y, p.z));
        affected.dedup();
        affected
    }
}

impl FluidWorldAccess for World {
    fn set_block_at(&mut self, pos: IVec3, block: BlockType) -> Vec<IVec3> {
        self.set_block_at(pos, block)
    }
}

impl BlockReader for HashMap<IVec3, ChunkSection> {
    #[inline]
    fn get_block_at(&self, x: i32, y: i32, z: i32) -> BlockType {
        let size = SECTION_SIZE as i32;
        let sec_pos = IVec3::new(x.div_euclid(size), y.div_euclid(size), z.div_euclid(size));
        if let Some(sec) = self.get(&sec_pos) {
            sec.get_block(
                x.rem_euclid(size) as usize,
                y.rem_euclid(size) as usize,
                z.rem_euclid(size) as usize,
            )
        } else {
            BlockType::Air
        }
    }
}

impl BlockReader for World {
    #[inline]
    fn get_block_at(&self, x: i32, y: i32, z: i32) -> BlockType {
        self.sections.get_block_at(x, y, z)
    }

    #[inline]
    fn get_light_at(&self, x: i32, y: i32, z: i32) -> u8 {
        self.lighting.get_combined_light(IVec3::new(x, y, z))
    }

    #[inline]
    fn get_fluid_level(&self, x: i32, y: i32, z: i32) -> Option<u8> {
        let pos = IVec3::new(x, y, z);
        if let Some(state) = self.fluids.get_fluid_state(pos) {
            Some(state.level)
        } else {
            let b = self.get_block_at(x, y, z);
            if b.is_fluid() {
                Some(8)
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boundary_remesh_detection() {
        let mut world = World::new();

        // Internal block at (5, 5, 5) -> only affects section (0, 0, 0)
        let affected_internal = world.set_block_at(IVec3::new(5, 5, 5), BlockType::Stone);
        assert_eq!(affected_internal, vec![IVec3::ZERO]);

        // Boundary block at x = 0 (0, 5, 5) -> affects section (0, 0, 0) and (-1, 0, 0)
        let affected_x_min = world.set_block_at(IVec3::new(0, 5, 5), BlockType::Dirt);
        assert!(affected_x_min.contains(&IVec3::ZERO));
        assert!(affected_x_min.contains(&IVec3::new(-1, 0, 0)));

        // Boundary block at x = 15 (15, 5, 5) -> affects section (0, 0, 0) and (1, 0, 0)
        let affected_x_max = world.set_block_at(IVec3::new(15, 5, 5), BlockType::Dirt);
        assert!(affected_x_max.contains(&IVec3::ZERO));
        assert!(affected_x_max.contains(&IVec3::new(1, 0, 0)));

        // Boundary corner block at (0, 0, 0) -> affects (-1, 0, 0), (0, -1, 0), (0, 0, -1), and (0, 0, 0)
        let affected_corner = world.set_block_at(IVec3::new(0, 0, 0), BlockType::Cobblestone);
        assert!(affected_corner.contains(&IVec3::ZERO));
        assert!(affected_corner.contains(&IVec3::new(-1, 0, 0)));
        assert!(affected_corner.contains(&IVec3::new(0, -1, 0)));
        assert!(affected_corner.contains(&IVec3::new(0, 0, -1)));
    }
}
