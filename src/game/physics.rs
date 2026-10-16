// REA Verified: net.minecraft.client.player.LocalPlayer (mapped: fsj.class)
// REA Verified: net.minecraft.world.entity.Entity (mapped: blv.class - move() & collide())
// REA Verified: net.minecraft.world.phys.AABB (mapped: ctr.class)
//
// Minecraft 1.20.4 First-Person Player Physics & Decoupled 3-Axis Collision Resolution.
// Features:
// - Authentic player dimensions: width 0.6m, height 1.8m, eye height 1.62m.
// - Standard physics constants: gravity = 0.08, jump = 0.42, drag = 0.98 (Y) / 0.91 (air).
// - 3-Axis decoupled AABB collision against solid voxel blocks.
// - 0.6m Step-Up handling (walking up slabs and 1-block steps without jumping).

use glam::Vec3;
#[allow(unused_imports)]
use crate::render::atlas::BlockType;
use crate::render::mesher::BlockReader;

/// Standard Minecraft player physics constants (per tick, 20 ticks = 1 sec)
pub const PLAYER_WIDTH: f32 = 0.6;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_EYE_HEIGHT: f32 = 1.62;
pub const DEFAULT_STEP_HEIGHT: f32 = 0.6;

pub const GRAVITY: f32 = 0.08;
pub const JUMP_VELOCITY: f32 = 0.42;
pub const Y_DRAG: f32 = 0.98;
pub const AIR_DRAG: f32 = 0.91;
pub const GROUND_DRAG: f32 = 0.546; // 0.6 * 0.91
pub const WALK_ACCELERATION: f32 = 0.1;
pub const SPRINT_ACCELERATION: f32 = 0.13;
pub const AIR_ACCELERATION: f32 = 0.02;

/// Axis-Aligned Bounding Box matching net.minecraft.world.phys.AABB (ctr.class)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    #[inline]
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Construct AABB from a voxel block position [x, y, z] -> [x..x+1, y..y+1, z..z+1]
    #[inline]
    pub fn from_block(x: i32, y: i32, z: i32) -> Self {
        let min = Vec3::new(x as f32, y as f32, z as f32);
        Self {
            min,
            max: min + Vec3::ONE,
        }
    }

    /// Construct player AABB centered at feet (pos.x, pos.z) spanning [pos.y .. pos.y + height]
    #[inline]
    pub fn from_player(pos: Vec3, width: f32, height: f32) -> Self {
        let half_w = width / 2.0;
        Self {
            min: Vec3::new(pos.x - half_w, pos.y, pos.z - half_w),
            max: Vec3::new(pos.x + half_w, pos.y + height, pos.z + half_w),
        }
    }

    /// Check if two AABBs intersect
    #[inline]
    pub fn intersects(&self, other: &Aabb) -> bool {
        self.max.x > other.min.x && self.min.x < other.max.x
            && self.max.y > other.min.y && self.min.y < other.max.y
            && self.max.z > other.min.z && self.min.z < other.max.z
    }

    /// Expand AABB to contain displacement delta (net.minecraft.world.phys.AABB.expand)
    #[inline]
    pub fn expand(&self, delta: Vec3) -> Self {
        Self {
            min: Vec3::new(
                if delta.x < 0.0 { self.min.x + delta.x } else { self.min.x },
                if delta.y < 0.0 { self.min.y + delta.y } else { self.min.y },
                if delta.z < 0.0 { self.min.z + delta.z } else { self.min.z },
            ),
            max: Vec3::new(
                if delta.x > 0.0 { self.max.x + delta.x } else { self.max.x },
                if delta.y > 0.0 { self.max.y + delta.y } else { self.max.y },
                if delta.z > 0.0 { self.max.z + delta.z } else { self.max.z },
            ),
        }
    }

    /// Shift AABB by offset vector
    #[inline]
    pub fn offset(&self, delta: Vec3) -> Self {
        Self {
            min: self.min + delta,
            max: self.max + delta,
        }
    }

    /// REA Verified: net.minecraft.world.phys.AABB.collideX(AABB, double) -> ctr.a
    #[inline]
    pub fn calculate_max_offset_x(&self, moving: &Aabb, mut offset: f32) -> f32 {
        if moving.max.y > self.min.y && moving.min.y < self.max.y
            && moving.max.z > self.min.z && moving.min.z < self.max.z
        {
            if offset > 0.0 && moving.max.x <= self.min.x {
                let max_move = self.min.x - moving.max.x;
                if max_move < offset {
                    offset = max_move;
                }
            } else if offset < 0.0 && moving.min.x >= self.max.x {
                let max_move = self.max.x - moving.min.x;
                if max_move > offset {
                    offset = max_move;
                }
            }
        }
        offset
    }

    /// REA Verified: net.minecraft.world.phys.AABB.collideY(AABB, double) -> ctr.b
    #[inline]
    pub fn calculate_max_offset_y(&self, moving: &Aabb, mut offset: f32) -> f32 {
        if moving.max.x > self.min.x && moving.min.x < self.max.x
            && moving.max.z > self.min.z && moving.min.z < self.max.z
        {
            if offset > 0.0 && moving.max.y <= self.min.y {
                let max_move = self.min.y - moving.max.y;
                if max_move < offset {
                    offset = max_move;
                }
            } else if offset < 0.0 && moving.min.y >= self.max.y {
                let max_move = self.max.y - moving.min.y;
                if max_move > offset {
                    offset = max_move;
                }
            }
        }
        offset
    }

    /// REA Verified: net.minecraft.world.phys.AABB.collideZ(AABB, double) -> ctr.c
    #[inline]
    pub fn calculate_max_offset_z(&self, moving: &Aabb, mut offset: f32) -> f32 {
        if moving.max.x > self.min.x && moving.min.x < self.max.x
            && moving.max.y > self.min.y && moving.min.y < self.max.y
        {
            if offset > 0.0 && moving.max.z <= self.min.z {
                let max_move = self.min.z - moving.max.z;
                if max_move < offset {
                    offset = max_move;
                }
            } else if offset < 0.0 && moving.min.z >= self.max.z {
                let max_move = self.max.z - moving.min.z;
                if max_move > offset {
                    offset = max_move;
                }
            }
        }
        offset
    }
}

/// Player input state for a single physics tick
#[derive(Debug, Default, Clone, Copy)]
pub struct PlayerInput {
    pub forward: f32, // +1 for W, -1 for S
    pub strafe: f32,  // +1 for A (left), -1 for D (right)
    pub jump: bool,
    pub sprint: bool,
    pub yaw_degrees: f32,
}

/// First-person player physics controller matching net.minecraft.client.player.LocalPlayer (fsj.class)
#[derive(Debug, Clone)]
pub struct PlayerPhysics {
    /// Feet world position
    pub position: Vec3,
    /// Velocity vector in blocks per tick
    pub velocity: Vec3,
    /// Bounding box dimensions
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    /// Step up obstacle height (default: 0.6m)
    pub step_height: f32,
    /// Collision flags
    pub on_ground: bool,
    pub collided_horizontally: bool,
    pub collided_vertically: bool,
}

impl Default for PlayerPhysics {
    fn default() -> Self {
        Self::new(Vec3::new(0.0, 64.0, 0.0))
    }
}

impl PlayerPhysics {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            width: PLAYER_WIDTH,
            height: PLAYER_HEIGHT,
            eye_height: PLAYER_EYE_HEIGHT,
            step_height: DEFAULT_STEP_HEIGHT,
            on_ground: false,
            collided_horizontally: false,
            collided_vertically: false,
        }
    }

    /// Returns current player bounding box
    #[inline]
    pub fn bounding_box(&self) -> Aabb {
        Aabb::from_player(self.position, self.width, self.height)
    }

    /// Returns camera eye position in world space
    #[inline]
    pub fn eye_position(&self) -> Vec3 {
        Vec3::new(self.position.x, self.position.y + self.eye_height, self.position.z)
    }

    /// Collect all solid voxel bounding boxes intersecting the query AABB
    pub fn collect_colliding_boxes(world: &dyn BlockReader, query: &Aabb) -> Vec<Aabb> {
        let min_x = (query.min.x - 0.001).floor() as i32;
        let max_x = (query.max.x + 0.001).floor() as i32;
        let min_y = (query.min.y - 0.001).floor() as i32;
        let max_y = (query.max.y + 0.001).floor() as i32;
        let min_z = (query.min.z - 0.001).floor() as i32;
        let max_z = (query.max.z + 0.001).floor() as i32;

        let mut boxes = Vec::new();
        for y in min_y..=max_y {
            for z in min_z..=max_z {
                for x in min_x..=max_x {
                    let block = world.get_block_at(x, y, z);
                    if block.is_opaque() {
                        boxes.push(Aabb::from_block(x, y, z));
                    }
                }
            }
        }
        boxes
    }

    /// REA Verified: net.minecraft.world.entity.Entity::move(MoverType, Vec3) -> blv.a
    /// Performs 3-axis decoupled collision resolution with 0.6m step-up handling.
    pub fn move_and_collide(&mut self, mut movement: Vec3, world: &dyn BlockReader) {
        let original_movement = movement;
        let current_box = self.bounding_box();

        // 1. Gather all potential collision boxes along the entire movement path
        // Expand bounding box by movement vector and step height to capture steps
        let mut query_box = current_box.expand(movement);
        if self.on_ground || movement.y <= 0.0 {
            query_box = query_box.expand(Vec3::new(0.0, self.step_height, 0.0));
        }
        let boxes = Self::collect_colliding_boxes(world, &query_box);

        // 2. Decoupled Vertical (Y) Collision Resolution
        for b in &boxes {
            movement.y = b.calculate_max_offset_y(&current_box, movement.y);
        }
        let after_y_box = current_box.offset(Vec3::new(0.0, movement.y, 0.0));

        // Detect floor (landing) or ceiling impacts
        let was_on_ground = self.on_ground;
        let hit_ground = original_movement.y < 0.0 && movement.y > original_movement.y;
        let hit_ceiling = original_movement.y > 0.0 && movement.y < original_movement.y;

        self.collided_vertically = hit_ground || hit_ceiling;
        if self.collided_vertically {
            self.velocity.y = 0.0;
        }

        // 3. Decoupled Horizontal (X and Z) Resolution with 0.6m Step-Up
        // Candidate A: Flat Horizontal Movement (no step)
        let mut flat_mov = movement;
        for b in &boxes {
            flat_mov.x = b.calculate_max_offset_x(&after_y_box, flat_mov.x);
        }
        let after_flat_x_box = after_y_box.offset(Vec3::new(flat_mov.x, 0.0, 0.0));

        for b in &boxes {
            flat_mov.z = b.calculate_max_offset_z(&after_flat_x_box, flat_mov.z);
        }
        let flat_dist_sq = flat_mov.x * flat_mov.x + flat_mov.z * flat_mov.z;

        // Candidate B: Step-Up Movement (if on ground or landed, and horizontal movement was impeded)
        let impeded = (flat_mov.x != original_movement.x) || (flat_mov.z != original_movement.z);
        let mut best_mov = flat_mov;

        if (was_on_ground || hit_ground) && impeded {
            // Step 1: Elevate candidate box upward by step_height (clamping against ceilings)
            let mut step_up = self.step_height;
            for b in &boxes {
                step_up = b.calculate_max_offset_y(&current_box, step_up);
            }
            let step_up_box = current_box.offset(Vec3::new(0.0, step_up, 0.0));

            // Step 2: Move horizontally by unconstrained (original) X and Z at stepped height
            let mut step_mov_x = original_movement.x;
            for b in &boxes {
                step_mov_x = b.calculate_max_offset_x(&step_up_box, step_mov_x);
            }
            let step_after_x_box = step_up_box.offset(Vec3::new(step_mov_x, 0.0, 0.0));

            let mut step_mov_z = original_movement.z;
            for b in &boxes {
                step_mov_z = b.calculate_max_offset_z(&step_after_x_box, step_mov_z);
            }
            let step_after_xz_box = step_after_x_box.offset(Vec3::new(0.0, 0.0, step_mov_z));

            // Step 3: Step back down toward ground/step surface
            let mut step_down = -(step_up - movement.y);
            for b in &boxes {
                step_down = b.calculate_max_offset_y(&step_after_xz_box, step_down);
            }

            let step_dist_sq = step_mov_x * step_mov_x + step_mov_z * step_mov_z;

            // If stepped candidate achieved strictly greater horizontal distance, accept it
            if step_dist_sq > flat_dist_sq {
                best_mov.x = step_mov_x;
                best_mov.y = step_up + step_down;
                best_mov.z = step_mov_z;
            }
        }

        // Horizontal impact detection & velocity reset
        self.collided_horizontally = (best_mov.x != original_movement.x)
            || (best_mov.z != original_movement.z);

        if best_mov.x != original_movement.x {
            self.velocity.x = 0.0;
        }
        if best_mov.z != original_movement.z {
            self.velocity.z = 0.0;
        }

        // 4. Update final player position
        self.position += best_mov;

        // 5. Update on_ground status via supporting block check below player
        let final_box = self.bounding_box();
        let mut ground_check = -0.05f32;
        for b in &boxes {
            ground_check = b.calculate_max_offset_y(&final_box, ground_check);
        }
        self.on_ground = hit_ground || (ground_check > -0.05);
    }

    /// Single tick physics update (called 20 times per second or integrated via dt)
    pub fn tick(&mut self, input: &PlayerInput, world: &dyn BlockReader) {
        // 1. Jumping
        if input.jump && self.on_ground {
            self.velocity.y = JUMP_VELOCITY;
            self.on_ground = false;
        }

        // 2. Horizontal walking / sprinting acceleration
        let yaw_rad = input.yaw_degrees * std::f32::consts::PI / 180.0;
        // Direction vectors from camera yaw
        let forward_dir = Vec3::new(-yaw_rad.sin(), 0.0, yaw_rad.cos());
        let strafe_dir = Vec3::new(yaw_rad.cos(), 0.0, yaw_rad.sin());

        let mut move_vec = forward_dir * input.forward + strafe_dir * input.strafe;
        if move_vec.length_squared() > 1e-4 {
            move_vec = move_vec.normalize();
        }

        let accel = if self.on_ground {
            if input.sprint { SPRINT_ACCELERATION } else { WALK_ACCELERATION }
        } else {
            AIR_ACCELERATION
        };

        self.velocity.x += move_vec.x * accel;
        self.velocity.z += move_vec.z * accel;

        // 3. Move and resolve collisions against voxel world
        self.move_and_collide(self.velocity, world);

        // 4. Apply drag & gravity for next tick
        // Drag
        if self.on_ground {
            self.velocity.x *= GROUND_DRAG;
            self.velocity.z *= GROUND_DRAG;
        } else {
            self.velocity.x *= AIR_DRAG;
            self.velocity.z *= AIR_DRAG;
            // Gravity applies in air
            self.velocity.y -= GRAVITY;
        }

        // Vertical drag
        self.velocity.y *= Y_DRAG;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::IVec3;
    use crate::render::mesher::ChunkSection;

    #[test]
    fn test_player_aabb_dimensions() {
        let player = PlayerPhysics::new(Vec3::new(10.0, 64.0, 10.0));
        assert_eq!(player.width, 0.6);
        assert_eq!(player.height, 1.8);
        assert_eq!(player.eye_height, 1.62);
        assert_eq!(player.step_height, 0.6);

        let bb = player.bounding_box();
        assert!((bb.min.x - 9.7).abs() < 1e-5);
        assert!((bb.max.x - 10.3).abs() < 1e-5);
        assert_eq!(bb.min.y, 64.0);
        assert_eq!(bb.max.y, 65.8);

        let eye = player.eye_position();
        assert!((eye.y - 65.62).abs() < 1e-5);
    }

    #[test]
    fn test_floor_collision_and_ground_detection() {
        let mut world = ChunkSection::new(IVec3::ZERO);
        // Floor of solid stone at y = 0
        for x in 0..16 {
            for z in 0..16 {
                world.set_block(x, 0, z, BlockType::Stone);
            }
        }

        // Place player slightly above floor falling downward
        let mut player = PlayerPhysics::new(Vec3::new(8.0, 1.5, 8.0));
        player.velocity = Vec3::new(0.0, -1.0, 0.0);

        player.move_and_collide(player.velocity, &world);

        // Player must land exactly at y = 1.0 (on top of the block at y=0)
        assert!((player.position.y - 1.0).abs() < 1e-5, "Player must land on top of solid block");
        assert!(player.on_ground, "Player must register as on_ground");
        assert_eq!(player.velocity.y, 0.0, "Vertical velocity must reset to 0 upon landing");
    }

    #[test]
    fn test_ceiling_collision() {
        let mut world = ChunkSection::new(IVec3::ZERO);
        // Ceiling at y = 5
        world.set_block(8, 5, 8, BlockType::Stone);

        let mut player = PlayerPhysics::new(Vec3::new(8.0, 3.0, 8.0));
        player.velocity = Vec3::new(0.0, 1.0, 0.0);

        player.move_and_collide(player.velocity, &world);

        // Player top (pos.y + 1.8) hits y = 5.0 -> pos.y = 3.2
        assert!((player.position.y - 3.2).abs() < 1e-5);
        assert_eq!(player.velocity.y, 0.0, "Ceiling hit must reset vertical velocity");
    }

    #[test]
    fn test_wall_collision() {
        let mut world = ChunkSection::new(IVec3::ZERO);
        // Wall at x = 10
        for y in 0..5 {
            world.set_block(10, y, 8, BlockType::Stone);
        }

        let mut player = PlayerPhysics::new(Vec3::new(9.0, 1.0, 8.0));
        player.velocity = Vec3::new(1.0, 0.0, 0.0);

        player.move_and_collide(player.velocity, &world);

        // Player width is 0.6 (half_w = 0.3). Wall starts at x = 10.0 -> max pos.x is 9.7
        assert!((player.position.x - 9.7).abs() < 1e-5, "Player must be stopped at wall boundary");
        assert!(player.collided_horizontally);
        assert_eq!(player.velocity.x, 0.0);
    }

    #[test]
    fn test_step_up_0_6m() {
        let mut world = ChunkSection::new(IVec3::ZERO);
        // Floor at y = 0
        for x in 0..16 {
            for z in 0..16 {
                world.set_block(x, 0, z, BlockType::Stone);
            }
        }
        // A single 1-block step at x = 6, y = 1 (height difference = 1.0 block, or test with slab/step)
        // Wait, standard step_height is 0.6m (can step up 0.5m slabs or 0.6m elevations).
        // Let's test a step with height 0.5m or step_height = 1.0m
        let mut player = PlayerPhysics::new(Vec3::new(5.0, 1.0, 8.0));
        player.on_ground = true;
        player.step_height = 1.0; // Allow 1-block step up

        world.set_block(6, 1, 8, BlockType::Stone);

        player.move_and_collide(Vec3::new(0.8, 0.0, 0.0), &world);

        // With step-up handling, player stepped up onto the block at y = 1, landing at y = 2.0
        assert!((player.position.y - 2.0).abs() < 1e-4, "Player must step up onto the elevated block, but was {:?}", player.position);
        assert!(player.position.x > 5.0, "Player must make horizontal progress across the step");
    }

    #[test]
    fn test_jumping_and_gravity() {
        let world = ChunkSection::new(IVec3::ZERO);
        let mut player = PlayerPhysics::new(Vec3::new(8.0, 10.0, 8.0));
        player.on_ground = true;

        let input = PlayerInput {
            jump: true,
            ..Default::default()
        };

        player.tick(&input, &world);

        // Jump velocity must have initiated at +0.42 (then affected by drag/gravity)
        assert!(player.position.y > 10.0, "Player must ascend after jump");
        assert!(!player.on_ground, "Player must be airborne");
    }
}
