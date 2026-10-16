// REA Verified: net.minecraft.client.gui.screens.inventory.InventoryScreen (mapped: ffa.class)
// REA Verified: net.minecraft.world.inventory.InventoryMenu (mapped: cjc.class)
//
// Survival Inventory Screen & 2x2 Crafting Grid GUI:
// - Window dimensions: 176x166
// - Slot positions matching cjc.class:
//   * Crafting Result (Slot 0): (154, 28)
//   * 2x2 Crafting Grid (Slots 1..4):
//       Slot 1: (98, 18), Slot 2: (116, 18)
//       Slot 3: (98, 36), Slot 4: (116, 36)
//   * Armor Slots (Slots 5..8):
//       Slot 5 (Helmet): (8, 8)
//       Slot 6 (Chestplate): (8, 26)
//       Slot 7 (Leggings): (8, 44)
//       Slot 8 (Boots): (8, 62)
//   * Main Storage 3x9 (Slots 9..35): x = 8 + col * 18, y = 84 + row * 18
//   * Hotbar 1x9 (Slots 36..44): x = 8 + col * 18, y = 142
//   * Offhand Slot (Slot 45): (77, 62)
// - Opens/closes with key 'E' and grabs/releases mouse cursor.
// - Supports left-click (full stack pick/place) and right-click (split / single item place).

use winit::event::MouseButton;
use winit::window::{CursorGrabMode, Window};

#[allow(unused_imports)]
use crate::game::inventory::{
    ItemStack, PlayerInventory, ARMOR_SLOT_END, ARMOR_SLOT_START, CRAFT_SLOT_END,
    CRAFT_SLOT_START, HOTBAR_SLOT_END, HOTBAR_SLOT_START, INV_SLOT_END, INV_SLOT_START,
    RESULT_SLOT, SHIELD_SLOT, TOTAL_SLOTS,
};
use crate::render::gui::{GuiAtlas, GuiRect, GuiVertex};

pub const INVENTORY_WIDTH: f32 = 176.0;
pub const INVENTORY_HEIGHT: f32 = 166.0;
pub const SLOT_SIZE: f32 = 16.0;

/// GUI State for Survival Inventory Screen
pub struct InventoryGui {
    pub is_open: bool,
    pub mouse_pos: [f32; 2],
    pub hovered_slot: Option<usize>,
}

impl Default for InventoryGui {
    fn default() -> Self {
        Self::new()
    }
}

impl InventoryGui {
    pub fn new() -> Self {
        Self {
            is_open: false,
            mouse_pos: [0.0, 0.0],
            hovered_slot: None,
        }
    }

    /// Open inventory: release cursor and enable mouse navigation
    pub fn open(&mut self, window: &Window) {
        self.is_open = true;
        let _ = window.set_cursor_grab(CursorGrabMode::None);
        window.set_cursor_visible(true);
    }

    /// Close inventory: lock cursor back to first-person mouse look
    pub fn close(&mut self, window: &Window) {
        self.is_open = false;
        let _ = window
            .set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
        window.set_cursor_visible(false);
    }

    /// Toggle inventory open/closed
    pub fn toggle(&mut self, window: &Window) {
        if self.is_open {
            self.close(window);
        } else {
            self.open(window);
        }
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

    /// Compute top-left pixel position of the 176x166 window
    pub fn get_window_bounds(screen_width: f32, screen_height: f32) -> (f32, f32, f32, f32) {
        let scale = Self::get_scale(screen_height);
        let win_w = INVENTORY_WIDTH * scale;
        let win_h = INVENTORY_HEIGHT * scale;
        let left = ((screen_width - win_w) / 2.0).round();
        let top = ((screen_height - win_h) / 2.0).round();
        (left, top, win_w, win_h)
    }

    /// Returns the local (x, y) offset within the 176x166 container for slot index (0..45)
    pub fn get_slot_local_pos(slot_idx: usize) -> Option<(f32, f32)> {
        match slot_idx {
            // Crafting result slot
            RESULT_SLOT => Some((154.0, 28.0)),
            // 2x2 Crafting grid (1=top-left, 2=top-right, 3=bottom-left, 4=bottom-right)
            CRAFT_SLOT_START..=4 => {
                let offset = slot_idx - CRAFT_SLOT_START;
                let col = (offset % 2) as f32;
                let row = (offset / 2) as f32;
                Some((98.0 + col * 18.0, 18.0 + row * 18.0))
            }
            // Armor slots (5=helmet, 6=chestplate, 7=leggings, 8=boots)
            ARMOR_SLOT_START..=8 => {
                let offset = (slot_idx - ARMOR_SLOT_START) as f32;
                Some((8.0, 8.0 + offset * 18.0))
            }
            // Main 3x9 storage
            INV_SLOT_START..=35 => {
                let offset = slot_idx - INV_SLOT_START;
                let col = (offset % 9) as f32;
                let row = (offset / 9) as f32;
                Some((8.0 + col * 18.0, 84.0 + row * 18.0))
            }
            // Hotbar 1x9
            HOTBAR_SLOT_START..=44 => {
                let col = (slot_idx - HOTBAR_SLOT_START) as f32;
                Some((8.0 + col * 18.0, 142.0))
            }
            // Offhand / Shield slot
            SHIELD_SLOT => Some((77.0, 62.0)),
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

    /// Update mouse coordinates and calculate hovered slot
    pub fn update_mouse(&mut self, mouse_x: f32, mouse_y: f32, screen_width: f32, screen_height: f32) {
        self.mouse_pos = [mouse_x, mouse_y];

        if !self.is_open {
            self.hovered_slot = None;
            return;
        }

        self.hovered_slot = None;
        for slot in 0..TOTAL_SLOTS {
            if let Some((sx, sy, sw, sh)) = Self::get_slot_screen_bounds(slot, screen_width, screen_height) {
                if mouse_x >= sx && mouse_x <= sx + sw && mouse_y >= sy && mouse_y <= sy + sh {
                    self.hovered_slot = Some(slot);
                    break;
                }
            }
        }
    }

    /// Process mouse click on inventory
    pub fn handle_mouse_click(&mut self, button: MouseButton, inventory: &mut PlayerInventory) {
        if !self.is_open {
            return;
        }

        if let Some(slot_idx) = self.hovered_slot {
            match button {
                MouseButton::Left => inventory.handle_left_click(slot_idx),
                MouseButton::Right => inventory.handle_right_click(slot_idx),
                _ => {}
            }
        }
    }

    /// Generates vertices and indices for rendering the full inventory screen
    pub fn build_render_mesh(
        &self,
        inventory: &PlayerInventory,
        atlas: &GuiAtlas,
        screen_width: f32,
        screen_height: f32,
        vertices: &mut Vec<GuiVertex>,
        indices: &mut Vec<u32>,
    ) {
        if !self.is_open {
            return;
        }

        let scale = Self::get_scale(screen_height);
        let (win_x, win_y, win_w, win_h) = Self::get_window_bounds(screen_width, screen_height);

        // 1. Fullscreen dark backdrop dim (RGBA 0, 0, 0, 0.6)
        Self::add_solid_quad(
            vertices,
            indices,
            0.0,
            0.0,
            screen_width,
            screen_height,
            atlas.white_uv,
            [0.0, 0.0, 0.0, 0.6],
        );

        // 2. Inventory container background (176x166)
        Self::add_textured_quad(
            vertices,
            indices,
            win_x,
            win_y,
            win_w,
            win_h,
            atlas.inventory_uv,
            [1.0, 1.0, 1.0, 1.0],
        );

        // 3. Render items in slots (0..45)
        for slot_idx in 0..TOTAL_SLOTS {
            if let Some(stack) = inventory.get_slot(slot_idx) {
                if let Some((sx, sy, sw, sh)) =
                    Self::get_slot_screen_bounds(slot_idx, screen_width, screen_height)
                {
                    Self::render_item_stack(vertices, indices, atlas, sx, sy, sw, sh, stack);
                }
            }
        }

        // 4. Hover highlight over hovered slot
        if let Some(hovered) = self.hovered_slot {
            if let Some((hx, hy, hw, hh)) =
                Self::get_slot_screen_bounds(hovered, screen_width, screen_height)
            {
                Self::add_solid_quad(
                    vertices,
                    indices,
                    hx,
                    hy,
                    hw,
                    hh,
                    atlas.white_uv,
                    [1.0, 1.0, 1.0, 0.35],
                );
            }
        }

        // 5. Floating cursor item following mouse
        if let Some(cursor_stack) = inventory.cursor_item {
            let item_size = SLOT_SIZE * scale;
            let cx = self.mouse_pos[0] - item_size / 2.0;
            let cy = self.mouse_pos[1] - item_size / 2.0;
            Self::render_item_stack(
                vertices,
                indices,
                atlas,
                cx,
                cy,
                item_size,
                item_size,
                cursor_stack,
            );
        }
    }

    /// Render a single item stack with distinct colored item icon and count badge
    fn render_item_stack(
        vertices: &mut Vec<GuiVertex>,
        indices: &mut Vec<u32>,
        atlas: &GuiAtlas,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        stack: ItemStack,
    ) {
        if stack.is_empty() {
            return;
        }

        // Item icon tint color
        let item_color = match stack.id {
            crate::game::inventory::ITEM_OAK_LOG => [0.42, 0.30, 0.18, 1.0],
            crate::game::inventory::ITEM_OAK_PLANKS => [0.72, 0.58, 0.37, 1.0],
            crate::game::inventory::ITEM_STICK => [0.55, 0.40, 0.22, 1.0],
            crate::game::inventory::ITEM_CRAFTING_TABLE => [0.60, 0.45, 0.25, 1.0],
            crate::game::inventory::ITEM_COBBLESTONE => [0.50, 0.50, 0.50, 1.0],
            crate::game::inventory::ITEM_STONE => [0.58, 0.58, 0.58, 1.0],
            crate::game::inventory::ITEM_DIRT => [0.53, 0.37, 0.24, 1.0],
            crate::game::inventory::ITEM_GRASS_BLOCK => [0.35, 0.65, 0.25, 1.0],
            crate::game::inventory::ITEM_DEEPSLATE => [0.30, 0.30, 0.32, 1.0],
            crate::game::inventory::ITEM_GLASS => [0.85, 0.95, 1.0, 0.8],
            crate::game::inventory::ITEM_BRICKS => [0.65, 0.30, 0.25, 1.0],
            _ => [0.70, 0.70, 0.70, 1.0],
        };

        // Inner item quad (14x14 within 16x16)
        let pad = w * 0.08;
        Self::add_solid_quad(
            vertices,
            indices,
            x + pad,
            y + pad,
            w - pad * 2.0,
            h - pad * 2.0,
            atlas.white_uv,
            item_color,
        );

        // Highlight inner border
        Self::add_solid_quad(
            vertices,
            indices,
            x + pad,
            y + pad,
            w - pad * 2.0,
            2.0,
            atlas.white_uv,
            [1.0, 1.0, 1.0, 0.3],
        );

        // If stack count > 1, draw count badge indicator in bottom-right corner
        if stack.count > 1 {
            let badge_w = w * 0.45;
            let badge_h = h * 0.35;
            let bx = x + w - badge_w - 1.0;
            let by = y + h - badge_h - 1.0;

            // Dark background badge
            Self::add_solid_quad(
                vertices,
                indices,
                bx,
                by,
                badge_w,
                badge_h,
                atlas.white_uv,
                [0.1, 0.1, 0.1, 0.85],
            );

            // Light text color badge
            Self::add_solid_quad(
                vertices,
                indices,
                bx + 1.0,
                by + 1.0,
                badge_w - 2.0,
                badge_h - 2.0,
                atlas.white_uv,
                [1.0, 1.0, 1.0, 0.9],
            );
        }
    }

    fn add_solid_quad(
        vertices: &mut Vec<GuiVertex>,
        indices: &mut Vec<u32>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rect: GuiRect,
        color: [f32; 4],
    ) {
        let base_idx = vertices.len() as u32;
        vertices.push(GuiVertex::new([x, y], [rect.u_min, rect.v_min], color));
        vertices.push(GuiVertex::new([x + w, y], [rect.u_max, rect.v_min], color));
        vertices.push(GuiVertex::new([x + w, y + h], [rect.u_max, rect.v_max], color));
        vertices.push(GuiVertex::new([x, y + h], [rect.u_min, rect.v_max], color));

        indices.extend_from_slice(&[
            base_idx,
            base_idx + 1,
            base_idx + 2,
            base_idx,
            base_idx + 2,
            base_idx + 3,
        ]);
    }

    fn add_textured_quad(
        vertices: &mut Vec<GuiVertex>,
        indices: &mut Vec<u32>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rect: GuiRect,
        tint: [f32; 4],
    ) {
        Self::add_solid_quad(vertices, indices, x, y, w, h, rect, tint);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_centering() {
        let (left, top, w, h) = InventoryGui::get_window_bounds(1280.0, 720.0);
        assert_eq!(w, 352.0); // 176 * 2
        assert_eq!(h, 332.0); // 166 * 2
        assert_eq!(left, (1280.0 - 352.0) / 2.0);
        assert_eq!(top, (720.0 - 332.0) / 2.0);
    }

    #[test]
    fn test_all_slot_screen_bounds_valid() {
        for slot in 0..TOTAL_SLOTS {
            let bounds = InventoryGui::get_slot_screen_bounds(slot, 1920.0, 1080.0);
            assert!(bounds.is_some(), "Slot {slot} should have valid bounds");
            let (_, _, w, h) = bounds.unwrap();
            assert_eq!(w, 32.0); // 16 * 2
            assert_eq!(h, 32.0);
        }
    }

    #[test]
    fn test_slot_hover_detection() {
        let mut gui = InventoryGui::new();
        gui.is_open = true;

        // Hover over hotbar slot 0 (index 36)
        let (hx, hy, hw, hh) =
            InventoryGui::get_slot_screen_bounds(36, 1280.0, 720.0).expect("slot 36 exists");
        gui.update_mouse(hx + hw / 2.0, hy + hh / 2.0, 1280.0, 720.0);
        assert_eq!(gui.hovered_slot, Some(36));

        // Hover over crafting result (index 0)
        let (rx, ry, rw, rh) =
            InventoryGui::get_slot_screen_bounds(0, 1280.0, 720.0).expect("slot 0 exists");
        gui.update_mouse(rx + rw / 2.0, ry + rh / 2.0, 1280.0, 720.0);
        assert_eq!(gui.hovered_slot, Some(0));

        // Hover outside window
        gui.update_mouse(0.0, 0.0, 1280.0, 720.0);
        assert_eq!(gui.hovered_slot, None);
    }
}
