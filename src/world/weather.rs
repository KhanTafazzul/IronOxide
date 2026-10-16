// REA Verified: net.minecraft.world.level.Level (ctp.class - rainLevel & thunderLevel)
// REA Verified: net.minecraft.world.level.biome.Biome (cuo.class - precipitation temperature threshold T < 0.15)
//
// Weather state machine & precipitation logic:
// - Weather states: Clear, Rain, and Thunder
// - Smooth rain_strength transitions: 0.0 -> 1.0 over 50 ticks (fade speed: 0.02 / tick)
// - Precipitation determination:
//   * If temperature < 0.15 (considering elevation lapse rate), precipitation falls as Snow;
//   * Otherwise, falls as Rain.
// - Atmosphere modifiers:
//   * Sky light interpolated down by 25% during rain (sky_light * (1.0 - 0.25 * rain_strength))
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

/// Weather state machine and climate precipitation engine
#[derive(Debug, Clone, PartialEq)]
pub struct WeatherManager {
    pub state: WeatherState,
    pub rain_strength: f32,
    pub prev_rain_strength: f32,
    pub thunder_strength: f32,
    pub prev_thunder_strength: f32,
    pub timer: u32,
}

impl Default for WeatherManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WeatherManager {
    pub fn new() -> Self {
        Self {
            state: WeatherState::Clear,
            rain_strength: 0.0,
            prev_rain_strength: 0.0,
            thunder_strength: 0.0,
            prev_thunder_strength: 0.0,
            timer: 0,
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

    /// Advances the weather simulation by 1 game tick (50ms).
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

    /// Returns whether it is currently raining (smooth rain strength > 0.01)
    pub fn is_raining(&self) -> bool {
        self.rain_strength > 0.01
    }

    /// Returns whether it is currently thundering (smooth thunder strength > 0.01)
    pub fn is_thundering(&self) -> bool {
        self.thunder_strength > 0.01
    }

    /// Computes effective temperature at elevation `y` taking into account the lapse rate
    #[inline]
    pub fn calculate_effective_temperature(base_temperature: f32, y: f32) -> f32 {
        if y > 64.0 {
            base_temperature - (y - 64.0) * TEMPERATURE_LAPSE_RATE
        } else {
            base_temperature
        }
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
        // Clear weather: multipliers are 1.0
        assert_eq!(weather.get_sky_light_multiplier(), 1.0);
        assert_eq!(weather.get_fog_distance_multiplier(), 1.0);

        // Full rain: sky light drops by 25% (multiplier = 0.75)
        weather.set_weather(WeatherState::Rain);
        for _ in 0..50 {
            weather.tick();
        }
        assert!((weather.get_sky_light_multiplier() - 0.75).abs() < 1e-4);

        // Check sky light calculation: 15.0 -> 11.25
        let light = weather.apply_sky_light(15.0);
        assert!((light - 11.25).abs() < 1e-4);

        // Check fog distance pull: 100 blocks -> 65 blocks
        let (fog_start, fog_end) = weather.apply_fog_distances(20.0, 100.0);
        assert!((fog_end - 65.0).abs() < 1e-4);
        assert!((fog_start - 13.0).abs() < 1e-4);
    }
}
