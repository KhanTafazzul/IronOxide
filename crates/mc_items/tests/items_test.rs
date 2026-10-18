use mc_items::{ArmorSlot, ArmorTier, ItemId, ToolTier, ToolType};

#[test]
fn test_item_id_name_roundtrip() {
    assert_eq!(ItemId::DiamondSword.name(), "minecraft:diamond_sword");
    assert_eq!(ItemId::from_name("minecraft:diamond_sword"), Some(ItemId::DiamondSword));
    assert_eq!(ItemId::from_name("diamond_sword"), Some(ItemId::DiamondSword));

    assert_eq!(ItemId::Crafter.name(), "minecraft:crafter");
    assert_eq!(ItemId::from_name("crafter"), Some(ItemId::Crafter));

    assert_eq!(ItemId::Mace.name(), "minecraft:mace");
    assert_eq!(ItemId::from_name("mace"), Some(ItemId::Mace));

    assert_eq!(ItemId::OakPlanks.name(), "minecraft:oak_planks");
    assert_eq!(ItemId::from_name("oak_planks"), Some(ItemId::OakPlanks));
}

#[test]
fn test_item_max_stack_sizes() {
    assert_eq!(ItemId::Stone.max_stack_size(), 64);
    assert_eq!(ItemId::EnderPearl.max_stack_size(), 16);
    assert_eq!(ItemId::Snowball.max_stack_size(), 16);
    assert_eq!(ItemId::OakSign.max_stack_size(), 16);
    assert_eq!(ItemId::Bucket.max_stack_size(), 16);
    assert_eq!(ItemId::WaterBucket.max_stack_size(), 1);
    assert_eq!(ItemId::DiamondSword.max_stack_size(), 1);
    assert_eq!(ItemId::IronChestplate.max_stack_size(), 1);
}

#[test]
fn test_tool_tier_attributes_parity() {
    // Wood
    assert_eq!(ToolTier::Wood.durability(), 59);
    assert_eq!(ToolTier::Wood.mining_speed_multiplier(), 2.0);
    assert_eq!(ToolTier::Wood.attack_damage_bonus(), 0.0);
    assert_eq!(ToolTier::Wood.harvest_level(), 0);

    // Stone
    assert_eq!(ToolTier::Stone.durability(), 131);
    assert_eq!(ToolTier::Stone.mining_speed_multiplier(), 4.0);
    assert_eq!(ToolTier::Stone.attack_damage_bonus(), 1.0);
    assert_eq!(ToolTier::Stone.harvest_level(), 1);

    // Iron
    assert_eq!(ToolTier::Iron.durability(), 250);
    assert_eq!(ToolTier::Iron.mining_speed_multiplier(), 6.0);
    assert_eq!(ToolTier::Iron.attack_damage_bonus(), 2.0);
    assert_eq!(ToolTier::Iron.harvest_level(), 2);

    // Diamond
    assert_eq!(ToolTier::Diamond.durability(), 1561);
    assert_eq!(ToolTier::Diamond.mining_speed_multiplier(), 8.0);
    assert_eq!(ToolTier::Diamond.attack_damage_bonus(), 3.0);
    assert_eq!(ToolTier::Diamond.harvest_level(), 3);

    // Netherite
    assert_eq!(ToolTier::Netherite.durability(), 2031);
    assert_eq!(ToolTier::Netherite.mining_speed_multiplier(), 9.0);
    assert_eq!(ToolTier::Netherite.attack_damage_bonus(), 4.0);
    assert_eq!(ToolTier::Netherite.harvest_level(), 4);

    // Gold
    assert_eq!(ToolTier::Gold.durability(), 32);
    assert_eq!(ToolTier::Gold.mining_speed_multiplier(), 12.0);
    assert_eq!(ToolTier::Gold.attack_damage_bonus(), 0.0);
    assert_eq!(ToolTier::Gold.harvest_level(), 0);
}

#[test]
fn test_tool_attack_damage_and_speed() {
    // Diamond Sword: 4.0 base + 3.0 tier bonus = 7.0 attack damage, 1.6 attack speed
    assert_eq!(ToolType::Sword.attack_damage(ToolTier::Diamond), 7.0);
    assert_eq!(ToolType::Sword.attack_speed(ToolTier::Diamond), 1.6);

    // Netherite Sword: 4.0 base + 4.0 tier bonus = 8.0 attack damage, 1.6 attack speed
    assert_eq!(ToolType::Sword.attack_damage(ToolTier::Netherite), 8.0);
    assert_eq!(ToolType::Sword.attack_speed(ToolTier::Netherite), 1.6);

    // Diamond Axe: 9.0 damage, 1.0 speed
    assert_eq!(ToolType::Axe.attack_damage(ToolTier::Diamond), 9.0);
    assert_eq!(ToolType::Axe.attack_speed(ToolTier::Diamond), 1.0);

    // Netherite Axe: 10.0 damage, 1.0 speed
    assert_eq!(ToolType::Axe.attack_damage(ToolTier::Netherite), 10.0);
    assert_eq!(ToolType::Axe.attack_speed(ToolTier::Netherite), 1.0);
}

#[test]
fn test_armor_tier_defense_and_durability() {
    // Diamond Armor: Head=3, Chest=8, Legs=6, Boots=3 (Total 20)
    assert_eq!(ArmorTier::Diamond.defense_points(ArmorSlot::Helmet), 3);
    assert_eq!(ArmorTier::Diamond.defense_points(ArmorSlot::Chestplate), 8);
    assert_eq!(ArmorTier::Diamond.defense_points(ArmorSlot::Leggings), 6);
    assert_eq!(ArmorTier::Diamond.defense_points(ArmorSlot::Boots), 3);
    assert_eq!(ArmorTier::Diamond.toughness(), 2.0);
    assert_eq!(ArmorTier::Diamond.durability(ArmorSlot::Chestplate), 528);

    // Netherite Armor: Head=3, Chest=8, Legs=6, Boots=3, Toughness=3.0, Knockback Resistance=0.1
    assert_eq!(ArmorTier::Netherite.defense_points(ArmorSlot::Chestplate), 8);
    assert_eq!(ArmorTier::Netherite.toughness(), 3.0);
    assert_eq!(ArmorTier::Netherite.knockback_resistance(), 0.1);
    assert_eq!(ArmorTier::Netherite.durability(ArmorSlot::Chestplate), 592);
}
