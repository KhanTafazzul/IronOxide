// REA Verified: net.minecraft.world.entity.ai.navigation.WaterBoundPathNavigation (mapped: bvx.class)
// REA Verified: net.minecraft.world.entity.LivingEntity.baseTick / handleAirSupply (mapped: bmm.class)
//
// 3D Water Path Navigation, buoyancy curves, water drag, and breath/drowning meters.

use glam::Vec3;

/// Respiration mode for aquatic and terrestrial entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RespirationType {
    /// Terrestrial mobs that breathe air and drown when submerged in water.
    #[default]
    AirBreather,
    /// Aquatic fish that breathe water and suffocate when stranded on dry land.
    WaterBreather,
    /// Amphibious mobs (e.g. Turtles, Axolotls, Frogs) with extended or dual respiration.
    Amphibious,
}

/// Breath Meter tracking oxygen supply and drowning/suffocation ticks.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BreathMeter {
    pub respiration: RespirationType,
    /// Remaining air supply ticks (default 300 ticks = 15 seconds).
    pub air_supply: i32,
    pub max_air_supply: i32,
    /// Countdown ticks between drowning damage strikes (every 20 ticks = 1s).
    pub damage_cooldown: u32,
}

impl Default for BreathMeter {
    fn default() -> Self {
        Self::new(RespirationType::AirBreather, 300)
    }
}

impl BreathMeter {
    pub const DEFAULT_MAX_AIR: i32 = 300; // 15 seconds of air
    pub const DROWN_DAMAGE_INTERVAL: u32 = 20; // 1 damage per second

    pub fn new(respiration: RespirationType, max_air: i32) -> Self {
        Self {
            respiration,
            air_supply: max_air,
            max_air_supply: max_air,
            damage_cooldown: 0,
        }
    }

    pub fn fish() -> Self {
        Self::new(RespirationType::WaterBreather, 300)
    }

    pub fn amphibious() -> Self {
        Self::new(RespirationType::Amphibious, 1200) // Amphibians have extended breath
    }

    /// Ticks oxygen consumption based on current fluid immersion:
    /// - Air-breathers lose air in water, take drowning damage when air <= -20.
    /// - Water-breathers lose air on dry land, take suffocation damage when air <= -20.
    /// Returns `Some(damage_amount)` when a drowning/suffocation damage tick occurs.
    pub fn tick(&mut self, is_submerged_in_water: bool) -> Option<f32> {
        match self.respiration {
            RespirationType::AirBreather => {
                if is_submerged_in_water {
                    self.air_supply -= 1;
                    if self.air_supply <= -20 {
                        self.air_supply = 0;
                        if self.damage_cooldown == 0 {
                            self.damage_cooldown = Self::DROWN_DAMAGE_INTERVAL;
                            return Some(2.0); // 1 heart of drowning damage
                        }
                    }
                } else {
                    // Rehydrate / replenish air rapidly on surface
                    self.air_supply = (self.air_supply + 5).min(self.max_air_supply);
                }
            }
            RespirationType::WaterBreather => {
                if !is_submerged_in_water {
                    self.air_supply -= 1;
                    if self.air_supply <= -20 {
                        self.air_supply = 0;
                        if self.damage_cooldown == 0 {
                            self.damage_cooldown = Self::DROWN_DAMAGE_INTERVAL;
                            return Some(1.0); // Suffocation damage on land
                        }
                    }
                } else {
                    self.air_supply = self.max_air_supply;
                }
            }
            RespirationType::Amphibious => {
                // Amphibians can stay indefinitely in water or on land
                self.air_supply = self.max_air_supply;
            }
        }

        if self.damage_cooldown > 0 {
            self.damage_cooldown -= 1;
        }

        None
    }
}

/// 3D Water Path Navigation for free-swimming fish, dolphins, squids, and turtles.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WaterPathNavigation {
    pub target_pos: Option<Vec3>,
    pub speed_modifier: f32,
    pub allow_breaching: bool,
    pub water_drag: f32,
}

impl Default for WaterPathNavigation {
    fn default() -> Self {
        Self {
            target_pos: None,
            speed_modifier: 1.0,
            allow_breaching: false,
            water_drag: 0.8,
        }
    }
}

impl WaterPathNavigation {
    pub fn new(speed_modifier: f32, allow_breaching: bool) -> Self {
        Self {
            target_pos: None,
            speed_modifier,
            allow_breaching,
            water_drag: 0.8,
        }
    }

    /// Sets 3D swim destination in water.
    pub fn set_destination(&mut self, pos: Vec3) {
        self.target_pos = Some(pos);
    }

    /// Computes 3D propulsion velocity towards target position with water resistance drag.
    pub fn compute_swim_velocity(&self, current_pos: Vec3, current_vel: Vec3) -> Vec3 {
        if let Some(target) = self.target_pos {
            let diff = target - current_pos;
            let dist = diff.length();

            if dist > 0.5 {
                let dir = diff / dist;
                let desired_speed = 0.25 * self.speed_modifier;
                let target_vel = dir * desired_speed;

                // Blend with current velocity using water drag
                current_vel * self.water_drag + target_vel * (1.0 - self.water_drag)
            } else {
                current_vel * self.water_drag
            }
        } else {
            current_vel * self.water_drag
        }
    }
}
