// REA Verified: net.minecraft.world.level.material.FlowingFluid (mapped: eep.class)
// REA Verified: net.minecraft.world.level.material.WaterFluid (mapped: eeu.class)
// REA Verified: net.minecraft.world.level.material.LavaFluid (mapped: eex.class)
// REA Verified: net.minecraft.world.level.block.LiquidBlock (mapped: dbf.class)
//
// Cellular Automaton Fluid Engine:
// - Fluid state tracking: level (1..=8), falling (bool).
// - Source block = Level 8 (falling = false).
// - Downward flow: if block below is Air or permeable, falling water retains Level 8 with falling = true.
// - Horizontal propagation: drops by 1 level per block traveled (Water spreads up to 7 blocks; Lava 3 blocks in Overworld).
// - Fluid collision interactions:
//   * Water contacting stationary Lava source -> converts Lava to Obsidian (BlockType::Obsidian).
//   * Water contacting flowing Lava -> converts Lava to Cobblestone (BlockType::Cobblestone).
//   * Lava contacting Water -> converts to Cobblestone.
// - Flow direction vectors: gradient of fluid heights and downward pull for entity swimming push forces.

use std::collections::{HashMap, VecDeque};
use glam::{IVec3, Vec3};
use crate::render::atlas::BlockType;
use crate::render::mesher::{BlockReader, SECTION_SIZE};

pub const WATER_MAX_SPREAD: u8 = 7;
pub const LAVA_MAX_SPREAD: u8 = 3;

/// Supported fluid types in Minecraft
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FluidType {
    Water,
    Lava,
}

impl FluidType {
    pub fn to_block(&self) -> BlockType {
        match self {
            FluidType::Water => BlockType::Water,
            FluidType::Lava => BlockType::Lava,
        }
    }

    pub fn from_block(block: BlockType) -> Option<Self> {
        match block {
            BlockType::Water => Some(FluidType::Water),
            BlockType::Lava => Some(FluidType::Lava),
            _ => None,
        }
    }

    pub fn max_spread_distance(&self) -> u8 {
        match self {
            FluidType::Water => WATER_MAX_SPREAD,
            FluidType::Lava => LAVA_MAX_SPREAD,
        }
    }

    pub fn drop_off(&self) -> u8 {
        match self {
            FluidType::Water => 1,
            FluidType::Lava => 2, // In Overworld, lava level drops by 2 per block
        }
    }
}

/// Dynamic fluid state per voxel:
/// - `level`: 1..=8 (8 = source block or falling column)
/// - `falling`: true if flowing vertically downward
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FluidState {
    pub fluid_type: FluidType,
    pub level: u8,
    pub falling: bool,
}

impl FluidState {
    pub fn new_source(fluid_type: FluidType) -> Self {
        Self {
            fluid_type,
            level: 8,
            falling: false,
        }
    }

    pub fn new_falling(fluid_type: FluidType) -> Self {
        Self {
            fluid_type,
            level: 8,
            falling: true,
        }
    }

    pub fn new_flowing(fluid_type: FluidType, level: u8) -> Self {
        Self {
            fluid_type,
            level: level.clamp(1, 8),
            falling: false,
        }
    }

    #[inline]
    pub fn is_source(&self) -> bool {
        self.level == 8 && !self.falling
    }

    /// Height multiplier in range (0.0, 1.0] proportional to fluid level (h = level / 8.0)
    #[inline]
    pub fn height(&self) -> f32 {
        if self.falling {
            1.0
        } else {
            (self.level as f32) / 8.0
        }
    }
}

/// Abstract mutable world access for fluid simulation
pub trait FluidWorldAccess: BlockReader {
    fn set_block_at(&mut self, pos: IVec3, block: BlockType) -> Vec<IVec3>;
}

/// Cellular Automaton Fluid Simulation Engine
#[derive(Debug, Clone, Default)]
pub struct FluidEngine {
    pub states: HashMap<IVec3, FluidState>,
    pub pending_updates: VecDeque<IVec3>,
}

impl FluidEngine {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            pending_updates: VecDeque::new(),
        }
    }

    #[inline]
    pub fn get_fluid_state(&self, pos: IVec3) -> Option<FluidState> {
        self.states.get(&pos).copied()
    }

    #[inline]
    pub fn set_fluid_state(&mut self, pos: IVec3, state: FluidState) {
        self.states.insert(pos, state);
    }

    #[inline]
    pub fn remove_fluid_state(&mut self, pos: IVec3) -> Option<FluidState> {
        self.states.remove(&pos)
    }

    /// Schedule a voxel position for fluid simulation update
    pub fn schedule_update(&mut self, pos: IVec3) {
        if !self.pending_updates.contains(&pos) {
            self.pending_updates.push_back(pos);
        }
    }

    /// Handle fluid placement (e.g. from bucket or world generation)
    pub fn on_fluid_placed(
        &mut self,
        pos: IVec3,
        fluid_type: FluidType,
        world: &mut dyn FluidWorldAccess,
    ) -> Vec<IVec3> {
        let state = FluidState::new_source(fluid_type);
        self.set_fluid_state(pos, state);
        let mut affected = world.set_block_at(pos, fluid_type.to_block());

        self.schedule_update(pos);
        let mut sim_affected = self.propagate_at(pos, world);
        affected.append(&mut sim_affected);

        affected.sort_by_key(|p| (p.x, p.y, p.z));
        affected.dedup();
        affected
    }

    /// Handle fluid removal (e.g. scooped by bucket or replaced by solid block)
    pub fn on_fluid_removed(&mut self, pos: IVec3, world: &mut dyn FluidWorldAccess) -> Vec<IVec3> {
        self.remove_fluid_state(pos);
        let mut affected = world.set_block_at(pos, BlockType::Air);

        // Schedule neighbors to drain or retract
        let dirs = [
            IVec3::new(1, 0, 0),
            IVec3::new(-1, 0, 0),
            IVec3::new(0, 1, 0),
            IVec3::new(0, -1, 0),
            IVec3::new(0, 0, 1),
            IVec3::new(0, 0, -1),
        ];
        for d in dirs {
            let neighbor = pos + d;
            if self.states.contains_key(&neighbor) {
                self.schedule_update(neighbor);
            }
        }

        let mut sim_affected = self.tick(world);
        affected.append(&mut sim_affected);

        affected.sort_by_key(|p| (p.x, p.y, p.z));
        affected.dedup();
        affected
    }

    /// Check fluid collision interactions between Water and Lava matching vanilla:
    /// - Water contacting stationary Lava source -> converts Lava to Obsidian.
    /// - Water contacting flowing Lava -> converts Lava to Cobblestone.
    /// - Lava contacting Water -> converts Lava to Cobblestone.
    /// Returns true if a collision reaction took place and converted the block.
    pub fn check_fluid_collision(
        &mut self,
        pos: IVec3,
        incoming_fluid: FluidType,
        world: &mut dyn FluidWorldAccess,
        affected_sections: &mut Vec<IVec3>,
    ) -> bool {
        let existing_block = world.get_block_at(pos.x, pos.y, pos.z);
        let existing_fluid = self.get_fluid_state(pos);

        match (incoming_fluid, existing_block, existing_fluid) {
            // Water entering Lava:
            (FluidType::Water, BlockType::Lava, Some(lava_state)) => {
                let result_block = if lava_state.is_source() {
                    BlockType::Obsidian // Water hits stationary Lava source -> Obsidian
                } else {
                    BlockType::Cobblestone // Water hits flowing Lava -> Cobblestone
                };
                self.remove_fluid_state(pos);
                let mut aff = world.set_block_at(pos, result_block);
                affected_sections.append(&mut aff);
                true
            }
            // Lava entering Water:
            (FluidType::Lava, BlockType::Water, Some(_)) => {
                // Lava flows into Water -> converts to Cobblestone
                self.remove_fluid_state(pos);
                let mut aff = world.set_block_at(pos, BlockType::Cobblestone);
                affected_sections.append(&mut aff);
                true
            }
            _ => false,
        }
    }

    /// Execute cellular automaton propagation for fluid at pos
    pub fn propagate_at(
        &mut self,
        pos: IVec3,
        world: &mut dyn FluidWorldAccess,
    ) -> Vec<IVec3> {
        let mut affected = Vec::new();
        let cur_state = match self.get_fluid_state(pos) {
            Some(s) => s,
            None => return affected,
        };

        let below_pos = pos + IVec3::new(0, -1, 0);
        let below_block = world.get_block_at(below_pos.x, below_pos.y, below_pos.z);

        // 1. Check collision interaction with block below
        if self.check_fluid_collision(below_pos, cur_state.fluid_type, world, &mut affected) {
            return affected;
        }

        // 2. Downward Flow: If block below is Air or permeable, falling water retains Level 8 with falling = true
        let can_flow_down = below_block.is_air() || (below_block.is_fluid() && below_block != cur_state.fluid_type.to_block());

        if can_flow_down {
            let falling_state = FluidState::new_falling(cur_state.fluid_type);
            self.set_fluid_state(below_pos, falling_state);
            let mut aff = world.set_block_at(below_pos, cur_state.fluid_type.to_block());
            affected.append(&mut aff);
            self.schedule_update(below_pos);

            // In vanilla, when fluid flows straight down, horizontal spread is deferred
            return affected;
        }

        // 3. Horizontal Propagation: drops by 1 level per block traveled for Water; 2 for Lava
        let drop = cur_state.fluid_type.drop_off();
        if cur_state.level <= drop {
            return affected; // Flow exhausted
        }

        let next_level = cur_state.level - drop;
        let horizontal_dirs = [
            IVec3::new(1, 0, 0),
            IVec3::new(-1, 0, 0),
            IVec3::new(0, 0, 1),
            IVec3::new(0, 0, -1),
        ];

        for dir in horizontal_dirs {
            let neighbor_pos = pos + dir;

            // Check collision with opposite fluid at neighbor
            if self.check_fluid_collision(neighbor_pos, cur_state.fluid_type, world, &mut affected) {
                continue;
            }

            let neighbor_block = world.get_block_at(neighbor_pos.x, neighbor_pos.y, neighbor_pos.z);
            let existing_neighbor_fluid = self.get_fluid_state(neighbor_pos);

            // Air or replaceable block
            if neighbor_block.is_air() {
                let flowing_state = FluidState::new_flowing(cur_state.fluid_type, next_level);
                self.set_fluid_state(neighbor_pos, flowing_state);
                let mut aff = world.set_block_at(neighbor_pos, cur_state.fluid_type.to_block());
                affected.append(&mut aff);
                self.schedule_update(neighbor_pos);
            } else if neighbor_block == cur_state.fluid_type.to_block() {
                if let Some(mut existing) = existing_neighbor_fluid {
                    if !existing.falling && existing.level < next_level {
                        existing.level = next_level;
                        self.set_fluid_state(neighbor_pos, existing);
                        self.schedule_update(neighbor_pos);
                        let (sec, _, _, _) = Self::world_to_section_and_local(neighbor_pos);
                        affected.push(sec);
                    }
                }
            }
        }

        affected.sort_by_key(|p| (p.x, p.y, p.z));
        affected.dedup();
        affected
    }

    /// Process queued fluid cellular automaton ticks
    pub fn tick(&mut self, world: &mut dyn FluidWorldAccess) -> Vec<IVec3> {
        let mut affected = Vec::new();
        let mut updates_to_process = Vec::new();

        while let Some(pos) = self.pending_updates.pop_front() {
            updates_to_process.push(pos);
        }

        for pos in updates_to_process {
            let mut aff = self.propagate_at(pos, world);
            affected.append(&mut aff);
        }

        affected.sort_by_key(|p| (p.x, p.y, p.z));
        affected.dedup();
        affected
    }

    /// REA Verified: FlowingFluid.getFlow(BlockGetter, BlockPos, FluidState)
    /// Calculates flow direction vector based on height gradient and downward falling flow.
    /// Entities (player, mobs, item drops) receive directional push forces from this vector.
    pub fn get_flow_direction(&self, pos: IVec3, world: &dyn BlockReader) -> Vec3 {
        let cur_state = match self.get_fluid_state(pos) {
            Some(s) => s,
            None => {
                let b = world.get_block_at(pos.x, pos.y, pos.z);
                if let Some(ft) = FluidType::from_block(b) {
                    FluidState::new_source(ft)
                } else {
                    return Vec3::ZERO;
                }
            }
        };

        let mut flow = Vec3::ZERO;

        // If falling vertically, dominant flow is straight down
        if cur_state.falling {
            return Vec3::new(0.0, -1.0, 0.0);
        }

        let cur_height = cur_state.height();

        // 4 Horizontal cardinal directions
        let cardinals = [
            (IVec3::new(1, 0, 0), Vec3::new(1.0, 0.0, 0.0)),
            (IVec3::new(-1, 0, 0), Vec3::new(-1.0, 0.0, 0.0)),
            (IVec3::new(0, 0, 1), Vec3::new(0.0, 0.0, 1.0)),
            (IVec3::new(0, 0, -1), Vec3::new(0.0, 0.0, -1.0)),
        ];

        for (offset, dir_vec) in cardinals {
            let neighbor_pos = pos + offset;
            let neighbor_block = world.get_block_at(neighbor_pos.x, neighbor_pos.y, neighbor_pos.z);

            if neighbor_block.is_opaque() {
                continue; // Blocked
            }

            if let Some(n_state) = self.get_fluid_state(neighbor_pos) {
                if n_state.fluid_type == cur_state.fluid_type {
                    let diff = cur_height - n_state.height();
                    flow += dir_vec * diff;
                }
            } else if neighbor_block.is_air() {
                // If neighbor is air and block below neighbor is air, it's an open ledge drop
                let below_neighbor = neighbor_pos + IVec3::new(0, -1, 0);
                let below_n_block = world.get_block_at(below_neighbor.x, below_neighbor.y, below_neighbor.z);
                if below_n_block.is_air() || self.get_fluid_state(below_neighbor).is_some() {
                    flow += dir_vec * cur_height;
                }
            }
        }

        // Check downward pull into air below
        let below_pos = pos + IVec3::new(0, -1, 0);
        let below_block = world.get_block_at(below_pos.x, below_pos.y, below_pos.z);
        if below_block.is_air() {
            flow += Vec3::new(0.0, -0.5, 0.0);
        }

        if flow.length_squared() > 0.0001 {
            flow.normalize()
        } else {
            Vec3::ZERO
        }
    }

    #[inline(always)]
    fn world_to_section_and_local(pos: IVec3) -> (IVec3, usize, usize, usize) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockWorld {
        blocks: HashMap<IVec3, BlockType>,
    }

    impl MockWorld {
        fn new() -> Self {
            Self {
                blocks: HashMap::new(),
            }
        }
    }

    impl BlockReader for MockWorld {
        fn get_block_at(&self, x: i32, y: i32, z: i32) -> BlockType {
            self.blocks.get(&IVec3::new(x, y, z)).copied().unwrap_or(BlockType::Air)
        }
    }

    impl FluidWorldAccess for MockWorld {
        fn set_block_at(&mut self, pos: IVec3, block: BlockType) -> Vec<IVec3> {
            self.blocks.insert(pos, block);
            let sec = IVec3::new(
                pos.x.div_euclid(16),
                pos.y.div_euclid(16),
                pos.z.div_euclid(16),
            );
            vec![sec]
        }
    }

    #[test]
    fn test_fluid_state_source_and_falling() {
        let water_source = FluidState::new_source(FluidType::Water);
        assert_eq!(water_source.level, 8);
        assert!(!water_source.falling);
        assert!(water_source.is_source());
        assert_eq!(water_source.height(), 1.0);

        let falling_water = FluidState::new_falling(FluidType::Water);
        assert_eq!(falling_water.level, 8);
        assert!(falling_water.falling);
        assert!(!falling_water.is_source());
        assert_eq!(falling_water.height(), 1.0);

        let flowing_water = FluidState::new_flowing(FluidType::Water, 4);
        assert_eq!(flowing_water.level, 4);
        assert!(!flowing_water.falling);
        assert_eq!(flowing_water.height(), 0.5);
    }

    #[test]
    fn test_downward_falling_retains_level_8() {
        let mut world = MockWorld::new();
        let mut engine = FluidEngine::new();

        // Place water source at (0, 10, 0) in open air
        engine.on_fluid_placed(IVec3::new(0, 10, 0), FluidType::Water, &mut world);

        // Below block is Air -> must flow downward retaining Level 8 with falling = true
        let state_below = engine.get_fluid_state(IVec3::new(0, 9, 0)).expect("Missing fluid below");
        assert_eq!(state_below.level, 8);
        assert!(state_below.falling);
        assert_eq!(world.get_block_at(0, 9, 0), BlockType::Water);
    }

    #[test]
    fn test_water_horizontal_propagation_7_blocks() {
        let mut world = MockWorld::new();
        let mut engine = FluidEngine::new();

        // Floor at y = 0
        for x in -8..=8 {
            for z in -8..=8 {
                world.set_block_at(IVec3::new(x, 0, z), BlockType::Stone);
            }
        }

        // Place water source at (0, 1, 0) on stone floor
        engine.on_fluid_placed(IVec3::new(0, 1, 0), FluidType::Water, &mut world);

        // Water should drop 1 level per block: (0, 1, 0)=8, (1, 1, 0)=7
        let st1 = engine.get_fluid_state(IVec3::new(1, 1, 0)).expect("Missing level 7");
        assert_eq!(st1.level, 7);
        assert!(!st1.falling);

        // Run updates to spread to edge
        for _ in 0..10 {
            engine.tick(&mut world);
        }

        // Distance 7 from source should have level 1
        let st7 = engine.get_fluid_state(IVec3::new(7, 1, 0));
        assert!(st7.is_some());
        assert_eq!(st7.unwrap().level, 1);

        // Distance 8 should be empty (water travels max 7 blocks horizontally)
        let st8 = engine.get_fluid_state(IVec3::new(8, 1, 0));
        assert!(st8.is_none());
    }

    #[test]
    fn test_lava_horizontal_propagation_3_blocks() {
        let mut world = MockWorld::new();
        let mut engine = FluidEngine::new();

        // Floor at y = 0
        for x in -5..=5 {
            for z in -5..=5 {
                world.set_block_at(IVec3::new(x, 0, z), BlockType::Stone);
            }
        }

        // Place lava source at (0, 1, 0)
        engine.on_fluid_placed(IVec3::new(0, 1, 0), FluidType::Lava, &mut world);

        // In Overworld, Lava drops by 2 per block: 8 -> 6 -> 4 -> 2 (3 blocks max)
        for _ in 0..6 {
            engine.tick(&mut world);
        }

        let st1 = engine.get_fluid_state(IVec3::new(1, 1, 0));
        assert!(st1.is_some());
        assert_eq!(st1.unwrap().level, 6);

        let st2 = engine.get_fluid_state(IVec3::new(2, 1, 0));
        assert!(st2.is_some());
        assert_eq!(st2.unwrap().level, 4);

        let st3 = engine.get_fluid_state(IVec3::new(3, 1, 0));
        assert!(st3.is_some());
        assert_eq!(st3.unwrap().level, 2);

        // Distance 4 should have no lava (travels up to 3 blocks in Overworld)
        let st4 = engine.get_fluid_state(IVec3::new(4, 1, 0));
        assert!(st4.is_none());
    }

    #[test]
    fn test_water_contacts_stationary_lava_converts_to_obsidian() {
        let mut world = MockWorld::new();
        let mut engine = FluidEngine::new();

        // Stationary Lava source at (1, 1, 0)
        world.set_block_at(IVec3::new(1, 1, 0), BlockType::Lava);
        engine.set_fluid_state(IVec3::new(1, 1, 0), FluidState::new_source(FluidType::Lava));

        // Solid floor
        world.set_block_at(IVec3::new(0, 0, 0), BlockType::Stone);
        world.set_block_at(IVec3::new(1, 0, 0), BlockType::Stone);

        // Place water source at (0, 1, 0)
        engine.on_fluid_placed(IVec3::new(0, 1, 0), FluidType::Water, &mut world);

        // Water flows into stationary lava source -> converts to Obsidian!
        assert_eq!(world.get_block_at(1, 1, 0), BlockType::Obsidian);
        assert!(engine.get_fluid_state(IVec3::new(1, 1, 0)).is_none());
    }

    #[test]
    fn test_water_contacts_flowing_lava_converts_to_cobblestone() {
        let mut world = MockWorld::new();
        let mut engine = FluidEngine::new();

        // Flowing Lava (level 4) at (1, 1, 0)
        world.set_block_at(IVec3::new(1, 1, 0), BlockType::Lava);
        engine.set_fluid_state(IVec3::new(1, 1, 0), FluidState::new_flowing(FluidType::Lava, 4));

        // Solid floor
        world.set_block_at(IVec3::new(0, 0, 0), BlockType::Stone);
        world.set_block_at(IVec3::new(1, 0, 0), BlockType::Stone);

        // Place water source at (0, 1, 0)
        engine.on_fluid_placed(IVec3::new(0, 1, 0), FluidType::Water, &mut world);

        // Water flows into flowing lava -> converts to Cobblestone!
        assert_eq!(world.get_block_at(1, 1, 0), BlockType::Cobblestone);
        assert!(engine.get_fluid_state(IVec3::new(1, 1, 0)).is_none());
    }

    #[test]
    fn test_flow_direction_vector() {
        let mut world = MockWorld::new();
        let mut engine = FluidEngine::new();

        // Solid floor under the water
        for x in -2..=2 {
            for z in -2..=2 {
                world.set_block_at(IVec3::new(x, 0, z), BlockType::Stone);
            }
        }

        // Water source at (0, 1, 0) and flowing water at (1, 1, 0)
        world.set_block_at(IVec3::new(0, 1, 0), BlockType::Water);
        engine.set_fluid_state(IVec3::new(0, 1, 0), FluidState::new_source(FluidType::Water));

        world.set_block_at(IVec3::new(1, 1, 0), BlockType::Water);
        engine.set_fluid_state(IVec3::new(1, 1, 0), FluidState::new_flowing(FluidType::Water, 4));

        // Flow vector at (0, 1, 0) should point towards +X
        let flow = engine.get_flow_direction(IVec3::new(0, 1, 0), &world);
        assert!(flow.x > 0.5, "Expected flow.x > 0.5, got {:?}", flow);

        // Falling water at (5, 5, 5) should point straight down
        engine.set_fluid_state(IVec3::new(5, 5, 5), FluidState::new_falling(FluidType::Water));
        let falling_flow = engine.get_flow_direction(IVec3::new(5, 5, 5), &world);
        assert_eq!(falling_flow, Vec3::new(0.0, -1.0, 0.0));
    }
}
