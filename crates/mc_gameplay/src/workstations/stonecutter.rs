//! Stonecutter workstation container menu and recipes.
//! REA Reference: net.minecraft.world.inventory.StonecutterMenu (cka.class)

use mc_crafting::ItemStack;
use mc_items::ItemId;

/// Stonecutter cutting recipe.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct StonecutterRecipe {
    pub input: ItemId,
    pub output: ItemId,
    pub count: u8,
}

impl StonecutterRecipe {
    pub const fn new(input: ItemId, output: ItemId, count: u8) -> Self {
        Self { input, output, count }
    }
}

/// Returns all available stonecutter outputs for a given input block item.
pub fn get_stonecutter_recipes(input: ItemId) -> Vec<StonecutterRecipe> {
    match input {
        ItemId::Stone => vec![
            StonecutterRecipe::new(input, ItemId::StoneSlab, 2),
            StonecutterRecipe::new(input, ItemId::StoneStairs, 1),
            StonecutterRecipe::new(input, ItemId::StoneBricks, 1),
            StonecutterRecipe::new(input, ItemId::ChiseledStoneBricks, 1),
        ],
        ItemId::Cobblestone => vec![
            StonecutterRecipe::new(input, ItemId::CobblestoneSlab, 2),
            StonecutterRecipe::new(input, ItemId::CobblestoneStairs, 1),
            StonecutterRecipe::new(input, ItemId::CobblestoneWall, 1),
        ],
        ItemId::StoneBricks => vec![
            StonecutterRecipe::new(input, ItemId::StoneBrickSlab, 2),
            StonecutterRecipe::new(input, ItemId::StoneBrickStairs, 1),
            StonecutterRecipe::new(input, ItemId::StoneBrickWall, 1),
            StonecutterRecipe::new(input, ItemId::ChiseledStoneBricks, 1),
        ],
        ItemId::Andesite => vec![
            StonecutterRecipe::new(input, ItemId::AndesiteSlab, 2),
            StonecutterRecipe::new(input, ItemId::AndesiteStairs, 1),
            StonecutterRecipe::new(input, ItemId::AndesiteWall, 1),
            StonecutterRecipe::new(input, ItemId::PolishedAndesite, 1),
        ],
        ItemId::Diorite => vec![
            StonecutterRecipe::new(input, ItemId::DioriteSlab, 2),
            StonecutterRecipe::new(input, ItemId::DioriteStairs, 1),
            StonecutterRecipe::new(input, ItemId::DioriteWall, 1),
            StonecutterRecipe::new(input, ItemId::PolishedDiorite, 1),
        ],
        ItemId::Granite => vec![
            StonecutterRecipe::new(input, ItemId::GraniteSlab, 2),
            StonecutterRecipe::new(input, ItemId::GraniteStairs, 1),
            StonecutterRecipe::new(input, ItemId::GraniteWall, 1),
            StonecutterRecipe::new(input, ItemId::PolishedGranite, 1),
        ],
        ItemId::Deepslate => vec![
            StonecutterRecipe::new(input, ItemId::CobbledDeepslate, 1),
            StonecutterRecipe::new(input, ItemId::PolishedDeepslate, 1),
            StonecutterRecipe::new(input, ItemId::DeepslateBricks, 1),
            StonecutterRecipe::new(input, ItemId::DeepslateTiles, 1),
            StonecutterRecipe::new(input, ItemId::ChiseledDeepslate, 1),
        ],
        ItemId::CopperBlock => vec![
            StonecutterRecipe::new(input, ItemId::CutCopper, 4),
            StonecutterRecipe::new(input, ItemId::CopperGrate, 4),
        ],
        _ => Vec::new(),
    }
}

/// Interactive Stonecutter workstation container menu.
/// REA Reference: net.minecraft.world.inventory.StonecutterMenu (cka.class)
pub struct StonecutterMenu {
    /// Slot 0: Input block item
    pub input_slot: Option<ItemStack>,
    /// Selected recipe index within available recipes
    pub selected_recipe_index: Option<usize>,
}

impl StonecutterMenu {
    pub fn new() -> Self {
        Self {
            input_slot: None,
            selected_recipe_index: None,
        }
    }

    /// Sets input slot and resets selected recipe index.
    pub fn set_input(&mut self, stack: Option<ItemStack>) {
        self.input_slot = stack;
        self.selected_recipe_index = None;
    }

    /// Returns the list of possible cutting recipes for the current input item.
    pub fn available_recipes(&self) -> Vec<StonecutterRecipe> {
        if let Some(input) = self.input_slot {
            get_stonecutter_recipes(input.item)
        } else {
            Vec::new()
        }
    }

    /// Selects a recipe by index and evaluates the preview output.
    pub fn select_recipe(&mut self, recipe_idx: usize) -> Option<ItemStack> {
        let recipes = self.available_recipes();
        if recipe_idx < recipes.len() {
            self.selected_recipe_index = Some(recipe_idx);
            let rec = &recipes[recipe_idx];
            Some(ItemStack::new(rec.output, rec.count))
        } else {
            self.selected_recipe_index = None;
            None
        }
    }

    /// Takes the stonecut output item, consuming 1 input item.
    pub fn take_output(&mut self) -> Option<ItemStack> {
        let selected_idx = self.selected_recipe_index?;
        let recipes = self.available_recipes();
        let rec = recipes.get(selected_idx)?;

        let input = self.input_slot.as_mut()?;
        if input.count <= 1 {
            self.input_slot = None;
            self.selected_recipe_index = None;
        } else {
            input.count -= 1;
        }

        Some(ItemStack::new(rec.output, rec.count))
    }

    /// Handles closing the UI and dropping any remaining input item.
    pub fn close_and_drop_items(&mut self) -> Option<ItemStack> {
        self.selected_recipe_index = None;
        self.input_slot.take()
    }
}
