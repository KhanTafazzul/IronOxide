use mc_crafting::{ItemStack, RecipeLookupCache};

/// Interactive 3x3 Crafting Table workstation container menu.
/// REA Reference: net.minecraft.world.inventory.CraftingMenu (cis.class)
#[derive(Clone, Debug, Default)]
pub struct CraftingTableMenu {
    /// 3x3 input crafting matrix (indices 0..8)
    pub grid: [Option<ItemStack>; 9],
    /// Current evaluated recipe output preview (slot 0 in container packet protocol)
    pub output_preview: Option<ItemStack>,
}

impl CraftingTableMenu {
    pub fn new() -> Self {
        Self {
            grid: [None; 9],
            output_preview: None,
        }
    }

    /// Sets or clears an item stack in the 3x3 matrix and updates the output preview in O(1).
    pub fn set_slot(&mut self, slot: usize, stack: Option<ItemStack>, cache: &RecipeLookupCache) {
        if slot < 9 {
            self.grid[slot] = stack;
            self.update_output(cache);
        }
    }

    /// Gets the item stack at the given slot (0..8 for input grid).
    pub fn get_slot(&self, slot: usize) -> Option<ItemStack> {
        if slot < 9 {
            self.grid[slot]
        } else {
            None
        }
    }

    /// Updates the output preview based on the current 3x3 matrix contents using the lookup cache.
    pub fn update_output(&mut self, cache: &RecipeLookupCache) {
        self.output_preview = cache.compute_output(&self.grid);
    }

    /// Crafts and takes the output item.
    /// Decrements 1 item from each populated slot in the 3x3 grid and recalculates the output preview.
    pub fn take_output(&mut self, cache: &RecipeLookupCache) -> Option<ItemStack> {
        let output = self.output_preview?;

        // Decrement 1 from every populated slot
        for slot in &mut self.grid {
            if let Some(stack) = slot {
                if stack.count <= 1 {
                    *slot = None;
                } else {
                    stack.count -= 1;
                }
            }
        }

        // Re-evaluate output preview with updated grid
        self.update_output(cache);

        Some(output)
    }

    /// Handles closing the crafting table UI (net.minecraft.world.inventory.CraftingMenu.removed / cis.b).
    /// Gathers all remaining items in the 3x3 grid to be returned to the player or dropped onto the ground.
    /// Clears both the crafting grid and the output preview.
    pub fn close_and_drop_items(&mut self) -> Vec<ItemStack> {
        let mut dropped = Vec::new();

        for slot in &mut self.grid {
            if let Some(stack) = slot.take() {
                dropped.push(stack);
            }
        }

        self.output_preview = None;
        dropped
    }
}
