//! Anvil workstation with repair, enchantment merging, renaming, and prior work penalty.
//! REA Reference: net.minecraft.world.inventory.AnvilMenu (cid.class)

use mc_items::ItemId;

/// Enchantment instance with identifier and level.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EnchantmentInstance {
    pub id: String,
    pub level: u32,
    pub max_level: u32,
    pub rarity_multiplier: u32,
}

impl EnchantmentInstance {
    pub fn new(id: impl Into<String>, level: u32, max_level: u32, rarity_multiplier: u32) -> Self {
        Self {
            id: id.into(),
            level,
            max_level,
            rarity_multiplier,
        }
    }
}

/// Anvil item representation holding durability damage, prior work penalty, and custom name.
#[derive(Clone, PartialEq, Debug)]
pub struct AnvilItem {
    pub item: ItemId,
    pub count: u8,
    pub damage: u32,
    /// Vanilla repairCost / prior work penalty: p = 2^n - 1
    pub prior_work_penalty: u32,
    pub custom_name: Option<String>,
    pub enchantments: Vec<EnchantmentInstance>,
}

impl AnvilItem {
    pub fn new(item: ItemId) -> Self {
        Self {
            item,
            count: 1,
            damage: 0,
            prior_work_penalty: 0,
            custom_name: None,
            enchantments: Vec::new(),
        }
    }

    pub fn with_damage(mut self, damage: u32) -> Self {
        self.damage = damage;
        self
    }

    pub fn with_prior_work(mut self, penalty: u32) -> Self {
        self.prior_work_penalty = penalty;
        self
    }

    pub fn with_custom_name(mut self, name: impl Into<String>) -> Self {
        self.custom_name = Some(name.into());
        self
    }

    pub fn with_enchantment(mut self, ench: EnchantmentInstance) -> Self {
        self.enchantments.push(ench);
        self
    }

    /// Max durability for tools/armor, or 0 if non-durable.
    pub fn max_durability(&self) -> u32 {
        if let Some((tier, _)) = self.item.tool_info() {
            tier.durability()
        } else if let Some((tier, slot)) = self.item.armor_info() {
            tier.durability(slot)
        } else {
            0
        }
    }
}

/// Calculates the next prior work penalty from the previous penalty.
/// Exact formula from AnvilMenu.calculateIncreasedRepairCost (cid.e):
/// p_next = p * 2 + 1, equivalent to 2^(n+1) - 1.
#[inline]
pub fn calculate_next_prior_work_penalty(prior_work: u32) -> u32 {
    prior_work.saturating_mul(2).saturating_add(1)
}

/// Calculates the repair material for a given tool or armor piece.
pub fn get_repair_material(item: ItemId) -> Option<ItemId> {
    if let Some((tier, _)) = item.tool_info() {
        match tier {
            mc_items::ToolTier::Wood => Some(ItemId::OakPlanks),
            mc_items::ToolTier::Stone => Some(ItemId::Cobblestone),
            mc_items::ToolTier::Iron => Some(ItemId::IronIngot),
            mc_items::ToolTier::Diamond => Some(ItemId::Diamond),
            mc_items::ToolTier::Gold => Some(ItemId::GoldIngot),
            mc_items::ToolTier::Netherite => Some(ItemId::NetheriteIngot),
        }
    } else if let Some((tier, _)) = item.armor_info() {
        match tier {
            mc_items::ArmorTier::Leather => Some(ItemId::Leather),
            mc_items::ArmorTier::Iron => Some(ItemId::IronIngot),
            mc_items::ArmorTier::Gold => Some(ItemId::GoldIngot),
            mc_items::ArmorTier::Diamond => Some(ItemId::Diamond),
            mc_items::ArmorTier::Netherite => Some(ItemId::NetheriteIngot),
            mc_items::ArmorTier::Turtle => Some(ItemId::Scute),
            mc_items::ArmorTier::Chainmail => Some(ItemId::IronIngot),
        }
    } else {
        None
    }
}

/// Anvil degradation state.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum AnvilDamageState {
    Intact,
    Chipped,
    Damaged,
    Broken,
}

/// Result of evaluating an Anvil recipe.
#[derive(Clone, PartialEq, Debug)]
pub struct AnvilResult {
    pub output: AnvilItem,
    /// Total experience level cost
    pub level_cost: u32,
    /// Base cost before prior work penalties
    pub base_cost: u32,
    /// Material count consumed from right slot (if unit repair was used)
    pub material_consumed: u8,
    /// Whether cost exceeds survival cap (>= 40 levels)
    pub too_expensive: bool,
}

/// Anvil workstation container menu.
/// REA Reference: net.minecraft.world.inventory.AnvilMenu (cid.class)
pub struct AnvilMenu {
    pub left_slot: Option<AnvilItem>,
    pub right_slot: Option<AnvilItem>,
    pub custom_name_input: Option<String>,
    pub anvil_state: AnvilDamageState,
}

impl AnvilMenu {
    pub fn new() -> Self {
        Self {
            left_slot: None,
            right_slot: None,
            custom_name_input: None,
            anvil_state: AnvilDamageState::Intact,
        }
    }

    /// Evaluates the anvil operation and computes the output and experience level cost.
    /// Implements exact logic from cid.m() in client.jar.
    pub fn compute_result(&self) -> Option<AnvilResult> {
        let left = self.left_slot.as_ref()?;
        let mut base_cost = 0u32;
        let mut material_consumed = 0u8;

        let max_dur = left.max_durability();
        let mut result_item = left.clone();

        // 1. Check Rename
        let is_renaming = match (&self.custom_name_input, &left.custom_name) {
            (Some(new_name), Some(old_name)) => new_name != old_name && !new_name.is_empty(),
            (Some(new_name), None) => !new_name.is_empty(),
            _ => false,
        };

        if is_renaming {
            base_cost += 1;
            result_item.custom_name = self.custom_name_input.clone();
        }

        // 2. Right slot interactions
        if let Some(right) = self.right_slot.as_ref() {
            let repair_material = get_repair_material(left.item);

            // Case A: Unit material repair (e.g. Iron Ingot on Iron Pickaxe)
            if repair_material == Some(right.item) && left.damage > 0 {
                let repair_per_unit = max_dur / 4;
                let mut current_damage = left.damage;
                let mut units_used = 0u8;

                while current_damage > 0 && units_used < right.count {
                    current_damage = current_damage.saturating_sub(repair_per_unit);
                    units_used += 1;
                    base_cost += 1;
                }

                result_item.damage = current_damage;
                material_consumed = units_used;
            }
            // Case B: Same item combining (e.g. two Diamond Swords)
            else if left.item == right.item && max_dur > 0 {
                // Vanilla durability bonus: 12% of max damage
                let bonus = (max_dur * 12) / 100;
                let remaining_right = max_dur.saturating_sub(right.damage);
                let repaired_damage = left.damage.saturating_sub(remaining_right + bonus);

                result_item.damage = repaired_damage;
                base_cost += 2;
                material_consumed = 1;

                // Merge enchantments
                for r_ench in &right.enchantments {
                    let mut found = false;
                    for l_ench in &mut result_item.enchantments {
                        if l_ench.id == r_ench.id {
                            found = true;
                            if l_ench.level < r_ench.level {
                                l_ench.level = r_ench.level;
                                base_cost += r_ench.level * r_ench.rarity_multiplier;
                            } else if l_ench.level == r_ench.level && l_ench.level < l_ench.max_level {
                                l_ench.level += 1;
                                base_cost += l_ench.level * l_ench.rarity_multiplier;
                            } else {
                                base_cost += 1;
                            }
                            break;
                        }
                    }
                    if !found {
                        result_item.enchantments.push(r_ench.clone());
                        base_cost += r_ench.level * r_ench.rarity_multiplier;
                    }
                }
            }
            // Case C: Enchanted Book application
            else if right.item == ItemId::EnchantedBook {
                for r_ench in &right.enchantments {
                    let mut found = false;
                    for l_ench in &mut result_item.enchantments {
                        if l_ench.id == r_ench.id {
                            found = true;
                            if l_ench.level < r_ench.level {
                                l_ench.level = r_ench.level;
                                base_cost += r_ench.level * r_ench.rarity_multiplier;
                            } else if l_ench.level == r_ench.level && l_ench.level < l_ench.max_level {
                                l_ench.level += 1;
                                base_cost += l_ench.level * l_ench.rarity_multiplier;
                            } else {
                                base_cost += 1;
                            }
                            break;
                        }
                    }
                    if !found {
                        result_item.enchantments.push(r_ench.clone());
                        base_cost += r_ench.level * r_ench.rarity_multiplier;
                    }
                }
                material_consumed = 1;
            } else {
                // Invalid combination
                if !is_renaming {
                    return None;
                }
            }
        } else if !is_renaming {
            return None;
        }

        // Prior work penalties
        let left_penalty = left.prior_work_penalty;
        let right_penalty = self.right_slot.as_ref().map_or(0, |r| r.prior_work_penalty);

        let total_level_cost = base_cost + left_penalty + right_penalty;
        let too_expensive = total_level_cost >= 40;

        // Resulting prior work penalty: max(p1, p2) * 2 + 1
        result_item.prior_work_penalty = calculate_next_prior_work_penalty(left_penalty.max(right_penalty));

        Some(AnvilResult {
            output: result_item,
            level_cost: total_level_cost,
            base_cost,
            material_consumed,
            too_expensive,
        })
    }

    /// Takes the result of the anvil operation, deducting experience levels and degrading the anvil.
    pub fn take_output(&mut self, player_levels: &mut u32, is_creative: bool) -> Option<AnvilItem> {
        let result = self.compute_result()?;

        if result.too_expensive && !is_creative {
            return None;
        }

        if !is_creative {
            if *player_levels < result.level_cost {
                return None;
            }
            *player_levels -= result.level_cost;
        }

        // Consume left slot
        self.left_slot = None;

        // Consume right slot material
        if let Some(right) = &mut self.right_slot {
            if right.count <= result.material_consumed {
                self.right_slot = None;
            } else {
                right.count -= result.material_consumed;
            }
        }

        // Clear custom name
        self.custom_name_input = None;

        Some(result.output)
    }

    /// Degrades the anvil state on use (12% chance in vanilla).
    pub fn degrade_anvil(&mut self) -> AnvilDamageState {
        self.anvil_state = match self.anvil_state {
            AnvilDamageState::Intact => AnvilDamageState::Chipped,
            AnvilDamageState::Chipped => AnvilDamageState::Damaged,
            AnvilDamageState::Damaged | AnvilDamageState::Broken => AnvilDamageState::Broken,
        };
        self.anvil_state
    }
}
