// REA Verified: net.minecraft.world.level.Level (ctp.class - rainLevel & thunderLevel)
// REA Verified: net.minecraft.server.level.ServerLevel (and.class - tickPrecipitation & weather timers)
// REA Verified: net.minecraft.world.level.biome.Biome (cuo.class - shouldFreeze & shouldSnow thresholds)
//
// Dynamic Weather & Accumulation Engine:
// - Global Weather Cycle:
//   * clear_weather_time: random timer (12,000 to 168,000 ticks)
//   * rain_time: random timer (12,000 to 24,000 ticks)
//   * thunder_time: random timer (12,000 to 168,000 ticks)
//   * Transitions: smoothly over 50 ticks (fade speed: 0.02 / tick)
// - Random Chunk Precipitation Ticking:
//   * Altitude lapse rate: T_effective = T_base - (y - 64) * 0.00166667
//   * If raining and T_effective < 0.15:
//     - Still water source exposed to sky freezes into Ice
//     - Solid surface places or increments SnowLayer (height 1 to 8)
//   * When clear weather returns: slowly thaws snow layers exposed to light level > 11
// - Atmosphere modifiers:
//   * Sky light interpolated down by 25% during rain
//   * Fog distance reduced for atmospheric occlusion

use crate::render::colormap::Biome;

/// Number of game ticks for full smooth weather fade (0.0 <-> 1.0)
pub const WEATHER_FADE_TICKS: f32 = 50.0;
pub const RAIN_FADE_PER_TICK: f32 = 1.0 / WEATHER_FADE_TICKS; // 0.02

/// Vanilla elevation lapse rate: temperature decreases with height above sea level (y = 64)
pub const TEMPERATURE_LAPSE_RATE: f32 = 0.00166667; // ~1.0 drop per 600 blocks above y=64
pub const SNOW_TEMPERATURE_THRESHOLD: f32 = 0.15;

/// Weather state enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeatherState {
    #[default]
    Clear,
    Rain,
    Thunder,
}

/// Active precipitation type falling at a specific location
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrecipitationType {
    None,
    Rain,
    Snow,
}

/// Simplified block state enum representing surfaces subject to freezing and snow accumulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrecipitationBlock {
    Air,
    SolidSurface,
    WaterSource,
    FlowingWater,
    Ice,
    PackedIce,
    SnowLayer(u8), // Height 1..=8 (8 represents a full block equivalent)
    SnowBlock,
}

impl PrecipitationBlock {
    /// Whether this block can support a layer of snow resting directly on top of it
    pub fn can_support_snow(&self) -> bool {
        match self {
            PrecipitationBlock::SolidSurface
            | PrecipitationBlock::SnowBlock
            | PrecipitationBlock::Ice
            | PrecipitationBlock::PackedIce => true,
            _ => false,
        }
    }
}

/// Result of a random chunk precipitation tick evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrecipitationTickResult {
    /// Water source block freezes into ice
    FreezeWater { x: i32, y: i32, z: i32 },
    /// New snow layer placed onto empty air over solid block
    PlaceSnow { x: i32, y: i32, z: i32, height: u8 },
    /// Existing snow layer height incremented (e.g. 1 -> 2 up to 8)
    AccumulateSnow { x: i32, y: i32, z: i32, new_height: u8 },
    /// Snow layer thaws due to warm clear weather or torch light > 11
    MeltSnow { x: i32, y: i32, z: i32, new_height: u8 },
    /// Ice thaws back into water source due to torch light > 11
    MeltIce { x: i32, y: i32, z: i32 },
}

/// Simple fast PRNG for weather timers
#[derive(Debug, Clone, PartialEq)]
pub struct WeatherRng {
    pub state: u64,
}

impl WeatherRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    pub fn next_range(&mut self, min: u32, max: u32) -> u32 {
        if min >= max {
            return min;
        }
        min + (self.next_u32() % (max - min))
    }
}

/// Weather state machine and climate precipitation engine
#[derive(Debug, Clone, PartialEq)]
pub struct WeatherManager {
    pub state: WeatherState,
    pub rain_strength: f32,
    pub prev_rain_strength: f32,
    pub thunder_strength: f32,
    pub prev_thunder_strength: f32,
    pub timer: u32,

    // Global weather cycle timers matching ServerLevel / ServerLevelData
    pub clear_weather_time: u32,
    pub rain_time: u32,
    pub thunder_time: u32,
    pub rng: WeatherRng,
}

impl Default for WeatherManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WeatherManager {
    pub fn new() -> Self {
        let mut rng = WeatherRng::new(123456789);
        let clear_time = rng.next_range(12000, 168000);
        let rain_time = rng.next_range(12000, 24000);
        let thunder_time = rng.next_range(12000, 168000);

        Self {
            state: WeatherState::Clear,
            rain_strength: 0.0,
            prev_rain_strength: 0.0,
            thunder_strength: 0.0,
            prev_thunder_strength: 0.0,
            timer: 0,
            clear_weather_time: clear_time,
            rain_time,
            thunder_time,
            rng,
        }
    }

    /// Sets the target weather state and begins smooth transition
    pub fn set_weather(&mut self, state: WeatherState) {
        self.state = state;
    }

    /// Target rain strength based on current state (Clear = 0.0, Rain/Thunder = 1.0)
    pub fn target_rain_strength(&self) -> f32 {
        match self.state {
            WeatherState::Clear => 0.0,
            WeatherState::Rain | WeatherState::Thunder => 1.0,
        }
    }

    /// Target thunder strength based on current state (Thunder = 1.0, otherwise 0.0)
    pub fn target_thunder_strength(&self) -> f32 {
        match self.state {
            WeatherState::Thunder => 1.0,
            _ => 0.0,
        }
    }

    /// Advances the smooth fade interpolation by 1 game tick (50ms).
    /// Smoothly fades rain_strength and thunder_strength toward target values over 50 ticks.
    pub fn tick(&mut self) {
        self.timer = self.timer.wrapping_add(1);
        self.prev_rain_strength = self.rain_strength;
        self.prev_thunder_strength = self.thunder_strength;

        let target_rain = self.target_rain_strength();
        if self.rain_strength < target_rain {
            self.rain_strength = (self.rain_strength + RAIN_FADE_PER_TICK).min(target_rain);
        } else if self.rain_strength > target_rain {
            self.rain_strength = (self.rain_strength - RAIN_FADE_PER_TICK).max(target_rain);
        }
        if (self.rain_strength - target_rain).abs() < 1e-5 {
            self.rain_strength = target_rain;
        }

        let target_thunder = self.target_thunder_strength();
        if self.thunder_strength < target_thunder {
            self.thunder_strength = (self.thunder_strength + RAIN_FADE_PER_TICK).min(target_thunder);
        } else if self.thunder_strength > target_thunder {
            self.thunder_strength = (self.thunder_strength - RAIN_FADE_PER_TICK).max(target_thunder);
        }
        if (self.thunder_strength - target_thunder).abs() < 1e-5 {
            self.thunder_strength = target_thunder;
        }
    }

    /// Full global weather simulation tick advancing cycle countdowns and smooth transitions.
    /// REA Reference: net.minecraft.server.level.ServerLevel.advanceWeatherCycle
    pub fn tick_weather_cycle(&mut self) {
        match self.state {
            WeatherState::Clear => {
                if self.clear_weather_time > 0 {
                    self.clear_weather_time -= 1;
                }
                if self.clear_weather_time == 0 {
                    // Clear weather expired: transition to Rain or Thunder
                    let is_thunder = (self.rng.next_u32() % 4) == 0;
                    if is_thunder {
                        self.set_weather(WeatherState::Thunder);
                        self.thunder_time = self.rng.next_range(12000, 168000);
                        self.rain_time = self.thunder_time.max(self.rng.next_range(12000, 24000));
                    } else {
                        self.set_weather(WeatherState::Rain);
                        self.rain_time = self.rng.next_range(12000, 24000);
                    }
                }
            }
            WeatherState::Rain => {
                if self.rain_time > 0 {
                    self.rain_time -= 1;
                }
                if self.rain_time == 0 {
                    // Rain expired: return to Clear weather
                    self.set_weather(WeatherState::Clear);
                    self.clear_weather_time = self.rng.next_range(12000, 168000);
                }
            }
            WeatherState::Thunder => {
                if self.thunder_time > 0 {
                    self.thunder_time -= 1;
                }
                if self.rain_time > 0 {
                    self.rain_time -= 1;
                }

                if self.thunder_time == 0 {
                    if self.rain_time > 0 {
                        self.set_weather(WeatherState::Rain);
                    } else {
                        self.set_weather(WeatherState::Clear);
                        self.clear_weather_time = self.rng.next_range(12000, 168000);
                    }
                }
            }
        }

        // Advance smooth visual and atmospheric fade
        self.tick();
    }

    /// Returns whether it is currently raining (smooth rain strength > 0.01)
    pub fn is_raining(&self) -> bool {
        self.rain_strength > 0.01
    }

    /// Returns whether it is currently thundering (smooth thunder strength > 0.01)
    pub fn is_thundering(&self) -> bool {
        self.thunder_strength > 0.01
    }

    /// Computes effective temperature at elevation `y` taking into account the lapse rate.
    /// REA Reference: net.minecraft.world.level.biome.Biome.getTemperature(BlockPos pos)
    #[inline]
    pub fn calculate_effective_temperature(base_temperature: f32, y: f32) -> f32 {
        if y > 64.0 {
            base_temperature - (y - 64.0) * TEMPERATURE_LAPSE_RATE
        } else {
            base_temperature
        }
    }

    /// Evaluates whether water at the specified position should freeze into ice.
    /// REA Reference: net.minecraft.world.level.biome.Biome.shouldFreeze (cuo.a)
    #[inline]
    pub fn should_freeze(
        base_temp: f32,
        y: f32,
        block: PrecipitationBlock,
        block_light: u8,
        sky_exposed: bool,
    ) -> bool {
        if !sky_exposed {
            return false;
        }
        if block != PrecipitationBlock::WaterSource {
            return false;
        }
        // Vanilla light threshold: block light must be < 11 (light >= 11 prevents freezing)
        if block_light >= 11 {
            return false;
        }
        let eff_temp = Self::calculate_effective_temperature(base_temp, y);
        eff_temp < SNOW_TEMPERATURE_THRESHOLD
    }

    /// Evaluates whether snow should accumulate onto the specified block.
    /// REA Reference: net.minecraft.world.level.biome.Biome.shouldSnow (cuo.b)
    #[inline]
    pub fn should_snow(
        base_temp: f32,
        y: f32,
        block_below: PrecipitationBlock,
        block_at: PrecipitationBlock,
        block_light: u8,
        sky_exposed: bool,
    ) -> bool {
        if !sky_exposed {
            return false;
        }
        if block_light >= 11 {
            return false;
        }
        let eff_temp = Self::calculate_effective_temperature(base_temp, y);
        if eff_temp >= SNOW_TEMPERATURE_THRESHOLD {
            return false;
        }

        match block_at {
            PrecipitationBlock::Air => block_below.can_support_snow(),
            PrecipitationBlock::SnowLayer(h) => h < 8,
            _ => false,
        }
    }

    /// Evaluates a random precipitation tick for a single block column in an active chunk.
    /// REA Reference: net.minecraft.server.level.ServerLevel.tickPrecipitation (and.a)
    pub fn tick_chunk_precipitation(
        &self,
        x: i32,
        z: i32,
        top_y: i32,
        top_block: PrecipitationBlock,
        above_block: PrecipitationBlock,
        base_temp: f32,
        downfall: f32,
        block_light: u8,
        sky_exposed: bool,
    ) -> Option<PrecipitationTickResult> {
        let y_f = top_y as f32;
        let eff_temp = Self::calculate_effective_temperature(base_temp, y_f);

        // Case 1: Active precipitation falling
        if self.is_raining() && downfall > 0.0 && sky_exposed {
            // Cold enough to snow & freeze
            if eff_temp < SNOW_TEMPERATURE_THRESHOLD {
                // Freezing exposed water source into Ice
                if Self::should_freeze(base_temp, y_f, top_block, block_light, sky_exposed) {
                    return Some(PrecipitationTickResult::FreezeWater { x, y: top_y, z });
                }

                // Snow accumulation: placing new layer or incrementing existing layers
                if Self::should_snow(base_temp, y_f + 1.0, top_block, above_block, block_light, sky_exposed) {
                    match above_block {
                        PrecipitationBlock::Air => {
                            return Some(PrecipitationTickResult::PlaceSnow {
                                x,
                                y: top_y + 1,
                                z,
                                height: 1,
                            });
                        }
                        PrecipitationBlock::SnowLayer(h) if h < 8 => {
                            return Some(PrecipitationTickResult::AccumulateSnow {
                                x,
                                y: top_y + 1,
                                z,
                                new_height: h + 1,
                            });
                        }
                        _ => {}
                    }
                }

                // If top block itself is already a snow layer, increment it up to 8
                if let PrecipitationBlock::SnowLayer(h) = top_block {
                    if h < 8 && block_light < 11 {
                        return Some(PrecipitationTickResult::AccumulateSnow {
                            x,
                            y: top_y,
                            z,
                            new_height: h + 1,
                        });
                    }
                }
            }
        }

        // Case 2: Thawing when clear weather returns or when exposed to artificial light level > 11
        // REA Reference: SnowLayerBlock.randomTick / IceBlock.melt
        if !self.is_raining() || eff_temp >= SNOW_TEMPERATURE_THRESHOLD || block_light > 11 {
            // Check snow layer above
            if let PrecipitationBlock::SnowLayer(h) = above_block {
                if block_light > 11 {
                    let next_h = h.saturating_sub(1);
                    return Some(PrecipitationTickResult::MeltSnow {
                        x,
                        y: top_y + 1,
                        z,
                        new_height: next_h,
                    });
                }
            }

            // Check top block itself
            match top_block {
                PrecipitationBlock::SnowLayer(h) => {
                    if block_light > 11 {
                        let next_h = h.saturating_sub(1);
                        return Some(PrecipitationTickResult::MeltSnow {
                            x,
                            y: top_y,
                            z,
                            new_height: next_h,
                        });
                    }
                }
                PrecipitationBlock::Ice => {
                    if block_light > 11 {
                        return Some(PrecipitationTickResult::MeltIce { x, y: top_y, z });
                    }
                }
                _ => {}
            }
        }

        None
    }

    /// Determines precipitation type (None, Rain, or Snow) based on temperature and height.
    /// REA Verified: if T < 0.15 precipitation falls as Snow; otherwise, Rain.
    pub fn get_precipitation_type(&self, base_temperature: f32, y: f32) -> PrecipitationType {
        if !self.is_raining() {
            return PrecipitationType::None;
        }

        let temp = Self::calculate_effective_temperature(base_temperature, y);
        if temp < SNOW_TEMPERATURE_THRESHOLD {
            PrecipitationType::Snow
        } else {
            PrecipitationType::Rain
        }
    }

    /// Determines precipitation type for a specific Biome at height `y`
    pub fn get_biome_precipitation(&self, biome: Biome, y: f32) -> PrecipitationType {
        // Desert, Savanna, and Badlands have 0 downfall in vanilla (no rain)
        if biome.downfall() <= 0.0 {
            return PrecipitationType::None;
        }
        self.get_precipitation_type(biome.temperature(), y)
    }

    /// REA Verified: During rain, interpolate sky light down by 25%.
    /// Returns multiplier in [0.75, 1.0].
    #[inline]
    pub fn get_sky_light_multiplier(&self) -> f32 {
        1.0 - 0.25 * self.rain_strength
    }

    /// REA Verified: Pulls fog distance closer during rain to simulate reduced visibility.
    /// Full rain reduces viewing distance by 35%.
    #[inline]
    pub fn get_fog_distance_multiplier(&self) -> f32 {
        1.0 - 0.35 * self.rain_strength
    }

    /// Applies rain light reduction to a base sky light value
    #[inline]
    pub fn apply_sky_light(&self, base_sky_light: f32) -> f32 {
        base_sky_light * self.get_sky_light_multiplier()
    }

    /// Applies rain fog occlusion to base fog planes (fog_start, fog_end)
    #[inline]
    pub fn apply_fog_distances(&self, fog_start: f32, fog_end: f32) -> (f32, f32) {
        let mult = self.get_fog_distance_multiplier();
        (fog_start * mult, fog_end * mult)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_state_transitions_and_fade() {
        let mut weather = WeatherManager::new();
        assert_eq!(weather.state, WeatherState::Clear);
        assert_eq!(weather.rain_strength, 0.0);

        // Transition Clear -> Rain
        weather.set_weather(WeatherState::Rain);

        // Tick 25 times: should be exactly halfway (0.50)
        for _ in 0..25 {
            weather.tick();
        }
        assert!((weather.rain_strength - 0.50).abs() < 1e-4);

        // Tick another 25 times: should reach full strength 1.0
        for _ in 0..25 {
            weather.tick();
        }
        assert!((weather.rain_strength - 1.0).abs() < 1e-4);
        assert!(weather.is_raining());
        assert!(!weather.is_thundering());

        // Transition Rain -> Thunder
        weather.set_weather(WeatherState::Thunder);
        for _ in 0..50 {
            weather.tick();
        }
        assert_eq!(weather.rain_strength, 1.0);
        assert!((weather.thunder_strength - 1.0).abs() < 1e-4);
        assert!(weather.is_thundering());

        // Transition Thunder -> Clear
        weather.set_weather(WeatherState::Clear);
        for _ in 0..50 {
            weather.tick();
        }
        assert_eq!(weather.rain_strength, 0.0);
        assert_eq!(weather.thunder_strength, 0.0);
        assert!(!weather.is_raining());
    }

    #[test]
    fn test_precipitation_temperature_and_elevation_threshold() {
        let mut weather = WeatherManager::new();
        weather.set_weather(WeatherState::Rain);
        for _ in 0..50 {
            weather.tick();
        }

        // 1. Plains at sea level (T = 0.8, y = 64): Rain
        assert_eq!(
            weather.get_precipitation_type(0.8, 64.0),
            PrecipitationType::Rain
        );

        // 2. Cold Taiga at sea level (T = 0.10 < 0.15, y = 64): Snow
        assert_eq!(
            weather.get_precipitation_type(0.10, 64.0),
            PrecipitationType::Snow
        );

        // 3. High elevation lapse rate:
        // Mountain peak with base T = 0.25 at y = 184 (120 blocks above 64):
        // lapse = 120 * 0.00166667 = 0.20 -> effective T = 0.05 < 0.15 -> Snow!
        assert_eq!(
            weather.get_precipitation_type(0.25, 184.0),
            PrecipitationType::Snow
        );

        // But at foot of the mountain (y = 64), T = 0.25 >= 0.15 -> Rain!
        assert_eq!(
            weather.get_precipitation_type(0.25, 64.0),
            PrecipitationType::Rain
        );
    }

    #[test]
    fn test_sky_light_and_fog_reduction_during_rain() {
        let mut weather = WeatherManager::new();
        assert_eq!(weather.get_sky_light_multiplier(), 1.0);
        assert_eq!(weather.get_fog_distance_multiplier(), 1.0);

        weather.set_weather(WeatherState::Rain);
        for _ in 0..50 {
            weather.tick();
        }
        assert!((weather.get_sky_light_multiplier() - 0.75).abs() < 1e-4);

        let light = weather.apply_sky_light(15.0);
        assert!((light - 11.25).abs() < 1e-4);

        let (fog_start, fog_end) = weather.apply_fog_distances(20.0, 100.0);
        assert!((fog_end - 65.0).abs() < 1e-4);
        assert!((fog_start - 13.0).abs() < 1e-4);
    }

    #[test]
    fn test_global_weather_cycle_timers_and_transitions() {
        let mut weather = WeatherManager::new();
        weather.clear_weather_time = 3; // 3 ticks of clear weather remaining

        assert_eq!(weather.state, WeatherState::Clear);
        weather.tick_weather_cycle();
        assert_eq!(weather.clear_weather_time, 2);

        weather.tick_weather_cycle();
        assert_eq!(weather.clear_weather_time, 1);

        // Clear weather timer expires on this tick: transitions to Rain or Thunder
        weather.tick_weather_cycle();
        assert!(weather.state == WeatherState::Rain || weather.state == WeatherState::Thunder);
        assert!(weather.rain_time >= 12000 && weather.rain_time <= 168000);

        // Smooth transition begins
        assert!(weather.rain_strength > 0.0);

        // Advance 50 ticks to reach full strength
        for _ in 0..49 {
            weather.tick_weather_cycle();
        }
        assert!((weather.rain_strength - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_chunk_precipitation_water_freezing_and_snow_accumulation() {
        let mut weather = WeatherManager::new();
        weather.set_weather(WeatherState::Rain);
        for _ in 0..50 {
            weather.tick();
        }
        assert!(weather.is_raining());

        // 1. Cold biome (T = 0.05 < 0.15), water source exposed to sky with low block light (< 11)
        let freeze_result = weather.tick_chunk_precipitation(
            10,
            20,
            63,
            PrecipitationBlock::WaterSource,
            PrecipitationBlock::Air,
            0.05,
            0.8,
            0,
            true,
        );
        assert_eq!(
            freeze_result,
            Some(PrecipitationTickResult::FreezeWater { x: 10, y: 63, z: 20 })
        );

        // 2. Torch light >= 11 prevents freezing
        let torch_prevented = weather.tick_chunk_precipitation(
            10,
            20,
            63,
            PrecipitationBlock::WaterSource,
            PrecipitationBlock::Air,
            0.05,
            0.8,
            12,
            true,
        );
        assert_eq!(torch_prevented, None);

        // 3. Snow accumulation: solid surface exposed to sky with cold temperature
        let snow_place = weather.tick_chunk_precipitation(
            5,
            15,
            70,
            PrecipitationBlock::SolidSurface,
            PrecipitationBlock::Air,
            0.05,
            0.8,
            0,
            true,
        );
        assert_eq!(
            snow_place,
            Some(PrecipitationTickResult::PlaceSnow {
                x: 5,
                y: 71,
                z: 15,
                height: 1
            })
        );

        // 4. Continued precipitation: increment snow layer height (1 -> 2)
        let snow_accumulate = weather.tick_chunk_precipitation(
            5,
            15,
            70,
            PrecipitationBlock::SolidSurface,
            PrecipitationBlock::SnowLayer(1),
            0.05,
            0.8,
            0,
            true,
        );
        assert_eq!(
            snow_accumulate,
            Some(PrecipitationTickResult::AccumulateSnow {
                x: 5,
                y: 71,
                z: 15,
                new_height: 2
            })
        );
    }

    #[test]
    fn test_snow_and_ice_thawing_under_light() {
        let mut weather = WeatherManager::new();
        weather.set_weather(WeatherState::Clear);
        for _ in 0..50 {
            weather.tick();
        }
        assert!(!weather.is_raining());

        // Snow layer exposed to torch light level 14 melts down (height 2 -> 1)
        let melt_snow = weather.tick_chunk_precipitation(
            10,
            20,
            70,
            PrecipitationBlock::SolidSurface,
            PrecipitationBlock::SnowLayer(2),
            0.05,
            0.8,
            14,
            true,
        );
        assert_eq!(
            melt_snow,
            Some(PrecipitationTickResult::MeltSnow {
                x: 10,
                y: 71,
                z: 20,
                new_height: 1
            })
        );

        // Ice exposed to torch light level 14 melts to water source
        let melt_ice = weather.tick_chunk_precipitation(
            10,
            20,
            63,
            PrecipitationBlock::Ice,
            PrecipitationBlock::Air,
            0.05,
            0.8,
            14,
            true,
        );
        assert_eq!(
            melt_ice,
            Some(PrecipitationTickResult::MeltIce { x: 10, y: 63, z: 20 })
        );
    }
}
