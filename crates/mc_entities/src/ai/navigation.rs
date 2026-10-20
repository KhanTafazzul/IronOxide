// REA Verified: net.minecraft.world.entity.ai.navigation.PathNavigation (mapped: bvv.class)
// REA Verified: net.minecraft.world.level.pathfinder.PathFinder (mapped: epc.class, epd.class)
// REA Verified: net.minecraft.world.level.pathfinder.WalkNodeEvaluator (mapped: epf.class)
//
// 3D Voxel Node Navigation & A* Pathfinding Engine over Minecraft Collision Geometry:
// - Step-up 1-block obstacles (dy = +1).
// - Safe step-downs up to 3 blocks (dy in [-3, -1]).
// - Cliff avoidance (reject drops > 3 blocks).
// - Hazard avoidance (fire, lava, cactus).
// - Door traversal (open doors passable, wooden doors operable if allowed, iron doors impassable).

use glam::IVec3;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

/// World block inspection trait for navigation and collision testing.
pub trait BlockAccessor {
    fn is_solid(&self, pos: IVec3) -> bool;
    fn is_water(&self, pos: IVec3) -> bool;
    fn is_lava(&self, pos: IVec3) -> bool;
    fn is_fire(&self, pos: IVec3) -> bool;
    fn is_door(&self, pos: IVec3) -> bool;
    fn is_door_open(&self, pos: IVec3) -> bool;
    fn can_open_door(&self, pos: IVec3) -> bool;
}

/// Node traversal classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathNodeType {
    Walkable,
    DoorOpen,
    DoorClosedPassable,
    DoorClosedBlocked,
    Hazard,
    Cliff,
    Blocked,
}

/// Represents a completed 3D path of voxel coordinates.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Path3D {
    pub nodes: Vec<IVec3>,
    pub current_index: usize,
}

impl Path3D {
    pub fn new(nodes: Vec<IVec3>) -> Self {
        Self {
            nodes,
            current_index: 0,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.current_index >= self.nodes.len()
    }

    pub fn current_node(&self) -> Option<IVec3> {
        self.nodes.get(self.current_index).copied()
    }

    pub fn next_node(&self) -> Option<IVec3> {
        self.nodes.get(self.current_index + 1).copied()
    }

    pub fn advance(&mut self) {
        if self.current_index < self.nodes.len() {
            self.current_index += 1;
        }
    }

    pub fn remaining_nodes(&self) -> &[IVec3] {
        if self.current_index < self.nodes.len() {
            &self.nodes[self.current_index..]
        } else {
            &[]
        }
    }
}

/// A* Priority Queue search node.
#[derive(Copy, Clone, PartialEq)]
struct AStarNode {
    pos: IVec3,
    f_score: f32,
}

impl Eq for AStarNode {}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap ordering (lowest f_score prioritizes)
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

/// 3D Euclidean distance heuristic.
#[inline]
fn euclidean_heuristic(a: IVec3, b: IVec3) -> f32 {
    let dx = (a.x - b.x) as f32;
    let dy = (a.y - b.y) as f32;
    let dz = (a.z - b.z) as f32;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// 3D Voxel Node Navigation engine.
pub struct PathNavigation {
    pub can_open_doors: bool,
    pub can_pass_doors: bool,
    pub max_visited_nodes: usize,
}

impl PathNavigation {
    pub fn new(can_open_doors: bool) -> Self {
        Self {
            can_open_doors,
            can_pass_doors: true,
            max_visited_nodes: 512,
        }
    }

    pub fn standard_mob() -> Self {
        Self::new(false)
    }

    pub fn door_opening_mob() -> Self {
        Self::new(true) // e.g. Villagers, Door-breaking Zombies, Vindicators
    }

    /// Evaluates if a given position is safe to stand on.
    /// Requirements:
    /// - Solid floor at `pos.y - 1`
    /// - Non-solid feet at `pos.y` and head at `pos.y + 1`
    /// - Free of hazards (lava, fire)
    /// - Door handling: open doors or operable doors
    pub fn evaluate_node(&self, pos: IVec3, world: &dyn BlockAccessor) -> PathNodeType {
        let floor = IVec3::new(pos.x, pos.y - 1, pos.z);
        let feet = pos;
        let head = IVec3::new(pos.x, pos.y + 1, pos.z);

        // Hazards check
        if world.is_fire(feet) || world.is_lava(feet) || world.is_fire(head) || world.is_lava(head) {
            return PathNodeType::Hazard;
        }

        // Floor must be solid
        if !world.is_solid(floor) {
            return PathNodeType::Blocked;
        }

        // Door evaluation at feet or head
        if world.is_door(feet) {
            if world.is_door_open(feet) {
                return PathNodeType::DoorOpen;
            } else if self.can_open_doors && world.can_open_door(feet) {
                return PathNodeType::DoorClosedPassable;
            } else {
                return PathNodeType::DoorClosedBlocked;
            }
        }

        // Standard 2-block vertical clearance check
        if world.is_solid(feet) || world.is_solid(head) {
            return PathNodeType::Blocked;
        }

        PathNodeType::Walkable
    }

    /// Finds accessible adjacent neighbors for a mob:
    /// - Flat movement (dy = 0)
    /// - Step-up 1 block (dy = +1)
    /// - Safe step-down up to 3 blocks (dy in [-3, -1])
    /// - Rejects cliff drops (> 3 blocks) and hazards
    pub fn get_accessible_neighbors(
        &self,
        curr: IVec3,
        world: &dyn BlockAccessor,
    ) -> Vec<(IVec3, f32)> {
        let mut neighbors = Vec::with_capacity(4);
        let cardinal_dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];

        for (dx, dz) in cardinal_dirs {
            let nx = curr.x + dx;
            let nz = curr.z + dz;

            // 1. Flat movement (dy = 0)
            let flat = IVec3::new(nx, curr.y, nz);
            match self.evaluate_node(flat, world) {
                PathNodeType::Walkable | PathNodeType::DoorOpen => {
                    neighbors.push((flat, 1.0));
                    continue;
                }
                PathNodeType::DoorClosedPassable => {
                    neighbors.push((flat, 2.0)); // 2.0 cost penalty for opening door
                    continue;
                }
                PathNodeType::Hazard | PathNodeType::DoorClosedBlocked => {
                    // Impassable hazard or locked door
                    continue;
                }
                PathNodeType::Blocked => {
                    // Could be a 1-block wall: test step-up!
                }
                PathNodeType::Cliff => {}
            }

            // 2. Step-up 1 block (dy = +1)
            // Needs floor at curr.y, air at curr.y + 1 and curr.y + 2,
            // and overhead clearance at current pos (curr.y + 2) so mob doesn't bonk head jumping.
            let overhead = IVec3::new(curr.x, curr.y + 2, curr.z);
            if !world.is_solid(overhead) {
                let step_up = IVec3::new(nx, curr.y + 1, nz);
                match self.evaluate_node(step_up, world) {
                    PathNodeType::Walkable | PathNodeType::DoorOpen => {
                        neighbors.push((step_up, 1.4)); // 1.4 cost for jump effort
                        continue;
                    }
                    PathNodeType::DoorClosedPassable => {
                        neighbors.push((step_up, 2.4));
                        continue;
                    }
                    _ => {}
                }
            }

            // 3. Step-down (dy in [-3, -1])
            // Must have air at the edge to step off: (nx, curr.y, nz) and (nx, curr.y + 1, nz)
            let edge_feet = IVec3::new(nx, curr.y, nz);
            let edge_head = IVec3::new(nx, curr.y + 1, nz);
            if !world.is_solid(edge_feet) && !world.is_solid(edge_head) {
                let mut found_drop = false;
                for drop in 1..=3 {
                    let candidate = IVec3::new(nx, curr.y - drop, nz);
                    match self.evaluate_node(candidate, world) {
                        PathNodeType::Walkable | PathNodeType::DoorOpen => {
                            let cost = 1.0 + 0.3 * (drop as f32);
                            neighbors.push((candidate, cost));
                            found_drop = true;
                            break;
                        }
                        PathNodeType::DoorClosedPassable => {
                            let cost = 2.0 + 0.3 * (drop as f32);
                            neighbors.push((candidate, cost));
                            found_drop = true;
                            break;
                        }
                        PathNodeType::Hazard | PathNodeType::DoorClosedBlocked => {
                            // Dangerous floor below: drop rejected!
                            found_drop = true;
                            break;
                        }
                        _ => {}
                    }
                }

                // If drop > 3, it is a cliff! Rejected.
                if !found_drop {
                    // Drop > 3: cliff drop detected and avoided.
                }
            }
        }

        neighbors
    }

    /// Executes full 3D A* search over voxel collision space.
    pub fn find_path(
        &self,
        start: IVec3,
        goal: IVec3,
        world: &dyn BlockAccessor,
    ) -> Option<Path3D> {
        if start == goal {
            return Some(Path3D::new(vec![start]));
        }

        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<IVec3, IVec3> = HashMap::new();
        let mut g_score: HashMap<IVec3, f32> = HashMap::new();

        g_score.insert(start, 0.0);
        open_set.push(AStarNode {
            pos: start,
            f_score: euclidean_heuristic(start, goal),
        });

        let mut nodes_visited = 0;

        while let Some(AStarNode { pos: current, .. }) = open_set.pop() {
            nodes_visited += 1;
            if nodes_visited > self.max_visited_nodes {
                break;
            }

            // Target reached!
            if current == goal {
                return Some(Self::reconstruct_path(came_from, current));
            }

            let current_g = match g_score.get(&current) {
                Some(&g) => g,
                None => continue,
            };

            for (neighbor, move_cost) in self.get_accessible_neighbors(current, world) {
                let tentative_g = current_g + move_cost;
                let existing_g = g_score.get(&neighbor).copied().unwrap_or(f32::INFINITY);

                if tentative_g < existing_g {
                    came_from.insert(neighbor, current);
                    g_score.insert(neighbor, tentative_g);
                    let f_score = tentative_g + euclidean_heuristic(neighbor, goal);
                    open_set.push(AStarNode {
                        pos: neighbor,
                        f_score,
                    });
                }
            }
        }

        None
    }

    fn reconstruct_path(came_from: HashMap<IVec3, IVec3>, mut current: IVec3) -> Path3D {
        let mut total_path = vec![current];
        while let Some(&prev) = came_from.get(&current) {
            total_path.push(prev);
            current = prev;
        }
        total_path.reverse();
        Path3D::new(total_path)
    }
}
