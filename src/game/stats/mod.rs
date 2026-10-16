// REA Verified: net.minecraft.world.food.FoodData (mapped: chx.class)
// REA Verified: net.minecraft.world.entity.player.Player.causeFoodExhaustion (mapped: cfi.A)
//
// Survival Vitals, Hunger Mechanics & Natural Regeneration:
// - Health (0.0 to 20.0, 10 hearts).
// - Food Level (0 to 20, 10 drumsticks).
// - Saturation (0.0 to 20.0).
// - Exhaustion (0.0 to 4.0).
// - Physical activity triggers:
//   * Sprinting: 0.1 exhaustion per meter.
//   * Jumping: 0.05 exhaustion (sprint-jump: 0.2).
//   * Breaking a block: 0.005 exhaustion.
// - Natural Regeneration:
//   * Fast regen: food == 20 && saturation > 0.0 -> heal 1.0 every 10 ticks.
//   * Normal regen: food >= 18 -> heal 1.0 every 80 ticks, add 6.0 exhaustion.
//   * Starvation: food == 0 -> deal 1.0 damage every 80 ticks.

/// Player survival statistics tracking health, hunger, saturation, and physical exhaustion
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerStats {
    pub health: f32,
    pub max_health: f32,
    pub food_level: u32,
    pub saturation: f32,
    pub exhaustion: f32,
    pub food_tick_timer: u32,
    pub tick_accumulator: f32,
    pub hurt_time: f32, // Remaining damage-shake time in seconds
    pub time: f32,      // Global running time in seconds
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerStats {
    pub const MAX_HEALTH: f32 = 20.0;
    pub const MAX_FOOD: u32 = 20;
    pub const EXHAUSTION_THRESHOLD: f32 = 4.0;
    pub const TICK_DURATION: f32 = 0.05; // 20 ticks per second

    pub fn new() -> Self {
        Self {
            health: Self::MAX_HEALTH,
            max_health: Self::MAX_HEALTH,
            food_level: Self::MAX_FOOD,
            saturation: 5.0, // Vanilla default starting saturation
            exhaustion: 0.0,
            food_tick_timer: 0,
            tick_accumulator: 0.0,
            hurt_time: 0.0,
            time: 0.0,
        }
    }

    /// REA Verified: net.minecraft.world.food.FoodData.addExhaustion (chx.a(F))
    /// When exhaustion exceeds 4.0: subtract 1.0 from saturation (or food level if saturation is 0)
    pub fn add_exhaustion(&mut self, amount: f32) {
        self.exhaustion += amount;
        while self.exhaustion >= Self::EXHAUSTION_THRESHOLD {
            self.exhaustion -= Self::EXHAUSTION_THRESHOLD;
            if self.saturation > 0.0 {
                self.saturation = (self.saturation - 1.0).max(0.0);
            } else {
                self.food_level = self.food_level.saturating_sub(1);
            }
        }
    }

    /// Sprinting adds 0.1 exhaustion per meter
    pub fn on_sprint_move(&mut self, meters: f32) {
        if meters > 0.0 {
            self.add_exhaustion(0.1 * meters);
        }
    }

    /// Jumping adds 0.05 exhaustion (sprint-jump adds 0.2)
    pub fn on_jump(&mut self, is_sprinting: bool) {
        let cost = if is_sprinting { 0.20 } else { 0.05 };
        self.add_exhaustion(cost);
    }

    /// Breaking a block adds 0.005 exhaustion
    pub fn on_block_break(&mut self) {
        self.add_exhaustion(0.005);
    }

    /// Applies damage to player health and triggers damage shake animation
    pub fn damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
        self.hurt_time = 0.5; // 0.5 seconds damage shake
    }

    /// Heals player health up to maximum health
    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    /// Advance time and run 20Hz tick simulation for natural regeneration and starvation
    pub fn update(&mut self, dt: f32) {
        self.time += dt;
        self.hurt_time = (self.hurt_time - dt).max(0.0);

        self.tick_accumulator += dt;
        while self.tick_accumulator >= Self::TICK_DURATION {
            self.tick_accumulator -= Self::TICK_DURATION;
            self.tick_food_data();
        }
    }

    /// REA Verified: net.minecraft.world.food.FoodData.tick (chx.a(cfi))
    /// Natural Regeneration & Starvation rules:
    /// - Fast regen: if food == 20 && saturation > 0 -> heal 1 heart every 10 ticks
    /// - Normal regen: if food >= 18 -> heal 1 heart every 80 ticks
    /// - Starvation: if food == 0 -> deal 1 damage every 80 ticks
    pub fn tick_food_data(&mut self) {
        if self.food_level >= 20 && self.saturation > 0.0 && self.health < self.max_health {
            self.food_tick_timer += 1;
            if self.food_tick_timer >= 10 {
                let drain = self.saturation.min(6.0);
                self.heal(drain / 6.0);
                self.add_exhaustion(drain);
                self.food_tick_timer = 0;
            }
        } else if self.food_level >= 18 && self.health < self.max_health {
            self.food_tick_timer += 1;
            if self.food_tick_timer >= 80 {
                self.heal(1.0);
                self.add_exhaustion(6.0);
                self.food_tick_timer = 0;
            }
        } else if self.food_level == 0 {
            self.food_tick_timer += 1;
            if self.food_tick_timer >= 80 {
                self.damage(1.0);
                self.food_tick_timer = 0;
            }
        } else {
            self.food_tick_timer = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_stats() {
        let stats = PlayerStats::new();
        assert_eq!(stats.health, 20.0);
        assert_eq!(stats.max_health, 20.0);
        assert_eq!(stats.food_level, 20);
        assert_eq!(stats.saturation, 5.0);
        assert_eq!(stats.exhaustion, 0.0);
    }

    #[test]
    fn test_sprint_and_jump_exhaustion() {
        let mut stats = PlayerStats::new();
        // 10 meters sprint -> 1.0 exhaustion
        stats.on_sprint_move(10.0);
        assert!((stats.exhaustion - 1.0).abs() < 1e-4);

        // Jump normal -> +0.05
        stats.on_jump(false);
        assert!((stats.exhaustion - 1.05).abs() < 1e-4);

        // Sprint jump -> +0.2
        stats.on_jump(true);
        assert!((stats.exhaustion - 1.25).abs() < 1e-4);

        // Breaking a block -> +0.005
        stats.on_block_break();
        assert!((stats.exhaustion - 1.255).abs() < 1e-4);
    }

    #[test]
    fn test_exhaustion_threshold_saturation_drain() {
        let mut stats = PlayerStats::new();
        stats.saturation = 2.0;
        stats.food_level = 20;

        // Add 4.0 exhaustion -> drains 1.0 saturation, exhaustion resets
        stats.add_exhaustion(4.0);
        assert!((stats.saturation - 1.0).abs() < 1e-4);
        assert_eq!(stats.food_level, 20);
        assert_eq!(stats.exhaustion, 0.0);

        // Add another 4.0 exhaustion -> drains last 1.0 saturation
        stats.add_exhaustion(4.0);
        assert_eq!(stats.saturation, 0.0);
        assert_eq!(stats.food_level, 20);

        // Add 4.0 more exhaustion -> saturation is 0, so food level drops to 19!
        stats.add_exhaustion(4.0);
        assert_eq!(stats.saturation, 0.0);
        assert_eq!(stats.food_level, 19);
    }

    #[test]
    fn test_fast_natural_regeneration() {
        let mut stats = PlayerStats::new();
        stats.health = 18.0;
        stats.food_level = 20;
        stats.saturation = 6.0;

        // Run 9 ticks -> no heal yet
        for _ in 0..9 {
            stats.tick_food_data();
        }
        assert_eq!(stats.health, 18.0);

        // 10th tick -> heals 1.0 (to 19.0)
        stats.tick_food_data();
        assert_eq!(stats.health, 19.0);
        assert!(stats.saturation < 6.0); // Saturation consumed by heal exhaustion
    }

    #[test]
    fn test_normal_natural_regeneration() {
        let mut stats = PlayerStats::new();
        stats.health = 15.0;
        stats.food_level = 18;
        stats.saturation = 0.0;

        // Run 79 ticks -> no heal
        for _ in 0..79 {
            stats.tick_food_data();
        }
        assert_eq!(stats.health, 15.0);

        // 80th tick -> heals 1.0 (to 16.0)
        stats.tick_food_data();
        assert_eq!(stats.health, 16.0);
    }

    #[test]
    fn test_starvation_damage() {
        let mut stats = PlayerStats::new();
        stats.health = 20.0;
        stats.food_level = 0;
        stats.saturation = 0.0;

        for _ in 0..79 {
            stats.tick_food_data();
        }
        assert_eq!(stats.health, 20.0);

        // 80th tick at 0 food -> deals 1.0 starvation damage!
        stats.tick_food_data();
        assert_eq!(stats.health, 19.0);
        assert!(stats.hurt_time > 0.0);
    }
}
