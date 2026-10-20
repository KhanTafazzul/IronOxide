// REA Verified: net.minecraft.world.entity.ai.goal.GoalSelector (mapped: bti.class)
//
// Goal Selector prioritization engine with flag mutual exclusion and lifecycle transitions.

use std::collections::HashMap;
use super::goal::{Goal, GoalContext, GoalFlags, WrappedGoal};

/// Behavior Goal Selector prioritization engine.
/// Manages a list of goals, selects active goals according to integer priority,
/// and enforces subsystem flag mutual exclusion (e.g., MOVE, LOOK, JUMP, TARGET).
pub struct GoalSelector {
    goals: Vec<WrappedGoal>,
    /// Maps each flag bit (1, 2, 4, 8) to the index in `goals` of the currently running goal holding it.
    locked_flags: HashMap<u8, usize>,
    disabled_flags: GoalFlags,
    new_goal_rate: u32,
    tick_count: u32,
}

impl GoalSelector {
    pub fn new() -> Self {
        Self {
            goals: Vec::new(),
            locked_flags: HashMap::new(),
            disabled_flags: GoalFlags::empty(),
            new_goal_rate: 1, // evaluate candidates every tick
            tick_count: 0,
        }
    }

    /// Registers a goal with an integer priority (0 = highest priority).
    pub fn add_goal(&mut self, priority: i32, goal: Box<dyn Goal>) {
        self.goals.push(WrappedGoal::new(priority, goal));
        // Sort goals by priority ascending (lower integer first)
        self.goals.sort_by_key(|g| g.priority);
    }

    /// Removes a goal by its diagnostic name.
    pub fn remove_goal_by_name(&mut self, name: &str, ctx: &mut GoalContext) {
        if let Some(pos) = self.goals.iter().position(|g| g.name() == name) {
            let mut removed = self.goals.remove(pos);
            if removed.is_running {
                removed.goal.stop(ctx);
            }
            self.rebuild_locked_flags();
        }
    }

    /// Disables specific control flags (e.g. disable MOVE while mob is riding or stunned).
    pub fn disable_control_flag(&mut self, flag: GoalFlags) {
        self.disabled_flags.insert(flag);
    }

    /// Enables specific control flags.
    pub fn enable_control_flag(&mut self, flag: GoalFlags) {
        self.disabled_flags.remove(flag);
    }

    pub fn disabled_flags(&self) -> GoalFlags {
        self.disabled_flags
    }

    /// Returns whether a goal with the given name is currently active and running.
    pub fn is_running(&self, name: &str) -> bool {
        self.goals.iter().any(|g| g.name() == name && g.is_running)
    }

    /// Returns the diagnostic names and priorities of all currently running goals.
    pub fn running_goals(&self) -> Vec<(&str, i32)> {
        self.goals
            .iter()
            .filter(|g| g.is_running)
            .map(|g| (g.name(), g.priority))
            .collect()
    }

    /// Total count of registered goals.
    pub fn total_goals(&self) -> usize {
        self.goals.len()
    }

    fn rebuild_locked_flags(&mut self) {
        self.locked_flags.clear();
        for (idx, g) in self.goals.iter().enumerate() {
            if g.is_running {
                let bits = g.flags().bits();
                for bit_pos in 0..8 {
                    let bit = 1 << bit_pos;
                    if (bits & bit) != 0 {
                        self.locked_flags.insert(bit, idx);
                    }
                }
            }
        }
    }

    /// Checks if a candidate goal's flags can replace all existing goals on conflicting flags.
    fn can_replace_all_conflicting_flags(&self, candidate_idx: usize) -> bool {
        let candidate = &self.goals[candidate_idx];
        let bits = candidate.flags().bits();

        for bit_pos in 0..8 {
            let bit = 1 << bit_pos;
            if (bits & bit) != 0 {
                if let Some(&holder_idx) = self.locked_flags.get(&bit) {
                    if holder_idx != candidate_idx {
                        let holder = &self.goals[holder_idx];
                        if holder.is_running && !holder.can_be_replaced_by(candidate.priority) {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    /// Ticks the goal selector: evaluates lifecycle states, preemption, and execution.
    pub fn tick(&mut self, ctx: &mut GoalContext) {
        self.tick_count = self.tick_count.wrapping_add(1);

        // 1. Verify all currently running goals
        for idx in 0..self.goals.len() {
            let g = &mut self.goals[idx];
            if g.is_running {
                let has_disabled = self.disabled_flags.intersects(g.flags());
                let can_continue = !has_disabled && g.goal.can_continue_to_use(ctx);
                if !can_continue {
                    g.goal.stop(ctx);
                    g.is_running = false;
                }
            }
        }

        // Rebuild lock table after terminations
        self.rebuild_locked_flags();

        // 2. Candidate Evaluation (according to priority order)
        if self.tick_count % self.new_goal_rate == 0 {
            for idx in 0..self.goals.len() {
                if self.goals[idx].is_running {
                    continue;
                }

                let flags = self.goals[idx].flags();
                if self.disabled_flags.intersects(flags) {
                    continue;
                }

                if self.can_replace_all_conflicting_flags(idx) {
                    let can_use = self.goals[idx].goal.can_use(ctx);
                    if can_use {
                        // Preempt conflicting running goals
                        let bits = flags.bits();
                        let mut to_stop = Vec::new();

                        for bit_pos in 0..8 {
                            let bit = 1 << bit_pos;
                            if (bits & bit) != 0 {
                                if let Some(&holder_idx) = self.locked_flags.get(&bit) {
                                    if holder_idx != idx && self.goals[holder_idx].is_running {
                                        to_stop.push(holder_idx);
                                    }
                                }
                            }
                        }

                        // Stop preempted goals
                        for holder_idx in to_stop {
                            let holder = &mut self.goals[holder_idx];
                            if holder.is_running {
                                holder.goal.stop(ctx);
                                holder.is_running = false;
                            }
                        }

                        // Start new goal
                        let candidate = &mut self.goals[idx];
                        candidate.is_running = true;
                        candidate.goal.start(ctx);

                        // Lock flags for candidate
                        for bit_pos in 0..8 {
                            let bit = 1 << bit_pos;
                            if (bits & bit) != 0 {
                                self.locked_flags.insert(bit, idx);
                            }
                        }
                    }
                }
            }
        }

        // 3. Tick all running goals
        for idx in 0..self.goals.len() {
            let g = &mut self.goals[idx];
            if g.is_running {
                g.goal.tick(ctx);
            }
        }
    }
}

impl Default for GoalSelector {
    fn default() -> Self {
        Self::new()
    }
}
