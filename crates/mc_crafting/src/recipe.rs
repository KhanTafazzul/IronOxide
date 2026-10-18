use crate::item_stack::ItemStack;
use mc_items::ItemId;

/// Crafting ingredient containing a set of allowed ItemIds (from single item, tag, or list).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Ingredient {
    pub allowed: Vec<ItemId>,
}

impl Ingredient {
    pub fn new(allowed: Vec<ItemId>) -> Self {
        Self { allowed }
    }

    pub fn single(item: ItemId) -> Self {
        Self { allowed: vec![item] }
    }

    pub fn matches(&self, item: ItemId) -> bool {
        self.allowed.contains(&item)
    }
}

/// Shaped crafting recipe matching a 2D matrix layout with horizontal mirror support.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ShapedRecipe {
    pub id: String,
    pub width: usize,
    pub height: usize,
    pub pattern: Vec<Option<Ingredient>>,
    pub result: ItemStack,
    pub mirrored: bool,
}

impl ShapedRecipe {
    pub fn new(
        id: String,
        width: usize,
        height: usize,
        pattern: Vec<Option<Ingredient>>,
        result: ItemStack,
        mirrored: bool,
    ) -> Self {
        Self {
            id,
            width,
            height,
            pattern,
            result,
            mirrored,
        }
    }

    /// Evaluates if this shaped recipe matches the given 3x3 crafting grid.
    pub fn matches(&self, grid: &[Option<ItemStack>; 9]) -> Option<ItemStack> {
        if self.width > 3 || self.height > 3 {
            return None;
        }

        // Try every possible (offset_x, offset_y) placement in the 3x3 matrix
        for oy in 0..=(3 - self.height) {
            for ox in 0..=(3 - self.width) {
                // Try standard unmirrored placement
                if self.check_match_at(grid, ox, oy, false) {
                    return Some(self.result);
                }

                // If symmetrical / mirrored allowed, try horizontally mirrored placement
                if self.mirrored && self.check_match_at(grid, ox, oy, true) {
                    return Some(self.result);
                }
            }
        }

        None
    }

    fn check_match_at(&self, grid: &[Option<ItemStack>; 9], ox: usize, oy: usize, mirrored: bool) -> bool {
        for gy in 0..3 {
            for gx in 0..3 {
                let slot_idx = gy * 3 + gx;
                let is_inside_recipe = gx >= ox && gx < ox + self.width && gy >= oy && gy < oy + self.height;

                if is_inside_recipe {
                    let rx = gx - ox;
                    let ry = gy - oy;
                    let pattern_x = if mirrored { (self.width - 1) - rx } else { rx };
                    let pattern_idx = ry * self.width + pattern_x;
                    let expected_ingredient = &self.pattern[pattern_idx];

                    match (expected_ingredient, &grid[slot_idx]) {
                        (Some(ing), Some(stack)) => {
                            if !ing.matches(stack.item) {
                                return false;
                            }
                        }
                        (None, None) => {}
                        _ => return false,
                    }
                } else {
                    // Any slot outside the recipe boundaries must be empty
                    if grid[slot_idx].is_some() {
                        return false;
                    }
                }
            }
        }

        true
    }
}

/// Shapeless crafting recipe where item slot positions in the crafting matrix do not matter.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ShapelessRecipe {
    pub id: String,
    pub ingredients: Vec<Ingredient>,
    pub result: ItemStack,
}

impl ShapelessRecipe {
    pub fn new(id: String, ingredients: Vec<Ingredient>, result: ItemStack) -> Self {
        Self {
            id,
            ingredients,
            result,
        }
    }

    /// Evaluates if this shapeless recipe matches the given 3x3 crafting grid.
    pub fn matches(&self, grid: &[Option<ItemStack>; 9]) -> Option<ItemStack> {
        let items: Vec<ItemId> = grid.iter().filter_map(|s| s.map(|st| st.item)).collect();

        if items.len() != self.ingredients.len() {
            return None;
        }

        if Self::match_backtrack(&items, &self.ingredients, 0, 0) {
            Some(self.result)
        } else {
            None
        }
    }

    fn match_backtrack(
        items: &[ItemId],
        ingredients: &[Ingredient],
        matched_mask: u16,
        item_idx: usize,
    ) -> bool {
        if item_idx == items.len() {
            return true;
        }

        let current_item = items[item_idx];

        for (ing_idx, ing) in ingredients.iter().enumerate() {
            if (matched_mask & (1 << ing_idx)) == 0 && ing.matches(current_item) {
                if Self::match_backtrack(items, ingredients, matched_mask | (1 << ing_idx), item_idx + 1) {
                    return true;
                }
            }
        }

        false
    }
}

/// Unified crafting recipe enum.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CraftingRecipe {
    Shaped(ShapedRecipe),
    Shapeless(ShapelessRecipe),
}

impl CraftingRecipe {
    pub fn matches(&self, grid: &[Option<ItemStack>; 9]) -> Option<ItemStack> {
        match self {
            CraftingRecipe::Shaped(r) => r.matches(grid),
            CraftingRecipe::Shapeless(r) => r.matches(grid),
        }
    }

    pub fn id(&self) -> &str {
        match self {
            CraftingRecipe::Shaped(r) => &r.id,
            CraftingRecipe::Shapeless(r) => &r.id,
        }
    }

    pub fn result(&self) -> ItemStack {
        match self {
            CraftingRecipe::Shaped(r) => r.result,
            CraftingRecipe::Shapeless(r) => r.result,
        }
    }
}
