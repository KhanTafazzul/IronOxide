use mc_items::ItemId;
use serde::{Deserialize, Serialize};

/// Represents an item stack with an ItemId and count.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ItemStack {
    pub item: ItemId,
    pub count: u8,
}

impl ItemStack {
    pub const fn new(item: ItemId, count: u8) -> Self {
        Self { item, count }
    }

    pub const fn single(item: ItemId) -> Self {
        Self { item, count: 1 }
    }
}

impl From<ItemId> for ItemStack {
    fn from(item: ItemId) -> Self {
        Self::single(item)
    }
}

