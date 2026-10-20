// Tests for Task 52: Aquatic, Amphibious & Fluid Mobs.

use glam::Vec3;
use mc_entities::aquatic::*;
use mc_entities::components::{EntityId, EntityType};
use mc_entities::passive::DyeColor;
use mc_items::ItemId;

#[test]
fn test_breath_meter_and_drowning() {
    // Air breather in water
    let mut land_mob_breath = BreathMeter::new(RespirationType::AirBreather, 10);
    let mut drown_dmg = None;
    for _ in 0..35 {
        if let Some(dmg) = land_mob_breath.tick(true) {
            drown_dmg = Some(dmg);
            break;
        }
    }
    assert_eq!(drown_dmg, Some(2.0));

    // Replenishing air on surface
    land_mob_breath.tick(false);
    assert!(land_mob_breath.air_supply > 0);

    // Fish on land
    let mut fish_breath = BreathMeter::fish();
    fish_breath.air_supply = 0;
    let mut suffocation_dmg = None;
    for _ in 0..25 {
        if let Some(dmg) = fish_breath.tick(false) {
            suffocation_dmg = Some(dmg);
            break;
        }
    }
    assert_eq!(suffocation_dmg, Some(1.0));

    // Amphibious mob
    let mut amphibian_breath = BreathMeter::amphibious();
    assert_eq!(amphibian_breath.tick(true), None);
    assert_eq!(amphibian_breath.air_supply, 1200);
}

#[test]
fn test_water_path_navigation_drag() {
    let mut nav = WaterPathNavigation::new(1.0, false);
    nav.set_destination(Vec3::new(10.0, 5.0, 0.0));

    let current_pos = Vec3::new(0.0, 5.0, 0.0);
    let current_vel = Vec3::ZERO;
    let next_vel = nav.compute_swim_velocity(current_pos, current_vel);

    assert!(next_vel.x > 0.0);
    assert_eq!(next_vel.y, 0.0);
}

#[test]
fn test_fish_schooling_cod_and_salmon() {
    let mut cod_leader = Cod::new();
    let mut cod_follower = Cod::new();

    let leader_id = EntityId(101);
    let follower_id = EntityId(102);

    assert!(cod_leader.school.add_follower(follower_id));
    assert!(cod_leader.school.is_leader());
    assert_eq!(cod_leader.school.followers.len(), 1);

    cod_follower.school.start_following(leader_id);
    assert!(cod_follower.school.has_leader());

    let vel = cod_follower.school.compute_school_velocity(
        Vec3::new(0.0, 10.0, 0.0),
        Vec3::new(3.0, 10.0, 0.0),
        Vec3::new(0.1, 0.0, 0.0),
    );
    assert!(vel.x > 0.0);

    // Bucketing
    assert_eq!(
        cod_leader.interact_bucket(ItemId::WaterBucket),
        Some(ItemId::CodBucket)
    );

    let salmon = Salmon::new();
    assert!(salmon.can_leap_waterfalls);
    assert_eq!(
        salmon.interact_bucket(ItemId::WaterBucket),
        Some(ItemId::SalmonBucket)
    );
}

#[test]
fn test_pufferfish_inflation_and_poison() {
    let mut puffer = Pufferfish::new();
    assert_eq!(puffer.puff_state, PuffState::Small);
    assert_eq!(puffer.touch_target(), None);

    // Threat approaches within 2.0 blocks (Mid puff)
    puffer.update_threat_distance(Some(2.0));
    assert_eq!(puffer.puff_state, PuffState::Mid);
    let sting_mid = puffer.touch_target().unwrap();
    assert_eq!(sting_mid.damage, 1.0);
    assert_eq!(sting_mid.poison_duration_ticks, 80);

    // Threat gets very close (1.0 block -> Full puff)
    puffer.update_threat_distance(Some(1.0));
    assert_eq!(puffer.puff_state, PuffState::Full);
    let sting_full = puffer.touch_target().unwrap();
    assert_eq!(sting_full.damage, 2.0);
    assert_eq!(sting_full.poison_duration_ticks, 140);
    assert_eq!(sting_full.poison_amplifier, 1);

    // Deflating when threat departs
    for _ in 0..Pufferfish::DEFLATE_TIME_TICKS + 1 {
        puffer.update_threat_distance(None);
    }
    assert_eq!(puffer.puff_state, PuffState::Mid);

    // Bucketing
    assert_eq!(
        puffer.interact_bucket(ItemId::WaterBucket),
        Some(ItemId::PufferfishBucket)
    );
}

#[test]
fn test_tropical_fish_presets() {
    let clownfish = TropicalFish::clownfish();
    assert_eq!(clownfish.base_color, DyeColor::Orange);
    assert_eq!(clownfish.pattern_color, DyeColor::White);
    assert_eq!(clownfish.pattern, TropicalFishPattern::Flopper);

    let blue_tang = TropicalFish::blue_tang();
    assert_eq!(blue_tang.base_color, DyeColor::LightBlue);
    assert_eq!(blue_tang.pattern_color, DyeColor::Gray);

    assert_eq!(
        clownfish.interact_bucket(ItemId::WaterBucket),
        Some(ItemId::TropicalFishBucket)
    );
}

#[test]
fn test_squid_jet_thrust_and_ink_defense() {
    let mut squid = Squid::new();
    let heading = Vec3::new(0.0, 1.0, 0.0);

    // Periodic jet thrust every 40 ticks
    let mut impulse = None;
    for _ in 0..CephalopodMovement::THRUST_INTERVAL {
        impulse = squid.movement.tick(true, heading);
    }
    assert!(impulse.is_some());
    assert!(impulse.unwrap().y > 0.1);

    // Ink defense on damage
    let (ink_item, escape_vec) = squid.on_damaged();
    assert_eq!(ink_item, ItemId::InkSac);
    assert!(escape_vec.y > 0.0);

    // Glow Squid
    let mut glow_squid = GlowSquid::new();
    assert!(glow_squid.is_glowing());
    let (glow_ink, _) = glow_squid.on_damaged();
    assert_eq!(glow_ink, ItemId::GlowInkSac);
    assert!(!glow_squid.is_glowing());

    for _ in 0..GlowSquid::DARK_DURATION_TICKS {
        glow_squid.tick();
    }
    assert!(glow_squid.is_glowing());
}

#[test]
fn test_turtle_home_beach_and_scute_growth() {
    let mut turtle = Turtle::new();
    let beach = Vec3::new(128.0, 64.0, 256.0);
    turtle.set_home_pos(beach);
    assert_eq!(turtle.home_pos, Some(beach));

    // Crawling vs Swimming speed
    assert_eq!(turtle.current_speed_modifier(false), 0.25);
    assert_eq!(turtle.current_speed_modifier(true), 1.0);

    // Breeding & Gravid state
    assert!(turtle.interact(EntityId(1), ItemId::Seagrass));
    turtle.become_gravid();
    assert!(turtle.has_egg);

    // Digging sand on home beach
    turtle.start_digging_sand();
    assert!(turtle.is_digging_sand);

    for _ in 0..Turtle::DIG_SAND_DURATION_TICKS {
        assert_eq!(turtle.tick_digging(), None);
    }
    let eggs = turtle.tick_digging().unwrap();
    assert_eq!(eggs.len(), 3);
    assert_eq!(eggs[0], ItemId::TurtleEgg);
    assert!(!turtle.has_egg);

    // Baby growing into adult drops Scute
    let mut baby_turtle = Turtle::baby();
    baby_turtle.animal_data.age = -1;
    let shed_drop = baby_turtle.tick_growth();
    assert_eq!(shed_drop, Some(ItemId::Scute));
    assert!(!baby_turtle.animal_data.is_baby());
}

#[test]
fn test_frog_variants_and_froglight_consumption() {
    let mut temperate_frog = Frog::new(FrogVariant::Temperate);
    let mut warm_frog = Frog::new(FrogVariant::Warm);
    let mut cold_frog = Frog::new(FrogVariant::Cold);

    // Eating small slime
    assert_eq!(
        temperate_frog.swallow_prey(FrogPreyType::SmallSlime),
        ItemId::SlimeBall
    );

    // Eating small magma cube -> 3 unique Froglights
    assert_eq!(
        temperate_frog.swallow_prey(FrogPreyType::SmallMagmaCube),
        ItemId::PearlescentFroglight
    );
    assert_eq!(
        warm_frog.swallow_prey(FrogPreyType::SmallMagmaCube),
        ItemId::OchreFroglight
    );
    assert_eq!(
        cold_frog.swallow_prey(FrogPreyType::SmallMagmaCube),
        ItemId::VerdantFroglight
    );
}

#[test]
fn test_tadpole_metamorphosis() {
    let mut tadpole = Tadpole::new();
    assert_eq!(
        tadpole.interact_bucket(ItemId::WaterBucket),
        Some(ItemId::TadpoleBucket)
    );

    // Slime ball acceleration
    tadpole.feed_slime_ball();
    assert!(tadpole.age > -24000);

    // Metamorphosis in cold biome (snowy, temp = 0.0)
    tadpole.age = -1;
    let cold_frog = tadpole.tick(0.0).unwrap();
    assert_eq!(cold_frog.variant, FrogVariant::Cold);

    // Metamorphosis in warm biome (desert, temp = 1.0)
    let mut tadpole_warm = Tadpole::new();
    tadpole_warm.age = -1;
    let warm_frog = tadpole_warm.tick(1.0).unwrap();
    assert_eq!(warm_frog.variant, FrogVariant::Warm);
}

#[test]
fn test_axolotl_play_dead_and_hunting() {
    let mut axolotl = Axolotl::new(AxolotlVariant::Lucy);
    assert_eq!(
        axolotl.interact_bucket(ItemId::WaterBucket),
        Some(ItemId::AxolotlBucket)
    );
    assert!(axolotl.can_be_targeted_by_hostiles());

    // Takes non-lethal heavy damage underwater (> 50% HP lost: 14.0 - 8.0 = 6.0 HP left)
    axolotl.take_damage(8.0, true);
    assert!(axolotl.is_playing_dead);
    assert!(!axolotl.can_be_targeted_by_hostiles());

    // Regeneration while playing dead
    let prev_hp = axolotl.health;
    for _ in 0..50 {
        axolotl.tick_play_dead();
    }
    assert_eq!(axolotl.health, prev_hp + 1.0);

    // Hunts aquatic fish and drowned
    assert!(axolotl.is_hunt_target(EntityType::Cod));
    assert!(axolotl.is_hunt_target(EntityType::Salmon));
    assert!(axolotl.is_hunt_target(EntityType::TropicalFish));
    assert!(axolotl.is_hunt_target(EntityType::Drowned));
    assert!(!axolotl.is_hunt_target(EntityType::Cow));
}

#[test]
fn test_strider_lava_walking_and_shivering() {
    let mut strider = Strider::new();

    // In lava: warm and fast
    strider.update_temperature(true);
    assert!(!strider.is_cold);
    assert!((strider.current_speed() - Strider::BASE_LAVA_SPEED).abs() < 1e-4);

    // Out of lava: shivering, purple-gray, speed penalty
    strider.update_temperature(false);
    assert!(strider.is_cold);
    let cold_speed = strider.current_speed();
    assert!(cold_speed < Strider::BASE_LAVA_SPEED * 0.5);

    // Saddling
    assert!(strider.interact(EntityId(1), ItemId::Saddle));
    assert!(strider.is_saddled);

    // Warped fungus boost
    strider.rider = Some(EntityId(1));
    assert!(strider.interact(EntityId(1), ItemId::WarpedFungusOnAStick));
    assert!(strider.boost_ticks > 0);
    assert!(strider.current_speed() > cold_speed);

    // Water damage
    let water_dmg = strider.tick_water_damage(true);
    assert_eq!(water_dmg, Some(1.0));
    assert_eq!(strider.health, 19.0);

    // Drops saddle upon death
    let death_drops = strider.get_death_drops();
    assert!(death_drops.contains(&ItemId::Saddle));
    assert!(death_drops.contains(&ItemId::String));
}
