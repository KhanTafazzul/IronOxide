//! Canyon and Ravine carver generating steep, narrow surface gorges.
//! REA Reference: net.minecraft.world.level.levelgen.carver.CanyonWorldCarver (dqn.class)
//! REA Reference: data/minecraft/worldgen/configured_carver/canyon.json

use super::config::CanyonCarverConfiguration;
use crate::structures::LegacyRandom;

/// Canyon step node along a canyon's curving path.
#[derive(Debug, Clone)]
pub struct CanyonNode {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub width: f64,
    pub height: f64,
}

/// Canyon / Ravine procedural carver.
#[derive(Clone, Debug)]
pub struct CanyonWorldCarver {
    pub config: CanyonCarverConfiguration,
}

impl CanyonWorldCarver {
    pub fn new(config: CanyonCarverConfiguration) -> Self {
        Self { config }
    }

    /// Determines if a canyon should spawn in chunk (chunk_x, chunk_z).
    pub fn is_canyon_chunk(&self, chunk_x: i32, chunk_z: i32, world_seed: i64) -> bool {
        let mut rng = LegacyRandom::from_chunk_seed(world_seed, chunk_x, chunk_z, 10387319);
        rng.next_float() < self.config.base.probability
    }

    /// Generates canyon trajectory nodes originating in chunk (chunk_x, chunk_z).
    pub fn generate_canyon_path(&self, chunk_x: i32, chunk_z: i32, world_seed: i64) -> Vec<CanyonNode> {
        let mut rng = LegacyRandom::from_chunk_seed(world_seed, chunk_x, chunk_z, 10387319);
        let mut nodes = Vec::new();

        let start_x = (chunk_x * 16 + rng.next_int(16)) as f64;
        let start_z = (chunk_z * 16 + rng.next_int(16)) as f64;
        let start_y = (rng.next_int((self.config.base.max_y - self.config.base.min_y) as u32) as i32
            + self.config.base.min_y) as f64;

        let total_steps = 112; // Vanilla standard canyon length
        let mut current_x = start_x;
        let mut current_y = start_y;
        let mut current_z = start_z;

        let mut yaw = rng.next_float() * std::f32::consts::TAU;
        let mut pitch = (rng.next_float() - 0.5) * 0.25;

        let base_thickness = 1.5 + (rng.next_float() as f64) * 3.0;

        for step in 0..total_steps {
            let progress = (step as f64) / (total_steps as f64);
            // Parabolic tapering at canyon ends
            let factor = (std::f64::consts::PI * progress).sin();
            let width = base_thickness * factor * (self.config.distance_factor_min as f64);
            let height = width * (self.config.y_scale as f64);

            nodes.push(CanyonNode {
                x: current_x,
                y: current_y,
                z: current_z,
                width,
                height,
            });

            // Advance step along pitch and yaw
            let step_dist = 1.0;
            current_x += (yaw as f64).cos() * (pitch as f64).cos() * step_dist;
            current_y += (pitch as f64).sin() * step_dist;
            current_z += (yaw as f64).sin() * (pitch as f64).cos() * step_dist;

            // Slight organic curvature drift
            yaw += (rng.next_float() - 0.5) * 0.1;
            pitch = (pitch + (rng.next_float() - 0.5) * 0.05).clamp(-0.125, 0.125);
        }

        nodes
    }

    /// Tests if a block coordinate (x, y, z) falls within the carved volume of the canyon path.
    pub fn is_carved_by_canyon(&self, x: i32, y: i32, z: i32, nodes: &[CanyonNode]) -> bool {
        let xf = x as f64 + 0.5;
        let yf = y as f64 + 0.5;
        let zf = z as f64 + 0.5;

        for node in nodes {
            let dx = (xf - node.x) / node.width.max(0.1);
            let dy = (yf - node.y) / node.height.max(0.1);
            let dz = (zf - node.z) / node.width.max(0.1);

            // Ellipsoid intersection
            if (dx * dx + dy * dy + dz * dz) < 1.0 {
                return true;
            }
        }

        false
    }
}
