// REA Verified: net.minecraft.client.gui.screens.inventory.ContainerScreen (mapped: fel.class)
// REA Verified: net.minecraft.world.inventory.ChestMenu (mapped: cii.class)
//
// Double-Layer Chest Container GUI:
// - Window dimensions: 176x168 (for standard 3-row 27-slot chest)
// - Layout:
//   * Top section: 27 chest storage slots (3 rows of 9) at (8 + col * 18, 18 + row * 18)
//   * Bottom section: 36 player inventory slots:
//       - 27 main inventory slots (3 rows of 9) at (8 + col * 18, 84 + row * 18)
//       - 9 hotbar slots (1 row of 9) at (8 + col * 18, 142)
// - Supports:
//   * Left-click: pick up, place, stack, and swap floating cursor items
//   * Right-click: split stack in half or place single item
//   * Shift-click: instant transfer between chest storage and player hotbar / main inventory
//   * Orthographic 2D mesh generation with item icons and stack count digits

use glam::IVec3;
use winit::event::MouseButton;
use winit::window::{CursorGrabMode, Window};

use crate::game::inventory::{
    ItemStack, PlayerInventory, HOTBAR_SLOT_END, HOTBAR_SLOT_START, INV_SLOT_END, INV_SLOT_START,
};
use crate::render::atlas::TextureAtlas;
use crate::render::gui::{GuiAtlas, GuiRect, GuiVertex};
use crate::world::tile_entity::{ChestTileEntity, CHEST_SLOT_COUNT};

pub const CONTAINER_WIDTH: f32 = 176.0;
pub const CONTAINER_HEIGHT: f32 = 168.0;
pub const SLOT_SIZE: f32 = 16.0;

// Slot ranges in ContainerScreen
pub const CHEST_SLOT_START: usize = 0;
pub const CHEST_SLOT_END: usize = 27; // 0..27 (len: 27)
pub const PLAYER_MAIN_START: usize = 27;
pub const PLAYER_MAIN_END: usize = 54; // 27..54 (len: 27 -> PlayerInventory slots 9..36)
pub const PLAYER_HOTBAR_START: usize = 54;
pub const PLAYER_HOTBAR_END: usize = 63; // 54..63 (len: 9 -> PlayerInventory slots 36..45)
pub const TOTAL_CONTAINER_SLOTS: usize = 63;

/// Chest Screen GUI state
pub struct ChestScreenGui {
    pub is_open: bool,
    pub chest_pos: Option<IVec3>,
    pub mouse_pos: [f32; 2],
    pub hovered_slot: Option<usize>,
}

impl Default for ChestScreenGui {
    fn default() -> Self {
        Self::new()
    }
}

impl ChestScreenGui {
    pub fn new() -> Self {
        Self {
            is_open: false,
            chest_pos: None,
            mouse_pos: [0.0, 0.0],
            hovered_slot: None,
        }
    }

    /// Open chest screen at target world block coordinate: releases mouse cursor
    pub fn open(&mut self, chest_pos: IVec3, window: &Window) {
        self.is_open = true;
        self.chest_pos = Some(chest_pos);
        let _ = window.set_cursor_grab(CursorGrabMode::None);
        window.set_cursor_visible(true);
    }

    /// Close chest screen: locks cursor back to first-person mouse look
    pub fn close(&mut self, window: &Window) {
        self.is_open = false;
        self.chest_pos = None;
        let _ = window
            .set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
        window.set_cursor_visible(false);
    }

    /// Compute UI integer scale: 2.0x for >= 720p, 1.0x otherwise
    #[inline]
    pub fn get_scale(screen_height: f32) -> f32 {
        if screen_height >= 720.0 {
            2.0
        } else {
            1.0
        }
    }

    /// Compute top-left pixel position of the 176x168 window
    pub fn get_window_bounds(screen_width: f32, screen_height: f32) -> (f32, f32, f32, f32) {
        let scale = Self::get_scale(screen_height);
        let win_w = CONTAINER_WIDTH * scale;
        let win_h = CONTAINER_HEIGHT * scale;
        let left = ((screen_width - win_w) / 2.0).round();
        let top = ((screen_height - win_h) / 2.0).round();
        (left, top, win_w, win_h)
    }

    /// Returns the local (x, y) offset within the 176x168 container for slot index (0..62)
    pub fn get_slot_local_pos(slot_idx: usize) -> Option<(f32, f32)> {
        match slot_idx {
            // Chest storage: 3 rows of 9 slots (0..27)
            CHEST_SLOT_START..=26 => {
                let col = (slot_idx % 9) as f32;
                let row = (slot_idx / 9) as f32;
                Some((8.0 + col * 18.0, 18.0 + row * 18.0))
            }
            // Player main inventory: 3 rows of 9 slots (27..54)
            PLAYER_MAIN_START..=53 => {
                let offset = slot_idx - PLAYER_MAIN_START;
                let col = (offset % 9) as f32;
                let row = (offset / 9) as f32;
                Some((8.0 + col * 18.0, 84.0 + row * 18.0))
            }
            // Player hotbar: 1 row of 9 slots (54..63)
            PLAYER_HOTBAR_START..=62 => {
                let col = (slot_idx - PLAYER_HOTBAR_START) as f32;
                Some((8.0 + col * 18.0, 142.0))
            }
            _ => None,
        }
    }

    /// Returns the screen pixel bounding box (x, y, w, h) for a slot index
    pub fn get_slot_screen_bounds(
        slot_idx: usize,
        screen_width: f32,
        screen_height: f32,
    ) -> Option<(f32, f32, f32, f32)> {
        let (win_x, win_y, _, _) = Self::get_window_bounds(screen_width, screen_height);
        let scale = Self::get_scale(screen_height);
        let (lx, ly) = Self::get_slot_local_pos(slot_idx)?;

        Some((
            win_x + lx * scale,
            win_y + ly * scale,
            SLOT_SIZE * scale,
            SLOT_SIZE * scale,
        ))
    }

    /// Update mouse coordinates and compute hovered slot index
    pub fn update_mouse(
        &mut self,
        mouse_x: f32,
        mouse_y: f32,
        screen_width: f32,
        screen_height: f32,
    ) {
        self.mouse_pos = [mouse_x, mouse_y];

        if !self.is_open {
            self.hovered_slot = None;
            return;
        }

        self.hovered_slot = None;
        for slot in 0..TOTAL_CONTAINER_SLOTS {
            if let Some((sx, sy, sw, sh)) =
                Self::get_slot_screen_bounds(slot, screen_width, screen_height)
            {
                if mouse_x >= sx && mouse_x <= sx + sw && mouse_y >= sy && mouse_y <= sy + sh {
                    self.hovered_slot = Some(slot);
                    break;
                }
            }
        }
    }

    /// Maps container slot index to player inventory slot index (if player slot)
    #[inline]
    pub fn to_player_slot(slot_idx: usize) -> Option<usize> {
        match slot_idx {
            PLAYER_MAIN_START..=53 => {
                let offset = slot_idx - PLAYER_MAIN_START;
                Some(INV_SLOT_START + offset) // 9..36
            }
            PLAYER_HOTBAR_START..=62 => {
                let offset = slot_idx - PLAYER_HOTBAR_START;
                Some(HOTBAR_SLOT_START + offset) // 36..45
            }
            _ => None,
        }
    }

    /// Handles mouse clicks on container slots (supports left, right, and shift-click)
    pub fn handle_mouse_click(
        &mut self,
        button: MouseButton,
        is_shift_held: bool,
        chest: &mut ChestTileEntity,
        inventory: &mut PlayerInventory,
    ) {
        if !self.is_open {
            return;
        }

        let slot_idx = match self.hovered_slot {
            Some(s) => s,
            None => return,
        };

        if is_shift_held {
            Self::quick_move_stack(slot_idx, chest, inventory);
            return;
        }

        // Regular mouse click (Left: pickup/place/swap, Right: split/place single)
        if slot_idx < CHEST_SLOT_END {
            // Click inside Chest
            match button {
                MouseButton::Left => Self::handle_chest_left_click(slot_idx, chest, inventory),
                MouseButton::Right => Self::handle_chest_right_click(slot_idx, chest, inventory),
                _ => {}
            }
        } else if let Some(player_slot) = Self::to_player_slot(slot_idx) {
            // Click inside Player inventory
            match button {
                MouseButton::Left => inventory.handle_left_click(player_slot),
                MouseButton::Right => inventory.handle_right_click(player_slot),
                _ => {}
            }
        }
    }

    /// Left click interaction for chest slot
    fn handle_chest_left_click(
        slot: usize,
        chest: &mut ChestTileEntity,
        inventory: &mut PlayerInventory,
    ) {
        let slot_item = chest.get_item(slot);
        let cursor = inventory.cursor_item;

        match (slot_item, cursor) {
            // Case 1: Pick up whole stack
            (Some(stack), None) => {
                inventory.cursor_item = Some(stack);
                chest.set_item(slot, None);
            }
            // Case 2: Place whole cursor stack into empty slot
            (None, Some(cursor_item)) => {
                chest.set_item(slot, Some(cursor_item));
                inventory.cursor_item = None;
            }
            // Case 3: Merge same items
            (Some(mut dest), Some(mut src)) if dest.can_stack_with(&src) => {
                let space = dest.max_stack.saturating_sub(dest.count);
                let to_add = space.min(src.count);
                dest.count += to_add;
                src.count -= to_add;

                chest.set_item(slot, Some(dest));
                if src.count == 0 {
                    inventory.cursor_item = None;
                } else {
                    inventory.cursor_item = Some(src);
                }
            }
            // Case 4: Swap different items
            (Some(dest), Some(src)) => {
                chest.set_item(slot, Some(src));
                inventory.cursor_item = Some(dest);
            }
            (None, None) => {}
        }
    }

    /// Right click interaction for chest slot
    fn handle_chest_right_click(
        slot: usize,
        chest: &mut ChestTileEntity,
        inventory: &mut PlayerInventory,
    ) {
        let slot_item = chest.get_item(slot);
        let cursor = inventory.cursor_item;

        match (slot_item, cursor) {
            // Case 1: Split stack in half into cursor
            (Some(mut stack), None) => {
                let take_count = (stack.count + 1) / 2; // ceil(N / 2)
                let leave_count = stack.count - take_count;

                inventory.cursor_item = Some(ItemStack::with_max_stack(
                    stack.id,
                    take_count,
                    stack.max_stack,
                ));
                if leave_count > 0 {
                    stack.count = leave_count;
                    chest.set_item(slot, Some(stack));
                } else {
                    chest.set_item(slot, None);
                }
            }
            // Case 2: Drop 1 item into empty slot
            (None, Some(mut cursor_item)) => {
                let single = ItemStack::with_max_stack(cursor_item.id, 1, cursor_item.max_stack);
                chest.set_item(slot, Some(single));
                cursor_item.count -= 1;
                if cursor_item.count == 0 {
                    inventory.cursor_item = None;
                } else {
                    inventory.cursor_item = Some(cursor_item);
                }
            }
            // Case 3: Drop 1 item into matching stack
            (Some(mut dest), Some(mut src)) if dest.can_stack_with(&src) => {
                if dest.count < dest.max_stack {
                    dest.count += 1;
                    src.count -= 1;
                    chest.set_item(slot, Some(dest));
                    if src.count == 0 {
                    inventory.cursor_item = None;
                } else {
                    inventory.cursor_item = Some(src);
                }
                }
            }
            // Case 4: Swap different items on right-click
            (Some(dest), Some(src)) => {
                chest.set_item(slot, Some(src));
                inventory.cursor_item = Some(dest);
            }
            (None, None) => {}
        }
    }

    /// REA Verified: net.minecraft.world.inventory.ChestMenu.quickMoveStack (cii.a)
    /// Shift-click instant transfer between chest storage and player hotbar / main inventory
    pub fn quick_move_stack(
        slot_idx: usize,
        chest: &mut ChestTileEntity,
        inventory: &mut PlayerInventory,
    ) {
        if slot_idx < CHEST_SLOT_END {
            // 1. Move from Chest -> Player Inventory
            let mut stack = match chest.get_item(slot_idx) {
                Some(s) if !s.is_empty() => s,
                _ => return,
            };

            // Phase A: Merge into player hotbar
            for h_slot in HOTBAR_SLOT_START..HOTBAR_SLOT_END {
                if let Some(mut target) = inventory.get_slot(h_slot) {
                    if target.can_stack_with(&stack) && target.count < target.max_stack {
                        let space = target.max_stack - target.count;
                        let add = space.min(stack.count);
                        target.count += add;
                        stack.count -= add;
                        inventory.set_slot(h_slot, Some(target));
                        if stack.count == 0 {
                            break;
                        }
                    }
                }
            }

            // Phase B: Merge into player main inventory
            if stack.count > 0 {
                for m_slot in INV_SLOT_START..INV_SLOT_END {
                    if let Some(mut target) = inventory.get_slot(m_slot) {
                        if target.can_stack_with(&stack) && target.count < target.max_stack {
                            let space = target.max_stack - target.count;
                            let add = space.min(stack.count);
                            target.count += add;
                            stack.count -= add;
                            inventory.set_slot(m_slot, Some(target));
                            if stack.count == 0 {
                                break;
                            }
                        }
                    }
                }
            }

            // Phase C: Place into empty player hotbar slot
            if stack.count > 0 {
                for h_slot in HOTBAR_SLOT_START..HOTBAR_SLOT_END {
                    if inventory.get_slot(h_slot).is_none() {
                        inventory.set_slot(h_slot, Some(stack));
                        stack.count = 0;
                        break;
                    }
                }
            }

            // Phase D: Place into empty player main inventory slot
            if stack.count > 0 {
                for m_slot in INV_SLOT_START..INV_SLOT_END {
                    if inventory.get_slot(m_slot).is_none() {
                        inventory.set_slot(m_slot, Some(stack));
                        stack.count = 0;
                        break;
                    }
                }
            }

            // Update remaining stack in chest
            if stack.count == 0 {
                chest.set_item(slot_idx, None);
            } else {
                chest.set_item(slot_idx, Some(stack));
            }
        } else if let Some(player_slot) = Self::to_player_slot(slot_idx) {
            // 2. Move from Player Inventory -> Chest
            let mut stack = match inventory.get_slot(player_slot) {
                Some(s) if !s.is_empty() => s,
                _ => return,
            };

            // Phase A: Merge into existing matching chest slots (0..27)
            for c_slot in 0..CHEST_SLOT_COUNT {
                if let Some(mut target) = chest.get_item(c_slot) {
                    if target.can_stack_with(&stack) && target.count < target.max_stack {
                        let space = target.max_stack - target.count;
                        let add = space.min(stack.count);
                        target.count += add;
                        stack.count -= add;
                        chest.set_item(c_slot, Some(target));
                        if stack.count == 0 {
                            break;
                        }
                    }
                }
            }

            // Phase B: Place into first empty chest slot (0..27)
            if stack.count > 0 {
                for c_slot in 0..CHEST_SLOT_COUNT {
                    if chest.get_item(c_slot).is_none() {
                        chest.set_item(c_slot, Some(stack));
                        stack.count = 0;
                        break;
                    }
                }
            }

            // Update remaining stack in player inventory
            if stack.count == 0 {
                inventory.set_slot(player_slot, None);
            } else {
                inventory.set_slot(player_slot, Some(stack));
            }
        }
    }

    /// Generates 2D vertices and indices for rendering the full Chest Container GUI
    pub fn build_render_mesh(
        &self,
        chest: &ChestTileEntity,
        inventory: &PlayerInventory,
        atlas: &GuiAtlas,
        terrain_atlas: &TextureAtlas,
        screen_width: f32,
        screen_height: f32,
        vertices: &mut Vec<GuiVertex>,
        indices: &mut Vec<u32>,
    ) {
        if !self.is_open {
            return;
        }

        let (win_x, win_y, win_w, win_h) = Self::get_window_bounds(screen_width, screen_height);
        let scale = Self::get_scale(screen_height);

        // 1. Draw 176x168 Container Background
        Self::push_quad(
            vertices,
            indices,
            win_x,
            win_y,
            win_w,
            win_h,
            atlas.chest_container_uv,
            [1.0, 1.0, 1.0, 1.0],
        );

        // 2. Draw Slot Items & Highlights
        for slot in 0..TOTAL_CONTAINER_SLOTS {
            let (sx, sy, sw, sh) = match Self::get_slot_screen_bounds(slot, screen_width, screen_height) {
                Some(b) => b,
                None => continue,
            };

            // Highlight hovered slot
            if self.hovered_slot == Some(slot) {
                Self::push_quad(
                    vertices,
                    indices,
                    sx,
                    sy,
                    sw,
                    sh,
                    atlas.white_uv,
                    [1.0, 1.0, 1.0, 0.35],
                );
            }

            // Get item stack in slot
            let item_opt = if slot < CHEST_SLOT_END {
                chest.get_item(slot)
            } else if let Some(p_slot) = Self::to_player_slot(slot) {
                inventory.get_slot(p_slot)
            } else {
                None
            };

            if let Some(stack) = item_opt {
                if !stack.is_empty() {
                    let uv = Self::get_item_uv(stack.id, atlas, terrain_atlas);
                    Self::push_quad(vertices, indices, sx, sy, sw, sh, uv, [1.0, 1.0, 1.0, 1.0]);

                    // Render stack count badge if count > 1
                    if stack.count > 1 {
                        Self::render_count_badge(
                            stack.count,
                            sx,
                            sy,
                            scale,
                            atlas,
                            vertices,
                            indices,
                        );
                    }
                }
            }
        }

        // 3. Draw Floating Cursor Stack
        if let Some(cursor) = inventory.cursor_item {
            if !cursor.is_empty() {
                let item_w = SLOT_SIZE * scale;
                let item_h = SLOT_SIZE * scale;
                let cx = self.mouse_pos[0] - item_w / 2.0;
                let cy = self.mouse_pos[1] - item_h / 2.0;

                let uv = Self::get_item_uv(cursor.id, atlas, terrain_atlas);
                Self::push_quad(vertices, indices, cx, cy, item_w, item_h, uv, [1.0, 1.0, 1.0, 1.0]);

                if cursor.count > 1 {
                    Self::render_count_badge(
                        cursor.count,
                        cx,
                        cy,
                        scale,
                        atlas,
                        vertices,
                        indices,
                    );
                }
            }
        }
    }

    /// Helper to render item stack count number in the bottom right corner of slot
    fn render_count_badge(
        count: u8,
        slot_x: f32,
        slot_y: f32,
        scale: f32,
        atlas: &GuiAtlas,
        vertices: &mut Vec<GuiVertex>,
        indices: &mut Vec<u32>,
    ) {
        let count_str = format!("{count}");
        let char_w = 6.0 * scale;
        let char_h = 7.0 * scale;
        let badge_w = count_str.len() as f32 * char_w;
        let badge_x = slot_x + SLOT_SIZE * scale - badge_w;
        let badge_y = slot_y + SLOT_SIZE * scale - char_h;

        for (i, ch) in count_str.chars().enumerate() {
            let cx = badge_x + i as f32 * char_w;
            let uv = atlas.get_glyph_uv(ch as u8);

            // Drop shadow
            Self::push_quad(
                vertices,
                indices,
                cx + 1.0 * scale,
                badge_y + 1.0 * scale,
                char_w,
                char_h,
                uv,
                [0.15, 0.15, 0.15, 1.0],
            );
            // Main text (crisp white)
            Self::push_quad(
                vertices,
                indices,
                cx,
                badge_y,
                char_w,
                char_h,
                uv,
                [1.0, 1.0, 1.0, 1.0],
            );
        }
    }

    /// Resolves item ID to appropriate atlas UV rectangle
    fn get_item_uv(item_id: u16, atlas: &GuiAtlas, terrain_atlas: &TextureAtlas) -> GuiRect {
        if let Some(block) = ItemStack::new(item_id, 1).to_block() {
            let [u_min, v_min, u_max, v_max] =
                terrain_atlas.get_face_uv(block, crate::render::atlas::Direction::North);
            GuiRect {
                u_min,
                v_min,
                u_max,
                v_max,
            }
        } else {
            atlas.white_uv
        }
    }

    /// Pushes a textured 2D quad into the vertex and index buffers
    pub fn push_quad(
        vertices: &mut Vec<GuiVertex>,
        indices: &mut Vec<u32>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        uv: GuiRect,
        color: [f32; 4],
    ) {
        let base = vertices.len() as u32;

        vertices.push(GuiVertex::new([x, y], [uv.u_min, uv.v_min], color));
        vertices.push(GuiVertex::new([x + w, y], [uv.u_max, uv.v_min], color));
        vertices.push(GuiVertex::new([x + w, y + h], [uv.u_max, uv.v_max], color));
        vertices.push(GuiVertex::new([x, y + h], [uv.u_min, uv.v_max], color));

        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::inventory::*;

    #[test]
    fn test_chest_screen_slot_layout_positions() {
        // 1. First chest slot (0): (8.0, 18.0)
        let (x0, y0) = ChestScreenGui::get_slot_local_pos(0).unwrap();
        assert_eq!((x0, y0), (8.0, 18.0));

        // 2. 9th chest slot (row 0, col 8): 8 + 8 * 18 = 152
        let (x8, y8) = ChestScreenGui::get_slot_local_pos(8).unwrap();
        assert_eq!((x8, y8), (152.0, 18.0));

        // 3. Last chest slot (26): row 2, col 8 -> (152.0, 54.0)
        let (x26, y26) = ChestScreenGui::get_slot_local_pos(26).unwrap();
        assert_eq!((x26, y26), (152.0, 54.0));

        // 4. First player main inventory slot (27): (8.0, 84.0)
        let (xp0, yp0) = ChestScreenGui::get_slot_local_pos(27).unwrap();
        assert_eq!((xp0, yp0), (8.0, 84.0));

        // 5. First hotbar slot (54): (8.0, 142.0)
        let (xh0, yh0) = ChestScreenGui::get_slot_local_pos(54).unwrap();
        assert_eq!((xh0, yh0), (8.0, 142.0));
    }

    #[test]
    fn test_chest_to_player_shift_click_transfer() {
        let mut chest = ChestTileEntity::new(IVec3::ZERO);
        let mut inv = PlayerInventory::new();
        inv.hotbar = [None; 9];
        inv.main = [None; 27];

        // Place 40 stone in chest slot 0
        chest.set_item(0, Some(ItemStack::new(ITEM_STONE, 40)));
        // Place 30 stone in player hotbar slot 0 (slot index 36)
        inv.set_slot(36, Some(ItemStack::new(ITEM_STONE, 30)));

        // Shift click chest slot 0
        ChestScreenGui::quick_move_stack(0, &mut chest, &mut inv);

        // Hotbar slot 36 should have merged to 64!
        assert_eq!(inv.get_slot(36), Some(ItemStack::new(ITEM_STONE, 64)));
        // Remaining 6 stone placed into next empty hotbar slot (37)
        assert_eq!(inv.get_slot(37), Some(ItemStack::new(ITEM_STONE, 6)));
        // Chest slot 0 should now be empty
        assert_eq!(chest.get_item(0), None);
    }

    #[test]
    fn test_player_to_chest_shift_click_transfer() {
        let mut chest = ChestTileEntity::new(IVec3::ZERO);
        let mut inv = PlayerInventory::new();
        inv.hotbar = [None; 9];
        inv.main = [None; 27];

        // Place 16 torches in chest slot 5
        chest.set_item(5, Some(ItemStack::new(ITEM_TORCH, 16)));
        // Place 32 torches in player hotbar slot 0 (container slot 54)
        inv.set_slot(HOTBAR_SLOT_START, Some(ItemStack::new(ITEM_TORCH, 32)));

        // Shift click container slot 54 (hotbar slot 0)
        ChestScreenGui::quick_move_stack(54, &mut chest, &mut inv);

        // Chest slot 5 should have merged: 16 + 32 = 48 torches!
        assert_eq!(chest.get_item(5), Some(ItemStack::new(ITEM_TORCH, 48)));
        // Player hotbar slot 0 should now be empty
        assert_eq!(inv.get_slot(HOTBAR_SLOT_START), None);
    }

    #[test]
    fn test_chest_left_and_right_click_slot_stacking() {
        let mut gui = ChestScreenGui::new();
        gui.is_open = true;
        let mut chest = ChestTileEntity::new(IVec3::ZERO);
        let mut inv = PlayerInventory::new();

        // Put 10 dirt in chest slot 0
        chest.set_item(0, Some(ItemStack::new(ITEM_DIRT, 10)));
        gui.hovered_slot = Some(0);

        // Right-click slot 0: should split in half (5 in cursor, 5 in slot)
        gui.handle_mouse_click(MouseButton::Right, false, &mut chest, &mut inv);
        assert_eq!(chest.get_item(0), Some(ItemStack::new(ITEM_DIRT, 5)));
        assert_eq!(inv.cursor_item, Some(ItemStack::new(ITEM_DIRT, 5)));

        // Right-click slot 1 (empty): should place 1 dirt into slot 1
        gui.hovered_slot = Some(1);
        gui.handle_mouse_click(MouseButton::Right, false, &mut chest, &mut inv);
        assert_eq!(chest.get_item(1), Some(ItemStack::new(ITEM_DIRT, 1)));
        assert_eq!(inv.cursor_item, Some(ItemStack::new(ITEM_DIRT, 4)));

        // Left-click slot 1: should place remaining 4 dirt into slot 1 (1 + 4 = 5)
        gui.handle_mouse_click(MouseButton::Left, false, &mut chest, &mut inv);
        assert_eq!(chest.get_item(1), Some(ItemStack::new(ITEM_DIRT, 5)));
        assert_eq!(inv.cursor_item, None);
    }
}
