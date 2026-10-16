// REA Verified: net.minecraft.client.multiplayer.MultiPlayerGameMode.continueDestroyBlock (mapped: fnz.b)
// REA Verified: net.minecraft.world.level.block.Block.getDestroyProgress
//
// Block Hardness, Mining Speed & Tool Multipliers:
// - Dirt/Grass: 0.5s bare hand break.
// - Stone: 1.5s with wooden pickaxe, 7.5s with bare hand (wrong tool penalty).
// - Bedrock: Unbreakable (-1.0 hardness).
// - Deepslate: 3.0s with pickaxe, 15.0s with bare hand.
// - Tracks active break progress (0.0 to 1.0) and progressive destroy crack stages (0..9).

use glam::IVec3;
use crate::game::inventory::{ItemStack, ITEM_WOODEN_AXE, ITEM_WOODEN_PICKAXE};
use crate::render::atlas::{BlockType, Direction};

/// Returns the vanilla block hardness value (-1.0 for unbreakable)
pub fn get_block_hardness(block: BlockType) -> f32 {
    match block {
        BlockType::Air => 0.0,
        BlockType::Bedrock => -1.0, // Unbreakable
        BlockType::Dirt | BlockType::GrassBlock | BlockType::Sand => 0.5,
        BlockType::Stone => 1.5,
        BlockType::Cobblestone | BlockType::Bricks => 2.0,
        BlockType::OakLog | BlockType::OakPlanks => 2.0,
        BlockType::Deepslate => 3.0,
        BlockType::OakLeaves => 0.2,
        BlockType::Glass => 0.3,
        BlockType::Torch => 0.0,
        BlockType::Lava | BlockType::Water => -1.0,
        BlockType::Obsidian => 50.0,
    }
}

/// Computes the time (in seconds) required to break a block with the held item
pub fn calculate_break_time(block: BlockType, held_item: Option<ItemStack>) -> Option<f32> {
    let hardness = get_block_hardness(block);
    if hardness < 0.0 || block == BlockType::Air {
        return None; // Unbreakable
    }

    let held_id = held_item.map(|s| s.id).unwrap_or(0);
    let is_pickaxe = held_id == ITEM_WOODEN_PICKAXE;
    let is_axe = held_id == ITEM_WOODEN_AXE;

    let time = match block {
        // Dirt / Grass / Sand: 0.5s bare hand
        BlockType::Dirt | BlockType::GrassBlock | BlockType::Sand => 0.5,
        // Stone: 1.5s with wooden pickaxe, 7.5s with bare hand
        BlockType::Stone => {
            if is_pickaxe {
                1.5
            } else {
                7.5
            }
        }
        // Cobblestone / Bricks: 2.0s with pickaxe, 10.0s with bare hand
        BlockType::Cobblestone | BlockType::Bricks => {
            if is_pickaxe {
                2.0
            } else {
                10.0
            }
        }
        // Deepslate: 3.0s with pickaxe, 15.0s with bare hand
        BlockType::Deepslate => {
            if is_pickaxe {
                3.0
            } else {
                15.0
            }
        }
        // Wood logs & planks: 1.5s with axe, 3.0s with bare hand
        BlockType::OakLog | BlockType::OakPlanks => {
            if is_axe {
                1.5
            } else {
                3.0
            }
        }
        // Leaves: 0.3s
        BlockType::OakLeaves => 0.3,
        // Glass: 0.45s
        BlockType::Glass => 0.45,
        _ => hardness * 1.5,
    };

    Some(time)
}

/// Result of a mining tick update
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MiningUpdateResult {
    /// Actively mining block, progress in 0.0..1.0, destroy stage in 0..=9
    Mining {
        pos: IVec3,
        face: Direction,
        progress: f32,
        stage: usize,
    },
    /// Block broken: progress reached 1.0!
    Broken {
        pos: IVec3,
        face: Direction,
        block: BlockType,
    },
    /// Idle (left mouse not down, or looking at air/unbreakable)
    Idle,
}

/// Tracks ongoing block destruction progress
#[derive(Debug, Clone)]
pub struct BlockMiningTracker {
    pub target_pos: Option<IVec3>,
    pub target_face: Option<Direction>,
    pub target_block: BlockType,
    pub progress: f32,       // 0.0 to 1.0
    pub break_duration: f32, // total seconds to break
}

impl Default for BlockMiningTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockMiningTracker {
    pub fn new() -> Self {
        Self {
            target_pos: None,
            target_face: None,
            target_block: BlockType::Air,
            progress: 0.0,
            break_duration: 1.0,
        }
    }

    /// Reset ongoing mining progress
    pub fn reset(&mut self) {
        self.target_pos = None;
        self.target_face = None;
        self.target_block = BlockType::Air;
        self.progress = 0.0;
        self.break_duration = 1.0;
    }

    /// Update mining progress per frame.
    /// Returns `MiningUpdateResult` with active crack stage or block broken event.
    pub fn update(
        &mut self,
        dt: f32,
        is_left_held: bool,
        raycast_target: Option<(IVec3, Direction, BlockType)>,
        held_item: Option<ItemStack>,
    ) -> MiningUpdateResult {
        if !is_left_held {
            self.reset();
            return MiningUpdateResult::Idle;
        }

        let (pos, face, block) = match raycast_target {
            Some(target) => target,
            None => {
                self.reset();
                return MiningUpdateResult::Idle;
            }
        };

        let duration = match calculate_break_time(block, held_item) {
            Some(d) => d,
            None => {
                // Unbreakable (e.g. Bedrock)
                self.reset();
                return MiningUpdateResult::Idle;
            }
        };

        // If target changed, reset progress to zero
        if self.target_pos != Some(pos) || self.target_block != block {
            self.target_pos = Some(pos);
            self.target_face = Some(face);
            self.target_block = block;
            self.progress = 0.0;
            self.break_duration = duration;
        } else {
            self.target_face = Some(face);
            self.break_duration = duration;
        }

        // Advance progress
        if self.break_duration > 0.0 {
            self.progress += dt / self.break_duration;
        } else {
            self.progress = 1.0;
        }

        if self.progress >= 1.0 {
            let broken_block = self.target_block;
            self.reset();
            MiningUpdateResult::Broken {
                pos,
                face,
                block: broken_block,
            }
        } else {
            // REA Verified: MultiPlayerGameMode.getDestroyStage formula:
            // stage = (int)(destroyProgress * 10.0f)
            let stage = ((self.progress * 10.0).floor() as usize).min(9);
            MiningUpdateResult::Mining {
                pos,
                face,
                progress: self.progress,
                stage,
            }
        }
    }

    /// Returns the active destroy crack stage (0 to 9) if currently breaking a block
    pub fn active_destroy_stage(&self) -> Option<usize> {
        if self.target_pos.is_some() && self.progress > 0.0 {
            Some(((self.progress * 10.0).floor() as usize).min(9))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_hardness_values() {
        assert_eq!(get_block_hardness(BlockType::Bedrock), -1.0);
        assert_eq!(get_block_hardness(BlockType::Dirt), 0.5);
        assert_eq!(get_block_hardness(BlockType::GrassBlock), 0.5);
        assert_eq!(get_block_hardness(BlockType::Stone), 1.5);
        assert_eq!(get_block_hardness(BlockType::Cobblestone), 2.0);
        assert_eq!(get_block_hardness(BlockType::OakLog), 2.0);
        assert_eq!(get_block_hardness(BlockType::Deepslate), 3.0);
    }

    #[test]
    fn test_break_times_dirt_stone_bedrock() {
        // Dirt / Grass: 0.5s bare hand
        let dirt_time = calculate_break_time(BlockType::Dirt, None);
        assert_eq!(dirt_time, Some(0.5));

        let grass_time = calculate_break_time(BlockType::GrassBlock, None);
        assert_eq!(grass_time, Some(0.5));

        // Stone: 7.5s bare hand, 1.5s with wood pickaxe
        let stone_hand = calculate_break_time(BlockType::Stone, None);
        assert_eq!(stone_hand, Some(7.5));

        let wood_pick = Some(ItemStack::new(ITEM_WOODEN_PICKAXE, 1));
        let stone_pick = calculate_break_time(BlockType::Stone, wood_pick);
        assert_eq!(stone_pick, Some(1.5));

        // Bedrock: Unbreakable
        let bedrock_time = calculate_break_time(BlockType::Bedrock, wood_pick);
        assert_eq!(bedrock_time, None);
    }

    #[test]
    fn test_mining_progress_and_stages() {
        let mut tracker = BlockMiningTracker::new();
        let target = Some((IVec3::new(5, 1, 5), Direction::Up, BlockType::Dirt));

        // Initial update with dt = 0.25s (half of 0.5s duration)
        let res1 = tracker.update(0.25, true, target, None);
        match res1 {
            MiningUpdateResult::Mining { progress, stage, .. } => {
                assert!((progress - 0.5).abs() < 1e-4);
                assert_eq!(stage, 5); // 0.5 * 10 = 5
            }
            _ => panic!("Expected Mining state"),
        }

        // Another 0.25s completes the break!
        let res2 = tracker.update(0.25, true, target, None);
        match res2 {
            MiningUpdateResult::Broken { pos, block, .. } => {
                assert_eq!(pos, IVec3::new(5, 1, 5));
                assert_eq!(block, BlockType::Dirt);
            }
            _ => panic!("Expected Broken state"),
        }
    }

    #[test]
    fn test_target_switch_resets_progress() {
        let mut tracker = BlockMiningTracker::new();
        let target1 = Some((IVec3::new(1, 0, 1), Direction::Up, BlockType::Stone));
        let target2 = Some((IVec3::new(2, 0, 1), Direction::Up, BlockType::Stone));

        tracker.update(2.0, true, target1, None);
        assert!(tracker.progress > 0.0);

        // Switching target block resets progress
        let res = tracker.update(0.1, true, target2, None);
        match res {
            MiningUpdateResult::Mining { progress, .. } => {
                assert!(progress < 0.1);
            }
            _ => panic!("Expected Mining state"),
        }
    }
}
