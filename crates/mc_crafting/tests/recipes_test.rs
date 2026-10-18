use std::path::Path;
use mc_crafting::{Ingredient, ItemStack, RecipeLoader, ShapedRecipe, ShapelessRecipe};
use mc_items::ItemId;


fn get_client_jar_path() -> &'static str {
    if Path::new("../../../client.jar").exists() {
        "../../../client.jar"
    } else if Path::new("../../client.jar").exists() {
        "../../client.jar"
    } else if Path::new("../client.jar").exists() {
        "../client.jar"
    } else if Path::new("client.jar").exists() {
        "client.jar"
    } else {
        "client.jar"
    }
}

#[test]
fn test_serde_shaped_recipe_manual_matching() {
    // 3x Iron Ingot on top, sticks in middle column
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

    let mut grid: [Option<ItemStack>; 9] = [None; 9];
    grid[0] = Some(ItemStack::new(ItemId::IronIngot, 1));
    grid[1] = Some(ItemStack::new(ItemId::IronIngot, 1));
    grid[2] = Some(ItemStack::new(ItemId::IronIngot, 1));
    grid[4] = Some(ItemStack::new(ItemId::Stick, 1));
    grid[7] = Some(ItemStack::new(ItemId::Stick, 1));

    let result = iron_pick_recipe.matches(&grid);
    assert_eq!(result, Some(ItemStack::new(ItemId::IronPickaxe, 1)));
}

#[test]
fn test_serde_shapeless_recipe_manual_matching() {
    // 1 Oak Log -> 4 Oak Planks
    let oak_planks_recipe = ShapelessRecipe::new(
        "minecraft:oak_planks".to_string(),
        vec![Ingredient::new(vec![ItemId::OakLog, ItemId::StrippedOakLog])],
        ItemStack::new(ItemId::OakPlanks, 4),
    );

    // Place oak log in slot 4 (center)
    let mut grid: [Option<ItemStack>; 9] = [None; 9];
    grid[4] = Some(ItemStack::new(ItemId::OakLog, 1));

    let result = oak_planks_recipe.matches(&grid);
    assert_eq!(result, Some(ItemStack::new(ItemId::OakPlanks, 4)));

    // Place stripped oak log in slot 8 (bottom right)
    let mut grid2: [Option<ItemStack>; 9] = [None; 9];
    grid2[8] = Some(ItemStack::new(ItemId::StrippedOakLog, 1));

    let result2 = oak_planks_recipe.matches(&grid2);
    assert_eq!(result2, Some(ItemStack::new(ItemId::OakPlanks, 4)));
}

#[test]
fn test_load_all_recipes_from_client_jar_and_lookup() {
    let jar_path = get_client_jar_path();
    if !Path::new(jar_path).exists() {
        println!("Warning: client.jar not found at {}", jar_path);
        return;
    }

    let cache = RecipeLoader::load_from_jar(jar_path).expect("Failed to load recipes from client.jar");
    println!("Successfully loaded {} crafting recipes from client.jar!", cache.recipes_count());
    assert!(cache.recipes_count() > 800, "Expected > 800 recipes in client.jar");

    // 1. Test Iron Pickaxe recipe:
    // [Iron, Iron, Iron]
    // [Air,  Stick, Air ]
    // [Air,  Stick, Air ]
    let mut iron_pick_grid: [Option<ItemStack>; 9] = [None; 9];
    iron_pick_grid[0] = Some(ItemStack::new(ItemId::IronIngot, 1));
    iron_pick_grid[1] = Some(ItemStack::new(ItemId::IronIngot, 1));
    iron_pick_grid[2] = Some(ItemStack::new(ItemId::IronIngot, 1));
    iron_pick_grid[4] = Some(ItemStack::new(ItemId::Stick, 1));
    iron_pick_grid[7] = Some(ItemStack::new(ItemId::Stick, 1));

    let pick_out = cache.compute_output(&iron_pick_grid);
    assert_eq!(pick_out, Some(ItemStack::new(ItemId::IronPickaxe, 1)));

    // 2. Test Oak Planks shapeless recipe from Oak Log anywhere in grid (e.g. slot 5)
    let mut oak_log_grid: [Option<ItemStack>; 9] = [None; 9];
    oak_log_grid[5] = Some(ItemStack::new(ItemId::OakLog, 1));

    let planks_out = cache.compute_output(&oak_log_grid);
    assert_eq!(planks_out, Some(ItemStack::new(ItemId::OakPlanks, 4)));

    // 3. Test Birch Planks shapeless recipe
    let mut birch_log_grid: [Option<ItemStack>; 9] = [None; 9];
    birch_log_grid[2] = Some(ItemStack::new(ItemId::BirchLog, 1));

    let birch_out = cache.compute_output(&birch_log_grid);
    assert_eq!(birch_out, Some(ItemStack::new(ItemId::BirchPlanks, 4)));

    // 4. Test Sticks from 2 vertical planks (tested in column 0, rows 0 and 1)
    let mut stick_grid: [Option<ItemStack>; 9] = [None; 9];
    stick_grid[0] = Some(ItemStack::new(ItemId::OakPlanks, 1));
    stick_grid[3] = Some(ItemStack::new(ItemId::OakPlanks, 1));

    let stick_out = cache.compute_output(&stick_grid);
    assert_eq!(stick_out, Some(ItemStack::new(ItemId::Stick, 4)));

    // 5. Test Sticks shifted to column 2, rows 1 and 2
    let mut stick_grid_shifted: [Option<ItemStack>; 9] = [None; 9];
    stick_grid_shifted[5] = Some(ItemId::OakPlanks.into());
    stick_grid_shifted[8] = Some(ItemId::OakPlanks.into());

    let stick_shifted_out = cache.compute_output(&stick_grid_shifted);
    assert_eq!(stick_shifted_out, Some(ItemStack::new(ItemId::Stick, 4)));

    // 6. Test Crafting Table (2x2 planks)
    let mut table_grid: [Option<ItemStack>; 9] = [None; 9];
    table_grid[0] = Some(ItemStack::new(ItemId::OakPlanks, 1));
    table_grid[1] = Some(ItemStack::new(ItemId::OakPlanks, 1));
    table_grid[3] = Some(ItemStack::new(ItemId::OakPlanks, 1));
    table_grid[4] = Some(ItemStack::new(ItemId::OakPlanks, 1));

    let table_out = cache.compute_output(&table_grid);
    assert_eq!(table_out, Some(ItemStack::new(ItemId::CraftingTable, 1)));

    // 7. Test Torch (Coal over Stick)
    let mut torch_grid: [Option<ItemStack>; 9] = [None; 9];
    torch_grid[1] = Some(ItemStack::new(ItemId::Coal, 1));
    torch_grid[4] = Some(ItemStack::new(ItemId::Stick, 1));

    let torch_out = cache.compute_output(&torch_grid);
    assert_eq!(torch_out, Some(ItemStack::new(ItemId::Torch, 4)));

    // 8. Test invalid recipe returns None
    let mut invalid_grid: [Option<ItemStack>; 9] = [None; 9];
    invalid_grid[0] = Some(ItemStack::new(ItemId::Dirt, 1));
    invalid_grid[1] = Some(ItemStack::new(ItemId::Stone, 1));

    let invalid_out = cache.compute_output(&invalid_grid);
    assert_eq!(invalid_out, None);

    // 9. Verify O(1) performance: benchmark 100,000 queries in tight loop
    let start = std::time::Instant::now();
    for _ in 0..100_000 {
        let out = cache.compute_output(&iron_pick_grid);
        assert_eq!(out, Some(ItemStack::new(ItemId::IronPickaxe, 1)));
    }
    let elapsed = start.elapsed();
    println!("100,000 lookups completed in {:?} (avg {:?} per query)", elapsed, elapsed / 100_000);
    assert!(elapsed.as_millis() < 500, "O(1) lookup took too long: {:?}", elapsed);
}

