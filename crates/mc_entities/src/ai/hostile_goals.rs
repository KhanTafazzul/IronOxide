// REA Verified: net.minecraft.world.entity.ai.goal.target.NearestAttackableTargetGoal (mapped: bva.class)
// REA Verified: net.minecraft.world.entity.ai.goal.MeleeAttackGoal (mapped: btr.class)
// REA Verified: net.minecraft.world.entity.ai.goal.RangedAttackGoal (mapped: bug.class)
//
// Hostile Mob AI Goals (Target Acquisition, Melee Strike, Ranged Combat).

use glam::Vec3;
use super::goal::{AttackEvent, Goal, GoalContext, GoalFlags, TargetEntity};
use crate::components::EntityId;

/// NearestAttackableTargetGoal: Target selector goal that scans for nearest vulnerable player/entity.
pub struct NearestAttackableTargetGoal {
    pub detection_range: f32,
    pub max_tracking_range: f32,
    pub target_id: Option<EntityId>,
}

impl NearestAttackableTargetGoal {
    pub fn new(detection_range: f32, max_tracking_range: f32) -> Self {
        Self {
            detection_range,
            max_tracking_range,
            target_id: None,
        }
    }

    pub fn standard() -> Self {
        Self::new(16.0, 24.0)
    }
}

impl Default for NearestAttackableTargetGoal {
    fn default() -> Self {
        Self::standard()
    }
}

impl Goal for NearestAttackableTargetGoal {
    fn can_use(&mut self, ctx: &GoalContext) -> bool {
        // Find nearest living non-spectator player within detection range
        let mut closest_dist = self.detection_range;
        let mut chosen: Option<TargetEntity> = None;

        for player in &ctx.nearby_players {
            if player.is_creative_or_spectator || !player.has_line_of_sight {
                continue;
            }
            let dist = ctx.transform.position.distance(player.position);
            if dist < closest_dist {
                closest_dist = dist;
                chosen = Some(TargetEntity {
                    id: player.id,
                    entity_type: crate::components::EntityType::Player,
                    position: player.position,
                    health: 20.0,
                    is_alive: true,
                    has_line_of_sight: true,
                });
            }
        }

        if let Some(target) = chosen {
            self.target_id = Some(target.id);
            true
        } else {
            false
        }
    }

    fn can_continue_to_use(&mut self, ctx: &GoalContext) -> bool {
        let Some(tid) = self.target_id else { return false };
        if let Some(target) = ctx.target {
            if target.id == tid && target.is_alive {
                let dist = ctx.transform.position.distance(target.position);
                return dist <= self.max_tracking_range;
            }
        }
        false
    }

    fn start(&mut self, ctx: &mut GoalContext) {
        if let Some(tid) = self.target_id {
            if let Some(player) = ctx.nearby_players.iter().find(|p| p.id == tid) {
                ctx.target = Some(TargetEntity {
                    id: player.id,
                    entity_type: crate::components::EntityType::Player,
                    position: player.position,
                    health: 20.0,
                    is_alive: true,
                    has_line_of_sight: player.has_line_of_sight,
                });
            }
        }
    }

    fn stop(&mut self, ctx: &mut GoalContext) {
        self.target_id = None;
        ctx.target = None;
    }

    fn flags(&self) -> GoalFlags {
        GoalFlags::TARGET
    }

    fn name(&self) -> &'static str {
        "NearestAttackableTargetGoal"
    }
}

/// MeleeAttackGoal: Closes distance to target and deals melee damage on contact with cooldown.
pub struct MeleeAttackGoal {
    pub speed_modifier: f32,
    pub attack_reach: f32,
    pub attack_damage: f32,
    pub attack_interval: u32,
}

impl MeleeAttackGoal {
    pub fn new(speed_modifier: f32, attack_reach: f32, attack_damage: f32, attack_interval: u32) -> Self {
        Self {
            speed_modifier,
            attack_reach,
            attack_damage,
            attack_interval,
        }
    }

    pub fn standard_zombie() -> Self {
        Self::new(1.0, 1.8, 3.0, 20)
    }
}

impl Default for MeleeAttackGoal {
    fn default() -> Self {
        Self::standard_zombie()
    }
}

impl Goal for MeleeAttackGoal {
    fn can_use(&mut self, ctx: &GoalContext) -> bool {
        if let Some(target) = ctx.target {
            target.is_alive
        } else {
            false
        }
    }

    fn can_continue_to_use(&mut self, ctx: &GoalContext) -> bool {
        if let Some(target) = ctx.target {
            target.is_alive && ctx.transform.position.distance(target.position) <= 24.0
        } else {
            false
        }
    }

    fn tick(&mut self, ctx: &mut GoalContext) {
        let Some(target) = ctx.target else { return };

        // Look toward target
        ctx.look_target = Some(target.position);

        let dist = ctx.transform.position.distance(target.position);
        if dist <= self.attack_reach {
            // Close enough for melee attack
            ctx.requested_velocity = Some(Vec3::new(0.0, ctx.velocity.value.y, 0.0));

            if ctx.attack_cooldown == 0 {
                // Strike!
                ctx.attack_event = Some(AttackEvent {
                    attacker_id: ctx.entity_id,
                    target_id: target.id,
                    damage: self.attack_damage,
                    is_ranged: false,
                });
                ctx.attack_cooldown = self.attack_interval;
            } else {
                ctx.attack_cooldown -= 1;
            }
        } else {
            // Move toward target
            let dir = (target.position - ctx.transform.position).normalize_or_zero();
            let speed = 0.23 * self.speed_modifier;
            ctx.requested_velocity = Some(Vec3::new(dir.x * speed, ctx.velocity.value.y, dir.z * speed));

            if ctx.attack_cooldown > 0 {
                ctx.attack_cooldown -= 1;
            }
        }
    }

    fn flags(&self) -> GoalFlags {
        GoalFlags::MOVE | GoalFlags::LOOK
    }

    fn name(&self) -> &'static str {
        "MeleeAttackGoal"
    }
}

/// RangedAttackGoal: Mob strafes and maintains tactical distance while shooting projectiles.
pub struct RangedAttackGoal {
    pub min_combat_dist: f32,
    pub max_combat_dist: f32,
    pub projectile_damage: f32,
    pub attack_interval: u32,
    pub attack_cooldown: u32,
}

impl RangedAttackGoal {
    pub fn new(min_combat_dist: f32, max_combat_dist: f32, projectile_damage: f32, attack_interval: u32) -> Self {
        Self {
            min_combat_dist,
            max_combat_dist,
            projectile_damage,
            attack_interval,
            attack_cooldown: 0,
        }
    }

    pub fn standard_skeleton() -> Self {
        Self::new(6.0, 15.0, 4.0, 30) // Skeleton bow attack interval 30 ticks = 1.5s
    }
}

impl Default for RangedAttackGoal {
    fn default() -> Self {
        Self::standard_skeleton()
    }
}

impl Goal for RangedAttackGoal {
    fn can_use(&mut self, ctx: &GoalContext) -> bool {
        if let Some(target) = ctx.target {
            target.is_alive
        } else {
            false
        }
    }

    fn can_continue_to_use(&mut self, ctx: &GoalContext) -> bool {
        if let Some(target) = ctx.target {
            target.is_alive && ctx.transform.position.distance(target.position) <= self.max_combat_dist * 1.5
        } else {
            false
        }
    }

    fn tick(&mut self, ctx: &mut GoalContext) {
        let Some(target) = ctx.target else { return };

        ctx.look_target = Some(target.position);
        let dist = ctx.transform.position.distance(target.position);

        // Tactical positioning
        if dist < self.min_combat_dist {
            // Too close! Back away
            let back_dir = (ctx.transform.position - target.position).normalize_or_zero();
            ctx.requested_velocity = Some(Vec3::new(back_dir.x * 0.2, ctx.velocity.value.y, back_dir.z * 0.2));
        } else if dist > self.max_combat_dist {
            // Too far! Advance
            let fwd_dir = (target.position - ctx.transform.position).normalize_or_zero();
            ctx.requested_velocity = Some(Vec3::new(fwd_dir.x * 0.2, ctx.velocity.value.y, fwd_dir.z * 0.2));
        } else {
            // Sweet spot: hold position and aim
            ctx.requested_velocity = Some(Vec3::new(0.0, ctx.velocity.value.y, 0.0));
        }

        // Projectile attack timer
        if self.attack_cooldown == 0 {
            ctx.attack_event = Some(AttackEvent {
                attacker_id: ctx.entity_id,
                target_id: target.id,
                damage: self.projectile_damage,
                is_ranged: true,
            });
            self.attack_cooldown = self.attack_interval;
        } else {
            self.attack_cooldown -= 1;
        }
    }

    fn flags(&self) -> GoalFlags {
        GoalFlags::MOVE | GoalFlags::LOOK
    }

    fn name(&self) -> &'static str {
        "RangedAttackGoal"
    }
}
