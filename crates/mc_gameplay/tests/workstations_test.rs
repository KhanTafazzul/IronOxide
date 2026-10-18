use mc_crafting::{CraftingRecipe, Ingredient, ItemStack, RecipeLookupCache, ShapedRecipe};
use mc_gameplay::{
    calculate_next_prior_work_penalty, AnvilDamageState, AnvilItem, AnvilMenu,
    CraftingTableMenu, FurnaceTileEntity, FurnaceType,
    SmithingTableMenu, StonecutterMenu,
};
use mc_items::ItemId;

fn create_mock_recipe_cache() -> RecipeLookupCache {
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
        ItemStack::new(ItemId::IronPickaxe, 1),
        true,
    );
    RecipeLookupCache::new(vec![CraftingRecipe::Shaped(iron_pick_recipe)])
}

#[test]
fn test_crafting_table_preview_craft_and_auto_drop() {
    let cache = create_mock_recipe_cache();
    let mut table = CraftingTableMenu::new();

    // 1. Arrange 3 Iron Ingots on top and 2 Sticks in middle column
    table.set_slot(0, Some(ItemStack::new(ItemId::IronIngot, 2)), &cache);
    table.set_slot(1, Some(ItemStack::new(ItemId::IronIngot, 3)), &cache);
    table.set_slot(2, Some(ItemStack::new(ItemId::IronIngot, 1)), &cache);
    table.set_slot(4, Some(ItemStack::new(ItemId::Stick, 4)), &cache);
    table.set_slot(7, Some(ItemStack::new(ItemId::Stick, 2)), &cache);

    // Verify preview
    assert_eq!(table.output_preview, Some(ItemStack::new(ItemId::IronPickaxe, 1)));

    // 2. Take output
    let crafted = table.take_output(&cache);
    assert_eq!(crafted, Some(ItemStack::new(ItemId::IronPickaxe, 1)));

    // Slot 2 had 1 iron ingot, so it should now be None
    assert_eq!(table.get_slot(2), None);
    // Slot 0 had 2, now 1
    assert_eq!(table.get_slot(0), Some(ItemStack::new(ItemId::IronIngot, 1)));
    // Output preview should now be None because slot 2 is empty
    assert_eq!(table.output_preview, None);

    // 3. Close UI and auto-drop remaining items
    let dropped = table.close_and_drop_items();
    assert_eq!(dropped.len(), 4);
    assert_eq!(table.output_preview, None);
    assert!(table.grid.iter().all(|s| s.is_none()));
}

#[test]
fn test_furnace_20hz_ticking_and_fuels() {
    // 1. Standard Furnace smelting Raw Iron with Coal
    let mut furnace = FurnaceTileEntity::new(FurnaceType::Furnace);
    furnace.input = Some(ItemStack::new(ItemId::RawIron, 2));
    furnace.fuel = Some(ItemStack::new(ItemId::Coal, 1));

    assert!(!furnace.is_lit());

    // First tick: consumes 1 Coal (1600 ticks) and ignites
    furnace.tick();
    assert!(furnace.is_lit());
    assert_eq!(furnace.lit_time, 1600);
    assert_eq!(furnace.fuel, None);

    // Tick for 199 more ticks (total 200 ticks = 1 item cooked)
    for _ in 0..199 {
        furnace.tick();
    }

    assert_eq!(furnace.output, Some(ItemStack::new(ItemId::IronIngot, 1)));
    assert_eq!(furnace.input, Some(ItemStack::new(ItemId::RawIron, 1)));
    assert!(furnace.stored_experience > 0.0);

    // 2. Lava Bucket fuel test: consumes lava, leaves empty Bucket
    let mut lava_furnace = FurnaceTileEntity::new(FurnaceType::Furnace);
    lava_furnace.input = Some(ItemStack::new(ItemId::Cobblestone, 64));
    lava_furnace.fuel = Some(ItemStack::new(ItemId::LavaBucket, 1));

    lava_furnace.tick();
    assert!(lava_furnace.is_lit());
    assert_eq!(lava_furnace.lit_duration, 20000);
    assert_eq!(lava_furnace.fuel, Some(ItemStack::new(ItemId::Bucket, 1)));

    // 3. Blast Furnace: 2x speed (100 ticks) for ores, rejects food
    let mut blast_furnace = FurnaceTileEntity::new(FurnaceType::BlastFurnace);
    blast_furnace.input = Some(ItemStack::new(ItemId::RawGold, 1));
    blast_furnace.fuel = Some(ItemStack::new(ItemId::Coal, 1));

    for _ in 0..100 {
        blast_furnace.tick();
    }
    assert_eq!(blast_furnace.output, Some(ItemStack::new(ItemId::GoldIngot, 1)));

    // Rejects food
    let mut blast_food = FurnaceTileEntity::new(FurnaceType::BlastFurnace);
    blast_food.input = Some(ItemStack::new(ItemId::Beef, 1));
    blast_food.fuel = Some(ItemStack::new(ItemId::Coal, 1));
    blast_food.tick();
    assert!(!blast_food.is_lit(), "Blast furnace should reject beef");

    // 4. Smoker: 2x speed for food, rejects ores
    let mut smoker = FurnaceTileEntity::new(FurnaceType::Smoker);
    smoker.input = Some(ItemStack::new(ItemId::Beef, 1));
    smoker.fuel = Some(ItemStack::new(ItemId::Coal, 1));

    for _ in 0..100 {
        smoker.tick();
    }
    assert_eq!(smoker.output, Some(ItemStack::new(ItemId::CookedBeef, 1)));
}

#[test]
fn test_anvil_prior_work_penalty_and_repairs() {
    // 1. Prior work equation: 2^n - 1 -> next = p * 2 + 1
    assert_eq!(calculate_next_prior_work_penalty(0), 1);
    assert_eq!(calculate_next_prior_work_penalty(1), 3);
    assert_eq!(calculate_next_prior_work_penalty(3), 7);
    assert_eq!(calculate_next_prior_work_penalty(7), 15);
    assert_eq!(calculate_next_prior_work_penalty(15), 31);
    assert_eq!(calculate_next_prior_work_penalty(31), 63);

    // 2. Renaming test
    let mut anvil = AnvilMenu::new();
    anvil.left_slot = Some(AnvilItem::new(ItemId::DiamondSword).with_prior_work(3));
    anvil.custom_name_input = Some("Excalibur".to_string());

    let result = anvil.compute_result().expect("Rename should succeed");
    // Base cost 1 + prior work 3 = 4 levels
    assert_eq!(result.level_cost, 4);
    assert_eq!(result.base_cost, 1);
    assert_eq!(result.output.custom_name, Some("Excalibur".to_string()));

    // 3. Item combine repair: two damaged diamond swords
    let mut anvil2 = AnvilMenu::new();
    anvil2.left_slot = Some(AnvilItem::new(ItemId::DiamondSword).with_damage(1000).with_prior_work(1));
    anvil2.right_slot = Some(AnvilItem::new(ItemId::DiamondSword).with_damage(800).with_prior_work(3));

    let res2 = anvil2.compute_result().expect("Combine should succeed");
    // Base cost 2 + left(1) + right(3) = 6 levels
    assert_eq!(res2.level_cost, 6);
    // Damage significantly reduced with 12% bonus
    assert!(res2.output.damage < 100);
    // Next penalty: max(1, 3) * 2 + 1 = 7
    assert_eq!(res2.output.prior_work_penalty, 7);

    // 4. Too expensive test (cost >= 40)
    let mut anvil3 = AnvilMenu::new();
    anvil3.left_slot = Some(AnvilItem::new(ItemId::DiamondPickaxe).with_prior_work(31));
    anvil3.right_slot = Some(AnvilItem::new(ItemId::DiamondPickaxe).with_prior_work(15));
    let res3 = anvil3.compute_result().expect("Result computed");
    assert!(res3.level_cost >= 40);
    assert!(res3.too_expensive);

    // Survival player cannot take output if too expensive
    let mut player_levels = 100;
    let taken = anvil3.take_output(&mut player_levels, false);
    assert_eq!(taken, None);

    // Creative mode can bypass
    let creative_taken = anvil3.take_output(&mut player_levels, true);
    assert!(creative_taken.is_some());

    // 5. Anvil degradation
    assert_eq!(anvil.degrade_anvil(), AnvilDamageState::Chipped);
    assert_eq!(anvil.degrade_anvil(), AnvilDamageState::Damaged);
    assert_eq!(anvil.degrade_anvil(), AnvilDamageState::Broken);
}

#[test]
fn test_smithing_table_netherite_upgrade_and_trims() {
    let mut smithing = SmithingTableMenu::new();

    // 1. Netherite Upgrade
    smithing.template_slot = Some(ItemStack::new(ItemId::NetheriteUpgradeSmithingTemplate, 1));
    smithing.base_slot = Some(ItemStack::new(ItemId::DiamondPickaxe, 1));
    smithing.addition_slot = Some(ItemStack::new(ItemId::NetheriteIngot, 1));

    let res = smithing.compute_result().expect("Netherite upgrade should be valid");
    assert_eq!(res.output, ItemStack::new(ItemId::NetheritePickaxe, 1));

    // Take output
    let output = smithing.take_output();
    assert_eq!(output, Some(ItemStack::new(ItemId::NetheritePickaxe, 1)));
    assert_eq!(smithing.template_slot, None);
    assert_eq!(smithing.base_slot, None);
    assert_eq!(smithing.addition_slot, None);

    // 2. Armor Trim
    smithing.template_slot = Some(ItemStack::new(ItemId::CoastArmorTrimSmithingTemplate, 1));
    smithing.base_slot = Some(ItemStack::new(ItemId::DiamondChestplate, 1));
    smithing.addition_slot = Some(ItemStack::new(ItemId::AmethystShard, 1));

    let trim_res = smithing.compute_result().expect("Armor trim should be valid");
    assert_eq!(trim_res.output, ItemStack::new(ItemId::DiamondChestplate, 1));
}

#[test]
fn test_stonecutter_recipes_and_selection() {
    let mut stonecutter = StonecutterMenu::new();
    stonecutter.set_input(Some(ItemStack::new(ItemId::Stone, 5)));

    let recipes = stonecutter.available_recipes();
    assert!(!recipes.is_empty());
    assert!(recipes.iter().any(|r| r.output == ItemId::StoneSlab && r.count == 2));

    // Select Stone Slab recipe (index 0)
    let preview = stonecutter.select_recipe(0);
    assert_eq!(preview, Some(ItemStack::new(ItemId::StoneSlab, 2)));

    // Take output
    let output = stonecutter.take_output();
    assert_eq!(output, Some(ItemStack::new(ItemId::StoneSlab, 2)));
    assert_eq!(stonecutter.input_slot, Some(ItemStack::new(ItemId::Stone, 4)));

    // Close and drop remaining 4 stones
    let dropped = stonecutter.close_and_drop_items();
    assert_eq!(dropped, Some(ItemStack::new(ItemId::Stone, 4)));
    assert_eq!(stonecutter.input_slot, None);
}
