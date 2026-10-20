pub mod chunk_manager;
pub mod entities;
pub mod inventory;
pub mod mining;
pub mod mobs;
pub mod physics;
pub mod portals;
pub mod raycast;
pub mod stats;
pub mod world;

pub use chunk_manager::{ChunkManager, ChunkStatus, GpuChunkMesh};

pub use entities::{ItemEntity, ItemEntityManager};
pub use mobs::{MobEcs, MobEntity, MobEvent, MobType, MobAiState};
pub use portals::{
    Dimension, EndPortalRing, NetherPortalShape, PortalAxis, PortalTeleportEvent,
    PortalTeleportState, build_nether_portal_at, ensure_end_spawn_platform,
    find_existing_portal, translate_coordinates,
};

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

pub use mc_items::{ArmorSlot, ArmorTier, ItemId, ToolTier, ToolType};
pub use mc_crafting::{
    CraftingRecipe, Ingredient, RecipeLoader, RecipeLookupCache, ShapedRecipe, ShapelessRecipe,
    ItemStack as CraftingItemStack,
};
pub use mc_gameplay::workstations::{
    calculate_next_prior_work_penalty, get_fuel_burn_ticks, get_smelting_recipe,
    AnvilDamageState, AnvilItem, AnvilMenu, CraftingTableMenu, EnchantmentInstance,
    FurnaceTileEntity, FurnaceType, SmeltingRecipe, SmithingResult, SmithingTableMenu,
    StonecutterMenu, StonecutterRecipe,
};
pub mod workstations {
    pub use mc_gameplay::workstations::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iron_oxide_item_registry_and_crafting_integration() {
        // Test ItemId and ToolTier
        assert_eq!(ItemId::DiamondSword.name(), "minecraft:diamond_sword");
        assert_eq!(ToolTier::Diamond.durability(), 1561);
        assert_eq!(ToolType::Sword.attack_damage(ToolTier::Diamond), 7.0);

        // Test RecipeLookupCache with shaped Iron Pickaxe
        let iron_pick_recipe = ShapedRecipe::new(
            "minecraft:iron_pickaxe".to_string(),
            3,
            3,
            vec![
                Some(Ingredient::single(ItemId::IronIngot)),
                Some(Ingredient::single(ItemId::IronIngot)),
                Some(Ingredient::single(ItemId::IronIngot)),
                None,
                Some(Ingredient::single(ItemId::Stick)),
                None,
                None,
                Some(Ingredient::single(ItemId::Stick)),
                None,
            ],
            CraftingItemStack::new(ItemId::IronPickaxe, 1),
            true,
        );

        let cache = RecipeLookupCache::new(vec![CraftingRecipe::Shaped(iron_pick_recipe)]);

        let mut grid = [None; 9];
        grid[0] = Some(CraftingItemStack::new(ItemId::IronIngot, 1));
        grid[1] = Some(CraftingItemStack::new(ItemId::IronIngot, 1));
        grid[2] = Some(CraftingItemStack::new(ItemId::IronIngot, 1));
        grid[4] = Some(CraftingItemStack::new(ItemId::Stick, 1));
        grid[7] = Some(CraftingItemStack::new(ItemId::Stick, 1));

        let out = cache.compute_output(&grid);
        assert_eq!(out, Some(CraftingItemStack::new(ItemId::IronPickaxe, 1)));
    }

    #[test]
    fn test_iron_oxide_workstations_integration() {
        // Verify Crafting Table, Furnace, Anvil, Smithing Table, Stonecutter
        let mut furnace = FurnaceTileEntity::new(FurnaceType::BlastFurnace);
        furnace.input = Some(CraftingItemStack::new(ItemId::RawIron, 1));
        furnace.fuel = Some(CraftingItemStack::new(ItemId::Coal, 1));
        furnace.tick();
        assert!(furnace.is_lit());
        assert_eq!(furnace.lit_duration, 1600);

        // Anvil calculation
        assert_eq!(calculate_next_prior_work_penalty(0), 1);
        assert_eq!(calculate_next_prior_work_penalty(1), 3);
        assert_eq!(calculate_next_prior_work_penalty(3), 7);

        // Smithing Table Netherite Upgrade
        let mut smithing = SmithingTableMenu::new();
        smithing.template_slot = Some(CraftingItemStack::new(ItemId::NetheriteUpgradeSmithingTemplate, 1));
        smithing.base_slot = Some(CraftingItemStack::new(ItemId::DiamondChestplate, 1));
        smithing.addition_slot = Some(CraftingItemStack::new(ItemId::NetheriteIngot, 1));
        
        let result = smithing.compute_result();
        assert_eq!(result.map(|r| r.output), Some(CraftingItemStack::new(ItemId::NetheriteChestplate, 1)));
        let output = smithing.take_output();
        assert_eq!(output, Some(CraftingItemStack::new(ItemId::NetheriteChestplate, 1)));
        assert_eq!(smithing.template_slot, None);
        assert_eq!(smithing.base_slot, None);
        assert_eq!(smithing.addition_slot, None);
    }
}


