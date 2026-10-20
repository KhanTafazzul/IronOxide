// REA Verified: net.minecraft.world.entity.animal.Wolf (mapped: bzb.class)
// REA Verified: net.minecraft.world.entity.animal.Cat (mapped: byb.class)
// REA Verified: net.minecraft.world.entity.animal.Ocelot (mapped: bym.class)
// REA Verified: net.minecraft.world.entity.animal.Parrot (mapped: byo.class)
//
// Pets & Tameable Mobs: Wolf, Cat, Ocelot, and Parrot with exact taming state
// machines, sitting states, Creeper/Phantom scaring, shoulder perching, and teleport-to-owner mechanics.

use super::{AnimalData, DyeColor, TameableData};
use crate::components::EntityId;
use mc_items::ItemId;

/// Wolf entity implementation matching `net.minecraft.world.entity.animal.Wolf` (`bzb.class`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Wolf {
    pub animal_data: AnimalData,
    pub tame_data: TameableData,
    pub health: f32,
    pub max_health: f32,
    pub is_angry: bool,
    pub anger_target: Option<EntityId>,
}

impl Default for Wolf {
    fn default() -> Self {
        Self {
            animal_data: AnimalData::adult(),
            tame_data: TameableData::untamed(),
            health: 8.0,
            max_health: 8.0,
            is_angry: false,
            anger_target: None,
        }
    }
}

impl Wolf {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates tail angle in radians based on current health proportion:
    /// Vanilla formula: `(health / max_health) * 0.6 + 0.1` radians.
    pub fn tail_angle(&self) -> f32 {
        if self.is_angry {
            1.539 // ~88 degrees elevated when aggressive
        } else if self.tame_data.is_tamed {
            let ratio = (self.health / self.max_health).clamp(0.0, 1.0);
            ratio * 0.6 + 0.1
        } else {
            0.2 // Slightly lowered default tail
        }
    }

    /// Attempts to tame the wolf using bones.
    /// Vanilla has a 1/3 (33.3%) chance per bone.
    pub fn try_tame(&mut self, player: EntityId, held_item: ItemId, rand_mod_3: u32) -> bool {
        if !self.tame_data.is_tamed && matches!(held_item, ItemId::Bone) {
            if rand_mod_3 == 0 {
                self.tame_data.tame(player, Some(DyeColor::Red));
                self.max_health = 40.0;
                self.health = 40.0;
                self.is_angry = false;
                self.anger_target = None;
                return true;
            }
        }
        false
    }

    /// Checks if item is meat that can heal or breed the wolf.
    pub fn is_meat(item: ItemId) -> bool {
        matches!(
            item,
            ItemId::Beef
                | ItemId::CookedBeef
                | ItemId::Porkchop
                | ItemId::CookedPorkchop
                | ItemId::Chicken
                | ItemId::CookedChicken
                | ItemId::Mutton
                | ItemId::CookedMutton
                | ItemId::Rabbit
                | ItemId::CookedRabbit
                | ItemId::RottenFlesh
        )
    }

    /// Interacts with wolf: taming, healing, dyeing collar, or toggling sitting state.
    pub fn interact(&mut self, player: EntityId, held_item: ItemId, rand_mod_3: u32) -> bool {
        if !self.tame_data.is_tamed {
            return self.try_tame(player, held_item, rand_mod_3);
        }

        // Tamed interactions by owner
        if self.tame_data.owner_id == Some(player) {
            if Self::is_meat(held_item) {
                if self.health < self.max_health {
                    self.health = (self.health + 4.0).min(self.max_health);
                    return true;
                } else if self.animal_data.can_breed() {
                    self.animal_data.set_in_love(Some(player));
                    return true;
                }
            } else if let Some(dye) = DyeColor::from_wool_item_id(held_item) {
                self.tame_data.collar_color = Some(dye);
                return true;
            } else if held_item == ItemId::Air {
                self.tame_data.toggle_sitting();
                return true;
            }
        }
        false
    }

    /// Teleportation condition: Owner is further than 12 blocks away and wolf is not sitting.
    pub fn should_teleport_to_owner(&self, distance_to_owner: f32) -> bool {
        self.tame_data.is_tamed && !self.tame_data.is_sitting && distance_to_owner > 12.0
    }
}

/// Vanilla 11 Cat Skin Variants matching `net.minecraft.world.entity.animal.Cat$Variant`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CatVariant {
    #[default]
    Tabby = 0,
    Black = 1,
    Red = 2,
    Siamese = 3,
    BritishShorthair = 4,
    Calico = 5,
    Persian = 6,
    Ragdoll = 7,
    White = 8,
    Jellie = 9,
    AllBlack = 10,
}

/// Cat entity implementation matching `net.minecraft.world.entity.animal.Cat` (`byb.class`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cat {
    pub animal_data: AnimalData,
    pub tame_data: TameableData,
    pub variant: CatVariant,
    pub is_lying_down: bool,
}

impl Default for Cat {
    fn default() -> Self {
        Self {
            animal_data: AnimalData::adult(),
            tame_data: TameableData::untamed(),
            variant: CatVariant::Tabby,
            is_lying_down: false,
        }
    }
}

impl Cat {
    pub fn new(variant: CatVariant) -> Self {
        Self {
            animal_data: AnimalData::adult(),
            tame_data: TameableData::untamed(),
            variant,
            is_lying_down: false,
        }
    }

    /// Checks if item is fish used for taming/breeding cats (Raw Cod or Raw Salmon).
    pub fn is_fish(item: ItemId) -> bool {
        matches!(item, ItemId::Cod | ItemId::Salmon)
    }

    /// Attempts to tame the cat using raw fish (1/3 chance).
    pub fn try_tame(&mut self, player: EntityId, held_item: ItemId, rand_mod_3: u32) -> bool {
        if !self.tame_data.is_tamed && Self::is_fish(held_item) {
            if rand_mod_3 == 0 {
                self.tame_data.tame(player, Some(DyeColor::Red));
                return true;
            }
        }
        false
    }

    /// Deterrence aura: Creepers and Phantoms within 16 blocks are repelled and flee from cats.
    pub fn repels_creepers_and_phantoms(&self) -> bool {
        true
    }

    /// Teleportation condition: Owner > 12 blocks away and cat is not sitting.
    pub fn should_teleport_to_owner(&self, distance_to_owner: f32) -> bool {
        self.tame_data.is_tamed && !self.tame_data.is_sitting && distance_to_owner > 12.0
    }

    /// Interacts with cat: taming, breeding, dyeing collar, or toggling sitting state.
    pub fn interact(&mut self, player: EntityId, held_item: ItemId, rand_mod_3: u32) -> bool {
        if !self.tame_data.is_tamed {
            return self.try_tame(player, held_item, rand_mod_3);
        }

        if self.tame_data.owner_id == Some(player) {
            if Self::is_fish(held_item) && self.animal_data.can_breed() {
                self.animal_data.set_in_love(Some(player));
                return true;
            } else if let Some(dye) = DyeColor::from_wool_item_id(held_item) {
                self.tame_data.collar_color = Some(dye);
                return true;
            } else if held_item == ItemId::Air {
                self.tame_data.toggle_sitting();
                return true;
            }
        }
        false
    }
}

/// Ocelot entity implementation matching `net.minecraft.world.entity.animal.Ocelot` (`bym.class`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ocelot {
    pub animal_data: AnimalData,
    pub is_trusting: bool,
    pub trusted_player: Option<EntityId>,
}

impl Default for Ocelot {
    fn default() -> Self {
        Self {
            animal_data: AnimalData::adult(),
            is_trusting: false,
            trusted_player: None,
        }
    }
}

impl Ocelot {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feeds raw fish to gain the ocelot's trust (1/3 chance).
    /// Once trusting, the ocelot stops sprinting away from that player.
    pub fn feed_fish(&mut self, player: EntityId, held_item: ItemId, rand_mod_3: u32) -> bool {
        if Cat::is_fish(held_item) {
            if !self.is_trusting && rand_mod_3 == 0 {
                self.is_trusting = true;
                self.trusted_player = Some(player);
                return true;
            } else if self.is_trusting && self.animal_data.can_breed() {
                self.animal_data.set_in_love(Some(player));
                return true;
            }
        }
        false
    }
}

/// Parrot color variants matching `net.minecraft.world.entity.animal.Parrot$Variant`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParrotVariant {
    #[default]
    RedBlue = 0,
    Blue = 1,
    Green = 2,
    YellowBlue = 3,
    Gray = 4,
}

/// Parrot entity implementation matching `net.minecraft.world.entity.animal.Parrot` (`byo.class`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Parrot {
    pub tame_data: TameableData,
    pub variant: ParrotVariant,
    pub is_perched: bool,
    pub is_dancing: bool,
}

impl Default for Parrot {
    fn default() -> Self {
        Self {
            tame_data: TameableData::untamed(),
            variant: ParrotVariant::RedBlue,
            is_perched: false,
            is_dancing: false,
        }
    }
}

impl Parrot {
    pub fn new(variant: ParrotVariant) -> Self {
        Self {
            tame_data: TameableData::untamed(),
            variant,
            is_perched: false,
            is_dancing: false,
        }
    }

    /// Taming items: all standard seeds.
    pub fn is_tame_item(item: ItemId) -> bool {
        matches!(
            item,
            ItemId::WheatSeeds
                | ItemId::MelonSeeds
                | ItemId::PumpkinSeeds
                | ItemId::BeetrootSeeds
                | ItemId::TorchflowerSeeds
                | ItemId::PitcherPod
        )
    }

    /// Interacts with parrot:
    /// - Feeding Cookie triggers instant fatal poison (death)!
    /// - Feeding seeds has 1/3 chance to tame.
    /// - Empty hand toggles sitting.
    pub fn interact(&mut self, player: EntityId, held_item: ItemId, rand_mod_3: u32) -> Result<bool, &'static str> {
        if matches!(held_item, ItemId::Cookie) {
            // Vanilla parity: Cookies are fatal poison to parrots
            return Err("FATAL_COOKIE_POISON");
        }

        if !self.tame_data.is_tamed && Self::is_tame_item(held_item) {
            if rand_mod_3 == 0 {
                self.tame_data.tame(player, None);
                return Ok(true);
            }
            return Ok(false);
        }

        if self.tame_data.owner_id == Some(player) && held_item == ItemId::Air {
            self.tame_data.toggle_sitting();
            return Ok(true);
        }

        Ok(false)
    }

    /// Sets jukebox music listening state for the swaying dance animation.
    pub fn set_nearby_music(&mut self, playing: bool) {
        self.is_dancing = playing;
    }

    /// Mounts the player's shoulder when walked through by owner.
    pub fn mount_shoulder(&mut self) {
        if self.tame_data.is_tamed && !self.tame_data.is_sitting {
            self.is_perched = true;
        }
    }

    /// Dismounts shoulder on player jump or taking damage.
    pub fn dismount_shoulder(&mut self) {
        self.is_perched = false;
    }
}
