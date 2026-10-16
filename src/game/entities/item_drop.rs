// REA Verified: net.minecraft.world.entity.item.ItemEntity (mapped: cbu.class)
// REA Verified: net.minecraft.client.renderer.entity.ItemEntityRenderer (mapped: fzj.class)
//
// Minecraft 1.20.4 Dropped Item Entities:
// - Authentic dimensions: 0.25m x 0.25m x 0.25m AABB box.
// - Authentic physics: 0.04 blocks/tick downward gravity, ground and air drag.
// - 3-Axis decoupled AABB collision resolution against voxel blocks.
// - Player magnetism: pulls items within 1.5 blocks toward player eye coordinate.
// - Pickup resolution: absorbs item within 0.5 blocks into inventory and triggers pop sound.
// - Visual bobbing & spinning: y = y0 + sin(ticks * 0.1) * 0.1 and continuous Y-axis rotation.

use glam::{IVec3, Vec3};
use crate::game::inventory::{ItemStack, PlayerInventory, ITEM_COBBLESTONE, ITEM_DIRT};
use crate::game::physics::Aabb;
use crate::render::atlas::BlockType;
use crate::render::mesher::BlockReader;

/// Standard item physics constants matching vanilla cbu.class
pub const ITEM_WIDTH: f32 = 0.25;
pub const ITEM_HEIGHT: f32 = 0.25;
pub const ITEM_GRAVITY: f32 = 0.04;
pub const ITEM_AIR_DRAG: f32 = 0.98;
pub const ITEM_GROUND_DRAG: f32 = 0.588; // 0.6 * 0.98
pub const MAGNETISM_RADIUS: f32 = 1.5;
pub const PICKUP_RADIUS: f32 = 0.5;
pub const DEFAULT_PICKUP_DELAY: u32 = 10;

/// Dropped item entity in the voxel world
#[derive(Debug, Clone)]
pub struct ItemEntity {
    pub pos: Vec3,
    pub velocity: Vec3,
    pub item: ItemStack,
    pub age: u32,
    pub pickup_delay: u32,
    pub on_ground: bool,
    pub initial_y: f32,
    pub bob_offset: f32,
    pub is_dead: bool,
}

impl ItemEntity {
    /// Creates a new ItemEntity with specified initial position and velocity
    pub fn new(pos: Vec3, velocity: Vec3, item: ItemStack) -> Self {
        let initial_y = pos.y;
        Self {
            pos,
            velocity,
            item,
            age: 0,
            pickup_delay: DEFAULT_PICKUP_DELAY,
            on_ground: false,
            initial_y,
            bob_offset: 0.0,
            is_dead: false,
        }
    }

    /// Spawns an ItemEntity at the block center when broken, matching vanilla:
    /// Initial upward and lateral velocity: (dx: [-0.1, 0.1], dy: 0.2, dz: [-0.1, 0.1])
    pub fn spawn_at_block_center(block_pos: IVec3, item: ItemStack, seed: u64) -> Self {
        let center = block_pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5);

        // Deterministic pseudo-random generation based on seed
        let mut s = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let r1 = ((s >> 33) as f32) / (u32::MAX as f32); // [0.0, 1.0]
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        let r2 = ((s >> 33) as f32) / (u32::MAX as f32);
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        let r3 = ((s >> 33) as f32) / (u32::MAX as f32);

        let dx = (r1 - 0.5) * 0.2; // [-0.1, 0.1]
        let dy = 0.2;             // 0.2 upward
        let dz = (r2 - 0.5) * 0.2; // [-0.1, 0.1]
        let bob_offset = r3 * std::f32::consts::TAU;

        // Position feet at center bottom of the 0.25m bounding box
        let pos = center - Vec3::new(0.0, ITEM_HEIGHT * 0.5, 0.0);

        Self {
            pos,
            velocity: Vec3::new(dx, dy, dz),
            item,
            age: 0,
            pickup_delay: DEFAULT_PICKUP_DELAY,
            on_ground: false,
            initial_y: center.y,
            bob_offset,
            is_dead: false,
        }
    }

    /// Returns the 0.25m x 0.25m AABB bounding box for voxel collision queries
    #[inline]
    pub fn bounding_box(&self) -> Aabb {
        let half_w = ITEM_WIDTH * 0.5;
        Aabb::new(
            Vec3::new(self.pos.x - half_w, self.pos.y, self.pos.z - half_w),
            Vec3::new(self.pos.x + half_w, self.pos.y + ITEM_HEIGHT, self.pos.z + half_w),
        )
    }

    /// Center point of the item entity in 3D world space
    #[inline]
    pub fn center(&self) -> Vec3 {
        Vec3::new(self.pos.x, self.pos.y + ITEM_HEIGHT * 0.5, self.pos.z)
    }

    /// Visual elevation with sinusoidal bobbing matching vanilla:
    /// y = y0 + sin(ticks * 0.1) * 0.1
    #[inline]
    pub fn visual_y(&self) -> f32 {
        let y0 = self.pos.y + ITEM_HEIGHT * 0.5;
        y0 + (self.age as f32 * 0.1 + self.bob_offset).sin() * 0.1
    }

    /// Continuous Y-axis rotation angle in radians for rendering
    #[inline]
    pub fn rotation_angle(&self) -> f32 {
        self.age as f32 * 0.05 + self.bob_offset
    }

    /// Applies 3-axis decoupled AABB collision against solid voxel blocks
    pub fn update_physics(&mut self, world: &dyn BlockReader) {
        // Apply vanilla item gravity: 0.04 blocks/tick downward
        self.velocity.y -= ITEM_GRAVITY;

        let cur_box = self.bounding_box();
        let query_box = cur_box.expand(self.velocity);

        let min_x = (query_box.min.x - 0.001).floor() as i32;
        let max_x = (query_box.max.x + 0.001).floor() as i32;
        let min_y = (query_box.min.y - 0.001).floor() as i32;
        let max_y = (query_box.max.y + 0.001).floor() as i32;
        let min_z = (query_box.min.z - 0.001).floor() as i32;
        let max_z = (query_box.max.z + 0.001).floor() as i32;

        let mut boxes = Vec::new();
        for y in min_y..=max_y {
            for z in min_z..=max_z {
                for x in min_x..=max_x {
                    if world.get_block_at(x, y, z).is_opaque() {
                        boxes.push(Aabb::from_block(x, y, z));
                    }
                }
            }
        }

        // 1. Vertical resolution (Y)
        let orig_vy = self.velocity.y;
        let mut mov_y = orig_vy;
        for b in &boxes {
            mov_y = b.calculate_max_offset_y(&cur_box, mov_y);
        }
        self.pos.y += mov_y;
        let hit_ground = orig_vy < 0.0 && mov_y > orig_vy;
        let hit_ceiling = orig_vy > 0.0 && mov_y < orig_vy;
        self.on_ground = hit_ground;
        if hit_ground || hit_ceiling {
            self.velocity.y = 0.0;
        }

        let after_y_box = cur_box.offset(Vec3::new(0.0, mov_y, 0.0));

        // 2. Horizontal resolution (X)
        let orig_vx = self.velocity.x;
        let mut mov_x = orig_vx;
        for b in &boxes {
            mov_x = b.calculate_max_offset_x(&after_y_box, mov_x);
        }
        self.pos.x += mov_x;
        if mov_x != orig_vx {
            self.velocity.x = 0.0;
        }

        let after_x_box = after_y_box.offset(Vec3::new(mov_x, 0.0, 0.0));

        // 3. Horizontal resolution (Z)
        let orig_vz = self.velocity.z;
        let mut mov_z = orig_vz;
        for b in &boxes {
            mov_z = b.calculate_max_offset_z(&after_x_box, mov_z);
        }
        self.pos.z += mov_z;
        if mov_z != orig_vz {
            self.velocity.z = 0.0;
        }

        // 4. Drag and friction
        if self.on_ground {
            self.velocity.x *= ITEM_GROUND_DRAG;
            self.velocity.z *= ITEM_GROUND_DRAG;
        } else {
            self.velocity.x *= ITEM_AIR_DRAG;
            self.velocity.z *= ITEM_AIR_DRAG;
            self.velocity.y *= ITEM_AIR_DRAG;
        }
    }

    /// Player magnetism: when within 1.5 blocks, interpolate velocity toward player eye coordinate
    pub fn update_magnetism(&mut self, player_eye: Vec3) {
        let to_player = player_eye - self.center();
        let dist = to_player.length();

        if dist < MAGNETISM_RADIUS && dist > 0.001 {
            let dir = to_player / dist;
            let target_vel = dir * 0.35;
            // Smoothly interpolate the drop velocity toward the player's eye coordinate
            self.velocity = self.velocity.lerp(target_vel, 0.25);
        }
    }

    /// Pickup resolution: when within 0.5 blocks, transfer item to player inventory
    /// Returns true if item was picked up (either fully or partially)
    pub fn try_pickup(&mut self, player_eye: Vec3, inventory: &mut PlayerInventory) -> bool {
        if self.pickup_delay > 0 || self.is_dead {
            return false;
        }

        let dist = (player_eye - self.center()).length();
        if dist <= PICKUP_RADIUS {
            match inventory.add_item(self.item) {
                None => {
                    self.is_dead = true;
                    true
                }
                Some(rem) => {
                    if rem.count < self.item.count {
                        self.item = rem;
                        true
                    } else {
                        false
                    }
                }
            }
        } else {
            false
        }
    }

    /// Advance entity state by 1 tick
    /// Returns true if an item pickup occurred during this tick
    pub fn tick(
        &mut self,
        player_eye: Vec3,
        inventory: &mut PlayerInventory,
        world: &dyn BlockReader,
    ) -> bool {
        if self.is_dead {
            return false;
        }

        if self.pickup_delay > 0 {
            self.pickup_delay -= 1;
        }

        self.age = self.age.wrapping_add(1);

        // 1. Apply player magnetism
        self.update_magnetism(player_eye);

        // 2. Physics & collision against voxels
        self.update_physics(world);

        // 3. Check pickup
        self.try_pickup(player_eye, inventory)
    }
}

/// Manager that tracks and updates active dropped item entities
#[derive(Debug, Default)]
pub struct ItemEntityManager {
    pub entities: Vec<ItemEntity>,
    rng_state: u64,
}

impl ItemEntityManager {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            rng_state: 987654321,
        }
    }

    /// Spawns a raw ItemEntity
    pub fn spawn(&mut self, entity: ItemEntity) {
        self.entities.push(entity);
    }

    /// Spawns the appropriate item drop when a block finishes breaking
    pub fn spawn_block_drop(&mut self, block_pos: IVec3, block: BlockType) {
        if block == BlockType::Air || block == BlockType::Bedrock {
            return;
        }

        let item = match block {
            BlockType::GrassBlock => ItemStack::new(ITEM_DIRT, 1),
            BlockType::Stone => ItemStack::new(ITEM_COBBLESTONE, 1),
            other => ItemStack::from_block(other, 1).unwrap_or(ItemStack::new(ITEM_DIRT, 1)),
        };

        self.rng_state = self.rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let entity = ItemEntity::spawn_at_block_center(block_pos, item, self.rng_state);
        self.entities.push(entity);
    }

    /// Updates all dropped item entities
    /// Returns the 3D positions where pickups occurred (for pop sound events)
    pub fn update(
        &mut self,
        player_eye: Vec3,
        inventory: &mut PlayerInventory,
        world: &dyn BlockReader,
    ) -> Vec<Vec3> {
        let mut pickup_positions = Vec::new();

        for entity in &mut self.entities {
            let pos = entity.center();
            if entity.tick(player_eye, inventory, world) {
                pickup_positions.push(pos);
            }
        }

        // Retain only living entities
        self.entities.retain(|e| !e.is_dead);

        pickup_positions
    }

    /// Returns reference to all currently active dropped items
    pub fn active_items(&self) -> &[ItemEntity] {
        &self.entities
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct DummyWorld {
        blocks: HashMap<IVec3, BlockType>,
    }

    impl BlockReader for DummyWorld {
        fn get_block_at(&self, x: i32, y: i32, z: i32) -> BlockType {
            self.blocks
                .get(&IVec3::new(x, y, z))
                .copied()
                .unwrap_or(BlockType::Air)
        }
    }

    #[test]
    fn test_spawn_at_block_center_velocity_bounds() {
        let item = ItemStack::new(ITEM_DIRT, 1);
        let block_pos = IVec3::new(10, 4, -5);
        let entity = ItemEntity::spawn_at_block_center(block_pos, item, 12345);

        // Center must be around block_pos + (0.5, 0.5, 0.5)
        let center = entity.center();
        assert!((center.x - 10.5).abs() < 1e-4);
        assert!((center.y - 4.5).abs() < 1e-4);
        assert!((center.z - -4.5).abs() < 1e-4);

        // Velocity bounds: dx: [-0.1, 0.1], dy: 0.2, dz: [-0.1, 0.1]
        assert!(entity.velocity.x >= -0.1 && entity.velocity.x <= 0.1);
        assert!((entity.velocity.y - 0.2).abs() < 1e-4);
        assert!(entity.velocity.z >= -0.1 && entity.velocity.z <= 0.1);
    }

    #[test]
    fn test_entity_aabb_dimensions() {
        let entity = ItemEntity::new(Vec3::new(0.0, 1.0, 0.0), Vec3::ZERO, ItemStack::new(ITEM_DIRT, 1));
        let bb = entity.bounding_box();

        assert!(((bb.max.x - bb.min.x) - 0.25).abs() < 1e-4);
        assert!(((bb.max.y - bb.min.y) - 0.25).abs() < 1e-4);
        assert!(((bb.max.z - bb.min.z) - 0.25).abs() < 1e-4);
    }

    #[test]
    fn test_entity_ground_collision_and_friction() {
        let mut world = DummyWorld {
            blocks: HashMap::new(),
        };
        // Floor at y = 0
        world.blocks.insert(IVec3::new(0, 0, 0), BlockType::Stone);

        let mut entity = ItemEntity::new(
            Vec3::new(0.5, 1.2, 0.5),
            Vec3::new(0.1, -0.1, 0.0),
            ItemStack::new(ITEM_DIRT, 1),
        );

        // Step physics until entity lands on floor at y = 1.0
        for _ in 0..10 {
            entity.update_physics(&world);
        }

        assert!(entity.on_ground, "Entity should have landed on the floor");
        assert!(entity.pos.y >= 1.0 - 1e-4, "Entity should be resting on top of y=0 block");
        assert!(entity.velocity.y.abs() < 1e-4, "Vertical velocity should be zero on ground");
        assert!(entity.velocity.x < 0.1, "Ground friction should dampen horizontal velocity");
    }

    #[test]
    fn test_player_magnetism_within_1_5_blocks() {
        let mut entity = ItemEntity::new(
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::ZERO,
            ItemStack::new(ITEM_DIRT, 1),
        );

        let player_eye = Vec3::new(0.8, 1.2, 0.0);
        let dist = (player_eye - entity.center()).length();
        assert!(dist < 1.5, "Player is within 1.5 blocks magnetism reach");

        entity.update_magnetism(player_eye);

        // Velocity should have interpolated toward player eye (+X, +Y)
        assert!(entity.velocity.x > 0.0, "Velocity should pull toward player");
        assert!(entity.velocity.y > 0.0, "Velocity should pull upward toward eye");
    }

    #[test]
    fn test_pickup_resolution_within_0_5_blocks() {
        let mut inv = PlayerInventory::new();
        // Clear hotbar & main for testing
        inv.hotbar = [None; 9];
        inv.main = [None; 27];

        let mut entity = ItemEntity::new(
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::ZERO,
            ItemStack::new(ITEM_DIRT, 5),
        );
        entity.pickup_delay = 0;

        let player_eye = Vec3::new(0.1, 1.1, 0.1);
        let dist = (player_eye - entity.center()).length();
        assert!(dist < 0.5, "Within pickup radius");

        let picked_up = entity.try_pickup(player_eye, &mut inv);
        assert!(picked_up, "Item should be picked up");
        assert!(entity.is_dead, "Entity should be marked dead after complete pickup");
        assert_eq!(inv.hotbar[0].unwrap().count, 5);
        assert_eq!(inv.hotbar[0].unwrap().id, ITEM_DIRT);
    }
}
