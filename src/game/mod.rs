pub mod entities;
pub mod inventory;
pub mod mining;
pub mod physics;
pub mod raycast;
pub mod stats;
pub mod world;

pub use entities::{ItemEntity, ItemEntityManager};

#[allow(unused_imports)]
pub use stats::PlayerStats;

#[allow(unused_imports)]
pub use inventory::{
    ItemStack, PlayerInventory, RecipeRegistry, CRAFT_SLOT_END, CRAFT_SLOT_START,
    HOTBAR_SLOT_END, HOTBAR_SLOT_START, INV_SLOT_END, INV_SLOT_START, RESULT_SLOT,
    SHIELD_SLOT, TOTAL_SLOTS,
};
#[allow(unused_imports)]
pub use mining::{calculate_break_time, get_block_hardness, BlockMiningTracker, MiningUpdateResult};
#[allow(unused_imports)]
pub use physics::{Aabb, PlayerInput, PlayerPhysics};
#[allow(unused_imports)]
pub use physics::{
    DEFAULT_STEP_HEIGHT, GRAVITY, JUMP_VELOCITY, PLAYER_EYE_HEIGHT, PLAYER_HEIGHT, PLAYER_WIDTH,
};
#[allow(unused_imports)]
pub use raycast::{raycast_voxel, RaycastHit, DEFAULT_MAX_REACH};
pub use world::World;
#[allow(unused_imports)]
pub use crate::world::lighting;

