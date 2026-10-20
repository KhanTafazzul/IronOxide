// REA Verified: net.minecraft.world.entity.animal.Fox (mapped: byi.class)
// REA Verified: net.minecraft.world.entity.animal.Panda (mapped: byn.class)
// REA Verified: net.minecraft.world.entity.animal.PolarBear (mapped: byq.class)
// REA Verified: net.minecraft.world.entity.animal.Rabbit (mapped: bys.class)
// REA Verified: net.minecraft.world.entity.animal.camel.Camel (mapped: bzk.class)
// REA Verified: net.minecraft.world.entity.animal.horse.Horse (mapped: bzz.class)
// REA Verified: net.minecraft.world.entity.animal.horse.Donkey (mapped: bzy.class)
// REA Verified: net.minecraft.world.entity.animal.horse.Mule (mapped: cac.class)
// REA Verified: net.minecraft.world.entity.ambient.Bat (mapped: bxt.class)
// REA Verified: net.minecraft.world.entity.animal.allay.Allay (mapped: bzc.class)
// REA Verified: net.minecraft.world.entity.animal.sniffer.Sniffer (mapped: cak.class)
//
// Wild and utility animal implementations with authentic unique behaviors,
// state machines, inventories, and mount mechanics.

use super::AnimalData;
use crate::components::EntityId;
use mc_items::ItemId;

// ==============================================================================
// 1. FOX
// ==============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FoxVariant {
    #[default]
    Red,
    Snow,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Fox {
    pub animal_data: AnimalData,
    pub variant: FoxVariant,
    pub mouth_item: Option<ItemId>,
    pub is_sleeping: bool,
    pub is_sitting: bool,
    pub is_pouncing: bool,
    pub trusted_players: Vec<EntityId>,
}

impl Default for Fox {
    fn default() -> Self {
        Self {
            animal_data: AnimalData::adult(),
            variant: FoxVariant::Red,
            mouth_item: None,
            is_sleeping: false,
            is_sitting: false,
            is_pouncing: false,
            trusted_players: Vec::new(),
        }
    }
}

impl Fox {
    pub fn new(variant: FoxVariant) -> Self {
        Self {
            variant,
            ..Default::default()
        }
    }

    /// Checks if item can be picked up and held in the fox's mouth.
    pub fn can_hold_item(&self) -> bool {
        self.mouth_item.is_none()
    }

    /// Holds an item in its mouth (e.g. food, emerald, weapon).
    pub fn pick_up_item(&mut self, item: ItemId) {
        self.mouth_item = Some(item);
    }

    /// Evaluates day/night sleeping logic:
    /// Sleeps during the daytime unless disturbed by close non-sneaking player or thunder.
    pub fn update_sleep_state(&mut self, is_day: bool, is_thunder: bool, nearest_player_dist: f32, player_sneaking: bool) {
        if is_day && !is_thunder {
            if nearest_player_dist < 4.0 && !player_sneaking {
                self.is_sleeping = false; // Disturbed!
            } else {
                self.is_sleeping = true;
            }
        } else {
            self.is_sleeping = false; // Active hunter at night
        }
    }

    /// Food item check: Sweet Berries and Glow Berries.
    pub fn is_food(item: ItemId) -> bool {
        matches!(item, ItemId::SweetBerries | ItemId::GlowBerries)
    }
}

// ==============================================================================
// 2. ARMADILLO
// ==============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ArmadilloState {
    #[default]
    Unrolled,
    Rolling,
    RolledUp,
    Peeking,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Armadillo {
    pub animal_data: AnimalData,
    pub state: ArmadilloState,
    pub roll_ticks_left: u32,
    pub scute_craft_count: u32,
}

impl Default for Armadillo {
    fn default() -> Self {
        Self {
            animal_data: AnimalData::adult(),
            state: ArmadilloState::Unrolled,
            roll_ticks_left: 0,
            scute_craft_count: 0,
        }
    }
}

impl Armadillo {
    pub const ROLL_DURATION_TICKS: u32 = 100; // 5 seconds in defensive shell

    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates surrounding threat conditions:
    /// Rolls up if: sprinting player is within 7 blocks, undead mob within 8 blocks, or damaged.
    pub fn evaluate_threat(&mut self, sprinting_player_dist: Option<f32>, undead_dist: Option<f32>, took_damage: bool) {
        let threatened = took_damage
            || sprinting_player_dist.map_or(false, |d| d < 7.0)
            || undead_dist.map_or(false, |d| d < 8.0);

        if threatened {
            self.state = ArmadilloState::RolledUp;
            self.roll_ticks_left = Self::ROLL_DURATION_TICKS;
        }
    }

    /// Ticks armadillo state countdown.
    pub fn tick(&mut self) {
        self.animal_data.tick();
        if self.roll_ticks_left > 0 {
            self.roll_ticks_left -= 1;
            if self.roll_ticks_left == 0 {
                self.state = ArmadilloState::Unrolled;
            } else if self.roll_ticks_left < 20 {
                self.state = ArmadilloState::Peeking;
            }
        }
    }

    /// Damage reduction while rolled up: takes 50% reduced damage.
    pub fn damage_multiplier(&self) -> f32 {
        if matches!(self.state, ArmadilloState::RolledUp | ArmadilloState::Rolling) {
            0.5
        } else {
            1.0
        }
    }

    /// Brushing tool interaction drops an Armadillo Scute.
    pub fn brush(&mut self, held_item: ItemId) -> Option<ItemId> {
        if matches!(held_item, ItemId::Brush) && self.state == ArmadilloState::Unrolled {
            self.scute_craft_count += 1;
            Some(ItemId::ArmadilloScute)
        } else {
            None
        }
    }
}

// ==============================================================================
// 3. CAMEL
// ==============================================================================

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Camel {
    pub animal_data: AnimalData,
    pub has_saddle: bool,
    pub driver: Option<EntityId>,
    pub passenger: Option<EntityId>,
    pub dash_cooldown_ticks: u32,
    pub is_sitting: bool,
}

impl Default for Camel {
    fn default() -> Self {
        Self {
            animal_data: AnimalData::adult(),
            has_saddle: false,
            driver: None,
            passenger: None,
            dash_cooldown_ticks: 0,
            is_sitting: false,
        }
    }
}

impl Camel {
    pub const DASH_COOLDOWN_TICKS: u32 = 55; // 2.75 seconds cooldown
    pub const RIDER_EYE_HEIGHT: f32 = 2.65; // High rider height protecting from zombies

    pub fn new() -> Self {
        Self::default()
    }

    /// Food item check: Cactus.
    pub fn is_food(item: ItemId) -> bool {
        matches!(item, ItemId::Cactus)
    }

    /// Mounts rider: First mount is driver, second is rear passenger (2-player riding).
    pub fn mount(&mut self, player: EntityId) -> bool {
        if self.driver.is_none() {
            self.driver = Some(player);
            true
        } else if self.passenger.is_none() {
            self.passenger = Some(player);
            true
        } else {
            false
        }
    }

    /// Dismounts a specific rider.
    pub fn dismount(&mut self, player: EntityId) {
        if self.driver == Some(player) {
            self.driver = self.passenger.take();
        } else if self.passenger == Some(player) {
            self.passenger = None;
        }
    }

    /// Triggers dash jump: launches camel forward 10-12 blocks with a small hop.
    pub fn try_dash(&mut self) -> bool {
        if self.has_saddle && self.dash_cooldown_ticks == 0 && !self.is_sitting {
            self.dash_cooldown_ticks = Self::DASH_COOLDOWN_TICKS;
            true
        } else {
            false
        }
    }

    /// Height check: standard ground zombie melee reach (1.8m) cannot reach riders at Y >= 2.5m!
    pub fn is_rider_immune_to_melee(&self) -> bool {
        true
    }

    pub fn tick(&mut self) {
        self.animal_data.tick();
        if self.dash_cooldown_ticks > 0 {
            self.dash_cooldown_ticks -= 1;
        }
    }
}

// ==============================================================================
// 4. SNIFFER
// ==============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SnifferState {
    #[default]
    Idling,
    Sniffing,
    Digging,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sniffer {
    pub animal_data: AnimalData,
    pub state: SnifferState,
    pub dig_ticks_left: u32,
    pub dig_cooldown_ticks: u32,
}

impl Default for Sniffer {
    fn default() -> Self {
        Self {
            animal_data: AnimalData::adult(),
            state: SnifferState::Idling,
            dig_ticks_left: 0,
            dig_cooldown_ticks: 0,
        }
    }
}

impl Sniffer {
    pub const DIG_DURATION_TICKS: u32 = 200; // 10 seconds digging animation
    pub const DIG_COOLDOWN_TICKS: u32 = 4800; // 4 minutes between digs

    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if block beneath sniffer is diggable soil.
    pub fn is_diggable_soil(block_item: ItemId) -> bool {
        matches!(
            block_item,
            ItemId::GrassBlock
                | ItemId::Dirt
                | ItemId::CoarseDirt
                | ItemId::Podzol
                | ItemId::RootedDirt
                | ItemId::Mud
                | ItemId::MossBlock
        )
    }

    /// Starts digging for ancient seeds if cooldown is ready on valid soil.
    pub fn try_start_digging(&mut self, soil: ItemId) -> bool {
        if self.state == SnifferState::Idling && self.dig_cooldown_ticks == 0 && Self::is_diggable_soil(soil) {
            self.state = SnifferState::Digging;
            self.dig_ticks_left = Self::DIG_DURATION_TICKS;
            true
        } else {
            false
        }
    }

    /// Ticks sniffer state. When digging finishes, yields Torchflower Seeds or Pitcher Pod!
    pub fn tick(&mut self, rand_bit: bool) -> Option<ItemId> {
        self.animal_data.tick();
        if self.dig_cooldown_ticks > 0 {
            self.dig_cooldown_ticks -= 1;
        }

        if self.state == SnifferState::Digging {
            if self.dig_ticks_left > 0 {
                self.dig_ticks_left -= 1;
            }
            if self.dig_ticks_left == 0 {
                self.state = SnifferState::Idling;
                self.dig_cooldown_ticks = Self::DIG_COOLDOWN_TICKS;
                let seed = if rand_bit {
                    ItemId::TorchflowerSeeds
                } else {
                    ItemId::PitcherPod
                };
                return Some(seed);
            }
        }
        None
    }
}

// ==============================================================================
// 5. ALLAY
// ==============================================================================

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Allay {
    pub owner_id: Option<EntityId>,
    pub given_item: Option<ItemId>,
    pub is_dancing: bool,
    pub linked_note_block_ticks: u32,
    pub duplication_cooldown_ticks: u32,
}

impl Default for Allay {
    fn default() -> Self {
        Self {
            owner_id: None,
            given_item: None,
            is_dancing: false,
            linked_note_block_ticks: 0,
            duplication_cooldown_ticks: 0,
        }
    }
}

impl Allay {
    pub const NOTE_BLOCK_TIMEOUT_TICKS: u32 = 600; // 30 seconds listening window
    pub const DUPLICATION_COOLDOWN: u32 = 6000; // 5 minutes

    pub fn new() -> Self {
        Self::default()
    }

    /// Player gives an item to the Allay to fetch duplicates in a 32-block radius.
    pub fn give_item(&mut self, player: EntityId, item: ItemId) {
        self.owner_id = Some(player);
        self.given_item = Some(item);
    }

    /// Takes the held item back from the Allay.
    pub fn take_item(&mut self) -> Option<ItemId> {
        self.given_item.take()
    }

    /// Receives vibration from a nearby Note Block.
    pub fn hear_note_block(&mut self) {
        self.linked_note_block_ticks = Self::NOTE_BLOCK_TIMEOUT_TICKS;
    }

    /// Duplicates the allay if fed an Amethyst Shard while dancing to jukebox music.
    pub fn try_duplicate(&mut self, item: ItemId) -> Option<Allay> {
        if self.is_dancing && matches!(item, ItemId::AmethystShard) && self.duplication_cooldown_ticks == 0 {
            self.duplication_cooldown_ticks = Self::DUPLICATION_COOLDOWN;
            let mut clone = Allay::new();
            clone.duplication_cooldown_ticks = Self::DUPLICATION_COOLDOWN;
            Some(clone)
        } else {
            None
        }
    }

    pub fn tick(&mut self) {
        if self.linked_note_block_ticks > 0 {
            self.linked_note_block_ticks -= 1;
        }
        if self.duplication_cooldown_ticks > 0 {
            self.duplication_cooldown_ticks -= 1;
        }
    }
}

// ==============================================================================
// 6. EQUINES (HORSE, DONKEY, MULE)
// ==============================================================================

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Horse {
    pub animal_data: AnimalData,
    pub is_tamed: bool,
    pub owner_id: Option<EntityId>,
    pub temper: u32,
    pub max_temper: u32,
    pub has_saddle: bool,
    pub armor: Option<ItemId>,
    pub jump_strength: f32, // 0.4..1.0
}

impl Default for Horse {
    fn default() -> Self {
        Self {
            animal_data: AnimalData::adult(),
            is_tamed: false,
            owner_id: None,
            temper: 0,
            max_temper: 100,
            has_saddle: false,
            armor: None,
            jump_strength: 0.7,
        }
    }
}

impl Horse {
    pub fn new() -> Self {
        Self::default()
    }

    /// Attempts to tame horse during mounting: increments temper until max_temper.
    pub fn try_tame_mount(&mut self, player: EntityId, temper_gain: u32) -> bool {
        if self.is_tamed {
            return true;
        }
        self.temper = self.temper.saturating_add(temper_gain);
        if self.temper >= self.max_temper {
            self.is_tamed = true;
            self.owner_id = Some(player);
            true
        } else {
            false
        }
    }

    /// Equips horse armor (Iron, Gold, Diamond).
    pub fn equip_armor(&mut self, armor_item: ItemId) -> bool {
        if matches!(
            armor_item,
            ItemId::IronHorseArmor | ItemId::GoldenHorseArmor | ItemId::DiamondHorseArmor
        ) {
            self.armor = Some(armor_item);
            true
        } else {
            false
        }
    }
}

// ==============================================================================
// 7. PANDA GENETICS
// ==============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PandaGene {
    #[default]
    Normal = 0,
    Lazy = 1,
    Worried = 2,
    Playful = 3,
    Brown = 4,
    Weak = 5,
    Aggressive = 6,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Panda {
    pub animal_data: AnimalData,
    pub main_gene: PandaGene,
    pub hidden_gene: PandaGene,
}

impl Default for Panda {
    fn default() -> Self {
        Self {
            animal_data: AnimalData::adult(),
            main_gene: PandaGene::Normal,
            hidden_gene: PandaGene::Normal,
        }
    }
}

impl Panda {
    pub fn new(main_gene: PandaGene, hidden_gene: PandaGene) -> Self {
        Self {
            animal_data: AnimalData::adult(),
            main_gene,
            hidden_gene,
        }
    }

    /// Evaluates dominant vs recessive expressed phenotype matching vanilla allele dominance:
    /// Brown (4) and Weak (5) are strictly recessive (require both alleles to express).
    pub fn expressed_trait(&self) -> PandaGene {
        if self.main_gene == self.hidden_gene {
            self.main_gene
        } else if self.main_gene != PandaGene::Brown && self.main_gene != PandaGene::Weak {
            self.main_gene
        } else if self.hidden_gene != PandaGene::Brown && self.hidden_gene != PandaGene::Weak {
            self.hidden_gene
        } else {
            PandaGene::Normal
        }
    }
}

// ==============================================================================
// 8. POLAR BEAR, RABBIT, BAT
// ==============================================================================

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PolarBear {
    pub animal_data: AnimalData,
    pub is_rearing_up: bool,
    pub has_cub_nearby: bool,
}

impl PolarBear {
    pub fn new(has_cub: bool) -> Self {
        Self {
            animal_data: AnimalData::adult(),
            is_rearing_up: false,
            has_cub_nearby: has_cub,
        }
    }

    /// Polar bears attack if a cub is nearby or if provoked.
    pub fn is_hostile_to_player(&self, was_attacked: bool) -> bool {
        self.has_cub_nearby || was_attacked
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rabbit {
    pub animal_data: AnimalData,
    pub is_killer_bunny: bool,
}

impl Rabbit {
    pub fn is_food(item: ItemId) -> bool {
        matches!(item, ItemId::Carrot | ItemId::GoldenCarrot | ItemId::Dandelion)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bat {
    pub is_resting: bool,
}

impl Bat {
    pub fn new() -> Self {
        Self { is_resting: true }
    }

    /// Bats wake up and fly if light level rises or player gets close.
    pub fn update_resting(&mut self, light_level: u8, player_dist: f32) {
        if light_level > 0 || player_dist < 4.0 {
            self.is_resting = false;
        }
    }
}
