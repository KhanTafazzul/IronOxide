// REA Verified: net.minecraft.world.entity.npc.VillagerProfession (mapped: cey.class)
// REA Verified: net.minecraft.world.entity.npc.VillagerData (mapped: cew.class)
//
// 15 Villager Professions, workstation block bindings, trade tiers (Novice to Master),
// XP progression, permanent profession locking, and twice-daily trade restocking.

use mc_items::ItemId;

/// 15 Villager professions matching `net.minecraft.world.entity.npc.VillagerProfession` (`cey.class`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VillagerProfession {
    /// Unemployed villager searching for a workstation POI.
    #[default]
    None,
    /// Armorer (Workstation: Blast Furnace).
    Armorer,
    /// Butcher (Workstation: Smoker).
    Butcher,
    /// Cartographer (Workstation: Cartography Table).
    Cartographer,
    /// Cleric (Workstation: Brewing Stand).
    Cleric,
    /// Farmer (Workstation: Composter).
    Farmer,
    /// Fisherman (Workstation: Barrel).
    Fisherman,
    /// Fletcher (Workstation: Fletching Table).
    Fletcher,
    /// Leatherworker (Workstation: Cauldron).
    Leatherworker,
    /// Librarian (Workstation: Lectern).
    Librarian,
    /// Mason (Workstation: Stonecutter).
    Mason,
    /// Nitwit (Green robe - cannot claim workstations, trade, or work).
    Nitwit,
    /// Shepherd (Workstation: Loom).
    Shepherd,
    /// Toolsmith (Workstation: Smithing Table).
    Toolsmith,
    /// Weaponsmith (Workstation: Grindstone).
    Weaponsmith,
}

impl VillagerProfession {
    /// Returns the workstation block required to acquire this profession.
    pub fn workstation_item(&self) -> Option<ItemId> {
        match self {
            Self::Armorer => Some(ItemId::BlastFurnace),
            Self::Butcher => Some(ItemId::Smoker),
            Self::Cartographer => Some(ItemId::CartographyTable),
            Self::Cleric => Some(ItemId::BrewingStand),
            Self::Farmer => Some(ItemId::Composter),
            Self::Fisherman => Some(ItemId::Barrel),
            Self::Fletcher => Some(ItemId::FletchingTable),
            Self::Leatherworker => Some(ItemId::Cauldron),
            Self::Librarian => Some(ItemId::Lectern),
            Self::Mason => Some(ItemId::Stonecutter),
            Self::Shepherd => Some(ItemId::Loom),
            Self::Toolsmith => Some(ItemId::SmithingTable),
            Self::Weaponsmith => Some(ItemId::Grindstone),
            Self::None | Self::Nitwit => None,
        }
    }

    /// Resolves the corresponding profession from an unclaimed workstation block.
    pub fn from_workstation(item: ItemId) -> Option<Self> {
        match item {
            ItemId::BlastFurnace => Some(Self::Armorer),
            ItemId::Smoker => Some(Self::Butcher),
            ItemId::CartographyTable => Some(Self::Cartographer),
            ItemId::BrewingStand => Some(Self::Cleric),
            ItemId::Composter => Some(Self::Farmer),
            ItemId::Barrel => Some(Self::Fisherman),
            ItemId::FletchingTable => Some(Self::Fletcher),
            ItemId::Cauldron => Some(Self::Leatherworker),
            ItemId::Lectern => Some(Self::Librarian),
            ItemId::Stonecutter => Some(Self::Mason),
            ItemId::Loom => Some(Self::Shepherd),
            ItemId::SmithingTable => Some(Self::Toolsmith),
            ItemId::Grindstone => Some(Self::Weaponsmith),
            _ => None,
        }
    }

    /// Whether this villager profession can trade items.
    pub fn can_trade(&self) -> bool {
        !matches!(self, Self::None | Self::Nitwit)
    }
}

/// 5 Villager Trade Tiers matching vanilla tier leveling matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum TradeLevel {
    /// Level 1 (0 XP) - Unlocks 2 initial trades.
    #[default]
    Novice = 1,
    /// Level 2 (10 XP) - Unlocks Apprentice trades.
    Apprentice = 2,
    /// Level 3 (70 XP) - Unlocks Journeyman trades.
    Journeyman = 3,
    /// Level 4 (150 XP) - Unlocks Expert trades.
    Expert = 4,
    /// Level 5 (250 XP) - Unlocks Master trades.
    Master = 5,
}

impl TradeLevel {
    pub const NOVICE_XP: u32 = 0;
    pub const APPRENTICE_XP: u32 = 10;
    pub const JOURNEYMAN_XP: u32 = 70;
    pub const EXPERT_XP: u32 = 150;
    pub const MASTER_XP: u32 = 250;

    /// Evaluates TradeLevel based on accumulated trade XP.
    pub fn from_xp(xp: u32) -> Self {
        if xp >= Self::MASTER_XP {
            Self::Master
        } else if xp >= Self::EXPERT_XP {
            Self::Expert
        } else if xp >= Self::JOURNEYMAN_XP {
            Self::Journeyman
        } else if xp >= Self::APPRENTICE_XP {
            Self::Apprentice
        } else {
            Self::Novice
        }
    }

    /// Returns the minimum XP required to reach the next trade tier.
    pub fn next_tier_xp(&self) -> Option<u32> {
        match self {
            Self::Novice => Some(Self::APPRENTICE_XP),
            Self::Apprentice => Some(Self::JOURNEYMAN_XP),
            Self::Journeyman => Some(Self::EXPERT_XP),
            Self::Expert => Some(Self::MASTER_XP),
            Self::Master => None, // Max level reached
        }
    }
}

/// Villager Profession & Trading Career Tracker.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VillagerCareer {
    pub profession: VillagerProfession,
    pub level: TradeLevel,
    pub xp: u32,
    /// Number of times trades have been restocked today (max 2 per day in vanilla).
    pub restocks_today: u8,
    /// Last day time tick when a restock occurred.
    pub last_restock_tick: u32,
}

impl Default for VillagerCareer {
    fn default() -> Self {
        Self {
            profession: VillagerProfession::None,
            level: TradeLevel::Novice,
            xp: 0,
            restocks_today: 0,
            last_restock_tick: 0,
        }
    }
}

impl VillagerCareer {
    pub const MAX_RESTOCKS_PER_DAY: u8 = 2;

    pub fn new(profession: VillagerProfession) -> Self {
        Self {
            profession,
            level: TradeLevel::Novice,
            xp: 0,
            restocks_today: 0,
            last_restock_tick: 0,
        }
    }

    /// Creates a Nitwit villager.
    pub fn nitwit() -> Self {
        Self::new(VillagerProfession::Nitwit)
    }

    /// Whether this villager's profession is permanently locked.
    /// In vanilla, once a villager gains any trade experience (XP > 0), their profession is locked forever!
    pub fn is_locked(&self) -> bool {
        self.xp > 0
    }

    /// Claims a workstation block. If the villager is currently unemployed,
    /// adopts the matching profession and initializes Novice tier.
    pub fn try_claim_workstation(&mut self, workstation: ItemId) -> bool {
        if self.profession == VillagerProfession::None {
            if let Some(new_prof) = VillagerProfession::from_workstation(workstation) {
                self.profession = new_prof;
                self.level = TradeLevel::Novice;
                self.xp = 0;
                return true;
            }
        }
        false
    }

    /// Loses profession if workstation is broken, provided profession is not locked by prior trading.
    pub fn try_lose_workstation(&mut self) -> bool {
        if !self.is_locked() && self.profession != VillagerProfession::Nitwit {
            self.profession = VillagerProfession::None;
            self.level = TradeLevel::Novice;
            self.xp = 0;
            true
        } else {
            false
        }
    }

    /// Adds trade experience points and handles leveling up to higher trade tiers.
    /// Returns `true` if a new trade tier was unlocked.
    pub fn add_xp(&mut self, amount: u32) -> bool {
        if !self.profession.can_trade() {
            return false;
        }

        let old_level = self.level;
        self.xp += amount;
        self.level = TradeLevel::from_xp(self.xp);

        self.level > old_level
    }

    /// Restocks trades at the workstation during Work hours.
    /// Allowed up to twice daily in vanilla Minecraft.
    pub fn restock(&mut self, current_tick: u32) -> bool {
        if self.profession.can_trade() && self.restocks_today < Self::MAX_RESTOCKS_PER_DAY {
            self.restocks_today += 1;
            self.last_restock_tick = current_tick;
            true
        } else {
            false
        }
    }

    /// Resets the daily restock counter at dawn (tick 0..2000).
    pub fn reset_daily_restocks(&mut self) {
        self.restocks_today = 0;
    }
}
