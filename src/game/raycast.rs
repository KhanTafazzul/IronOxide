// REA Verified: net.minecraft.world.phys.BlockHitResult (mapped: elp.class)
// REA Verified: net.minecraft.world.level.BlockGetter::clip(ClipContext) (3D Raycast DDA)
//
// 3D Fast Voxel Traversal Algorithm (Amanatides-Woo DDA) for Voxel Selection and Block Interaction.
// Casts rays from player/camera eye position along forward look vector up to 5.0 blocks reach.

use glam::{IVec3, Vec3};
use crate::render::atlas::Direction;
use crate::render::mesher::{direction_to_normal, BlockReader};

pub const DEFAULT_MAX_REACH: f32 = 5.0;

/// Result of a successful voxel raycast hit
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaycastHit {
    /// Integer grid coordinates of the targeted block
    pub block_pos: IVec3,
    /// Targeted face direction (UP, DOWN, NORTH, SOUTH, WEST, EAST)
    pub face: Direction,
    /// Exact world coordinates of the intersection point
    pub hit_point: Vec3,
    /// Distance from ray origin to hit point
    pub distance: f32,
}

impl RaycastHit {
    pub fn new(block_pos: IVec3, face: Direction, hit_point: Vec3, distance: f32) -> Self {
        Self {
            block_pos,
            face,
            hit_point,
            distance,
        }
    }

    /// Block position where a new block should be placed against this face
    #[inline]
    pub fn place_pos(&self) -> IVec3 {
        let n = direction_to_normal(self.face);
        self.block_pos + IVec3::new(n[0], n[1], n[2])
    }
}

/// 3D Fast Voxel Traversal (Amanatides-Woo DDA)
/// Traverses discrete voxel grid cells along a continuous 3D ray in $O(\Delta x + \Delta y + \Delta z)$ time.
pub fn raycast_voxel(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    world: &dyn BlockReader,
) -> Option<RaycastHit> {
    if direction.length_squared() < 1e-6 || max_distance <= 0.0 {
        return None;
    }

    let dir = direction.normalize();

    // Current integer voxel coordinates
    let mut x = origin.x.floor() as i32;
    let mut y = origin.y.floor() as i32;
    let mut z = origin.z.floor() as i32;

    // Check if origin is already inside a solid block
    if world.get_block_at(x, y, z).is_opaque() {
        // Default face opposing ray direction
        let opposing_face = if dir.y.abs() > dir.x.abs().max(dir.z.abs()) {
            if dir.y > 0.0 { Direction::Down } else { Direction::Up }
        } else if dir.x.abs() > dir.z.abs() {
            if dir.x > 0.0 { Direction::West } else { Direction::East }
        } else {
            if dir.z > 0.0 { Direction::North } else { Direction::South }
        };

        return Some(RaycastHit {
            block_pos: IVec3::new(x, y, z),
            face: opposing_face,
            hit_point: origin,
            distance: 0.0,
        });
    }

    // Step direction along each axis (+1, -1, or 0)
    let step_x = if dir.x > 0.0 { 1 } else if dir.x < 0.0 { -1 } else { 0 };
    let step_y = if dir.y > 0.0 { 1 } else if dir.y < 0.0 { -1 } else { 0 };
    let step_z = if dir.z > 0.0 { 1 } else if dir.z < 0.0 { -1 } else { 0 };

    // Delta t (distance traveled along ray for coordinate to increment by 1 voxel)
    let t_delta_x = if step_x != 0 { (1.0 / dir.x).abs() } else { f32::INFINITY };
    let t_delta_y = if step_y != 0 { (1.0 / dir.y).abs() } else { f32::INFINITY };
    let t_delta_z = if step_z != 0 { (1.0 / dir.z).abs() } else { f32::INFINITY };

    // Initial distance to first voxel boundary along each axis
    let mut t_max_x = if step_x > 0 {
        ((x as f32 + 1.0) - origin.x) * t_delta_x
    } else if step_x < 0 {
        (origin.x - x as f32) * t_delta_x
    } else {
        f32::INFINITY
    };

    let mut t_max_y = if step_y > 0 {
        ((y as f32 + 1.0) - origin.y) * t_delta_y
    } else if step_y < 0 {
        (origin.y - y as f32) * t_delta_y
    } else {
        f32::INFINITY
    };

    let mut t_max_z = if step_z > 0 {
        ((z as f32 + 1.0) - origin.z) * t_delta_z
    } else if step_z < 0 {
        (origin.z - z as f32) * t_delta_z
    } else {
        f32::INFINITY
    };

    let mut last_face;
    let mut t = 0.0f32;

    while t <= max_distance {
        // Advance to the nearest boundary intersection
        if t_max_x < t_max_y {
            if t_max_x < t_max_z {
                x += step_x;
                t = t_max_x;
                t_max_x += t_delta_x;
                last_face = if step_x > 0 { Direction::West } else { Direction::East };
            } else {
                z += step_z;
                t = t_max_z;
                t_max_z += t_delta_z;
                last_face = if step_z > 0 { Direction::North } else { Direction::South };
            }
        } else {
            if t_max_y < t_max_z {
                y += step_y;
                t = t_max_y;
                t_max_y += t_delta_y;
                last_face = if step_y > 0 { Direction::Down } else { Direction::Up };
            } else {
                z += step_z;
                t = t_max_z;
                t_max_z += t_delta_z;
                last_face = if step_z > 0 { Direction::North } else { Direction::South };
            }
        }

        if t > max_distance {
            break;
        }

        // Check if targeted voxel is solid
        if world.get_block_at(x, y, z).is_opaque() {
            let hit_point = origin + dir * t;
            return Some(RaycastHit {
                block_pos: IVec3::new(x, y, z),
                face: last_face,
                hit_point,
                distance: t,
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::atlas::BlockType;
    use crate::render::mesher::ChunkSection;

    #[test]
    fn test_raycast_horizontal_hit() {
        let mut world = ChunkSection::new(IVec3::ZERO);
        // Stone block placed at (5, 1, 5)
        world.set_block(5, 1, 5, BlockType::Stone);

        // Ray cast from (2.5, 1.5, 5.5) pointing in +X direction (1.0, 0.0, 0.0)
        let origin = Vec3::new(2.5, 1.5, 5.5);
        let direction = Vec3::new(1.0, 0.0, 0.0);

        let hit = raycast_voxel(origin, direction, 5.0, &world).expect("Should hit block at x=5");

        assert_eq!(hit.block_pos, IVec3::new(5, 1, 5));
        // Stepping +X hits the West (-X) face of the block
        assert_eq!(hit.face, Direction::West);
        assert!((hit.distance - 2.5).abs() < 1e-4);
        assert_eq!(hit.place_pos(), IVec3::new(4, 1, 5));
    }

    #[test]
    fn test_raycast_vertical_floor_hit() {
        let mut world = ChunkSection::new(IVec3::ZERO);
        // Floor block at (4, 0, 4)
        world.set_block(4, 0, 4, BlockType::GrassBlock);

        // Ray cast downward from (4.5, 3.5, 4.5) pointing in -Y direction
        let origin = Vec3::new(4.5, 3.5, 4.5);
        let direction = Vec3::new(0.0, -1.0, 0.0);

        let hit = raycast_voxel(origin, direction, 5.0, &world).expect("Should hit floor");

        assert_eq!(hit.block_pos, IVec3::new(4, 0, 4));
        assert_eq!(hit.face, Direction::Up); // Top face
        assert!((hit.distance - 2.5).abs() < 1e-4);
        assert_eq!(hit.place_pos(), IVec3::new(4, 1, 4));
    }

    #[test]
    fn test_raycast_out_of_reach() {
        let mut world = ChunkSection::new(IVec3::ZERO);
        // Block placed at x = 10, reach is 5.0 from x = 0
        world.set_block(10, 0, 0, BlockType::Stone);

        let origin = Vec3::new(0.5, 0.5, 0.5);
        let direction = Vec3::new(1.0, 0.0, 0.0);

        let hit = raycast_voxel(origin, direction, 5.0, &world);
        assert!(hit.is_none(), "Block at 10.0 should be out of 5.0 reach");
    }

    #[test]
    fn test_raycast_passes_air() {
        let mut world = ChunkSection::new(IVec3::ZERO);
        // Air at (1, 0, 0), (2, 0, 0), Stone at (3, 0, 0)
        world.set_block(3, 0, 0, BlockType::Stone);

        let origin = Vec3::new(0.5, 0.5, 0.5);
        let direction = Vec3::new(1.0, 0.0, 0.0);

        let hit = raycast_voxel(origin, direction, 5.0, &world).expect("Should hit stone");
        assert_eq!(hit.block_pos, IVec3::new(3, 0, 0));
        assert_eq!(hit.face, Direction::West);
    }
}
