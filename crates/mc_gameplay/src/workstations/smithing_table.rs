//! Smithing Table workstation supporting Netherite upgrading and armor trims.
//! REA Reference: net.minecraft.world.inventory.SmithingMenu (cjx.class)

use mc_crafting::ItemStack;
use mc_items::ItemId;

/// Result of smithing upgrade operation.
#[derive(Clone, PartialEq, Debug)]
pub struct SmithingResult {
    pub output: ItemStack,
    pub template_consumed: u8,
    pub base_consumed: u8,
    pub addition_consumed: u8,
}

/// Interactive Smithing Table workstation container menu.
/// REA Reference: net.minecraft.world.inventory.SmithingMenu (cjx.class)
#[derive(Clone, Debug, Default)]
pub struct SmithingTableMenu {
    /// Slot 0: Smithing Template (e.g. Netherite Upgrade Template)
    pub template_slot: Option<ItemStack>,
    /// Slot 1: Base gear to upgrade (e.g. Diamond Pickaxe)
    pub base_slot: Option<ItemStack>,
    /// Slot 2: Material addition (e.g. Netherite Ingot)
    pub addition_slot: Option<ItemStack>,
}

impl SmithingTableMenu {
    pub fn new() -> Self {
        Self {
            template_slot: None,
            base_slot: None,
            addition_slot: None,
        }
    }

    /// Evaluates current input slots to compute output preview.
    pub fn compute_result(&self) -> Option<SmithingResult> {
        let template = self.template_slot.as_ref()?;
        let base = self.base_slot.as_ref()?;
        let addition = self.addition_slot.as_ref()?;

        // Case 1: Netherite Upgrade
        if template.item == ItemId::NetheriteUpgradeSmithingTemplate && addition.item == ItemId::NetheriteIngot {
            let upgraded_item = match base.item {
                ItemId::DiamondSword => Some(ItemId::NetheriteSword),
                ItemId::DiamondPickaxe => Some(ItemId::NetheritePickaxe),
                ItemId::DiamondAxe => Some(ItemId::NetheriteAxe),
                ItemId::DiamondShovel => Some(ItemId::NetheriteShovel),
                ItemId::DiamondHoe => Some(ItemId::NetheriteHoe),
                ItemId::DiamondHelmet => Some(ItemId::NetheriteHelmet),
                ItemId::DiamondChestplate => Some(ItemId::NetheriteChestplate),
                ItemId::DiamondLeggings => Some(ItemId::NetheriteLeggings),
                ItemId::DiamondBoots => Some(ItemId::NetheriteBoots),
                _ => None,
            }?;

            return Some(SmithingResult {
                output: ItemStack::new(upgraded_item, 1),
                template_consumed: 1,
                base_consumed: 1,
                addition_consumed: 1,
            });
        }

        // Case 2: Armor Trim Recipe
        if is_armor_trim_template(template.item) && base.item.is_armor() && is_trim_material(addition.item) {
            // Produces the trimmed armor
            return Some(SmithingResult {
                output: ItemStack::new(base.item, 1),
                template_consumed: 1,
                base_consumed: 1,
                addition_consumed: 1,
            });
        }

        None
    }

    /// Takes the smithing output and decrements inputs by 1.
    pub fn take_output(&mut self) -> Option<ItemStack> {
        let result = self.compute_result()?;

        // Decrement template
        if let Some(template) = &mut self.template_slot {
            if template.count <= result.template_consumed {
                self.template_slot = None;
            } else {
                template.count -= result.template_consumed;
            }
        }

        // Decrement base
        if let Some(base) = &mut self.base_slot {
            if base.count <= result.base_consumed {
                self.base_slot = None;
            } else {
                base.count -= result.base_consumed;
            }
        }

        // Decrement addition
        if let Some(addition) = &mut self.addition_slot {
            if addition.count <= result.addition_consumed {
                self.addition_slot = None;
            } else {
                addition.count -= result.addition_consumed;
            }
        }

        Some(result.output)
    }

    /// Handles closing the UI and returning all items to the player.
    pub fn close_and_drop_items(&mut self) -> Vec<ItemStack> {
        let mut dropped = Vec::new();
        if let Some(s) = self.template_slot.take() {
            dropped.push(s);
        }
        if let Some(s) = self.base_slot.take() {
            dropped.push(s);
        }
        if let Some(s) = self.addition_slot.take() {
            dropped.push(s);
        }
        dropped
    }
}

pub fn is_armor_trim_template(item: ItemId) -> bool {
    matches!(
        item,
        ItemId::CoastArmorTrimSmithingTemplate |
        ItemId::DuneArmorTrimSmithingTemplate |
        ItemId::EyeArmorTrimSmithingTemplate |
        ItemId::HostArmorTrimSmithingTemplate |
        ItemId::RaiserArmorTrimSmithingTemplate |
        ItemId::RibArmorTrimSmithingTemplate |
        ItemId::SentryArmorTrimSmithingTemplate |
        ItemId::ShaperArmorTrimSmithingTemplate |
        ItemId::SilenceArmorTrimSmithingTemplate |
        ItemId::SnoutArmorTrimSmithingTemplate |
        ItemId::SpireArmorTrimSmithingTemplate |
        ItemId::TideArmorTrimSmithingTemplate |
        ItemId::VexArmorTrimSmithingTemplate |
        ItemId::WardArmorTrimSmithingTemplate |
        ItemId::WayfinderArmorTrimSmithingTemplate |
        ItemId::WildArmorTrimSmithingTemplate
    )
}

pub fn is_trim_material(item: ItemId) -> bool {
    matches!(
        item,
        ItemId::AmethystShard |
        ItemId::CopperIngot |
        ItemId::Diamond |
        ItemId::Emerald |
        ItemId::GoldIngot |
        ItemId::IronIngot |
        ItemId::LapisLazuli |
        ItemId::NetheriteIngot |
        ItemId::Quartz |
        ItemId::Redstone
    )
}
