// Tests for Task 54: Hostile Mob Registry, Attack Logic & Projectiles.

use glam::Vec3;
use mc_entities::components::EntityId;
use mc_entities::hostile::*;
use mc_items::ItemId;

#[test]
fn test_sunlight_burning_and_zombie_drowning_conversion() {
    // Burning during day with sky exposure
    assert!(is_sun_sensitive_burning(1000, true, false, false));
    // Not burning at night
    assert!(!is_sun_sensitive_burning(13000, true, false, false));
    // Not burning if wearing helmet
    assert!(!is_sun_sensitive_burning(1000, true, false, true));
    // Not burning in water or rain
    assert!(!is_sun_sensitive_burning(1000, true, true, false));

    // Zombie underwater drowning conversion
    let mut zombie = ZombieMob::new();
    assert!(!zombie.tick_underwater(true));
    // Fast-forward submerged time
    zombie.submerged_ticks = ZombieMob::SUBMERGED_DROWN_TIME;
    zombie.conversion_ticks = ZombieMob::DROWN_CONVERSION_TIME - 1;
    // Next tick completes conversion to Drowned
    assert!(zombie.tick_underwater(true));

    // Zombie curing
    let mut zombie_villager = ZombieMob::new();
    zombie_villager.start_curing();
    assert!(zombie_villager.is_curing);
    zombie_villager.cure_ticks_remaining = 1;
    assert!(zombie_villager.tick_curing());
    assert!(!zombie_villager.is_curing);
}

#[test]
fn test_skeleton_strafing_bow_and_tipped_arrows() {
    let mut skeleton = SkeletonMob::skeleton();
    let current_pos = Vec3::new(0.0, 64.0, 0.0);
    let target_pos = Vec3::new(8.0, 64.0, 0.0);

    // Strafing while drawing bow
    let mut fired_arrow = None;
    for _ in 0..SkeletonMob::BOW_DRAW_DURATION {
        let (_move_vel, arrow) = skeleton.tick_bow_aim(current_pos, target_pos);
        if arrow.is_some() {
            fired_arrow = arrow;
        }
    }
    assert!(fired_arrow.is_some());
    let arrow = fired_arrow.unwrap();
    assert_eq!(arrow.effect, ArrowEffect::Normal);
    assert!(arrow.velocity.x > 0.0);

    // Stray tipped arrow of Slowness
    let stray = SkeletonMob::stray();
    assert_eq!(stray.arrow_effect, ArrowEffect::Slowness);

    // Bogged tipped arrow of Poison
    let bogged = SkeletonMob::bogged();
    assert_eq!(bogged.arrow_effect, ArrowEffect::Poison);
}

#[test]
fn test_spider_wall_climbing_and_pouncing() {
    let mut spider = SpiderMob::spider();

    // Normal horizontal movement
    let normal_vel = spider.compute_movement(false, Vec3::new(0.2, 0.0, 0.0));
    assert_eq!(normal_vel.y, 0.0);
    assert!(!spider.is_climbing);

    // Colliding horizontally triggers vertical climbing
    let climb_vel = spider.compute_movement(true, Vec3::new(0.2, 0.0, 0.0));
    assert_eq!(climb_vel.y, 0.15);
    assert!(spider.is_climbing);

    // Pounce leap towards target at 4 blocks
    let pounce = spider.try_pounce(Vec3::new(0.0, 64.0, 0.0), Vec3::new(4.0, 64.0, 0.0));
    assert!(pounce.is_some());
    let pounce_vel = pounce.unwrap();
    assert!(pounce_vel.y > 0.3); // upward leap arc
    assert!(pounce_vel.x > 0.3);
}

#[test]
fn test_creeper_fuse_swelling_and_charged_explosion() {
    let mut creeper = CreeperMob::new();
    let pos = Vec3::new(10.0, 64.0, 10.0);

    // Target is close (2m): fuse counts up
    for _ in 0..29 {
        assert_eq!(creeper.tick_fuse(Some(2.0), pos), None);
    }
    assert_eq!(creeper.fuse_ticks, 29);

    // Tick 30: detonates normal explosion
    let explosion = creeper.tick_fuse(Some(2.0), pos).unwrap();
    assert_eq!(explosion.power, 3.0);
    assert!(!explosion.causes_fire);

    // Charged Creeper has double explosion power (6.0)
    let mut charged_creeper = CreeperMob::charged();
    charged_creeper.fuse_ticks = 29;
    let charged_exp = charged_creeper.tick_fuse(Some(2.0), pos).unwrap();
    assert_eq!(charged_exp.power, 6.0);

    // Damage decay with distance
    let close_damage = charged_exp.compute_damage_at(pos + Vec3::new(1.0, 0.0, 0.0));
    let far_damage = charged_exp.compute_damage_at(pos + Vec3::new(8.0, 0.0, 0.0));
    assert!(close_damage > far_damage);
}

#[test]
fn test_phantom_insomnia_targeting_and_swoop() {
    // Insomnia check: only targets players with >= 72000 ticks without sleep
    assert!(!PhantomMob::can_target_player(50000));
    assert!(PhantomMob::can_target_player(72000));
    assert!(PhantomMob::can_target_player(100000));

    let mut phantom = PhantomMob::new();
    let current_pos = Vec3::new(0.0, 80.0, 0.0);
    let target_pos = Vec3::new(0.0, 64.0, 0.0);

    // High in sky: circles and initiates swoop
    let vel = phantom.compute_flight_velocity(current_pos, target_pos);
    assert!(vel.length() > 0.0);
}

#[test]
fn test_witch_defensive_potions_and_splash_throwing() {
    let mut witch = WitchMob::new();

    // Health drops low (< 11.0 HP): drinks Instant Health
    witch.health = 8.0;
    witch.tick_defensive_potions(false, false);
    assert_eq!(witch.drink_potion_ticks, 30);
    witch.drink_potion_ticks = 1;
    assert!(witch.tick_defensive_potions(false, false));
    assert_eq!(witch.health, 12.0);

    // Throws splash potion within 10 blocks
    assert!(witch.try_throw_splash_potion(6.0));
    assert_eq!(witch.attack_cooldown, 40);
    assert!(!witch.try_throw_splash_potion(6.0)); // On cooldown
}

#[test]
fn test_illagers_pillager_vindicator_and_evoker_fangs() {
    let mut pillager = PillagerMob::new();
    let pos = Vec3::new(0.0, 64.0, 0.0);
    let target = Vec3::new(10.0, 64.0, 0.0);

    // Crossbow loading cycle
    for _ in 0..PillagerMob::CROSSBOW_RELOAD_TICKS {
        assert_eq!(pillager.tick_attack(pos, target), None);
    }
    assert!(pillager.is_charged);
    let fired_bolt = pillager.tick_attack(pos, target);
    assert!(fired_bolt.is_some());
    assert!(!pillager.is_charged);

    // Vindicator melee strike
    let mut vindicator = VindicatorMob::new();
    assert_eq!(vindicator.try_strike(1.5), Some(13.0));
    assert_eq!(vindicator.try_strike(1.5), None); // On cooldown

    // Evoker fangs line
    let mut evoker = EvokerMob::new();
    let fangs = evoker.cast_fangs_line(pos, target);
    assert_eq!(fangs.len(), 16);
    assert_eq!(fangs[0].warmup_ticks, 2);
    assert_eq!(fangs[15].warmup_ticks, 32);

    // Fangs snap and deal damage
    let mut front_fang = fangs[0].clone();
    assert_eq!(front_fang.tick(), None);
    assert_eq!(front_fang.tick(), Some(6.0)); // Snaps at tick 2
    assert!(front_fang.is_snapped);

    // Guaranteed Totem of Undying drop
    assert!(evoker.get_death_drops().contains(&ItemId::TotemOfUndying));
}

#[test]
fn test_ravager_shield_stun_and_concussive_roar() {
    let mut ravager = RavagerMob::new();
    assert_eq!(ravager.ram_attack(), Some(12.0));

    // Player blocks with shield -> Stuns ravager for 40 ticks
    ravager.on_blocked_by_shield();
    assert!(ravager.is_stunned);
    assert_eq!(ravager.ram_attack(), None); // Cannot attack while stunned

    // Ticks down stun
    for _ in 0..RavagerMob::STUN_DURATION_TICKS - 1 {
        assert_eq!(ravager.tick(), None);
    }
    // Final stun tick unleashes concussive roar
    let roar = ravager.tick().unwrap();
    assert_eq!(roar.damage, 6.0);
    assert_eq!(roar.knockback_radius, 4.0);
    assert!(!ravager.is_stunned);
}

#[test]
fn test_nether_ghast_blaze_magmacube_and_piglin() {
    let mut ghast = GhastMob::new();
    let pos = Vec3::new(0.0, 100.0, 0.0);
    let target = Vec3::new(20.0, 80.0, 0.0);

    // Ghast charge & fireball release
    let mut fireball = None;
    for _ in 0..GhastMob::CHARGE_DURATION {
        if let Some(fb) = ghast.tick_attack(pos, target) {
            fireball = Some(fb);
        }
    }
    assert!(fireball.is_some());
    let fb = fireball.unwrap();
    assert!(fb.is_large);
    assert_eq!(fb.explosion_power, 1.0);

    // Blaze triple burst
    let mut blaze = BlazeMob::new();
    let mut burst_fireballs = Vec::new();
    for _ in 0..25 {
        if let Some(bfb) = blaze.tick_attack(pos, target) {
            burst_fireballs.push(bfb);
        }
    }
    assert_eq!(burst_fireballs.len(), 3);
    assert!(!burst_fireballs[0].is_large);

    // Magma Cube splitting
    let large_cube = MagmaCubeMob::new(SlimeSize::Large);
    let medium_cubes = large_cube.split_on_death().unwrap();
    assert_eq!(medium_cubes.len(), 3);
    assert_eq!(medium_cubes[0].size, SlimeSize::Medium);

    // Piglin gold bartering and Overworld zombification
    let mut piglin = PiglinMob::new();
    assert!(piglin.offer_gold_ingot(ItemId::GoldIngot));
    assert!(piglin.is_bartering);

    // In Overworld for 300 ticks transforms into Zombified Piglin
    for _ in 0..PiglinMob::ZOMBIFICATION_THRESHOLD {
        piglin.tick(true);
    }
    assert!(piglin.is_zombified);
}

#[test]
fn test_specialized_guardian_warden_breeze_and_creaking() {
    // Guardian beam lock (60 ticks)
    let mut guardian = GuardianMob::guardian();
    guardian.start_beam_lock(EntityId(1));
    for _ in 0..GuardianMob::BEAM_LOCK_DURATION - 1 {
        assert_eq!(guardian.tick_beam(), None);
    }
    assert_eq!(guardian.tick_beam(), Some(9.0));

    // Elder Guardian deals 12.0 damage
    let mut elder = GuardianMob::elder_guardian();
    elder.start_beam_lock(EntityId(1));
    elder.beam_lock_ticks = GuardianMob::BEAM_LOCK_DURATION - 1;
    assert_eq!(elder.tick_beam(), Some(12.0));

    // Thorns damage
    assert_eq!(guardian.on_melee_attacked(), 2.0);

    // Warden vibration & sonic boom
    let mut warden = WardenMob::new();
    assert!(warden.is_blind);
    warden.on_vibration(EntityId(1), 5);
    assert_eq!(warden.anger_level, 50);

    let pos = Vec3::new(0.0, 0.0, 0.0);
    let target = Vec3::new(10.0, 0.0, 0.0);
    for _ in 0..WardenMob::SONIC_BOOM_CHARGE_TICKS - 1 {
        assert_eq!(warden.tick_sonic_boom(pos, target), None);
    }
    let sonic_boom = warden.tick_sonic_boom(pos, target).unwrap();
    assert_eq!(sonic_boom.damage, 10.0);
    assert!(sonic_boom.knockback.x > 1.0);

    // Breeze projectile reflection
    assert!(BreezeMob::should_reflect_projectile());
    let mut breeze = BreezeMob::new();
    let wind_charge = breeze.shoot_wind_charge(pos, target);
    assert!(wind_charge.is_some());

    // Creaking invulnerability & eye-contact freezing
    let mut creaking = CreakingMob::new(Vec3::new(50.0, 64.0, 50.0));
    assert!(!CreakingMob::takes_damage_directly());

    // Player looks directly at Creaking (dot > 0.5)
    let player_pos = Vec3::new(0.0, 64.0, 0.0);
    let creaking_pos = Vec3::new(10.0, 64.0, 0.0);
    let looking_at = Vec3::new(1.0, 0.0, 0.0); // looking straight towards creaking
    assert!(creaking.update_player_gaze(creaking_pos, player_pos, looking_at));
    assert!(creaking.is_frozen);

    // Player looks away
    let looking_away = Vec3::new(-1.0, 0.0, 0.0);
    assert!(!creaking.update_player_gaze(creaking_pos, player_pos, looking_away));
    assert!(!creaking.is_frozen);
}
