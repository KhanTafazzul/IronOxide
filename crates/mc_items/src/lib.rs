//! Comprehensive Minecraft ItemId registry and Tool/Armor attributes.
//! Supports Minecraft 1.20.4 and 1.21 items.

pub mod item_id;
pub mod tool_tier;

pub use item_id::ItemId;
pub use tool_tier::{ArmorSlot, ArmorTier, ToolTier, ToolType};
