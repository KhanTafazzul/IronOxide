use mc_entities::components::EntityId;
use mc_entities::passive::*;
use mc_items::ItemId;

#[test]
fn test_cow_tempt_milking_and_breeding() {
    let mut cow1 = Cow::new();
    let mut cow2 = Cow::new();
    let player = EntityId(100);

    // Tempt food verification
    assert!(Cow::is_food(ItemId::Wheat));
    assert!(!Cow::is_food(ItemId::Carrot));

    // Milking with empty bucket
    let milk = cow1.interact(player, ItemId::Bucket);
    assert_eq!(milk, Some(ItemId::MilkBucket));

    // Feeding wheat triggers love mode
    assert!(cow1.animal_data.can_breed());
    let _ = cow1.interact(player, ItemId::Wheat);
    assert!(cow1.animal_data.in_love_ticks > 0);

    let _ = cow2.interact(player, ItemId::Wheat);
    assert!(cow2.animal_data.in_love_ticks > 0);

    // Breeding produces a baby cow
    let baby = cow1.breed(&mut cow2);
    assert!(baby.animal_data.is_baby());
    assert_eq!(baby.animal_data.age, -24000);
    assert_eq!(cow1.animal_data.breed_cooldown_ticks, 6000);
    assert_eq!(cow2.animal_data.breed_cooldown_ticks, 6000);
}

#[test]
fn test_pig_saddle_and_boost() {
    let mut pig = Pig::new();
    let player = EntityId(101);

    assert!(Pig::is_food(ItemId::Carrot));
    assert!(Pig::is_food(ItemId::Potato));
    assert!(Pig::is_food(ItemId::Beetroot));

    // Saddling the pig
    assert!(!pig.has_saddle);
    let saddled = pig.interact(player, ItemId::Saddle);
    assert!(saddled);
    assert!(pig.has_saddle);

    // Boosting with Carrot on a Stick
    assert_eq!(pig.speed_multiplier(), 1.0);
    let boosted = pig.interact(player, ItemId::CarrotOnAStick);
    assert!(boosted);
    assert_eq!(pig.speed_multiplier(), 1.35);

    // Ticking decrements boost
    for _ in 0..60 {
        pig.tick();
    }
    assert_eq!(pig.speed_multiplier(), 1.0);
}

#[test]
fn test_sheep_shearing_and_grass_regrowth() {
    let mut sheep = Sheep::new(DyeColor::Red);
    assert_eq!(sheep.color, DyeColor::Red);
    assert!(!sheep.is_sheared);

    // Shearing drops red wool
    let sheared = sheep.shear();
    assert!(sheared.is_some());
    let (item, count) = sheared.unwrap();
    assert_eq!(item, ItemId::RedWool);
    assert_eq!(count, 2);
    assert!(sheep.is_sheared);

    // Cannot shear twice
    assert!(sheep.shear().is_none());

    // Eating grass regrows wool
    sheep.ate_grass();
    assert!(!sheep.is_sheared);

    // Dyeing wool to Blue
    sheep.dye(DyeColor::Blue);
    assert_eq!(sheep.color, DyeColor::Blue);

    // Breeding sheep with color blending
    let mut parent2 = Sheep::new(DyeColor::Yellow);
    let baby = sheep.breed(&mut parent2);
    // Blue + Yellow = Green wool
    assert_eq!(baby.color, DyeColor::Green);
    assert!(baby.animal_data.is_baby());
}

#[test]
fn test_chicken_egg_laying_and_slow_falling() {
    let mut chicken = Chicken::new(6001);
    assert!(!chicken.animal_data.is_baby());

    // Seeds food verification
    assert!(Chicken::is_food(ItemId::WheatSeeds));
    assert!(Chicken::is_food(ItemId::MelonSeeds));
    assert!(Chicken::is_food(ItemId::PumpkinSeeds));

    // Slow falling caps terminal velocity
    let (vel_y, flapping) = chicken.slow_falling_velocity(-1.5);
    assert_eq!(vel_y, -0.05);
    assert!(flapping);

    // Egg laying timer countdown
    chicken.egg_lay_timer = 1;
    let egg = chicken.tick(2500);
    assert_eq!(egg, Some(ItemId::Egg));
    assert!(chicken.egg_lay_timer >= Chicken::MIN_EGG_TICKS);
}

#[test]
fn test_wolf_taming_tail_angle_and_teleport() {
    let mut wolf = Wolf::new();
    let player = EntityId(200);

    // Untamed tail angle
    assert_eq!(wolf.tail_angle(), 0.2);
    assert!(!wolf.tame_data.is_tamed);

    // Attempt taming with bone (success on rand_mod_3 == 0)
    let failed = wolf.interact(player, ItemId::Bone, 1);
    assert!(!failed);
    assert!(!wolf.tame_data.is_tamed);

    let success = wolf.interact(player, ItemId::Bone, 0);
    assert!(success);
    assert!(wolf.tame_data.is_tamed);
    assert_eq!(wolf.tame_data.owner_id, Some(player));
    assert_eq!(wolf.tame_data.collar_color, Some(DyeColor::Red));
    assert_eq!(wolf.max_health, 40.0);
    assert_eq!(wolf.health, 40.0);

    // Full health tail angle (~0.7 radians)
    assert!((wolf.tail_angle() - 0.7).abs() < 0.01);

    // Damage wolf & test healing with meat
    wolf.health = 20.0;
    assert!((wolf.tail_angle() - 0.4).abs() < 0.01);

    let healed = wolf.interact(player, ItemId::CookedBeef, 0);
    assert!(healed);
    assert_eq!(wolf.health, 24.0);

    // Sitting toggle with empty hand
    assert!(!wolf.tame_data.is_sitting);
    let sat = wolf.interact(player, ItemId::Air, 0);
    assert!(sat);
    assert!(wolf.tame_data.is_sitting);

    // Sitting wolves do NOT teleport
    assert!(!wolf.should_teleport_to_owner(25.0));

    // Unsit wolf and test teleportation (> 12m)
    wolf.tame_data.toggle_sitting();
    assert!(!wolf.tame_data.is_sitting);
    assert!(wolf.should_teleport_to_owner(12.5));
    assert!(!wolf.should_teleport_to_owner(8.0));
}

#[test]
fn test_cat_taming_and_creeper_repel() {
    let mut cat = Cat::new(CatVariant::Calico);
    let player = EntityId(201);

    assert!(Cat::is_fish(ItemId::Cod));
    assert!(Cat::is_fish(ItemId::Salmon));

    let tamed = cat.interact(player, ItemId::Cod, 0);
    assert!(tamed);
    assert!(cat.tame_data.is_tamed);
    assert!(cat.repels_creepers_and_phantoms());
}

#[test]
fn test_parrot_taming_and_cookie_poison() {
    let mut parrot = Parrot::new(ParrotVariant::Blue);
    let player = EntityId(202);

    // Feeding cookie results in fatal poison
    let poison_result = parrot.interact(player, ItemId::Cookie, 0);
    assert_eq!(poison_result, Err("FATAL_COOKIE_POISON"));

    // Taming with seeds
    let tamed = parrot.interact(player, ItemId::WheatSeeds, 0).unwrap();
    assert!(tamed);
    assert!(parrot.tame_data.is_tamed);

    // Shoulder perching
    parrot.mount_shoulder();
    assert!(parrot.is_perched);
    parrot.dismount_shoulder();
    assert!(!parrot.is_perched);

    // Jukebox dancing
    parrot.set_nearby_music(true);
    assert!(parrot.is_dancing);
}

#[test]
fn test_fox_sleeping_and_mouth_holding() {
    let mut fox = Fox::new(FoxVariant::Red);

    assert!(fox.can_hold_item());
    fox.pick_up_item(ItemId::Emerald);
    assert_eq!(fox.mouth_item, Some(ItemId::Emerald));
    assert!(!fox.can_hold_item());

    // Daytime sleep without nearby player
    fox.update_sleep_state(true, false, 10.0, false);
    assert!(fox.is_sleeping);

    // Non-sneaking player wakes fox up
    fox.update_sleep_state(true, false, 2.5, false);
    assert!(!fox.is_sleeping);

    // Sneaking player does NOT wake fox up
    fox.update_sleep_state(true, false, 2.5, true);
    assert!(fox.is_sleeping);

    // Night time: awake and hunting
    fox.update_sleep_state(false, false, 10.0, false);
    assert!(!fox.is_sleeping);
}

#[test]
fn test_armadillo_rolling_and_scute_brushing() {
    let mut armadillo = Armadillo::new();
    assert_eq!(armadillo.state, ArmadilloState::Unrolled);
    assert_eq!(armadillo.damage_multiplier(), 1.0);

    // Sprinting player within 7 blocks triggers rolling up
    armadillo.evaluate_threat(Some(5.0), None, false);
    assert_eq!(armadillo.state, ArmadilloState::RolledUp);
    assert_eq!(armadillo.damage_multiplier(), 0.5);

    // Brushing drops an ArmadilloScute when unrolled
    armadillo.state = ArmadilloState::Unrolled;
    let scute = armadillo.brush(ItemId::Brush);
    assert_eq!(scute, Some(ItemId::ArmadilloScute));
    assert_eq!(armadillo.scute_craft_count, 1);
}

#[test]
fn test_camel_two_player_mount_and_dash() {
    let mut camel = Camel::new();
    let player1 = EntityId(301);
    let player2 = EntityId(302);
    let player3 = EntityId(303);

    camel.has_saddle = true;

    // 2-Player riding
    assert!(camel.mount(player1));
    assert_eq!(camel.driver, Some(player1));
    assert!(camel.mount(player2));
    assert_eq!(camel.passenger, Some(player2));
    // 3rd player rejected
    assert!(!camel.mount(player3));

    // Dash jump
    assert!(camel.try_dash());
    assert_eq!(camel.dash_cooldown_ticks, Camel::DASH_COOLDOWN_TICKS);
    // Cannot dash while on cooldown
    assert!(!camel.try_dash());

    // Tall rider immunity against melee attacks
    assert!(camel.is_rider_immune_to_melee());
}

#[test]
fn test_sniffer_ancient_seed_digging() {
    let mut sniffer = Sniffer::new();
    assert_eq!(sniffer.state, SnifferState::Idling);

    // Valid soil check
    assert!(Sniffer::is_diggable_soil(ItemId::MossBlock));
    assert!(Sniffer::is_diggable_soil(ItemId::GrassBlock));
    assert!(!Sniffer::is_diggable_soil(ItemId::Stone));

    // Start digging
    assert!(sniffer.try_start_digging(ItemId::MossBlock));
    assert_eq!(sniffer.state, SnifferState::Digging);

    // Simulate digging ticks
    for _ in 0..Sniffer::DIG_DURATION_TICKS {
        let seed = sniffer.tick(true);
        if seed.is_some() {
            assert_eq!(seed, Some(ItemId::TorchflowerSeeds));
            break;
        }
    }
    assert_eq!(sniffer.state, SnifferState::Idling);
    assert_eq!(sniffer.dig_cooldown_ticks, Sniffer::DIG_COOLDOWN_TICKS);
}

#[test]
fn test_allay_fetching_and_duplication() {
    let mut allay = Allay::new();
    let player = EntityId(401);

    // Give item to fetch
    allay.give_item(player, ItemId::Diamond);
    assert_eq!(allay.given_item, Some(ItemId::Diamond));
    assert_eq!(allay.owner_id, Some(player));

    // Note block vibration linking
    allay.hear_note_block();
    assert_eq!(allay.linked_note_block_ticks, Allay::NOTE_BLOCK_TIMEOUT_TICKS);

    // Duplication while dancing near jukebox
    allay.is_dancing = true;
    let clone = allay.try_duplicate(ItemId::AmethystShard);
    assert!(clone.is_some());
    assert_eq!(allay.duplication_cooldown_ticks, Allay::DUPLICATION_COOLDOWN);
}

#[test]
fn test_panda_genetics_dominance() {
    // Normal + Normal = Normal
    let panda_normal = Panda::new(PandaGene::Normal, PandaGene::Normal);
    assert_eq!(panda_normal.expressed_trait(), PandaGene::Normal);

    // Playful (dominant) + Brown (recessive) = Playful
    let panda_carrier = Panda::new(PandaGene::Playful, PandaGene::Brown);
    assert_eq!(panda_carrier.expressed_trait(), PandaGene::Playful);

    // Brown + Brown (both recessive alleles) = Brown expressed!
    let panda_brown = Panda::new(PandaGene::Brown, PandaGene::Brown);
    assert_eq!(panda_brown.expressed_trait(), PandaGene::Brown);
}
