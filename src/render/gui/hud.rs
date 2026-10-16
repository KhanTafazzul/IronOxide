// REA Verified: net.minecraft.client.gui.Gui (mapped: ewt.class)
//
// 2D HUD Vitals Rendering:
// - 10 Heart icons (health) and 10 Hunger Drumstick icons (food level) above hotbar.
// - Full, half, and empty icon rendering matching vanilla health/hunger levels.
// - Dynamic shake animations:
//   * Damage shake: high-frequency bounce during hurt_time.
//   * Low health shake: continuous heartbeat vibration when health <= 4.0 (2 hearts).
//   * Low hunger rumble: rumble wobble when food_level <= 6 and saturation <= 0.0.

use crate::game::stats::PlayerStats;
use crate::render::gui::{GuiAtlas, GuiRect, GuiVertex};

pub struct HudRenderer;

impl HudRenderer {
    /// Add a 2D quad to vertex and index list
    pub fn add_quad(
        vertices: &mut Vec<GuiVertex>,
        indices: &mut Vec<u32>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rect: GuiRect,
        color: [f32; 4],
    ) {
        let base_idx = vertices.len() as u32;

        vertices.push(GuiVertex::new([x, y], [rect.u_min, rect.v_min], color));
        vertices.push(GuiVertex::new([x + w, y], [rect.u_max, rect.v_min], color));
        vertices.push(GuiVertex::new([x + w, y + h], [rect.u_max, rect.v_max], color));
        vertices.push(GuiVertex::new([x, y + h], [rect.u_min, rect.v_max], color));

        indices.extend_from_slice(&[
            base_idx,
            base_idx + 1,
            base_idx + 2,
            base_idx,
            base_idx + 2,
            base_idx + 3,
        ]);
    }

    /// Renders hearts and hunger drumsticks above the hotbar
    pub fn build_vitals_quads(
        stats: &PlayerStats,
        atlas: &GuiAtlas,
        hotbar_x: f32,
        hotbar_y: f32,
        hotbar_w: f32,
        scale: f32,
        vertices: &mut Vec<GuiVertex>,
        indices: &mut Vec<u32>,
    ) {
        let icon_size = 9.0 * scale;
        let icon_spacing = 8.0 * scale;
        // Position vitals 10px (scaled) above the top of the hotbar
        let vitals_y = hotbar_y - (10.0 * scale);
        let color = [1.0, 1.0, 1.0, 1.0];

        // 1. Render 10 Hearts (Health: 0..20)
        let is_taking_damage = stats.hurt_time > 0.0;
        let is_low_health = stats.health <= 4.0;

        for i in 0..10 {
            let x = hotbar_x + (i as f32 * icon_spacing);

            // Shaking animations
            let y_shake = if is_taking_damage {
                ((stats.hurt_time * 30.0 + i as f32 * 2.5).sin() * 3.0 * scale).round()
            } else if is_low_health {
                ((stats.time * 16.0 + i as f32 * 1.5).sin() * 1.5 * scale).round()
            } else {
                0.0
            };
            let y = vitals_y + y_shake;

            // Background empty heart container
            Self::add_quad(vertices, indices, x, y, icon_size, icon_size, atlas.heart_container_uv, color);

            // Foreground full / half heart
            let hp_for_slot = (stats.health.ceil() as i32) - (i * 2);
            if hp_for_slot >= 2 {
                Self::add_quad(vertices, indices, x, y, icon_size, icon_size, atlas.heart_full_uv, color);
            } else if hp_for_slot == 1 {
                Self::add_quad(vertices, indices, x, y, icon_size, icon_size, atlas.heart_half_uv, color);
            }
        }

        // 2. Render 10 Hunger Drumsticks (Food Level: 0..20)
        let is_low_hunger = stats.food_level <= 6 && stats.saturation <= 0.0;

        for i in 0..10 {
            // Anchor hunger icons on right side of hotbar, advancing right-to-left
            let x = hotbar_x + hotbar_w - icon_size - (i as f32 * icon_spacing);

            let y_shake = if is_low_hunger {
                ((stats.time * 12.0 + i as f32 * 1.2).sin() * 1.5 * scale).round()
            } else {
                0.0
            };
            let y = vitals_y + y_shake;

            // Background empty food container
            Self::add_quad(vertices, indices, x, y, icon_size, icon_size, atlas.food_empty_uv, color);

            // Foreground full / half drumstick
            let food_for_slot = (stats.food_level as i32) - (i * 2);
            if food_for_slot >= 2 {
                Self::add_quad(vertices, indices, x, y, icon_size, icon_size, atlas.food_full_uv, color);
            } else if food_for_slot == 1 {
                Self::add_quad(vertices, indices, x, y, icon_size, icon_size, atlas.food_half_uv, color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vitals_quads_full_health_and_food() {
        let stats = PlayerStats::new(); // health 20, food 20
        let atlas = GuiAtlas::load_or_create(None);

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        HudRenderer::build_vitals_quads(
            &stats,
            &atlas,
            100.0,
            600.0,
            182.0,
            1.0,
            &mut vertices,
            &mut indices,
        );

        // 10 containers + 10 full hearts + 10 empty food + 10 full food = 40 quads
        // 40 quads * 4 vertices = 160 vertices
        assert_eq!(vertices.len(), 160);
        assert_eq!(indices.len(), 240);
    }

    #[test]
    fn test_vitals_quads_half_health_and_food() {
        let mut stats = PlayerStats::new();
        stats.health = 1.0; // 0 full hearts, 1 half heart
        stats.food_level = 3; // 1 full drumstick, 1 half drumstick
        let atlas = GuiAtlas::load_or_create(None);

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        HudRenderer::build_vitals_quads(
            &stats,
            &atlas,
            100.0,
            600.0,
            182.0,
            1.0,
            &mut vertices,
            &mut indices,
        );

        // 10 heart containers + 1 half heart = 11 heart quads
        // 10 food containers + 1 full + 1 half food = 12 food quads
        // Total = 23 quads = 92 vertices
        assert_eq!(vertices.len(), 23 * 4);
    }

    #[test]
    fn test_damage_shake_applies_offset() {
        let mut stats = PlayerStats::new();
        stats.hurt_time = 0.4;
        let atlas = GuiAtlas::load_or_create(None);

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        HudRenderer::build_vitals_quads(
            &stats,
            &atlas,
            100.0,
            600.0,
            182.0,
            1.0,
            &mut vertices,
            &mut indices,
        );

        // Base vitals_y is 600 - 10 = 590.
        // With hurt_time > 0, at least one heart vertex Y should differ from exactly 590.0
        let base_y = 590.0;
        let any_shaken = vertices.iter().any(|v| (v.pos[1] - base_y).abs() > 0.1 && (v.pos[1] - (base_y + 9.0)).abs() > 0.1);
        assert!(any_shaken);
    }
}
