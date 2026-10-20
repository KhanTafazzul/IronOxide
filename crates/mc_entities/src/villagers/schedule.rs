// REA Verified: net.minecraft.world.entity.npc.Villager (mapped: cev.class)
// REA Verified: net.minecraft.world.entity.schedule.Activity (mapped: bxe.class)
// REA Verified: net.minecraft.world.entity.schedule.Schedule (mapped: bxf.class)
//
// Villager POI daily schedule clock, activity state machine (Work, Gather, Sleep, Rest),
// bed claiming and horizontal sleeping orientation.

use glam::Vec3;

/// Villager scheduled activities matching vanilla `net.minecraft.world.entity.schedule.Activity`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VillagerActivity {
    /// Wandering around village during morning free time or evening wind-down.
    #[default]
    Rest,
    /// Pathfinding to and working at workstation POI (ticks 2000..9000).
    Work,
    /// Gathering at the village bell to socialize and gossip (ticks 9000..11000).
    Gather,
    /// Sleeping horizontally in a claimed bed (ticks 12000..24000).
    Sleep,
    /// Fleeing from hostile zombies, illagers, or raids.
    Panic,
}

/// Villager POI daily schedule controller.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VillagerSchedule {
    /// Current activity state.
    pub current_activity: VillagerActivity,
    /// Target coordinates of claimed workstation POI.
    pub workstation_pos: Option<Vec3>,
    /// Target coordinates of claimed bed POI.
    pub bed_pos: Option<Vec3>,
    /// Target coordinates of village meeting bell POI.
    pub bell_pos: Option<Vec3>,
    /// Whether the villager is currently lying horizontally in bed.
    pub is_sleeping: bool,
    /// Sleep head rotation/yaw facing angle in radians.
    pub sleep_orientation: f32,
    /// Panic duration countdown ticks.
    pub panic_ticks: u32,
}

impl Default for VillagerSchedule {
    fn default() -> Self {
        Self {
            current_activity: VillagerActivity::Rest,
            workstation_pos: None,
            bed_pos: None,
            bell_pos: None,
            is_sleeping: false,
            sleep_orientation: 0.0,
            panic_ticks: 0,
        }
    }
}

impl VillagerSchedule {
    pub const WORK_START_TICK: u32 = 2000;
    pub const GATHER_START_TICK: u32 = 9000;
    pub const REST_START_TICK: u32 = 11000;
    pub const SLEEP_START_TICK: u32 = 12000;
    pub const DAY_CYCLE_TICKS: u32 = 24000;

    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates scheduled activity based on current day time (0..24000 ticks) and profession type:
    ///
    /// Standard Villager Schedule:
    /// - 0000..2000 (Dawn): Rest / Free wandering
    /// - 2000..9000 (Day): Work at workstation POI
    /// - 9000..11000 (Afternoon): Gather at Village Bell
    /// - 11000..12000 (Dusk): Rest / Head to bed
    /// - 12000..24000 (Night): Sleep horizontally in claimed bed
    ///
    /// Nitwit Villager Schedule:
    /// - 0000..3000: Sleeps in late
    /// - 3000..10000: Wanders aimlessly
    /// - 10000..12000: Joins gather late
    /// - 12000..14000: Stays up late
    /// - 14000..24000: Sleep
    pub fn update_schedule(&mut self, day_time_ticks: u32, is_nitwit: bool) -> VillagerActivity {
        if self.panic_ticks > 0 {
            self.panic_ticks -= 1;
            self.current_activity = VillagerActivity::Panic;
            self.wake_up();
            return VillagerActivity::Panic;
        }

        let time = day_time_ticks % Self::DAY_CYCLE_TICKS;

        if is_nitwit {
            self.current_activity = if time < 3000 || time >= 14000 {
                VillagerActivity::Sleep
            } else if (10000..12000).contains(&time) {
                VillagerActivity::Gather
            } else {
                VillagerActivity::Rest
            };
        } else {
            self.current_activity = if time < Self::WORK_START_TICK {
                VillagerActivity::Rest
            } else if time < Self::GATHER_START_TICK {
                VillagerActivity::Work
            } else if time < Self::REST_START_TICK {
                VillagerActivity::Gather
            } else if time < Self::SLEEP_START_TICK {
                VillagerActivity::Rest
            } else {
                VillagerActivity::Sleep
            };
        }

        if self.current_activity == VillagerActivity::Sleep {
            if self.bed_pos.is_some() {
                self.is_sleeping = true;
            }
        } else {
            self.wake_up();
        }

        self.current_activity
    }

    /// Triggers panic mode when threatened by zombies or illagers.
    pub fn trigger_panic(&mut self, duration_ticks: u32) {
        self.panic_ticks = duration_ticks;
        self.current_activity = VillagerActivity::Panic;
        self.wake_up();
    }

    /// Wakes up the villager from bed.
    pub fn wake_up(&mut self) {
        self.is_sleeping = false;
    }

    /// Claims a workstation POI.
    pub fn claim_workstation(&mut self, pos: Vec3) {
        self.workstation_pos = Some(pos);
    }

    /// Releases workstation POI.
    pub fn release_workstation(&mut self) {
        self.workstation_pos = None;
    }

    /// Claims a bed POI.
    pub fn claim_bed(&mut self, pos: Vec3, orientation: f32) {
        self.bed_pos = Some(pos);
        self.sleep_orientation = orientation;
    }

    /// Releases bed POI.
    pub fn release_bed(&mut self) {
        self.bed_pos = None;
        self.wake_up();
    }

    /// Claims a village meeting bell POI.
    pub fn claim_bell(&mut self, pos: Vec3) {
        self.bell_pos = Some(pos);
    }

    /// Returns the active POI navigation destination for the current activity.
    pub fn current_destination(&self) -> Option<Vec3> {
        match self.current_activity {
            VillagerActivity::Work => self.workstation_pos,
            VillagerActivity::Gather => self.bell_pos,
            VillagerActivity::Sleep => self.bed_pos,
            VillagerActivity::Rest | VillagerActivity::Panic => None,
        }
    }
}
