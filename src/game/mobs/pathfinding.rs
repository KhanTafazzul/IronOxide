// REA Verified: net.minecraft.world.entity.ai.navigation.PathNavigation (mapped: bvv.class)
// REA Verified: net.minecraft.world.level.pathfinder.PathFinder (mapped: epc.class, epd.class)
//
// 3D A* Grid Pathfinding over Voxel Collision Geometry:
// - Navigation over 3D voxel grid collision geometry.
// - Authentic mob capabilities:
//   * Jumping 1-block steps (dy = +1).
//   * Safe step-downs up to 3 blocks (dy in [-3, -1]).
//   * Avoiding dangerous cliff drops (> 3 blocks).
//   * Enforcing 2-block vertical clearance for humanoid mobs (zombie height 1.8m-1.95m).

use glam::IVec3;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use crate::render::mesher::BlockReader;

/// A* Priority Queue Node
#[derive(Copy, Clone, PartialEq)]
struct AStarNode {
    pos: IVec3,
    f_score: f32,
}

impl Eq for AStarNode {}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap
        other
            .f_score
            .partial_cmp(&self.f_score)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Heuristic: 3D Euclidean distance
#[inline]
fn heuristic(a: IVec3, b: IVec3) -> f32 {
    let dx = (a.x - b.x) as f32;
    let dy = (a.y - b.y) as f32;
    let dz = (a.z - b.z) as f32;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Checks if a voxel position has solid floor and 2-block head clearance
#[inline]
pub fn is_standable(pos: IVec3, world: &dyn BlockReader) -> bool {
    let floor_solid = world.get_block_at(pos.x, pos.y - 1, pos.z).is_opaque();
    let feet_clear = !world.get_block_at(pos.x, pos.y, pos.z).is_opaque();
    let head_clear = !world.get_block_at(pos.x, pos.y + 1, pos.z).is_opaque();
    floor_solid && feet_clear && head_clear
}

/// Finds accessible neighbors for a 2-block tall mob:
/// - Flat step (dy = 0)
/// - Step-up 1 block (dy = +1, jumping single slab/block)
/// - Safe step-down up to 3 blocks (dy in [-3, -1])
/// - Drops > 3 blocks are considered dangerous and rejected
pub fn get_neighbors(curr: IVec3, world: &dyn BlockReader) -> Vec<(IVec3, f32)> {
    let mut neighbors = Vec::with_capacity(4);
    let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];

    for (dx, dz) in dirs {
        let nx = curr.x + dx;
        let nz = curr.z + dz;

        // 1. Flat movement (dy = 0)
        let flat = IVec3::new(nx, curr.y, nz);
        if is_standable(flat, world) {
            neighbors.push((flat, 1.0));
            continue;
        }

        // 2. Step-up 1 block (dy = +1)
        // Requires floor at curr.y, air at curr.y + 1 and curr.y + 2, and overhead clearance above curr
        let step_up = IVec3::new(nx, curr.y + 1, nz);
        let curr_overhead_clear = !world.get_block_at(curr.x, curr.y + 2, curr.z).is_opaque();
        if curr_overhead_clear && is_standable(step_up, world) {
            neighbors.push((step_up, 1.4)); // 1.4 cost for jump effort
            continue;
        }

        // 3. Step-down (dy in [-3, -1])
        // Must avoid drops > 3 blocks!
        // Air must exist at step edge (nx, curr.y, nz) and (nx, curr.y + 1, nz)
        let step_edge_clear = !world.get_block_at(nx, curr.y, nz).is_opaque()
            && !world.get_block_at(nx, curr.y + 1, nz).is_opaque();

        if step_edge_clear {
            let mut found_safe_drop = false;
            for drop in 1..=3 {
                let target_y = curr.y - drop;
                let candidate = IVec3::new(nx, target_y, nz);
                if is_standable(candidate, world) {
                    let cost = 1.0 + 0.3 * (drop as f32);
                    neighbors.push((candidate, cost));
                    found_safe_drop = true;
                    break;
                }
            }
            if !found_safe_drop {
                // If drop is deeper than 3 blocks, it's a dangerous cliff; do not add neighbor!
            }
        }
    }

    neighbors
}

/// 3D A* Pathfinding implementation
/// Returns a sequence of waypoints from start to goal (excluding start).
pub fn find_path(
    start: IVec3,
    goal: IVec3,
    max_nodes: usize,
    world: &dyn BlockReader,
) -> Option<Vec<IVec3>> {
    if start == goal {
        return Some(vec![goal]);
    }

    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<IVec3, IVec3> = HashMap::new();
    let mut g_score: HashMap<IVec3, f32> = HashMap::new();

    g_score.insert(start, 0.0);
    open_set.push(AStarNode {
        pos: start,
        f_score: heuristic(start, goal),
    });

    let mut nodes_visited = 0;
    let mut closest_node = start;
    let mut min_h = heuristic(start, goal);

    while let Some(AStarNode { pos: current, .. }) = open_set.pop() {
        nodes_visited += 1;
        if nodes_visited > max_nodes {
            break;
        }

        let h_curr = heuristic(current, goal);
        if h_curr < min_h {
            min_h = h_curr;
            closest_node = current;
        }

        // Reached goal (exact match or within 1 block reach)
        if current == goal || (current.x == goal.x && current.z == goal.z && (current.y - goal.y).abs() <= 1) {
            let mut path = Vec::new();
            let mut curr = current;
            while curr != start {
                path.push(curr);
                if let Some(&prev) = came_from.get(&curr) {
                    curr = prev;
                } else {
                    break;
                }
            }
            path.reverse();
            return Some(path);
        }

        let current_g = *g_score.get(&current).unwrap_or(&f32::INFINITY);

        for (neighbor, cost) in get_neighbors(current, world) {
            let tentative_g = current_g + cost;
            let neighbor_g = *g_score.get(&neighbor).unwrap_or(&f32::INFINITY);

            if tentative_g < neighbor_g {
                came_from.insert(neighbor, current);
                g_score.insert(neighbor, tentative_g);
                let f_score = tentative_g + heuristic(neighbor, goal);
                open_set.push(AStarNode {
                    pos: neighbor,
                    f_score,
                });
            }
        }
    }

    // If exact goal unreachable, return path to closest explored node if it made progress
    if closest_node != start {
        let mut path = Vec::new();
        let mut curr = closest_node;
        while curr != start {
            path.push(curr);
            if let Some(&prev) = came_from.get(&curr) {
                curr = prev;
            } else {
                break;
            }
        }
        path.reverse();
        if !path.is_empty() {
            return Some(path);
        }
    }

    None
}

/// Line-of-sight raycast check between two 3D points
pub fn has_line_of_sight(from: glam::Vec3, to: glam::Vec3, world: &dyn BlockReader) -> bool {
    let diff = to - from;
    let dist = diff.length();
    if dist <= 0.001 {
        return true;
    }

    let dir = diff / dist;
    let steps = (dist * 4.0).ceil() as usize; // Check every 0.25 blocks
    let step_dist = dist / (steps as f32);

    for i in 1..steps {
        let check_pt = from + dir * (i as f32 * step_dist);
        let block_pos = check_pt.floor().as_ivec3();
        if world.get_block_at(block_pos.x, block_pos.y, block_pos.z).is_opaque() {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::atlas::BlockType;

    struct TestWorld {
        blocks: HashMap<IVec3, BlockType>,
    }

    impl BlockReader for TestWorld {
        fn get_block_at(&self, x: i32, y: i32, z: i32) -> BlockType {
            self.blocks
                .get(&IVec3::new(x, y, z))
                .copied()
                .unwrap_or(BlockType::Air)
        }
    }

    #[test]
    fn test_flat_pathfinding() {
        let mut world = TestWorld {
            blocks: HashMap::new(),
        };
        // Build 5-block flat walkway from x = 0..=5 at y = 0
        for x in 0..=5 {
            world.blocks.insert(IVec3::new(x, 0, 0), BlockType::Stone);
        }

        let start = IVec3::new(0, 1, 0);
        let goal = IVec3::new(5, 1, 0);
        let path = find_path(start, goal, 100, &world).expect("Should find flat path");

        assert_eq!(path.len(), 5);
        assert_eq!(path.last().unwrap(), &goal);
    }

    #[test]
    fn test_jump_1_block_step() {
        let mut world = TestWorld {
            blocks: HashMap::new(),
        };
        // Floor at y=0 for x=0..=1, then step up to y=1 at x=2..=3
        world.blocks.insert(IVec3::new(0, 0, 0), BlockType::Stone);
        world.blocks.insert(IVec3::new(1, 0, 0), BlockType::Stone);
        world.blocks.insert(IVec3::new(2, 1, 0), BlockType::Stone); // 1-block step up
        world.blocks.insert(IVec3::new(3, 1, 0), BlockType::Stone);

        let start = IVec3::new(1, 1, 0);
        let goal = IVec3::new(3, 2, 0);
        let path = find_path(start, goal, 100, &world).expect("Should jump 1-block step");

        assert_eq!(path[0], IVec3::new(2, 2, 0));
        assert_eq!(path[1], IVec3::new(3, 2, 0));
    }

    #[test]
    fn test_avoid_drops_greater_than_3_blocks() {
        let mut world = TestWorld {
            blocks: HashMap::new(),
        };
        // Upper platform at y=5 for x=0..=1
        world.blocks.insert(IVec3::new(0, 5, 0), BlockType::Stone);
        world.blocks.insert(IVec3::new(1, 5, 0), BlockType::Stone);

        // Deadly 5-block cliff drop: floor at y=0 (drop of 4 blocks from y=5 feet)
        world.blocks.insert(IVec3::new(2, 0, 0), BlockType::Stone);

        let curr = IVec3::new(1, 6, 0);
        let neighbors = get_neighbors(curr, &world);

        // Neighbor at x=2 should NOT be generated because drop is 4 blocks (> 3 blocks limit)
        let has_cliff_drop = neighbors.iter().any(|(pos, _)| pos.x == 2);
        assert!(!has_cliff_drop, "Should reject dangerous cliff drop > 3 blocks");
    }

    #[test]
    fn test_allow_safe_drop_up_to_3_blocks() {
        let mut world = TestWorld {
            blocks: HashMap::new(),
        };
        // Platform at y=4
        world.blocks.insert(IVec3::new(0, 4, 0), BlockType::Stone);
        // Landing at y=2 (drop of 2 blocks, <= 3)
        world.blocks.insert(IVec3::new(1, 2, 0), BlockType::Stone);

        let curr = IVec3::new(0, 5, 0);
        let neighbors = get_neighbors(curr, &world);

        let safe_drop = neighbors.iter().find(|(pos, _)| pos.x == 1);
        assert!(safe_drop.is_some(), "Should allow safe drop of 2 blocks");
        assert_eq!(safe_drop.unwrap().0, IVec3::new(1, 3, 0));
    }
}
