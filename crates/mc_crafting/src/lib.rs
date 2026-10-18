//! Automated Minecraft recipe parser, shaped/shapeless deserializer, and O(1) crafting lookup engine.

pub mod item_stack;
pub mod recipe;
pub mod recipe_loader;

pub use item_stack::ItemStack;
pub use recipe::{CraftingRecipe, Ingredient, ShapedRecipe, ShapelessRecipe};
pub use recipe_loader::{RecipeLoader, RecipeLookupCache};
