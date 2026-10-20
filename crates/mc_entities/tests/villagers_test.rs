// Tests for Task 53: Villager Profession Schedule, Trading & Golems.

use glam::Vec3;
use mc_entities::components::EntityType;
use mc_entities::villagers::*;
use mc_items::ItemId;

#[test]
fn test_villager_daily_schedule_and_sleep() {
    let mut schedule = VillagerSchedule::new();
    let bed_pos = Vec3::new(10.0, 64.0, 20.0);
    let work_pos = Vec3::new(12.0, 64.0, 18.0);
    let bell_pos = Vec3::new(0.0, 64.0, 0.0);

    schedule.claim_bed(bed_pos, 0.0);
    schedule.claim_workstation(work_pos);
    schedule.claim_bell(bell_pos);

    // Morning Rest (tick 1000)
    assert_eq!(schedule.update_schedule(1000, false), VillagerActivity::Rest);
    assert!(!schedule.is_sleeping);

    // Work Hours (tick 4000)
    assert_eq!(schedule.update_schedule(4000, false), VillagerActivity::Work);
    assert_eq!(schedule.current_destination(), Some(work_pos));

    // Gather Hours at Village Bell (tick 10000)
    assert_eq!(schedule.update_schedule(10000, false), VillagerActivity::Gather);
    assert_eq!(schedule.current_destination(), Some(bell_pos));

    // Sleep Hours (tick 13000)
    assert_eq!(schedule.update_schedule(13000, false), VillagerActivity::Sleep);
    assert!(schedule.is_sleeping);
    assert_eq!(schedule.current_destination(), Some(bed_pos));

    // Panic overrides sleep and wakes villager
    schedule.trigger_panic(100);
    assert_eq!(schedule.update_schedule(13000, false), VillagerActivity::Panic);
    assert!(!schedule.is_sleeping);
}

#[test]
fn test_nitwit_schedule_late_riser() {
    let mut nitwit_schedule = VillagerSchedule::new();
    // Nitwit sleeps until tick 3000
    assert_eq!(nitwit_schedule.update_schedule(2000, true), VillagerActivity::Sleep);
    // Wanders during normal work hours
    assert_eq!(nitwit_schedule.update_schedule(5000, true), VillagerActivity::Rest);
    // Stays up late until tick 14000
    assert_eq!(nitwit_schedule.update_schedule(13000, true), VillagerActivity::Rest);
    assert_eq!(nitwit_schedule.update_schedule(15000, true), VillagerActivity::Sleep);
}

#[test]
fn test_profession_workstation_binding_and_permanent_locking() {
    let mut career = VillagerCareer::new(VillagerProfession::None);
    assert_eq!(career.profession, VillagerProfession::None);
    assert!(!career.is_locked());

    // Claiming Lectern binds to Librarian
    assert!(career.try_claim_workstation(ItemId::Lectern));
    assert_eq!(career.profession, VillagerProfession::Librarian);

    // Losing workstation before trading reverts to Unemployed
    assert!(career.try_lose_workstation());
    assert_eq!(career.profession, VillagerProfession::None);

    // Re-claiming Composter binds to Farmer
    assert!(career.try_claim_workstation(ItemId::Composter));
    assert_eq!(career.profession, VillagerProfession::Farmer);

    // Gaining trade XP permanently locks profession!
    career.add_xp(5);
    assert!(career.is_locked());

    // Breaking workstation does NOT lose locked profession
    assert!(!career.try_lose_workstation());
    assert_eq!(career.profession, VillagerProfession::Farmer);
}

#[test]
fn test_trade_leveling_matrix_novice_to_master() {
    let mut career = VillagerCareer::new(VillagerProfession::Armorer);
    assert_eq!(career.level, TradeLevel::Novice);

    // Level up to Apprentice (10 XP)
    assert!(career.add_xp(10));
    assert_eq!(career.level, TradeLevel::Apprentice);

    // Level up to Journeyman (70 XP)
    assert!(career.add_xp(60));
    assert_eq!(career.level, TradeLevel::Journeyman);

    // Level up to Expert (150 XP)
    assert!(career.add_xp(80));
    assert_eq!(career.level, TradeLevel::Expert);

    // Level up to Master (250 XP)
    assert!(career.add_xp(100));
    assert_eq!(career.level, TradeLevel::Master);
}

#[test]
fn test_trade_offers_execution_and_restocking() {
    let mut offer = TradeOffer::simple(
        (ItemId::Wheat, 20),
        (ItemId::Emerald, 1),
        16,
        2,
        TradeLevel::Novice,
    );

    assert!(offer.can_trade());
    // Execute trade 16 times until locked
    for _ in 0..16 {
        assert_eq!(offer.execute_trade(), Some(2));
    }
    assert!(!offer.can_trade());
    assert_eq!(offer.execute_trade(), None);

    // Restock resets trade uses
    offer.restock();
    assert!(offer.can_trade());
}

#[test]
fn test_villager_twice_daily_restock_at_workstation() {
    let mut villager = Villager::new(VillagerProfession::Librarian);
    villager.schedule.claim_workstation(Vec3::new(10.0, 64.0, 10.0));

    // Exhaust trade
    for _ in 0..16 {
        villager.trade(0);
    }
    assert!(!villager.trades[0].can_trade());

    // Restock 1: during work hours (tick 3000)
    villager.tick(3000);
    assert_eq!(villager.career.restocks_today, 1);
    assert!(villager.trades[0].can_trade());

    // Exhaust trade again
    for _ in 0..16 {
        villager.trade(0);
    }

    // Restock 2: during work hours (tick 5000)
    villager.tick(5000);
    assert_eq!(villager.career.restocks_today, 2);
    assert!(villager.trades[0].can_trade());

    // Exhaust trade again
    for _ in 0..16 {
        villager.trade(0);
    }

    // Third restock attempt today is blocked (max 2 daily)
    villager.tick(7000);
    assert_eq!(villager.career.restocks_today, 2);
    assert!(!villager.trades[0].can_trade());

    // Next dawn (tick 0) resets daily restock counter
    villager.tick(0);
    assert_eq!(villager.career.restocks_today, 0);
}

#[test]
fn test_villager_food_breeding() {
    let mut villager = Villager::unemployed();
    assert!(!villager.animal_data.is_baby());

    // Feeding 2 bread (8 points) is not enough (< 12 points)
    assert!(villager.feed_food(ItemId::Bread, 2));
    assert_eq!(villager.food_points, 8);
    assert_eq!(villager.animal_data.in_love_ticks, 0);

    // Feeding 1 more bread (total 12 points) triggers love mode
    assert!(villager.feed_food(ItemId::Bread, 1));
    assert!(villager.animal_data.in_love_ticks > 0);
    assert_eq!(villager.food_points, 0);
}

#[test]
fn test_iron_golem_summon_checks_flinging_and_repair() {
    // Village spawn conditions: 3 beds + (3 panicked OR 5 gossiping)
    assert!(!IronGolem::can_spawn_in_village(2, 4, 3));
    assert!(IronGolem::can_spawn_in_village(3, 0, 3));
    assert!(IronGolem::can_spawn_in_village(0, 5, 3));

    let mut golem = IronGolem::new();
    assert_eq!(golem.crackiness(), IronGolemCrack::None);

    // Fling attack
    let attack = golem.fling_attack();
    assert_eq!(attack.damage, 14.0);
    assert_eq!(attack.vertical_knockback, 0.45);
    assert!(golem.attack_animation_ticks > 0);

    // Damage & 4-stage cracking
    golem.health = 70.0;
    assert_eq!(golem.crackiness(), IronGolemCrack::Low);
    golem.health = 45.0;
    assert_eq!(golem.crackiness(), IronGolemCrack::Medium);
    golem.health = 20.0;
    assert_eq!(golem.crackiness(), IronGolemCrack::High);

    // Repair with Iron Ingot (+25 HP)
    assert!(golem.repair(ItemId::IronIngot));
    assert_eq!(golem.health, 45.0);
    assert_eq!(golem.crackiness(), IronGolemCrack::Medium);

    // Hostile target filter
    assert!(IronGolem::is_hostile_target(EntityType::Zombie));
    assert!(IronGolem::is_hostile_target(EntityType::Pillager));
    assert!(!IronGolem::is_hostile_target(EntityType::Cow));
    assert!(!IronGolem::is_hostile_target(EntityType::Villager));
}

#[test]
fn test_snow_golem_snowball_defense_shearing_and_melting() {
    let mut snow_golem = SnowGolem::new();
    assert!(snow_golem.has_pumpkin);
    assert!(snow_golem.leaves_snow_trail);

    // Shearing carved pumpkin off head
    let drop = snow_golem.shear_pumpkin(ItemId::Shears);
    assert_eq!(drop, Some(ItemId::CarvedPumpkin));
    assert!(!snow_golem.has_pumpkin);

    // Melting in warm biome (temp = 1.0)
    let melt_dmg = snow_golem.tick_temperature_and_water(1.0, false);
    assert_eq!(melt_dmg, Some(1.0));
    assert_eq!(snow_golem.health, 3.0);

    // Snowball throwing at hostile in 10-block range
    let launch_vel = snow_golem.try_throw_snowball(
        Vec3::new(0.0, 64.0, 0.0),
        Vec3::new(10.0, 64.0, 0.0),
    );
    assert!(launch_vel.is_some());
    let vel = launch_vel.unwrap();
    assert!(vel.x > 0.0);
    assert!(vel.y > 0.0); // slight arc
}

#[test]
fn test_wandering_trader_invisibility_and_spawner() {
    let mut trader = WanderingTrader::new();
    assert!(!trader.is_invisible);
    assert!(!trader.trades.is_empty());

    // Dusk / Night (tick 13000): drinks invisibility potion
    trader.tick(13000, false);
    // Drinking animation takes 32 ticks
    for _ in 0..WanderingTrader::DRINK_DURATION_TICKS {
        trader.tick(13000, false);
    }
    assert!(trader.is_invisible);

    // Dawn (tick 1000): drinks milk, clearing invisibility
    trader.tick(1000, false);
    assert!(!trader.is_invisible);

    // Spawner guarantees spawn after 12 days
    let mut spawner = WanderingTraderSpawner::new();
    for _ in 0..(12 * WanderingTraderSpawner::DAY_TICKS) {
        if spawner.tick(0.0) {
            break;
        }
    }
    assert_eq!(spawner.days_without_spawn, 0);
}
