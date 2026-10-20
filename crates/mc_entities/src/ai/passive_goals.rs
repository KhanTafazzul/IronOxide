// REA Verified: net.minecraft.world.entity.ai.goal.FloatGoal (mapped: btb.class)
// REA Verified: net.minecraft.world.entity.ai.goal.PanicGoal (mapped: bua.class)
// REA Verified: net.minecraft.world.entity.ai.goal.BreedGoal (mapped: bst.class)
// REA Verified: net.minecraft.world.entity.ai.goal.TemptGoal (mapped: bup.class)
// REA Verified: net.minecraft.world.entity.ai.goal.RandomStrollGoal (mapped: bue.class)
// REA Verified: net.minecraft.world.entity.ai.goal.LookAtPlayerGoal (mapped: btp.class)
//
// Common Passive Mob Goals.

use glam::Vec3;
use super::goal::{Goal, GoalContext, GoalFlags};
use crate::components::EntityId;

/// FloatGoal: Mob jumps/swims upward when submerged in water or lava to prevent drowning.
pub struct FloatGoal;

impl FloatGoal {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FloatGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl Goal for FloatGoal {
    fn can_use(&mut self, ctx: &GoalContext) -> bool {
        ctx.is_in_water || ctx.is_in_lava
    }

    fn tick(&mut self, ctx: &mut GoalContext) {
        ctx.jump_requested = true;
        if ctx.is_in_water || ctx.is_in_lava {
            ctx.requested_velocity = Some(Vec3::new(
                ctx.velocity.value.x,
                0.3,
                ctx.velocity.value.z,
            ));
        }
    }

    fn flags(&self) -> GoalFlags {
        GoalFlags::JUMP
    }

    fn name(&self) -> &'static str {
        "FloatGoal"
    }
}

/// PanicGoal: Mob runs rapidly away in a random safe direction when hurt or burning.
pub struct PanicGoal {
    pub speed_modifier: f32,
    pub target_pos: Option<Vec3>,
    pub panic_ticks_left: u32,
    pub is_running: bool,
}

impl PanicGoal {
    pub fn new(speed_modifier: f32) -> Self {
        Self {
            speed_modifier,
            target_pos: None,
            panic_ticks_left: 0,
            is_running: false,
        }
    }

    pub fn standard() -> Self {
        Self::new(1.25)
    }
}

impl Default for PanicGoal {
    fn default() -> Self {
        Self::standard()
    }
}

impl Goal for PanicGoal {
    fn can_use(&mut self, ctx: &GoalContext) -> bool {
        let was_hurt = ctx.health.current < ctx.last_health;
        was_hurt || ctx.is_on_fire
    }

    fn can_continue_to_use(&mut self, ctx: &GoalContext) -> bool {
        if self.panic_ticks_left == 0 {
            return false;
        }
        if let Some(target) = self.target_pos {
            ctx.transform.position.distance(target) > 1.0
        } else {
            false
        }
    }

    fn start(&mut self, ctx: &mut GoalContext) {
        self.panic_ticks_left = 60; // 3 seconds panic duration
        self.is_running = true;
        ctx.panic_active = true;

        // Choose panic destination: forward direction + randomized offset
        let fwd = ctx.transform.forward();
        let target = ctx.transform.position + fwd * 8.0 + Vec3::new(1.5, 0.0, 1.5);
        self.target_pos = Some(target);
    }

    fn stop(&mut self, ctx: &mut GoalContext) {
        self.is_running = false;
        self.target_pos = None;
        self.panic_ticks_left = 0;
        ctx.panic_active = false;
    }

    fn tick(&mut self, ctx: &mut GoalContext) {
        if self.panic_ticks_left > 0 {
            self.panic_ticks_left -= 1;
        }

        if let Some(target) = self.target_pos {
            let dir = (target - ctx.transform.position).normalize_or_zero();
            let speed = 0.25 * self.speed_modifier;
            ctx.requested_velocity = Some(Vec3::new(dir.x * speed, ctx.velocity.value.y, dir.z * speed));
            ctx.look_target = Some(target);
        }
    }

    fn flags(&self) -> GoalFlags {
        GoalFlags::MOVE
    }

    fn name(&self) -> &'static str {
        "PanicGoal"
    }
}

/// BreedGoal: Two animals of the same species in love move toward each other and spawn an offspring.
pub struct BreedGoal {
    pub speed_modifier: f32,
    pub partner_id: Option<EntityId>,
    pub breed_ticks: u32,
}

impl BreedGoal {
    pub const BREED_REACH_DISTANCE: f32 = 2.0;

    pub fn new(speed_modifier: f32) -> Self {
        Self {
            speed_modifier,
            partner_id: None,
            breed_ticks: 0,
        }
    }

    pub fn standard() -> Self {
        Self::new(1.0)
    }
}

impl Default for BreedGoal {
    fn default() -> Self {
        Self::standard()
    }
}

impl Goal for BreedGoal {
    fn can_use(&mut self, ctx: &GoalContext) -> bool {
        if ctx.in_love_ticks == 0 {
            return false;
        }

        // Find nearby partner of same species also in love
        for entity in &ctx.nearby_entities {
            if entity.id != ctx.entity_id
                && entity.entity_type == ctx.entity_type
                && entity.is_alive
                && entity.in_love_ticks > 0
            {
                let dist = ctx.transform.position.distance(entity.position);
                if dist <= 12.0 {
                    self.partner_id = Some(entity.id);
                    return true;
                }
            }
        }
        false
    }

    fn can_continue_to_use(&mut self, ctx: &GoalContext) -> bool {
        if ctx.in_love_ticks == 0 {
            return false;
        }
        if let Some(partner_id) = self.partner_id {
            ctx.nearby_entities
                .iter()
                .any(|e| e.id == partner_id && e.is_alive && e.in_love_ticks > 0)
        } else {
            false
        }
    }

    fn start(&mut self, _ctx: &mut GoalContext) {
        self.breed_ticks = 0;
    }

    fn stop(&mut self, _ctx: &mut GoalContext) {
        self.partner_id = None;
        self.breed_ticks = 0;
    }

    fn tick(&mut self, ctx: &mut GoalContext) {
        let Some(partner_id) = self.partner_id else { return };
        let Some(partner) = ctx.nearby_entities.iter().find(|e| e.id == partner_id) else { return };

        let partner_pos = partner.position;
        let dist = ctx.transform.position.distance(partner_pos);

        // Turn towards partner
        ctx.look_target = Some(partner_pos);

        if dist > Self::BREED_REACH_DISTANCE {
            let dir = (partner_pos - ctx.transform.position).normalize_or_zero();
            let speed = 0.2 * self.speed_modifier;
            ctx.requested_velocity = Some(Vec3::new(dir.x * speed, ctx.velocity.value.y, dir.z * speed));
        } else {
            // Close enough to breed!
            self.breed_ticks += 1;
            ctx.requested_velocity = Some(Vec3::new(0.0, ctx.velocity.value.y, 0.0));

            if self.breed_ticks >= 20 { // 1 second of loving contact
                ctx.spawn_child_event = true;
                ctx.in_love_ticks = 0;
                self.partner_id = None;
            }
        }
    }

    fn flags(&self) -> GoalFlags {
        GoalFlags::MOVE | GoalFlags::LOOK
    }

    fn name(&self) -> &'static str {
        "BreedGoal"
    }
}

/// TemptGoal: Animal follows a player holding food/seeds that tempt it.
pub struct TemptGoal {
    pub speed_modifier: f32,
    pub tempt_distance: f32,
    pub stop_distance: f32,
    pub target_player_id: Option<EntityId>,
}

impl TemptGoal {
    pub fn new(speed_modifier: f32, tempt_distance: f32, stop_distance: f32) -> Self {
        Self {
            speed_modifier,
            tempt_distance,
            stop_distance,
            target_player_id: None,
        }
    }

    pub fn standard() -> Self {
        Self::new(1.2, 10.0, 2.5)
    }
}

impl Default for TemptGoal {
    fn default() -> Self {
        Self::standard()
    }
}

impl Goal for TemptGoal {
    fn can_use(&mut self, ctx: &GoalContext) -> bool {
        for player in &ctx.nearby_players {
            if player.is_creative_or_spectator {
                continue;
            }
            if let Some(held) = player.held_item {
                if ctx.entity_type.is_tempting_item(held) {
                    let dist = ctx.transform.position.distance(player.position);
                    if dist <= self.tempt_distance {
                        self.target_player_id = Some(player.id);
                        return true;
                    }
                }
            }
        }
        false
    }

    fn can_continue_to_use(&mut self, ctx: &GoalContext) -> bool {
        let Some(player_id) = self.target_player_id else { return false };
        if let Some(player) = ctx.nearby_players.iter().find(|p| p.id == player_id) {
            if player.is_creative_or_spectator {
                return false;
            }
            if let Some(held) = player.held_item {
                let dist = ctx.transform.position.distance(player.position);
                return dist <= self.tempt_distance && ctx.entity_type.is_tempting_item(held);
            }
        }
        false
    }

    fn stop(&mut self, _ctx: &mut GoalContext) {
        self.target_player_id = None;
    }

    fn tick(&mut self, ctx: &mut GoalContext) {
        let Some(player_id) = self.target_player_id else { return };
        let Some(player) = ctx.nearby_players.iter().find(|p| p.id == player_id) else { return };

        // Look at player's eyes
        ctx.look_target = Some(player.eye_position);

        let dist = ctx.transform.position.distance(player.position);
        if dist > self.stop_distance {
            // Move toward player
            let dir = (player.position - ctx.transform.position).normalize_or_zero();
            let speed = 0.2 * self.speed_modifier;
            ctx.requested_velocity = Some(Vec3::new(dir.x * speed, ctx.velocity.value.y, dir.z * speed));
        } else {
            // Within reach: hold position and gaze
            ctx.requested_velocity = Some(Vec3::new(0.0, ctx.velocity.value.y, 0.0));
        }
    }

    fn flags(&self) -> GoalFlags {
        GoalFlags::MOVE | GoalFlags::LOOK
    }

    fn name(&self) -> &'static str {
        "TemptGoal"
    }
}

/// RandomStrollGoal: Mob idly wanders around every interval ticks when not engaged in other goals.
pub struct RandomStrollGoal {
    pub speed_modifier: f32,
    pub interval: u32,
    pub timer: u32,
    pub target_pos: Option<Vec3>,
    pub wander_ticks_left: u32,
}

impl RandomStrollGoal {
    pub fn new(speed_modifier: f32, interval: u32) -> Self {
        Self {
            speed_modifier,
            interval,
            timer: 0,
            target_pos: None,
            wander_ticks_left: 0,
        }
    }

    pub fn standard() -> Self {
        Self::new(0.8, 80)
    }
}

impl Default for RandomStrollGoal {
    fn default() -> Self {
        Self::standard()
    }
}

impl Goal for RandomStrollGoal {
    fn can_use(&mut self, ctx: &GoalContext) -> bool {
        self.timer += 1;
        if self.timer >= self.interval {
            self.timer = 0;
            // Pick destination 6 blocks in forward/side direction
            let fwd = ctx.transform.forward();
            let target = ctx.transform.position + fwd * 6.0;
            self.target_pos = Some(target);
            return true;
        }
        false
    }

    fn can_continue_to_use(&mut self, ctx: &GoalContext) -> bool {
        if self.wander_ticks_left == 0 {
            return false;
        }
        if let Some(target) = self.target_pos {
            ctx.transform.position.distance(target) > 0.8
        } else {
            false
        }
    }

    fn start(&mut self, _ctx: &mut GoalContext) {
        self.wander_ticks_left = 60; // 3 seconds max stroll time
    }

    fn stop(&mut self, _ctx: &mut GoalContext) {
        self.target_pos = None;
        self.wander_ticks_left = 0;
    }

    fn tick(&mut self, ctx: &mut GoalContext) {
        if self.wander_ticks_left > 0 {
            self.wander_ticks_left -= 1;
        }

        if let Some(target) = self.target_pos {
            let dir = (target - ctx.transform.position).normalize_or_zero();
            let speed = 0.15 * self.speed_modifier;
            ctx.requested_velocity = Some(Vec3::new(dir.x * speed, ctx.velocity.value.y, dir.z * speed));
            ctx.look_target = Some(target);
        }
    }

    fn flags(&self) -> GoalFlags {
        GoalFlags::MOVE
    }

    fn name(&self) -> &'static str {
        "RandomStrollGoal"
    }
}

/// LookAtPlayerGoal: Mob turns its head/body to observe a player when close by.
pub struct LookAtPlayerGoal {
    pub look_distance: f32,
    pub look_time_left: u32,
    pub target_player_id: Option<EntityId>,
}

impl LookAtPlayerGoal {
    pub fn new(look_distance: f32) -> Self {
        Self {
            look_distance,
            look_time_left: 0,
            target_player_id: None,
        }
    }

    pub fn standard() -> Self {
        Self::new(8.0)
    }
}

impl Default for LookAtPlayerGoal {
    fn default() -> Self {
        Self::standard()
    }
}

impl Goal for LookAtPlayerGoal {
    fn can_use(&mut self, ctx: &GoalContext) -> bool {
        for player in &ctx.nearby_players {
            let dist = ctx.transform.position.distance(player.position);
            if dist <= self.look_distance {
                self.target_player_id = Some(player.id);
                return true;
            }
        }
        false
    }

    fn can_continue_to_use(&mut self, ctx: &GoalContext) -> bool {
        if self.look_time_left == 0 {
            return false;
        }
        if let Some(pid) = self.target_player_id {
            ctx.nearby_players
                .iter()
                .any(|p| p.id == pid && ctx.transform.position.distance(p.position) <= self.look_distance)
        } else {
            false
        }
    }

    fn start(&mut self, _ctx: &mut GoalContext) {
        self.look_time_left = 60; // 3 seconds gaze
    }

    fn stop(&mut self, _ctx: &mut GoalContext) {
        self.target_player_id = None;
        self.look_time_left = 0;
    }

    fn tick(&mut self, ctx: &mut GoalContext) {
        if self.look_time_left > 0 {
            self.look_time_left -= 1;
        }

        let Some(pid) = self.target_player_id else { return };
        if let Some(player) = ctx.nearby_players.iter().find(|p| p.id == pid) {
            ctx.look_target = Some(player.eye_position);
            let (yaw, pitch) = ctx.transform.look_at_angles(player.eye_position);
            ctx.transform.rotation.x = yaw;
            ctx.transform.rotation.y = pitch;
        }
    }

    fn flags(&self) -> GoalFlags {
        GoalFlags::LOOK
    }

    fn name(&self) -> &'static str {
        "LookAtPlayerGoal"
    }
}
