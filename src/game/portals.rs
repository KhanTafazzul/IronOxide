// REA Verified: net.minecraft.world.level.portal.PortalForcer (mapped: efn.class)
// REA Verified: net.minecraft.world.level.portal.PortalShape (mapped: efp.class)
// REA Verified: net.minecraft.world.level.block.NetherPortalBlock (mapped: dbt.class)
// REA Verified: net.minecraft.world.level.block.EndPortalBlock (mapped: cze.class)
// REA Verified: net.minecraft.world.level.block.EndPortalFrameBlock (mapped: czf.class)
//
// Nether & End Portal Linking Engine:
// - Nether Portal Shape Detection: 4x5 to 23x23 Obsidian rectangle (2x3 to 21x21 inner opening)
// - Lighting: Flint & Steel lights internal Air into NetherPortal blocks
// - Coordinate Translation: Overworld <-> Nether (8:1 ratio)
// - Portal Search & Forcer: 128-block radius search, dynamic 4x5 Obsidian frame construction with safe air pocket
// - Teleportation Timer: 80 ticks Survival, 0 ticks Creative, with purple swirling distortion and nausea warp
// - End Portal Mechanics: 12-block frame ring, 3x3 EndPortal area, fixed spawn platform at (100, 49, 0)

use glam::{IVec3, Vec3};
use crate::world::block_registry::BlockId;

/// Minecraft Dimension Classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Dimension {
    #[default]
    Overworld,
    Nether,
    TheEnd,
}

impl Dimension {
    /// Coordinate scale ratio relative to Overworld
    #[inline]
    pub fn coordinate_scale(&self) -> f64 {
        match self {
            Dimension::Overworld => 1.0,
            Dimension::Nether => 0.125, // 1 / 8
            Dimension::TheEnd => 1.0,
        }
    }

    /// Minimum valid Y coordinate for portal creation and player teleportation
    #[inline]
    pub fn min_y(&self) -> i32 {
        match self {
            Dimension::Overworld => -64,
            Dimension::Nether => 0,
            Dimension::TheEnd => 0,
        }
    }

    /// Maximum valid Y coordinate for portal creation and player teleportation
    #[inline]
    pub fn max_y(&self) -> i32 {
        match self {
            Dimension::Overworld => 320,
            Dimension::Nether => 128,
            Dimension::TheEnd => 128,
        }
    }

    /// Vanilla namespaced identifier
    #[inline]
    pub fn vanilla_identifier(&self) -> &'static str {
        match self {
            Dimension::Overworld => "minecraft:overworld",
            Dimension::Nether => "minecraft:the_nether",
            Dimension::TheEnd => "minecraft:the_end",
        }
    }

    /// Ambient atmospheric fog color (RGB in 0..1 range)
    pub fn ambient_fog_color(&self) -> [f32; 3] {
        match self {
            Dimension::Overworld => [0.75, 0.85, 1.0],
            Dimension::Nether => [0.20, 0.03, 0.03],
            Dimension::TheEnd => [0.08, 0.05, 0.10],
        }
    }
}

/// Alignment axis of a Nether Portal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortalAxis {
    /// Extends East-West along the X axis (portal face faces North/South)
    X,
    /// Extends North-South along the Z axis (portal face faces East/West)
    Z,
}

impl PortalAxis {
    /// Direction vector along the portal opening width
    #[inline]
    pub fn width_direction(&self) -> IVec3 {
        match self {
            PortalAxis::X => IVec3::new(1, 0, 0),
            PortalAxis::Z => IVec3::new(0, 0, 1),
        }
    }

    /// Direction vector perpendicular to the portal face
    #[inline]
    pub fn normal_direction(&self) -> IVec3 {
        match self {
            PortalAxis::X => IVec3::new(0, 0, 1),
            PortalAxis::Z => IVec3::new(1, 0, 0),
        }
    }
}

/// Reverse-Engineered Nether Portal Shape Detector matching efp.class (PortalShape.java)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetherPortalShape {
    /// Inner bottom-left corner of the portal opening (where portal blocks reside)
    pub bottom_left: IVec3,
    /// Orientation axis of the portal
    pub axis: PortalAxis,
    /// Inner width in blocks (vanilla MIN_WIDTH = 2, MAX_WIDTH = 21)
    pub width: i32,
    /// Inner height in blocks (vanilla MIN_HEIGHT = 3, MAX_HEIGHT = 21)
    pub height: i32,
}

impl NetherPortalShape {
    // REA Constants extracted from efp.class
    pub const MIN_WIDTH: i32 = 2;
    pub const MAX_WIDTH: i32 = 21;
    pub const MIN_HEIGHT: i32 = 3;
    pub const MAX_HEIGHT: i32 = 21;

    /// Attempts to detect and validate a Nether Portal frame containing or adjacent to `start_pos`
    pub fn detect<F: Fn(IVec3) -> BlockId>(start_pos: IVec3, get_block: &F) -> Option<Self> {
        // Test along X axis first, then Z axis
        if let Some(shape) = Self::detect_along_axis(start_pos, PortalAxis::X, get_block) {
            return Some(shape);
        }
        if let Some(shape) = Self::detect_along_axis(start_pos, PortalAxis::Z, get_block) {
            return Some(shape);
        }
        None
    }

    /// Detects a portal frame aligned with a specific axis
    pub fn detect_along_axis<F: Fn(IVec3) -> BlockId>(
        start_pos: IVec3,
        axis: PortalAxis,
        get_block: &F,
    ) -> Option<Self> {
        let dir = axis.width_direction();

        // 1. If start_pos is on an obsidian block, look for adjacent air/portal inside space
        let mut inner_pos = start_pos;
        if get_block(inner_pos) == BlockId::Obsidian {
            let candidates = [
                start_pos + IVec3::new(0, 1, 0),
                start_pos - dir,
                start_pos + dir,
                start_pos - IVec3::new(0, 1, 0),
            ];
            let mut found = false;
            for c in candidates {
                let b = get_block(c);
                if b == BlockId::Air || b == BlockId::NetherPortal {
                    inner_pos = c;
                    found = true;
                    break;
                }
            }
            if !found {
                return None;
            }
        }

        // 2. Scan down along -Y to find the bottom inner row (directly above bottom Obsidian frame)
        let mut cur_y = inner_pos.y;
        while cur_y > -64 {
            let below = IVec3::new(inner_pos.x, cur_y - 1, inner_pos.z);
            let b = get_block(below);
            if b == BlockId::Obsidian {
                break;
            }
            if b != BlockId::Air && b != BlockId::NetherPortal {
                return None;
            }
            cur_y -= 1;
            if inner_pos.y - cur_y > Self::MAX_HEIGHT {
                return None;
            }
        }

        let bottom_y = cur_y;

        // 3. Scan in negative width direction (-dir) to find the left inner column (directly adjacent to left Obsidian frame)
        let mut offset = 0;
        while offset <= Self::MAX_WIDTH {
            let check_pos = IVec3::new(inner_pos.x, bottom_y, inner_pos.z) - dir * (offset + 1);
            let b = get_block(check_pos);
            if b == BlockId::Obsidian {
                break;
            }
            if b != BlockId::Air && b != BlockId::NetherPortal {
                return None;
            }
            offset += 1;
        }

        if offset > Self::MAX_WIDTH {
            return None;
        }

        let bottom_left = IVec3::new(inner_pos.x, bottom_y, inner_pos.z) - dir * offset;

        // 4. Measure inner width along +dir
        let mut width = 0;
        while width <= Self::MAX_WIDTH {
            let pos = bottom_left + dir * width;
            let b = get_block(pos);
            if b == BlockId::Obsidian {
                break;
            }
            if b != BlockId::Air && b != BlockId::NetherPortal {
                return None;
            }
            // Check that the block below is Obsidian
            let below = pos - IVec3::new(0, 1, 0);
            if get_block(below) != BlockId::Obsidian {
                return None;
            }
            width += 1;
        }

        if width < Self::MIN_WIDTH || width > Self::MAX_WIDTH {
            return None;
        }

        // 5. Measure inner height along +Y
        let mut height = 0;
        while height <= Self::MAX_HEIGHT {
            // Check all columns across this row
            let mut row_is_obsidian_top = false;
            let mut row_is_inner_air = false;

            for w in 0..width {
                let pos = bottom_left + dir * w + IVec3::new(0, height, 0);
                let b = get_block(pos);
                if b == BlockId::Obsidian {
                    row_is_obsidian_top = true;
                } else if b == BlockId::Air || b == BlockId::NetherPortal {
                    row_is_inner_air = true;
                } else {
                    return None;
                }
            }

            if row_is_obsidian_top {
                // If this is the top frame, ALL blocks in this row must be Obsidian
                for w in 0..width {
                    let pos = bottom_left + dir * w + IVec3::new(0, height, 0);
                    if get_block(pos) != BlockId::Obsidian {
                        return None;
                    }
                }
                break;
            }

            if !row_is_inner_air {
                return None;
            }

            // Left pillar check at this height
            let left_pos = bottom_left - dir + IVec3::new(0, height, 0);
            if get_block(left_pos) != BlockId::Obsidian {
                return None;
            }

            // Right pillar check at this height
            let right_pos = bottom_left + dir * width + IVec3::new(0, height, 0);
            if get_block(right_pos) != BlockId::Obsidian {
                return None;
            }

            height += 1;
        }

        if height < Self::MIN_HEIGHT || height > Self::MAX_HEIGHT {
            return None;
        }

        Some(Self {
            bottom_left,
            axis,
            width,
            height,
        })
    }

    /// Returns all coordinates inside the portal opening
    pub fn inner_blocks(&self) -> Vec<IVec3> {
        let dir = self.axis.width_direction();
        let mut blocks = Vec::with_capacity((self.width * self.height) as usize);
        for y in 0..self.height {
            for w in 0..self.width {
                blocks.push(self.bottom_left + dir * w + IVec3::new(0, y, 0));
            }
        }
        blocks
    }

    /// Returns all bounding frame coordinates (bottom, top, left, right)
    pub fn frame_blocks(&self) -> Vec<IVec3> {
        let dir = self.axis.width_direction();
        let mut blocks = Vec::new();

        // Bottom row (under inner blocks)
        for w in 0..self.width {
            blocks.push(self.bottom_left + dir * w - IVec3::new(0, 1, 0));
        }
        // Top row (above inner blocks)
        for w in 0..self.width {
            blocks.push(self.bottom_left + dir * w + IVec3::new(0, self.height, 0));
        }
        // Left pillar
        for y in 0..self.height {
            blocks.push(self.bottom_left - dir + IVec3::new(0, y, 0));
        }
        // Right pillar
        for y in 0..self.height {
            blocks.push(self.bottom_left + dir * self.width + IVec3::new(0, y, 0));
        }

        blocks
    }

    /// Fills the interior of the portal with `BlockId::NetherPortal` blocks
    pub fn light<W: FnMut(IVec3, BlockId)>(&self, set_block: &mut W) {
        for pos in self.inner_blocks() {
            set_block(pos, BlockId::NetherPortal);
        }
    }

    /// Breaks the portal, converting all inner portal blocks to Air
    pub fn break_portal<W: FnMut(IVec3, BlockId)>(&self, set_block: &mut W) {
        for pos in self.inner_blocks() {
            set_block(pos, BlockId::Air);
        }
    }
}

/// Translates coordinates between dimensions matching vanilla mechanics
pub fn translate_coordinates(pos: Vec3, from: Dimension, to: Dimension) -> Vec3 {
    if from == to {
        return pos;
    }

    match (from, to) {
        (Dimension::Overworld, Dimension::Nether) => {
            // Overworld to Nether: scale by 1/8
            Vec3::new(
                pos.x / 8.0,
                pos.y.clamp(10.0, 120.0),
                pos.z / 8.0,
            )
        }
        (Dimension::Nether, Dimension::Overworld) => {
            // Nether to Overworld: scale by 8
            Vec3::new(
                pos.x * 8.0,
                pos.y.clamp(10.0, 250.0),
                pos.z * 8.0,
            )
        }
        (_, Dimension::TheEnd) => {
            // Teleporting to The End: fixed spawn platform
            Vec3::new(100.5, 49.0, 0.5)
        }
        (Dimension::TheEnd, Dimension::Overworld) => {
            // Leaving The End: return to world spawn (default 0.5, 64.0, 0.5)
            Vec3::new(0.5, 64.0, 0.5)
        }
        (Dimension::TheEnd, Dimension::Nether) => {
            Vec3::new(pos.x / 8.0, 64.0, pos.z / 8.0)
        }
        _ => pos,
    }
}

/// Searches within radius (vanilla 128 blocks) for an existing active Nether Portal
pub fn find_existing_portal<F: Fn(IVec3) -> BlockId>(
    target: IVec3,
    radius: i32,
    get_block: &F,
) -> Option<IVec3> {
    let mut closest_pos: Option<IVec3> = None;
    let mut closest_dist_sq = i64::MAX;

    // Search in concentric diamond/box pattern
    for dy in -32..=32 {
        let y = target.y + dy;
        if y < 5 || y > 250 {
            continue;
        }
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let dist_sq = (dx as i64) * (dx as i64) + (dz as i64) * (dz as i64) + (dy as i64) * (dy as i64);
                if dist_sq > (radius as i64) * (radius as i64) {
                    continue;
                }
                let check_pos = IVec3::new(target.x + dx, y, target.z + dz);
                if get_block(check_pos) == BlockId::NetherPortal {
                    if dist_sq < closest_dist_sq {
                        closest_dist_sq = dist_sq;
                        closest_pos = Some(check_pos);
                    }
                }
            }
        }
    }

    closest_pos
}

/// Constructs a standard vanilla 4x5 Obsidian frame with 2x3 portal opening and safe exit air pocket
pub fn build_nether_portal_at<W: FnMut(IVec3, BlockId)>(
    origin: IVec3,
    axis: PortalAxis,
    set_block: &mut W,
) -> Vec3 {
    let dir = axis.width_direction();
    let norm = axis.normal_direction();

    // Base row (y = 0): 4 Obsidian
    for w in 0..4 {
        set_block(origin + dir * w, BlockId::Obsidian);
    }
    // Top row (y = 4): 4 Obsidian
    for w in 0..4 {
        set_block(origin + dir * w + IVec3::new(0, 4, 0), BlockId::Obsidian);
    }
    // Left pillar (w = 0, y = 1..3)
    for y in 1..4 {
        set_block(origin + IVec3::new(0, y, 0), BlockId::Obsidian);
    }
    // Right pillar (w = 3, y = 1..3)
    for y in 1..4 {
        set_block(origin + dir * 3 + IVec3::new(0, y, 0), BlockId::Obsidian);
    }

    // Portal interior (w = 1..2, y = 1..3): 6 NetherPortal blocks
    for y in 1..4 {
        for w in 1..3 {
            set_block(origin + dir * w + IVec3::new(0, y, 0), BlockId::NetherPortal);
        }
    }

    // Safe air clearance on front and back (w = 1..2, y = 1..3, norm = -1 and +1)
    for side in [-1, 1] {
        for y in 1..4 {
            for w in 1..3 {
                set_block(origin + dir * w + norm * side + IVec3::new(0, y, 0), BlockId::Air);
            }
        }
        // Solid landing step under exit pockets
        for w in 1..3 {
            set_block(origin + dir * w + norm * side, BlockId::Obsidian);
        }
    }

    // Spawn player standing inside the portal at the center
    let center = origin + dir * 1 + IVec3::new(0, 1, 0);
    Vec3::new(
        center.x as f32 + 0.5 + (dir.x as f32 * 0.5),
        center.y as f32,
        center.z as f32 + 0.5 + (dir.z as f32 * 0.5),
    )
}

/// Teleport event generated when dimension switch completes
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortalTeleportEvent {
    pub destination: Dimension,
    pub position: Vec3,
}

/// Teleportation State Machine tracking tick accumulator, visual distortions, and dimension switching
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortalTeleportState {
    /// Number of consecutive ticks standing inside portal blocks
    pub portal_ticks: u32,
    /// Cooldown ticks remaining before next portal interaction is accepted
    pub cooldown_ticks: u32,
    /// Current dimension of the player
    pub current_dimension: Dimension,
    /// Visual portal overlay intensity (0.0 to 1.0)
    pub transition_intensity: f32,
    /// Swirling purple distortion amplitude factor
    pub distortion_factor: f32,
    /// Screen wobble / nausea warp factor
    pub nausea_factor: f32,
}

impl Default for PortalTeleportState {
    fn default() -> Self {
        Self::new(Dimension::Overworld)
    }
}

impl PortalTeleportState {
    /// Survival mode ticks required to trigger teleportation (80 ticks = 4.0 seconds)
    pub const SURVIVAL_TELEPORT_TICKS: u32 = 80;
    /// Creative mode ticks required (0 ticks = instantaneous)
    pub const CREATIVE_TELEPORT_TICKS: u32 = 0;
    /// Post-teleport cooldown ticks to avoid oscillating transitions (100 ticks = 5.0 seconds)
    pub const COOLDOWN_TICKS: u32 = 100;

    pub fn new(dimension: Dimension) -> Self {
        Self {
            portal_ticks: 0,
            cooldown_ticks: 0,
            current_dimension: dimension,
            transition_intensity: 0.0,
            distortion_factor: 0.0,
            nausea_factor: 0.0,
        }
    }

    /// Updates portal state machine by 1 game tick
    pub fn update<F: Fn(IVec3) -> BlockId, W: FnMut(IVec3, BlockId)>(
        &mut self,
        standing_block: BlockId,
        player_pos: Vec3,
        is_creative: bool,
        get_block_target_dim: &F,
        set_block_target_dim: &mut W,
    ) -> Option<PortalTeleportEvent> {
        // --- 1. End Portal: Instantaneous teleport to The End ---
        if standing_block == BlockId::EndPortal {
            if self.cooldown_ticks == 0 {
                let target_dim = if self.current_dimension == Dimension::TheEnd {
                    Dimension::Overworld
                } else {
                    Dimension::TheEnd
                };

                let dest_pos = if target_dim == Dimension::TheEnd {
                    // Generate End spawn platform
                    ensure_end_spawn_platform(set_block_target_dim);
                    Vec3::new(100.5, 49.0, 0.5)
                } else {
                    Vec3::new(0.5, 64.0, 0.5)
                };

                self.current_dimension = target_dim;
                self.portal_ticks = 0;
                self.cooldown_ticks = Self::COOLDOWN_TICKS;
                self.transition_intensity = 0.0;
                self.distortion_factor = 0.0;
                self.nausea_factor = 0.0;

                return Some(PortalTeleportEvent {
                    destination: target_dim,
                    position: dest_pos,
                });
            } else {
                self.cooldown_ticks = self.cooldown_ticks.saturating_sub(1);
                return None;
            }
        }

        // --- 2. Nether Portal: 80 ticks Survival, 0 ticks Creative ---
        if standing_block == BlockId::NetherPortal {
            if self.cooldown_ticks > 0 {
                self.cooldown_ticks -= 1;
                return None;
            }

            self.portal_ticks += 1;
            let threshold = if is_creative {
                Self::CREATIVE_TELEPORT_TICKS
            } else {
                Self::SURVIVAL_TELEPORT_TICKS
            };

            // Transition intensity ramps smoothly from 0.0 to 1.0
            self.transition_intensity = (self.portal_ticks as f32 / Self::SURVIVAL_TELEPORT_TICKS as f32).min(1.0);
            let time = self.portal_ticks as f32;
            self.distortion_factor = (time * 0.25).sin() * self.transition_intensity * 0.35;
            self.nausea_factor = (time * 0.12).sin() * self.transition_intensity * 0.20;

            if self.portal_ticks >= threshold {
                let target_dim = match self.current_dimension {
                    Dimension::Overworld => Dimension::Nether,
                    Dimension::Nether => Dimension::Overworld,
                    Dimension::TheEnd => Dimension::Overworld,
                };

                let translated = translate_coordinates(player_pos, self.current_dimension, target_dim);
                let target_ivec = IVec3::new(
                    translated.x.floor() as i32,
                    translated.y.floor() as i32,
                    translated.z.floor() as i32,
                );

                // Search for existing portal within 128 blocks in destination dimension
                let dest_pos = if let Some(existing) = find_existing_portal(target_ivec, 128, get_block_target_dim) {
                    Vec3::new(existing.x as f32 + 0.5, existing.y as f32, existing.z as f32 + 0.5)
                } else {
                    // No existing portal found: build a 4x5 Obsidian frame
                    build_nether_portal_at(target_ivec, PortalAxis::X, set_block_target_dim)
                };

                self.current_dimension = target_dim;
                self.portal_ticks = 0;
                self.cooldown_ticks = Self::COOLDOWN_TICKS;
                self.transition_intensity = 0.0;
                self.distortion_factor = 0.0;
                self.nausea_factor = 0.0;

                return Some(PortalTeleportEvent {
                    destination: target_dim,
                    position: dest_pos,
                });
            }
        } else {
            // Player stepped out: decay transition
            self.portal_ticks = self.portal_ticks.saturating_sub(2);
            self.cooldown_ticks = self.cooldown_ticks.saturating_sub(1);
            self.transition_intensity = (self.portal_ticks as f32 / Self::SURVIVAL_TELEPORT_TICKS as f32).min(1.0);
            self.distortion_factor = 0.0;
            self.nausea_factor = 0.0;
        }

        None
    }
}

/// End Portal Frame Ring Layout: 12 frame blocks surrounding 3x3 inner area
pub struct EndPortalRing;

impl EndPortalRing {
    /// Relative offsets for the 12 frame blocks of an End Portal
    pub const FRAME_OFFSETS: [IVec3; 12] = [
        // North side (facing South)
        IVec3::new(-1, 0, -2),
        IVec3::new(0, 0, -2),
        IVec3::new(1, 0, -2),
        // South side (facing North)
        IVec3::new(-1, 0, 2),
        IVec3::new(0, 0, 2),
        IVec3::new(1, 0, 2),
        // West side (facing East)
        IVec3::new(-2, 0, -1),
        IVec3::new(-2, 0, 0),
        IVec3::new(-2, 0, 1),
        // East side (facing West)
        IVec3::new(2, 0, -1),
        IVec3::new(2, 0, 0),
        IVec3::new(2, 0, 1),
    ];

    /// Validates whether a complete 12-block EndPortalFrame ring exists around `center`
    pub fn is_valid_ring<F: Fn(IVec3) -> BlockId>(center: IVec3, get_block: &F) -> bool {
        for offset in Self::FRAME_OFFSETS {
            if get_block(center + offset) != BlockId::EndPortalFrame {
                return false;
            }
        }
        true
    }

    /// Activates the End Portal: fills the 3x3 inner area with `BlockId::EndPortal`
    pub fn activate<F: Fn(IVec3) -> BlockId, W: FnMut(IVec3, BlockId)>(
        center: IVec3,
        get_block: &F,
        set_block: &mut W,
    ) -> bool {
        if !Self::is_valid_ring(center, get_block) {
            return false;
        }

        // Fill 3x3 interior at center.y with EndPortal blocks
        for dx in -1..=1 {
            for dz in -1..=1 {
                set_block(center + IVec3::new(dx, 0, dz), BlockId::EndPortal);
            }
        }
        true
    }
}

/// Generates the fixed 5x5 Obsidian spawn platform in The End at Y=48 with air clearance
pub fn ensure_end_spawn_platform<W: FnMut(IVec3, BlockId)>(set_block: &mut W) {
    let center = IVec3::new(100, 48, 0);

    // 5x5 Obsidian platform at Y = 48
    for dx in -2..=2 {
        for dz in -2..=2 {
            set_block(center + IVec3::new(dx, 0, dz), BlockId::Obsidian);
        }
    }

    // 5x3x5 Air clearance above platform (Y = 49..=51)
    for dy in 1..=3 {
        for dx in -2..=2 {
            for dz in -2..=2 {
                set_block(center + IVec3::new(dx, dy, dz), BlockId::Air);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_nether_portal_shape_detection_minimum_4x5() {
        let mut world = HashMap::<IVec3, BlockId>::new();

        // Build a 4-wide by 5-high Obsidian frame along X axis:
        // Inner opening: 2 wide (x = 1..2), 3 high (y = 1..3)
        let origin = IVec3::new(10, 60, 10);

        // Frame base (y = 0)
        for x in 0..4 {
            world.insert(origin + IVec3::new(x, 0, 0), BlockId::Obsidian);
        }
        // Frame top (y = 4)
        for x in 0..4 {
            world.insert(origin + IVec3::new(x, 4, 0), BlockId::Obsidian);
        }
        // Left pillar (x = 0)
        for y in 1..4 {
            world.insert(origin + IVec3::new(0, y, 0), BlockId::Obsidian);
        }
        // Right pillar (x = 3)
        for y in 1..4 {
            world.insert(origin + IVec3::new(3, y, 0), BlockId::Obsidian);
        }

        // Inner blocks are Air
        for y in 1..4 {
            for x in 1..3 {
                world.insert(origin + IVec3::new(x, y, 0), BlockId::Air);
            }
        }

        let get_block = |pos| *world.get(&pos).unwrap_or(&BlockId::Air);

        // Detect shape from inner coordinate (11, 61, 10)
        let detected = NetherPortalShape::detect(origin + IVec3::new(1, 1, 0), &get_block);
        assert!(detected.is_some(), "4x5 Obsidian portal frame should be detected");
        let shape = detected.unwrap();
        assert_eq!(shape.width, 2);
        assert_eq!(shape.height, 3);
        assert_eq!(shape.axis, PortalAxis::X);

        // Light portal: fills inner 2x3 blocks with NetherPortal
        let mut set_block = |pos, block| {
            world.insert(pos, block);
        };
        shape.light(&mut set_block);

        for y in 1..4 {
            for x in 1..3 {
                assert_eq!(
                    world.get(&(origin + IVec3::new(x, y, 0))),
                    Some(&BlockId::NetherPortal),
                    "Inner block should be NetherPortal"
                );
            }
        }
    }

    #[test]
    fn test_nether_portal_shape_detection_maximum_23x23() {
        let mut world = HashMap::<IVec3, BlockId>::new();
        let origin = IVec3::new(0, 70, 0);

        // Maximum vanilla portal: 21 wide x 21 high inner opening => 23x23 total frame
        let width = 21;
        let height = 21;

        // Frame base (y = 0)
        for z in 0..width + 2 {
            world.insert(origin + IVec3::new(0, 0, z), BlockId::Obsidian);
        }
        // Frame top (y = height + 1)
        for z in 0..width + 2 {
            world.insert(origin + IVec3::new(0, height + 1, z), BlockId::Obsidian);
        }
        // Left pillar (z = 0)
        for y in 1..=height {
            world.insert(origin + IVec3::new(0, y, 0), BlockId::Obsidian);
        }
        // Right pillar (z = width + 1)
        for y in 1..=height {
            world.insert(origin + IVec3::new(0, y, width + 1), BlockId::Obsidian);
        }

        // Inner blocks along Z axis
        for y in 1..=height {
            for z in 1..=width {
                world.insert(origin + IVec3::new(0, y, z), BlockId::Air);
            }
        }

        let get_block = |pos| *world.get(&pos).unwrap_or(&BlockId::Air);

        let detected = NetherPortalShape::detect(origin + IVec3::new(0, 5, 5), &get_block);
        assert!(detected.is_some(), "23x23 portal frame along Z axis should be detected");
        let shape = detected.unwrap();
        assert_eq!(shape.width, 21);
        assert_eq!(shape.height, 21);
        assert_eq!(shape.axis, PortalAxis::Z);
    }

    #[test]
    fn test_nether_portal_invalid_frame_missing_obsidian() {
        let mut world = HashMap::<IVec3, BlockId>::new();
        let origin = IVec3::new(20, 60, 20);

        // Build 4x5 frame but leave top frame as Air
        for x in 0..4 {
            world.insert(origin + IVec3::new(x, 0, 0), BlockId::Obsidian);
        }
        for y in 1..4 {
            world.insert(origin + IVec3::new(0, y, 0), BlockId::Obsidian);
            world.insert(origin + IVec3::new(3, y, 0), BlockId::Obsidian);
        }
        // Top frame missing!

        let get_block = |pos| *world.get(&pos).unwrap_or(&BlockId::Air);
        let detected = NetherPortalShape::detect(origin + IVec3::new(1, 1, 0), &get_block);
        assert!(detected.is_none(), "Incomplete frame without top obsidian must not detect");
    }

    #[test]
    fn test_coordinate_translation_overworld_to_nether_and_back() {
        let overworld_pos = Vec3::new(800.0, 64.0, -1600.0);
        let nether_pos = translate_coordinates(overworld_pos, Dimension::Overworld, Dimension::Nether);

        assert!((nether_pos.x - 100.0).abs() < 1e-4);
        assert!((nether_pos.y - 64.0).abs() < 1e-4);
        assert!((nether_pos.z - (-200.0)).abs() < 1e-4);

        let roundtrip = translate_coordinates(nether_pos, Dimension::Nether, Dimension::Overworld);
        assert!((roundtrip.x - overworld_pos.x).abs() < 1e-4);
        assert!((roundtrip.z - overworld_pos.z).abs() < 1e-4);
    }

    #[test]
    fn test_portal_forcer_search_and_generation() {
        use std::cell::RefCell;
        let world = RefCell::new(HashMap::<IVec3, BlockId>::new());
        let target = IVec3::new(50, 60, 50);

        let get_block = |pos| *world.borrow().get(&pos).unwrap_or(&BlockId::Air);

        // 1. When no portal exists within 128 blocks, search returns None
        assert_eq!(find_existing_portal(target, 128, &get_block), None);

        // 2. Generate new portal frame using forcer
        let mut set_block = |pos, block| {
            world.borrow_mut().insert(pos, block);
        };
        let spawn_pos = build_nether_portal_at(target, PortalAxis::X, &mut set_block);

        assert!(spawn_pos.x > 50.0);
        assert_eq!(spawn_pos.y, 61.0);

        // 3. Search now finds the newly generated portal!
        let found = find_existing_portal(target, 128, &get_block);
        assert!(found.is_some());
    }

    #[test]
    fn test_portal_teleport_state_survival_and_creative_timers() {
        use std::cell::RefCell;
        let world = RefCell::new(HashMap::<IVec3, BlockId>::new());
        let get_block = |pos| *world.borrow().get(&pos).unwrap_or(&BlockId::Air);
        let mut set_block = |pos, block| {
            world.borrow_mut().insert(pos, block);
        };

        let mut state = PortalTeleportState::new(Dimension::Overworld);

        // Survival mode: requires 80 ticks
        for tick in 1..80 {
            let event = state.update(
                BlockId::NetherPortal,
                Vec3::new(0.0, 64.0, 0.0),
                false, // Survival
                &get_block,
                &mut set_block,
            );
            assert_eq!(event, None, "Tick {tick} should not teleport yet");
            assert!(state.transition_intensity > 0.0);
        }

        // Tick 80 triggers teleport!
        let event = state.update(
            BlockId::NetherPortal,
            Vec3::new(80.0, 64.0, 80.0),
            false,
            &get_block,
            &mut set_block,
        );
        assert!(event.is_some(), "Tick 80 must trigger dimension switch");
        let ev = event.unwrap();
        assert_eq!(ev.destination, Dimension::Nether);
        assert_eq!(state.current_dimension, Dimension::Nether);
        assert_eq!(state.cooldown_ticks, PortalTeleportState::COOLDOWN_TICKS);

        // Creative mode: instant teleport on first tick
        let mut creative_state = PortalTeleportState::new(Dimension::Overworld);
        let creative_event = creative_state.update(
            BlockId::NetherPortal,
            Vec3::new(0.0, 64.0, 0.0),
            true, // Creative
            &get_block,
            &mut set_block,
        );
        assert!(creative_event.is_some(), "Creative mode should teleport on tick 0/1");
        assert_eq!(creative_event.unwrap().destination, Dimension::Nether);
    }

    #[test]
    fn test_end_portal_ring_activation_and_fixed_spawn_platform() {
        use std::cell::RefCell;
        let world = RefCell::new(HashMap::<IVec3, BlockId>::new());
        let center = IVec3::new(0, 50, 0);

        // Place 12 EndPortalFrame blocks
        for offset in EndPortalRing::FRAME_OFFSETS {
            world.borrow_mut().insert(center + offset, BlockId::EndPortalFrame);
        }

        let get_block = |pos| *world.borrow().get(&pos).unwrap_or(&BlockId::Air);
        assert!(EndPortalRing::is_valid_ring(center, &get_block));

        // Activate ring
        let mut set_block = |pos, block| {
            world.borrow_mut().insert(pos, block);
        };
        let activated = EndPortalRing::activate(center, &get_block, &mut set_block);
        assert!(activated);

        // Verify inner 3x3 is EndPortal
        for dx in -1..=1 {
            for dz in -1..=1 {
                assert_eq!(
                    world.borrow().get(&(center + IVec3::new(dx, 0, dz))),
                    Some(&BlockId::EndPortal),
                    "Inner 3x3 position should be EndPortal"
                );
            }
        }

        // Stepping into EndPortal: state triggers instant teleport to (100.5, 49.0, 0.5)
        let mut state = PortalTeleportState::new(Dimension::Overworld);
        let event = state.update(
            BlockId::EndPortal,
            Vec3::new(0.0, 50.0, 0.0),
            false,
            &get_block,
            &mut set_block,
        );
        assert!(event.is_some());
        let ev = event.unwrap();
        assert_eq!(ev.destination, Dimension::TheEnd);
        assert_eq!(ev.position, Vec3::new(100.5, 49.0, 0.5));

        // Verify End spawn platform was generated at Y=48
        assert_eq!(world.borrow().get(&IVec3::new(100, 48, 0)), Some(&BlockId::Obsidian));
        assert_eq!(world.borrow().get(&IVec3::new(100, 49, 0)), Some(&BlockId::Air));
    }
}
