use glam::{IVec3, Vec3};
use mc_entities::ai::*;
use mc_entities::components::*;
use mc_items::ItemId;
use std::collections::HashSet;

// -----------------------------------------------------------------------------
// Test Mock World for 3D Voxel Navigation
// -----------------------------------------------------------------------------

struct MockVoxelWorld {
    solid_blocks: HashSet<IVec3>,
    water_blocks: HashSet<IVec3>,
    lava_blocks: HashSet<IVec3>,
    fire_blocks: HashSet<IVec3>,
    doors: HashSet<IVec3>,
    open_doors: HashSet<IVec3>,
    wooden_doors: HashSet<IVec3>,
}

impl MockVoxelWorld {
    fn new() -> Self {
        Self {
            solid_blocks: HashSet::new(),
            water_blocks: HashSet::new(),
            lava_blocks: HashSet::new(),
            fire_blocks: HashSet::new(),
            doors: HashSet::new(),
            open_doors: HashSet::new(),
            wooden_doors: HashSet::new(),
        }
    }

    fn add_solid_floor(&mut self, min_x: i32, max_x: i32, y: i32, min_z: i32, max_z: i32) {
        for x in min_x..=max_x {
            for z in min_z..=max_z {
                self.solid_blocks.insert(IVec3::new(x, y, z));
            }
        }
    }

    fn add_hazard_fire(&mut self, pos: IVec3) {
        self.fire_blocks.insert(pos);
    }

    fn add_hazard_lava(&mut self, pos: IVec3) {
        self.lava_blocks.insert(pos);
    }

    fn add_door(&mut self, pos: IVec3, is_open: bool, is_wooden: bool) {
        self.doors.insert(pos);
        if is_open {
            self.open_doors.insert(pos);
        }
        if is_wooden {
            self.wooden_doors.insert(pos);
        }
    }
}

impl BlockAccessor for MockVoxelWorld {
    fn is_solid(&self, pos: IVec3) -> bool {
        self.solid_blocks.contains(&pos)
    }

    fn is_water(&self, pos: IVec3) -> bool {
        self.water_blocks.contains(&pos)
    }

    fn is_lava(&self, pos: IVec3) -> bool {
        self.lava_blocks.contains(&pos)
    }

    fn is_fire(&self, pos: IVec3) -> bool {
        self.fire_blocks.contains(&pos)
    }

    fn is_door(&self, pos: IVec3) -> bool {
        self.doors.contains(&pos)
    }

    fn is_door_open(&self, pos: IVec3) -> bool {
        self.open_doors.contains(&pos)
    }

    fn can_open_door(&self, pos: IVec3) -> bool {
        self.wooden_doors.contains(&pos)
    }
}

// -----------------------------------------------------------------------------
// 1. GoalSelector & Flag Preemption Tests
// -----------------------------------------------------------------------------

struct MockMoveGoal {
    name: &'static str,
    can_run: bool,
    started: bool,
    stopped: bool,
    ticked: bool,
}

impl MockMoveGoal {
    fn new(name: &'static str, can_run: bool) -> Self {
        Self {
            name,
            can_run,
            started: false,
            stopped: false,
            ticked: false,
        }
    }
}

impl Goal for MockMoveGoal {
    fn can_use(&mut self, _ctx: &GoalContext) -> bool {
        self.can_run
    }

    fn start(&mut self, _ctx: &mut GoalContext) {
        self.started = true;
    }

    fn stop(&mut self, _ctx: &mut GoalContext) {
        self.stopped = true;
    }

    fn tick(&mut self, _ctx: &mut GoalContext) {
        self.ticked = true;
    }

    fn flags(&self) -> GoalFlags {
        GoalFlags::MOVE
    }

    fn name(&self) -> &'static str {
        self.name
    }
}

#[test]
fn test_goal_selector_priority_preemption() {
    let mut selector = GoalSelector::new();
    let mut ctx = GoalContext::new(EntityId(1), EntityType::Zombie, Vec3::ZERO);

    // Goal A: Priority 3, uses MOVE flag
    selector.add_goal(3, Box::new(MockMoveGoal::new("WanderGoal", true)));
    // Goal B: Priority 1, uses MOVE flag (higher priority, initially can_run = false)
    selector.add_goal(1, Box::new(MockMoveGoal::new("PanicGoal", false)));

    // Tick 1: Goal A starts because B cannot run
    selector.tick(&mut ctx);
    assert!(selector.is_running("WanderGoal"));
    assert!(!selector.is_running("PanicGoal"));

    // Now make Goal B ready to run
    // Preemption: Goal B has priority 1 < 3, so it preempts Goal A on the MOVE flag
    selector.remove_goal_by_name("PanicGoal", &mut ctx);
    selector.add_goal(1, Box::new(MockMoveGoal::new("PanicGoal", true)));

    selector.tick(&mut ctx);
    assert!(!selector.is_running("WanderGoal"), "Lower priority goal should have been preempted");
    assert!(selector.is_running("PanicGoal"), "Higher priority goal should be running");
}

#[test]
fn test_goal_selector_concurrent_non_conflicting_flags() {
    let mut selector = GoalSelector::new();
    let mut ctx = GoalContext::new(EntityId(1), EntityType::Cow, Vec3::ZERO);

    // Move goal (MOVE flag)
    selector.add_goal(2, Box::new(MockMoveGoal::new("MoveGoal", true)));
    // Look goal (LOOK flag)
    selector.add_goal(3, Box::new(LookAtPlayerGoal::new(8.0)));

    ctx.nearby_players.push(NearbyPlayer {
        id: EntityId(10),
        position: Vec3::new(2.0, 0.0, 2.0),
        eye_position: Vec3::new(2.0, 1.62, 2.0),
        held_item: None,
        is_creative_or_spectator: false,
        has_line_of_sight: true,
    });

    selector.tick(&mut ctx);
    assert!(selector.is_running("MoveGoal"));
    assert!(selector.is_running("LookAtPlayerGoal"));
    assert_eq!(selector.running_goals().len(), 2);
}

#[test]
fn test_goal_selector_disabled_flags() {
    let mut selector = GoalSelector::new();
    let mut ctx = GoalContext::new(EntityId(1), EntityType::Zombie, Vec3::ZERO);

    selector.add_goal(2, Box::new(MockMoveGoal::new("MoveGoal", true)));
    selector.tick(&mut ctx);
    assert!(selector.is_running("MoveGoal"));

    // Disable MOVE flag (e.g. mob is stunned or riding a vehicle)
    selector.disable_control_flag(GoalFlags::MOVE);
    selector.tick(&mut ctx);
    assert!(!selector.is_running("MoveGoal"), "Move goal must stop when MOVE flag is disabled");
}

// -----------------------------------------------------------------------------
// 2. Passive Mob Goals Tests
// -----------------------------------------------------------------------------

#[test]
fn test_float_goal_submerged_in_water() {
    let mut float_goal = FloatGoal::new();
    let mut ctx = GoalContext::new(EntityId(1), EntityType::Cow, Vec3::ZERO);

    // Not in water
    assert!(!float_goal.can_use(&ctx));

    // In water
    ctx.is_in_water = true;
    assert!(float_goal.can_use(&ctx));

    float_goal.tick(&mut ctx);
    assert!(ctx.jump_requested);
    assert_eq!(ctx.requested_velocity.unwrap().y, 0.3);
}

#[test]
fn test_panic_goal_triggered_on_damage() {
    let mut panic_goal = PanicGoal::standard();
    let mut ctx = GoalContext::new(EntityId(1), EntityType::Pig, Vec3::ZERO);

    // Full health -> should not panic
    assert!(!panic_goal.can_use(&ctx));

    // Hurt pig
    ctx.health.current = 5.0; // was 10.0
    assert!(panic_goal.can_use(&ctx));

    panic_goal.start(&mut ctx);
    assert!(ctx.panic_active);

    panic_goal.tick(&mut ctx);
    assert!(ctx.requested_velocity.is_some());
    let vel = ctx.requested_velocity.unwrap();
    assert!(vel.length() > 0.0);
}

#[test]
fn test_tempt_goal_following_wheat_for_cow() {
    let mut tempt_goal = TemptGoal::standard();
    let mut ctx = GoalContext::new(EntityId(1), EntityType::Cow, Vec3::ZERO);

    // Player holding dirt: ignored
    ctx.nearby_players.push(NearbyPlayer {
        id: EntityId(100),
        position: Vec3::new(5.0, 0.0, 0.0),
        eye_position: Vec3::new(5.0, 1.62, 0.0),
        held_item: Some(ItemId::Dirt),
        is_creative_or_spectator: false,
        has_line_of_sight: true,
    });
    assert!(!tempt_goal.can_use(&ctx));

    // Player switches to Wheat: tempted!
    ctx.nearby_players[0].held_item = Some(ItemId::Wheat);
    assert!(tempt_goal.can_use(&ctx));

    tempt_goal.start(&mut ctx);
    tempt_goal.tick(&mut ctx);

    // Cow looks at player and moves toward player
    assert!(ctx.look_target.is_some());
    assert!(ctx.requested_velocity.is_some());
    let vel = ctx.requested_velocity.unwrap();
    assert!(vel.x > 0.0, "Should move towards player in positive X direction");

    // When close (within 2.5m reach), cow halts movement to await feeding
    ctx.transform.position = Vec3::new(4.0, 0.0, 0.0); // 1.0m away from player at (5, 0, 0)
    tempt_goal.tick(&mut ctx);
    assert_eq!(ctx.requested_velocity.unwrap().x, 0.0);
}

#[test]
fn test_breed_goal_partner_pairing() {
    let mut breed_goal = BreedGoal::standard();
    let mut ctx = GoalContext::new(EntityId(1), EntityType::Sheep, Vec3::ZERO);
    ctx.in_love_ticks = 600;

    // Partner sheep nearby also in love
    ctx.nearby_entities.push(NearbyEntity {
        id: EntityId(2),
        entity_type: EntityType::Sheep,
        position: Vec3::new(1.0, 0.0, 0.0),
        is_alive: true,
        in_love_ticks: 600,
    });

    assert!(breed_goal.can_use(&ctx));
    breed_goal.start(&mut ctx);

    // Simulate 20 ticks of contact
    for _ in 0..20 {
        breed_goal.tick(&mut ctx);
    }

    assert!(ctx.spawn_child_event, "Offspring should be spawned after breeding contact");
    assert_eq!(ctx.in_love_ticks, 0, "Love ticks should reset after breeding");
}

// -----------------------------------------------------------------------------
// 3. Hostile Mob Goals Tests
// -----------------------------------------------------------------------------

#[test]
fn test_nearest_attackable_target_goal_acquisition() {
    let mut target_goal = NearestAttackableTargetGoal::standard();
    let mut ctx = GoalContext::new(EntityId(1), EntityType::Zombie, Vec3::ZERO);

    // Player in creative mode: ignored
    ctx.nearby_players.push(NearbyPlayer {
        id: EntityId(50),
        position: Vec3::new(8.0, 0.0, 0.0),
        eye_position: Vec3::new(8.0, 1.62, 0.0),
        held_item: None,
        is_creative_or_spectator: true,
        has_line_of_sight: true,
    });
    assert!(!target_goal.can_use(&ctx));

    // Survival player: targeted!
    ctx.nearby_players[0].is_creative_or_spectator = false;
    assert!(target_goal.can_use(&ctx));

    target_goal.start(&mut ctx);
    assert!(ctx.target.is_some());
    assert_eq!(ctx.target.unwrap().id, EntityId(50));
}

#[test]
fn test_melee_attack_goal_damage_and_cooldown() {
    let mut attack_goal = MeleeAttackGoal::standard_zombie();
    let mut ctx = GoalContext::new(EntityId(1), EntityType::Zombie, Vec3::ZERO);

    ctx.target = Some(TargetEntity {
        id: EntityId(50),
        entity_type: EntityType::Player,
        position: Vec3::new(1.0, 0.0, 0.0), // within 1.8m reach
        health: 20.0,
        is_alive: true,
        has_line_of_sight: true,
    });

    assert!(attack_goal.can_use(&ctx));
    ctx.attack_cooldown = 0;

    // Tick: executes melee attack dealing 3 damage
    attack_goal.tick(&mut ctx);
    assert!(ctx.attack_event.is_some());
    let ev = ctx.attack_event.unwrap();
    assert_eq!(ev.damage, 3.0);
    assert_eq!(ev.target_id, EntityId(50));
    assert_eq!(ctx.attack_cooldown, 20, "Cooldown must be reset to 20 ticks");
}

#[test]
fn test_ranged_attack_goal_tactical_spacing() {
    let mut skeleton_ranged = RangedAttackGoal::standard_skeleton();
    let mut ctx = GoalContext::new(EntityId(1), EntityType::Skeleton, Vec3::ZERO);

    ctx.target = Some(TargetEntity {
        id: EntityId(50),
        entity_type: EntityType::Player,
        position: Vec3::new(3.0, 0.0, 0.0), // Too close! (< 6m)
        health: 20.0,
        is_alive: true,
        has_line_of_sight: true,
    });

    skeleton_ranged.tick(&mut ctx);
    let vel = ctx.requested_velocity.unwrap();
    assert!(vel.x < 0.0, "Skeleton must back away in negative X direction when player is too close");
}

// -----------------------------------------------------------------------------
// 4. 3D Voxel Node Navigation Tests
// -----------------------------------------------------------------------------

#[test]
fn test_navigation_flat_and_step_up_1_block() {
    let mut world = MockVoxelWorld::new();
    let nav = PathNavigation::standard_mob();

    // Floor from x=0..=5 at y=0, then steps up to y=1 at x=6..=10
    world.add_solid_floor(0, 5, 0, 0, 0);
    world.add_solid_floor(6, 10, 1, 0, 0);

    let start = IVec3::new(0, 1, 0);
    let goal = IVec3::new(8, 2, 0);

    let path = nav.find_path(start, goal, &world);
    assert!(path.is_some(), "Must find path stepping up 1 block obstacle");

    let p = path.unwrap();
    assert!(p.nodes.contains(&IVec3::new(6, 2, 0)), "Path must include the step-up voxel (6, 2, 0)");
}

#[test]
fn test_navigation_safe_step_down_and_cliff_avoidance() {
    let mut world = MockVoxelWorld::new();
    let nav = PathNavigation::standard_mob();

    // Floor at y=5, drops 2 blocks to y=3 (safe drop <= 3)
    world.add_solid_floor(0, 2, 5, 0, 0);
    world.add_solid_floor(3, 5, 3, 0, 0);

    let start = IVec3::new(1, 6, 0);
    let goal = IVec3::new(4, 4, 0);

    let path = nav.find_path(start, goal, &world);
    assert!(path.is_some(), "Safe 2-block step down must be traversable");

    // Dangerous cliff: drops 5 blocks to y=0 (> 3 blocks drop)
    let mut cliff_world = MockVoxelWorld::new();
    cliff_world.add_solid_floor(0, 2, 5, 0, 0);
    cliff_world.add_solid_floor(3, 5, 0, 0, 0);

    let cliff_path = nav.find_path(start, IVec3::new(4, 1, 0), &cliff_world);
    assert!(cliff_path.is_none(), "Cliff drop (> 3 blocks) must be strictly rejected");
}

#[test]
fn test_navigation_hazard_fire_and_lava_avoidance() {
    let mut world = MockVoxelWorld::new();
    let nav = PathNavigation::standard_mob();

    // 5x5 floor at y=0
    world.add_solid_floor(0, 4, 0, -2, 2);

    // Place lava block directly in direct path at (2, 1, 0) and fire at (2, 1, 1)
    world.add_hazard_lava(IVec3::new(2, 1, 0));
    world.add_hazard_fire(IVec3::new(2, 1, 1));

    let start = IVec3::new(0, 1, 0);
    let goal = IVec3::new(4, 1, 0);

    let path = nav.find_path(start, goal, &world);
    assert!(path.is_some(), "Pathfinder should navigate around the hazard");

    let p = path.unwrap();
    assert!(!p.nodes.contains(&IVec3::new(2, 1, 0)), "Path must NEVER traverse the lava voxel");
    assert!(!p.nodes.contains(&IVec3::new(2, 1, 1)), "Path must NEVER traverse the fire voxel");
}

#[test]
fn test_navigation_doors_open_vs_closed() {
    let mut world = MockVoxelWorld::new();
    world.add_solid_floor(0, 5, 0, 0, 0);

    // Closed iron door at (2, 1, 0)
    world.add_door(IVec3::new(2, 1, 0), false, false);

    let regular_nav = PathNavigation::standard_mob();
    let door_nav = PathNavigation::door_opening_mob();

    let start = IVec3::new(0, 1, 0);
    let goal = IVec3::new(4, 1, 0);

    // Regular mob cannot pass closed iron door
    assert!(regular_nav.find_path(start, goal, &world).is_none());
    // Door opening mob still cannot pass closed iron door
    assert!(door_nav.find_path(start, goal, &world).is_none());

    // Replace with closed wooden door
    world.wooden_doors.insert(IVec3::new(2, 1, 0));
    // Door opening mob can pass wooden door
    assert!(door_nav.find_path(start, goal, &world).is_some());
    // Regular mob still cannot open wooden door
    assert!(regular_nav.find_path(start, goal, &world).is_none());

    // Open door: everyone can pass!
    world.open_doors.insert(IVec3::new(2, 1, 0));
    assert!(regular_nav.find_path(start, goal, &world).is_some());
}

// -----------------------------------------------------------------------------
// 5. Component ECS Architecture Tests
// -----------------------------------------------------------------------------

#[test]
fn test_aabb_physics_and_raycast() {
    let box_a = Aabb::from_feet_and_dimensions(Vec3::new(0.0, 0.0, 0.0), 1.0, 2.0);
    let box_b = Aabb::from_feet_and_dimensions(Vec3::new(0.5, 0.0, 0.0), 1.0, 2.0);
    let box_c = Aabb::from_feet_and_dimensions(Vec3::new(5.0, 0.0, 0.0), 1.0, 2.0);

    assert!(box_a.intersects(&box_b));
    assert!(!box_a.intersects(&box_c));

    // Raycast hit
    let hit = box_a.ray_intersects(Vec3::new(-3.0, 1.0, 0.0), Vec3::new(1.0, 0.0, 0.0), 10.0);
    assert!(hit.is_some());
    assert!((hit.unwrap() - 2.5).abs() < 0.01);
}

#[test]
fn test_health_and_air_supply() {
    let mut health = Health::new(20.0);
    assert!(health.take_damage(6.0));
    assert_eq!(health.current, 14.0);

    // Damage cooldown prevents immediate subsequent hit
    assert!(!health.take_damage(4.0));
    assert_eq!(health.current, 14.0);

    let mut air = AirSupply::standard();
    assert_eq!(air.current, 300);

    // Simulate 300 underwater ticks
    for _ in 0..300 {
        assert!(!air.tick_underwater());
    }

    // Now out of air: next 20 ticks triggers drowning damage
    let mut took_drown_dmg = false;
    for _ in 0..20 {
        if air.tick_underwater() {
            took_drown_dmg = true;
        }
    }
    assert!(took_drown_dmg, "Entity must take drowning damage when out of air");

    air.replenish();
    assert_eq!(air.current, 300);
}

#[test]
fn test_entity_type_attributes() {
    let zombie = EntityType::Zombie;
    let cow = EntityType::Cow;
    let creeper = EntityType::Creeper;

    assert!(zombie.is_hostile());
    assert!(zombie.is_undead());
    assert!(!zombie.is_passive());

    assert!(cow.is_passive());
    assert!(!cow.is_hostile());
    assert!(cow.is_tempting_item(ItemId::Wheat));
    assert!(!cow.is_tempting_item(ItemId::WheatSeeds));

    assert!(creeper.is_hostile());
    assert!(!creeper.is_undead());
}
