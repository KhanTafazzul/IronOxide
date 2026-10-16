// REA Verified: net.minecraft.world.inventory.InventoryMenu (mapped: cjc.class)
// REA Verified: net.minecraft.world.item.crafting.CraftingRecipe (mapped: cpt.class)
//
// Survival Inventory & 2x2 Crafting Matrix.
// Slot layout:
// - Slot 0: Crafting result (1 slot)
// - Slots 1..4: 2x2 Crafting Matrix (4 slots: 1=top-left, 2=top-right, 3=bottom-left, 4=bottom-right)
// - Slots 5..8: Armor slots (4 slots: 5=helmet, 6=chestplate, 7=leggings, 8=boots)
// - Slots 9..35: Main storage (27 slots: 3 rows of 9)
// - Slots 36..44: Hotbar (9 slots: 1 row of 9)
// - Slot 45: Offhand / Shield (1 slot)

use crate::render::atlas::BlockType;

// Item IDs
pub const ITEM_AIR: u16 = 0;
pub const ITEM_GRASS_BLOCK: u16 = 1;
pub const ITEM_DIRT: u16 = 2;
pub const ITEM_STONE: u16 = 3;
pub const ITEM_COBBLESTONE: u16 = 4;
pub const ITEM_DEEPSLATE: u16 = 5;
pub const ITEM_OAK_PLANKS: u16 = 6;
pub const ITEM_OAK_LOG: u16 = 7;
pub const ITEM_OAK_LEAVES: u16 = 8;
pub const ITEM_BEDROCK: u16 = 9;
pub const ITEM_SAND: u16 = 10;
pub const ITEM_GLASS: u16 = 11;
pub const ITEM_BRICKS: u16 = 12;
pub const ITEM_CRAFTING_TABLE: u16 = 13;
pub const ITEM_STICK: u16 = 14;
pub const ITEM_WOODEN_PICKAXE: u16 = 15;
pub const ITEM_WOODEN_AXE: u16 = 16;
pub const ITEM_TORCH: u16 = 17;
pub const ITEM_LAVA: u16 = 18;
pub const ITEM_WATER: u16 = 19;

// REA Slot Constants from cjc.class
pub const RESULT_SLOT: usize = 0;
pub const CRAFT_SLOT_START: usize = 1;
pub const CRAFT_SLOT_END: usize = 5; // 1..4 (len: 4)
pub const ARMOR_SLOT_START: usize = 5;
pub const ARMOR_SLOT_END: usize = 9; // 5..8 (len: 4)
pub const INV_SLOT_START: usize = 9;
pub const INV_SLOT_END: usize = 36; // 9..35 (len: 27)
pub const HOTBAR_SLOT_START: usize = 36;
pub const HOTBAR_SLOT_END: usize = 45; // 36..44 (len: 9)
pub const SHIELD_SLOT: usize = 45; // (len: 1)
pub const TOTAL_SLOTS: usize = 46;

/// Minecraft Item Stack with ID, current count, and maximum stack limit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemStack {
    pub id: u16,
    pub count: u8,
    pub max_stack: u8,
}

impl ItemStack {
    pub fn new(id: u16, count: u8) -> Self {
        Self {
            id,
            count,
            max_stack: 64,
        }
    }

    pub fn with_max_stack(id: u16, count: u8, max_stack: u8) -> Self {
        Self {
            id,
            count,
            max_stack,
        }
    }

    pub fn empty() -> Self {
        Self {
            id: ITEM_AIR,
            count: 0,
            max_stack: 64,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.id == ITEM_AIR || self.count == 0
    }

    pub fn can_stack_with(&self, other: &ItemStack) -> bool {
        !self.is_empty() && !other.is_empty() && self.id == other.id
    }

    pub fn name(&self) -> &'static str {
        match self.id {
            ITEM_AIR => "Air",
            ITEM_GRASS_BLOCK => "Grass Block",
            ITEM_DIRT => "Dirt",
            ITEM_STONE => "Stone",
            ITEM_COBBLESTONE => "Cobblestone",
            ITEM_DEEPSLATE => "Deepslate",
            ITEM_OAK_PLANKS => "Oak Planks",
            ITEM_OAK_LOG => "Oak Log",
            ITEM_OAK_LEAVES => "Oak Leaves",
            ITEM_BEDROCK => "Bedrock",
            ITEM_SAND => "Sand",
            ITEM_GLASS => "Glass",
            ITEM_BRICKS => "Bricks",
            ITEM_TORCH => "Torch",
            ITEM_LAVA => "Lava",
            ITEM_WATER => "Water",
            ITEM_CRAFTING_TABLE => "Crafting Table",
            ITEM_STICK => "Stick",
            _ => "Unknown Item",
        }
    }

    pub fn from_block(block: BlockType, count: u8) -> Option<Self> {
        let id = match block {
            BlockType::Air => return None,
            BlockType::GrassBlock => ITEM_GRASS_BLOCK,
            BlockType::Dirt => ITEM_DIRT,
            BlockType::Stone => ITEM_STONE,
            BlockType::Cobblestone => ITEM_COBBLESTONE,
            BlockType::Deepslate => ITEM_DEEPSLATE,
            BlockType::OakPlanks => ITEM_OAK_PLANKS,
            BlockType::OakLog => ITEM_OAK_LOG,
            BlockType::OakLeaves => ITEM_OAK_LEAVES,
            BlockType::Bedrock => ITEM_BEDROCK,
            BlockType::Sand => ITEM_SAND,
            BlockType::Glass => ITEM_GLASS,
            BlockType::Bricks => ITEM_BRICKS,
            BlockType::Torch => ITEM_TORCH,
            BlockType::Lava => ITEM_LAVA,
            BlockType::Water => ITEM_WATER,
        };
        Some(Self::new(id, count))
    }

    pub fn to_block(&self) -> Option<BlockType> {
        match self.id {
            ITEM_GRASS_BLOCK => Some(BlockType::GrassBlock),
            ITEM_DIRT => Some(BlockType::Dirt),
            ITEM_STONE => Some(BlockType::Stone),
            ITEM_COBBLESTONE => Some(BlockType::Cobblestone),
            ITEM_DEEPSLATE => Some(BlockType::Deepslate),
            ITEM_OAK_PLANKS => Some(BlockType::OakPlanks),
            ITEM_OAK_LOG => Some(BlockType::OakLog),
            ITEM_OAK_LEAVES => Some(BlockType::OakLeaves),
            ITEM_BEDROCK => Some(BlockType::Bedrock),
            ITEM_SAND => Some(BlockType::Sand),
            ITEM_GLASS => Some(BlockType::Glass),
            ITEM_BRICKS => Some(BlockType::Bricks),
            ITEM_TORCH => Some(BlockType::Torch),
            ITEM_LAVA => Some(BlockType::Lava),
            ITEM_WATER => Some(BlockType::Water),
            _ => None,
        }
    }
}

/// Vanilla 2x2 Crafting Recipe Registry
pub struct RecipeRegistry;

impl RecipeRegistry {
    /// Evaluates a 2x2 crafting matrix (grid length 4) and returns the output ItemStack
    pub fn evaluate_2x2(grid: &[Option<ItemStack>; 4]) -> Option<ItemStack> {
        let mut count_items = 0;
        let mut first_item = None;

        for (i, slot) in grid.iter().enumerate() {
            if let Some(stack) = slot {
                if !stack.is_empty() {
                    count_items += 1;
                    if first_item.is_none() {
                        first_item = Some((i, *stack));
                    }
                }
            }
        }

        if count_items == 0 {
            return None;
        }

        // Recipe 1: 1 Oak Log -> 4 Oak Planks (Shapeless: exactly 1 log in any slot)
        if count_items == 1 {
            if let Some((_, stack)) = first_item {
                if stack.id == ITEM_OAK_LOG {
                    return Some(ItemStack::new(ITEM_OAK_PLANKS, 4));
                }
            }
        }

        // Recipe 2: 4 Planks -> 1 Crafting Table (2x2 full grid of Oak Planks)
        if count_items == 4 {
            let all_planks = grid.iter().all(|s| {
                if let Some(stack) = s {
                    stack.id == ITEM_OAK_PLANKS
                } else {
                    false
                }
            });
            if all_planks {
                return Some(ItemStack::new(ITEM_CRAFTING_TABLE, 1));
            }
        }

        // Recipe 3: 2 Planks -> 4 Sticks (Vertical column of 2 planks)
        // Grid indices:
        // [0, 1]
        // [2, 3]
        if count_items == 2 {
            // Column 0: slots 0 and 2
            let col0_match = match (grid[0], grid[2], grid[1], grid[3]) {
                (Some(s0), Some(s2), None, None)
                    if s0.id == ITEM_OAK_PLANKS && s2.id == ITEM_OAK_PLANKS => true,
                _ => false,
            };
            // Column 1: slots 1 and 3
            let col1_match = match (grid[1], grid[3], grid[0], grid[2]) {
                (Some(s1), Some(s3), None, None)
                    if s1.id == ITEM_OAK_PLANKS && s3.id == ITEM_OAK_PLANKS => true,
                _ => false,
            };

            if col0_match || col1_match {
                return Some(ItemStack::new(ITEM_STICK, 4));
            }
        }

        None
    }
}

/// Full Player Survival Inventory State
#[derive(Debug, Clone)]
pub struct PlayerInventory {
    // Slot 0: Crafting result
    pub crafting_result: Option<ItemStack>,
    // Slots 1..4: Crafting matrix (0=top-left, 1=top-right, 2=bottom-left, 3=bottom-right)
    pub crafting_grid: [Option<ItemStack>; 4],
    // Slots 5..8: Armor (0=helmet, 1=chestplate, 2=leggings, 3=boots)
    pub armor: [Option<ItemStack>; 4],
    // Slots 9..35: Main inventory storage (3 rows of 9)
    pub main: [Option<ItemStack>; 27],
    // Slots 36..44: Hotbar (9 slots)
    pub hotbar: [Option<ItemStack>; 9],
    // Slot 45: Offhand / Shield
    pub offhand: Option<ItemStack>,
    // Floating item held on cursor
    pub cursor_item: Option<ItemStack>,
}

impl Default for PlayerInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerInventory {
    pub fn new() -> Self {
        let mut inv = Self {
            crafting_result: None,
            crafting_grid: [None; 4],
            armor: [None; 4],
            main: [None; 27],
            hotbar: [None; 9],
            offhand: None,
            cursor_item: None,
        };

        // Populate starter survival items in hotbar
        inv.hotbar[0] = Some(ItemStack::new(ITEM_OAK_LOG, 16));
        inv.hotbar[1] = Some(ItemStack::new(ITEM_COBBLESTONE, 32));
        inv.hotbar[2] = Some(ItemStack::new(ITEM_DIRT, 64));
        inv.hotbar[3] = Some(ItemStack::new(ITEM_STONE, 64));

        inv
    }

    /// Adds an item stack to player inventory (hotbar first, then main storage).
    /// Returns Some(remaining_stack) if inventory is full, or None if fully added.
    pub fn add_item(&mut self, mut stack: ItemStack) -> Option<ItemStack> {
        if stack.is_empty() {
            return None;
        }

        // 1. Try to merge with existing stacks in hotbar
        for slot in &mut self.hotbar {
            if let Some(existing) = slot {
                if existing.can_stack_with(&stack) && existing.count < existing.max_stack {
                    let space = existing.max_stack - existing.count;
                    let to_add = stack.count.min(space);
                    existing.count += to_add;
                    stack.count -= to_add;
                    if stack.count == 0 {
                        return None;
                    }
                }
            }
        }

        // 2. Try to merge with existing stacks in main storage
        for slot in &mut self.main {
            if let Some(existing) = slot {
                if existing.can_stack_with(&stack) && existing.count < existing.max_stack {
                    let space = existing.max_stack - existing.count;
                    let to_add = stack.count.min(space);
                    existing.count += to_add;
                    stack.count -= to_add;
                    if stack.count == 0 {
                        return None;
                    }
                }
            }
        }

        // 3. Place into first empty slot in hotbar
        for slot in &mut self.hotbar {
            if slot.is_none() {
                *slot = Some(stack);
                return None;
            }
        }

        // 4. Place into first empty slot in main storage
        for slot in &mut self.main {
            if slot.is_none() {
                *slot = Some(stack);
                return None;
            }
        }

        // Inventory is completely full; return remainder
        Some(stack)
    }

    /// Retrieve an item from the unified slot index (0..45)
    pub fn get_slot(&self, index: usize) -> Option<ItemStack> {
        match index {
            RESULT_SLOT => self.crafting_result,
            1..=4 => self.crafting_grid[index - CRAFT_SLOT_START],
            5..=8 => self.armor[index - ARMOR_SLOT_START],
            9..=35 => self.main[index - INV_SLOT_START],
            36..=44 => self.hotbar[index - HOTBAR_SLOT_START],
            SHIELD_SLOT => self.offhand,
            _ => None,
        }
    }

    /// Set an item in the unified slot index (0..45)
    pub fn set_slot(&mut self, index: usize, stack: Option<ItemStack>) {
        let sanitized = stack.and_then(|s| if s.is_empty() { None } else { Some(s) });
        match index {
            RESULT_SLOT => self.crafting_result = sanitized,
            1..=4 => {
                self.crafting_grid[index - CRAFT_SLOT_START] = sanitized;
                self.update_crafting_result();
            }
            5..=8 => self.armor[index - ARMOR_SLOT_START] = sanitized,
            9..=35 => self.main[index - INV_SLOT_START] = sanitized,
            36..=44 => self.hotbar[index - HOTBAR_SLOT_START] = sanitized,
            SHIELD_SLOT => self.offhand = sanitized,
            _ => {}
        }
    }

    /// Recalculates the crafting output based on the 2x2 grid
    pub fn update_crafting_result(&mut self) {
        self.crafting_result = RecipeRegistry::evaluate_2x2(&self.crafting_grid);
    }

    /// Left-Click on a slot index (0..45):
    /// - Pick up full stack if cursor empty.
    /// - Place full stack if slot empty.
    /// - Merge stacks if same type.
    /// - Swap stacks if different types.
    /// - Craft output if clicking slot 0.
    pub fn handle_left_click(&mut self, slot_idx: usize) {
        if slot_idx >= TOTAL_SLOTS {
            return;
        }

        // Special handling for Crafting Result Slot (0)
        if slot_idx == RESULT_SLOT {
            self.take_crafting_result_left();
            return;
        }

        let slot_item = self.get_slot(slot_idx);

        match (self.cursor_item, slot_item) {
            // Cursor empty, slot has item -> pick up full stack
            (None, Some(item)) => {
                self.cursor_item = Some(item);
                self.set_slot(slot_idx, None);
            }
            // Cursor has item, slot empty -> place full stack
            (Some(cursor), None) => {
                self.set_slot(slot_idx, Some(cursor));
                self.cursor_item = None;
            }
            // Both have items
            (Some(mut cursor), Some(mut slot)) => {
                if cursor.can_stack_with(&slot) {
                    let available_space = slot.max_stack.saturating_sub(slot.count);
                    let to_transfer = cursor.count.min(available_space);
                    if to_transfer > 0 {
                        slot.count += to_transfer;
                        cursor.count -= to_transfer;
                        self.set_slot(slot_idx, Some(slot));
                        self.cursor_item = if cursor.count > 0 { Some(cursor) } else { None };
                    }
                } else {
                    // Different items: swap
                    self.set_slot(slot_idx, Some(cursor));
                    self.cursor_item = Some(slot);
                }
            }
            (None, None) => {}
        }
    }

    /// Right-Click on a slot index (0..45):
    /// - Split stack if cursor empty (take half rounded up into cursor).
    /// - Place 1 item if cursor has items and slot is empty or compatible.
    pub fn handle_right_click(&mut self, slot_idx: usize) {
        if slot_idx >= TOTAL_SLOTS {
            return;
        }

        // Special handling for Crafting Result Slot (0)
        if slot_idx == RESULT_SLOT {
            self.take_crafting_result_left();
            return;
        }

        let slot_item = self.get_slot(slot_idx);

        match (self.cursor_item, slot_item) {
            // Cursor empty, slot has items -> split half into cursor
            (None, Some(mut slot)) => {
                if slot.count == 1 {
                    self.cursor_item = Some(slot);
                    self.set_slot(slot_idx, None);
                } else {
                    let take_count = (slot.count + 1) / 2;
                    let remain_count = slot.count - take_count;
                    self.cursor_item = Some(ItemStack {
                        id: slot.id,
                        count: take_count,
                        max_stack: slot.max_stack,
                    });
                    slot.count = remain_count;
                    self.set_slot(slot_idx, Some(slot));
                }
            }
            // Cursor has items, slot empty -> place 1 item
            (Some(mut cursor), None) => {
                let single = ItemStack {
                    id: cursor.id,
                    count: 1,
                    max_stack: cursor.max_stack,
                };
                self.set_slot(slot_idx, Some(single));
                cursor.count -= 1;
                self.cursor_item = if cursor.count > 0 { Some(cursor) } else { None };
            }
            // Cursor has items, slot has items
            (Some(mut cursor), Some(mut slot)) => {
                if cursor.can_stack_with(&slot) && slot.count < slot.max_stack {
                    slot.count += 1;
                    cursor.count -= 1;
                    self.set_slot(slot_idx, Some(slot));
                    self.cursor_item = if cursor.count > 0 { Some(cursor) } else { None };
                }
            }
            (None, None) => {}
        }
    }

    /// Handles picking up or stacking crafting result output
    fn take_crafting_result_left(&mut self) {
        if let Some(result) = self.crafting_result {
            match self.cursor_item {
                None => {
                    self.cursor_item = Some(result);
                    self.consume_crafting_ingredients();
                }
                Some(mut cursor) if cursor.can_stack_with(&result) => {
                    if cursor.count + result.count <= cursor.max_stack {
                        cursor.count += result.count;
                        self.cursor_item = Some(cursor);
                        self.consume_crafting_ingredients();
                    }
                }
                _ => {}
            }
        }
    }

    /// Consumes 1 item from every non-empty slot in the 2x2 crafting grid
    fn consume_crafting_ingredients(&mut self) {
        for slot in &mut self.crafting_grid {
            if let Some(mut stack) = *slot {
                if stack.count > 1 {
                    stack.count -= 1;
                    *slot = Some(stack);
                } else {
                    *slot = None;
                }
            }
        }
        self.update_crafting_result();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_indexing() {
        let mut inv = PlayerInventory::new();

        // Hotbar slot 0 maps to index 36
        assert_eq!(inv.get_slot(36).map(|s| s.id), Some(ITEM_OAK_LOG));

        // Set crafting slot
        inv.set_slot(1, Some(ItemStack::new(ITEM_DIRT, 5)));
        assert_eq!(inv.get_slot(1), Some(ItemStack::new(ITEM_DIRT, 5)));
    }

    #[test]
    fn test_oak_log_to_planks_recipe() {
        let mut inv = PlayerInventory::new();
        inv.crafting_grid[0] = Some(ItemStack::new(ITEM_OAK_LOG, 1));
        inv.update_crafting_result();

        assert_eq!(
            inv.crafting_result,
            Some(ItemStack::new(ITEM_OAK_PLANKS, 4))
        );

        // Click result slot to take planks
        inv.handle_left_click(RESULT_SLOT);
        assert_eq!(inv.cursor_item, Some(ItemStack::new(ITEM_OAK_PLANKS, 4)));
        assert_eq!(inv.crafting_grid[0], None);
        assert_eq!(inv.crafting_result, None);
    }

    #[test]
    fn test_planks_to_crafting_table_recipe() {
        let mut inv = PlayerInventory::new();
        for i in 0..4 {
            inv.crafting_grid[i] = Some(ItemStack::new(ITEM_OAK_PLANKS, 1));
        }
        inv.update_crafting_result();

        assert_eq!(
            inv.crafting_result,
            Some(ItemStack::new(ITEM_CRAFTING_TABLE, 1))
        );

        inv.handle_left_click(RESULT_SLOT);
        assert_eq!(inv.cursor_item, Some(ItemStack::new(ITEM_CRAFTING_TABLE, 1)));
        for i in 0..4 {
            assert_eq!(inv.crafting_grid[i], None);
        }
    }

    #[test]
    fn test_planks_to_sticks_recipe() {
        let mut inv = PlayerInventory::new();
        // Column 0: slots 0 and 2
        inv.crafting_grid[0] = Some(ItemStack::new(ITEM_OAK_PLANKS, 2));
        inv.crafting_grid[2] = Some(ItemStack::new(ITEM_OAK_PLANKS, 2));
        inv.update_crafting_result();

        assert_eq!(inv.crafting_result, Some(ItemStack::new(ITEM_STICK, 4)));

        inv.handle_left_click(RESULT_SLOT);
        assert_eq!(inv.cursor_item, Some(ItemStack::new(ITEM_STICK, 4)));
        assert_eq!(inv.crafting_grid[0], Some(ItemStack::new(ITEM_OAK_PLANKS, 1)));
        assert_eq!(inv.crafting_grid[2], Some(ItemStack::new(ITEM_OAK_PLANKS, 1)));
        // Crafting result updates for remaining items
        assert_eq!(inv.crafting_result, Some(ItemStack::new(ITEM_STICK, 4)));
    }

    #[test]
    fn test_left_click_merge_and_swap() {
        let mut inv = PlayerInventory::new();
        inv.main[0] = Some(ItemStack::new(ITEM_COBBLESTONE, 30));
        inv.cursor_item = Some(ItemStack::new(ITEM_COBBLESTONE, 40));

        // Click slot 9 (main[0]) -> merges up to 64
        inv.handle_left_click(9);
        assert_eq!(inv.get_slot(9), Some(ItemStack::new(ITEM_COBBLESTONE, 64)));
        assert_eq!(inv.cursor_item, Some(ItemStack::new(ITEM_COBBLESTONE, 6)));

        // Click slot with different item -> swaps
        inv.set_slot(10, Some(ItemStack::new(ITEM_STONE, 10)));
        inv.handle_left_click(10);
        assert_eq!(inv.get_slot(10), Some(ItemStack::new(ITEM_COBBLESTONE, 6)));
        assert_eq!(inv.cursor_item, Some(ItemStack::new(ITEM_STONE, 10)));
    }

    #[test]
    fn test_right_click_split_and_place_single() {
        let mut inv = PlayerInventory::new();
        inv.set_slot(9, Some(ItemStack::new(ITEM_OAK_LOG, 10)));
        inv.cursor_item = None;

        // Right click slot 9 splits in half (5 and 5)
        inv.handle_right_click(9);
        assert_eq!(inv.get_slot(9), Some(ItemStack::new(ITEM_OAK_LOG, 5)));
        assert_eq!(inv.cursor_item, Some(ItemStack::new(ITEM_OAK_LOG, 5)));

        // Right click empty slot 10 places 1 item
        inv.handle_right_click(10);
        assert_eq!(inv.get_slot(10), Some(ItemStack::new(ITEM_OAK_LOG, 1)));
        assert_eq!(inv.cursor_item, Some(ItemStack::new(ITEM_OAK_LOG, 4)));
    }
}
