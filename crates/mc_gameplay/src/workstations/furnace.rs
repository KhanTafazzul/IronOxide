//! Furnace, Blast Furnace, and Smoker tile entities with 20 Hz ticking logic and fuel burn engine.
//! REA Reference: net.minecraft.world.level.block.entity.AbstractFurnaceBlockEntity (dgk.class)

use mc_crafting::ItemStack;
use mc_items::ItemId;

/// Category of furnace workstation.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FurnaceType {
    /// Standard Furnace (all items, 200 ticks standard cook time)
    Furnace,
    /// Blast Furnace (ores and metals only, 100 ticks / 2x speed)
    BlastFurnace,
    /// Smoker (food items only, 100 ticks / 2x speed)
    Smoker,
}

impl FurnaceType {
    /// Standard cooking duration in ticks (200 for standard, 100 for blast furnace & smoker).
    pub const fn default_cook_time(&self) -> u32 {
        match self {
            FurnaceType::Furnace => 200,
            FurnaceType::BlastFurnace | FurnaceType::Smoker => 100,
        }
    }
}

/// Smelting recipe definition.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SmeltingRecipe {
    pub input: ItemId,
    pub output: ItemId,
    pub experience: f32,
    pub cook_time: u32,
    pub is_blasting: bool,
    pub is_smoking: bool,
}

impl SmeltingRecipe {
    pub const fn new(
        input: ItemId,
        output: ItemId,
        experience: f32,
        cook_time: u32,
        is_blasting: bool,
        is_smoking: bool,
    ) -> Self {
        Self {
            input,
            output,
            experience,
            cook_time,
            is_blasting,
            is_smoking,
        }
    }
}

/// Returns the burn duration in ticks for any standard Minecraft fuel item.
/// REA Reference: dgk.f() in client.jar.
pub fn get_fuel_burn_ticks(item: ItemId) -> u32 {
    match item {
        // Lava bucket: 20,000 ticks (smelts 100 items)
        ItemId::LavaBucket => 20000,
        // Coal block: 16,000 ticks (smelts 80 items)
        ItemId::CoalBlock => 16000,
        // Blaze rod: 2,400 ticks (smelts 12 items)
        ItemId::BlazeRod => 2400,
        // Coal & Charcoal: 1,600 ticks (smelts 8 items)
        ItemId::Coal | ItemId::Charcoal => 1600,
        // Wood logs, wood, stripped wood: 300 ticks (smelts 1.5 items)
        ItemId::OakLog | ItemId::SpruceLog | ItemId::BirchLog | ItemId::JungleLog |
        ItemId::AcaciaLog | ItemId::DarkOakLog | ItemId::MangroveLog | ItemId::CherryLog |
        ItemId::OakWood | ItemId::SpruceWood | ItemId::BirchWood | ItemId::JungleWood |
        ItemId::AcaciaWood | ItemId::DarkOakWood | ItemId::MangroveWood | ItemId::CherryWood |
        ItemId::StrippedOakLog | ItemId::StrippedSpruceLog | ItemId::StrippedBirchLog |
        ItemId::StrippedJungleLog | ItemId::StrippedAcaciaLog | ItemId::StrippedDarkOakLog |
        ItemId::StrippedMangroveLog | ItemId::StrippedCherryLog |
        ItemId::StrippedOakWood | ItemId::StrippedSpruceWood | ItemId::StrippedBirchWood |
        ItemId::StrippedJungleWood | ItemId::StrippedAcaciaWood | ItemId::StrippedDarkOakWood |
        ItemId::StrippedMangroveWood | ItemId::StrippedCherryWood => 300,
        // Planks, crafting table, chest, wooden doors, fences: 300 ticks
        ItemId::OakPlanks | ItemId::SprucePlanks | ItemId::BirchPlanks | ItemId::JunglePlanks |
        ItemId::AcaciaPlanks | ItemId::DarkOakPlanks | ItemId::MangrovePlanks | ItemId::CherryPlanks |
        ItemId::BambooPlanks | ItemId::CraftingTable | ItemId::Chest | ItemId::TrappedChest |
        ItemId::Barrel | ItemId::Bookshelf | ItemId::ChiseledBookshelf |
        ItemId::OakFence | ItemId::SpruceFence | ItemId::BirchFence | ItemId::JungleFence |
        ItemId::AcaciaFence | ItemId::DarkOakFence | ItemId::MangroveFence | ItemId::CherryFence |
        ItemId::OakDoor | ItemId::SpruceDoor | ItemId::BirchDoor | ItemId::JungleDoor |
        ItemId::AcaciaDoor | ItemId::DarkOakDoor | ItemId::MangroveDoor | ItemId::CherryDoor |
        ItemId::OakTrapdoor | ItemId::SpruceTrapdoor | ItemId::BirchTrapdoor | ItemId::JungleTrapdoor |
        ItemId::AcaciaTrapdoor | ItemId::DarkOakTrapdoor | ItemId::MangroveTrapdoor | ItemId::CherryTrapdoor |
        ItemId::OakSign | ItemId::SpruceSign | ItemId::BirchSign | ItemId::JungleSign |
        ItemId::AcaciaSign | ItemId::DarkOakSign | ItemId::MangroveSign | ItemId::CherrySign |
        ItemId::OakBoat | ItemId::SpruceBoat | ItemId::BirchBoat | ItemId::JungleBoat |
        ItemId::AcaciaBoat | ItemId::DarkOakBoat | ItemId::MangroveBoat | ItemId::CherryBoat => 300,
        // Wooden slabs: 150 ticks
        ItemId::OakSlab | ItemId::SpruceSlab | ItemId::BirchSlab | ItemId::JungleSlab |
        ItemId::AcaciaSlab | ItemId::DarkOakSlab | ItemId::MangroveSlab | ItemId::CherrySlab |
        ItemId::BambooSlab => 150,
        // Sticks, saplings, bamboo: 100 ticks
        ItemId::Stick | ItemId::Bamboo |
        ItemId::OakSapling | ItemId::SpruceSapling | ItemId::BirchSapling | ItemId::JungleSapling |
        ItemId::AcaciaSapling | ItemId::DarkOakSapling | ItemId::CherrySapling => 100,
        _ => 0,
    }
}

/// Lookup for canonical smelting recipes.
pub fn get_smelting_recipe(input: ItemId, furnace_type: FurnaceType) -> Option<SmeltingRecipe> {
    let recipe = match input {
        // Raw ores & ores (Furnace & Blast Furnace)
        ItemId::RawIron | ItemId::IronOre | ItemId::DeepslateIronOre => {
            Some(SmeltingRecipe::new(input, ItemId::IronIngot, 0.7, 200, true, false))
        }
        ItemId::RawGold | ItemId::GoldOre | ItemId::DeepslateGoldOre | ItemId::NetherGoldOre => {
            Some(SmeltingRecipe::new(input, ItemId::GoldIngot, 1.0, 200, true, false))
        }
        ItemId::RawCopper | ItemId::CopperOre | ItemId::DeepslateCopperOre => {
            Some(SmeltingRecipe::new(input, ItemId::CopperIngot, 0.7, 200, true, false))
        }
        ItemId::AncientDebris => {
            Some(SmeltingRecipe::new(input, ItemId::NetheriteScrap, 2.0, 200, true, false))
        }
        // Blocks & building materials (Standard Furnace only)
        ItemId::Cobblestone => {
            Some(SmeltingRecipe::new(input, ItemId::Stone, 0.1, 200, false, false))
        }
        ItemId::Stone => {
            Some(SmeltingRecipe::new(input, ItemId::SmoothStone, 0.1, 200, false, false))
        }
        ItemId::Sand | ItemId::RedSand => {
            Some(SmeltingRecipe::new(input, ItemId::Glass, 0.1, 200, false, false))
        }
        ItemId::ClayBall => {
            Some(SmeltingRecipe::new(input, ItemId::Brick, 0.3, 200, false, false))
        }
        ItemId::Clay => {
            Some(SmeltingRecipe::new(input, ItemId::Terracotta, 0.35, 200, false, false))
        }
        ItemId::Cactus => {
            Some(SmeltingRecipe::new(input, ItemId::GreenDye, 1.0, 200, false, false))
        }
        // Logs -> Charcoal (Standard Furnace only)
        ItemId::OakLog | ItemId::SpruceLog | ItemId::BirchLog | ItemId::JungleLog |
        ItemId::AcaciaLog | ItemId::DarkOakLog | ItemId::MangroveLog | ItemId::CherryLog |
        ItemId::StrippedOakLog | ItemId::StrippedSpruceLog | ItemId::StrippedBirchLog |
        ItemId::StrippedJungleLog | ItemId::StrippedAcaciaLog | ItemId::StrippedDarkOakLog |
        ItemId::StrippedMangroveLog | ItemId::StrippedCherryLog => {
            Some(SmeltingRecipe::new(input, ItemId::Charcoal, 0.15, 200, false, false))
        }
        // Foods (Furnace & Smoker)
        ItemId::Beef => {
            Some(SmeltingRecipe::new(input, ItemId::CookedBeef, 0.35, 200, false, true))
        }
        ItemId::Porkchop => {
            Some(SmeltingRecipe::new(input, ItemId::CookedPorkchop, 0.35, 200, false, true))
        }
        ItemId::Mutton => {
            Some(SmeltingRecipe::new(input, ItemId::CookedMutton, 0.35, 200, false, true))
        }
        ItemId::Chicken => {
            Some(SmeltingRecipe::new(input, ItemId::CookedChicken, 0.35, 200, false, true))
        }
        ItemId::Cod => {
            Some(SmeltingRecipe::new(input, ItemId::CookedCod, 0.35, 200, false, true))
        }
        ItemId::Salmon => {
            Some(SmeltingRecipe::new(input, ItemId::CookedSalmon, 0.35, 200, false, true))
        }
        ItemId::Potato => {
            Some(SmeltingRecipe::new(input, ItemId::BakedPotato, 0.35, 200, false, true))
        }
        ItemId::Kelp => {
            Some(SmeltingRecipe::new(input, ItemId::DriedKelp, 0.1, 200, false, true))
        }
        _ => None,
    }?;

    match furnace_type {
        FurnaceType::Furnace => Some(recipe),
        FurnaceType::BlastFurnace => {
            if recipe.is_blasting {
                Some(SmeltingRecipe {
                    cook_time: recipe.cook_time / 2,
                    ..recipe
                })
            } else {
                None
            }
        }
        FurnaceType::Smoker => {
            if recipe.is_smoking {
                Some(SmeltingRecipe {
                    cook_time: recipe.cook_time / 2,
                    ..recipe
                })
            } else {
                None
            }
        }
    }
}

/// 20 Hz ticking Furnace tile entity.
/// REA Reference: net.minecraft.world.level.block.entity.AbstractFurnaceBlockEntity (dgk.class).
#[derive(Clone, Debug, PartialEq)]
pub struct FurnaceTileEntity {
    pub furnace_type: FurnaceType,
    /// Slot 0: Input item stack to be smelted
    pub input: Option<ItemStack>,
    /// Slot 1: Fuel item stack
    pub fuel: Option<ItemStack>,
    /// Slot 2: Output smelted item stack
    pub output: Option<ItemStack>,

    /// Remaining burn ticks for current active fuel (0 when unlit)
    pub lit_time: u32,
    /// Initial total burn duration of the currently burning fuel item
    pub lit_duration: u32,
    /// Current cooking progress ticks towards completing the current item
    pub cooking_progress: u32,
    /// Total ticks required to smelt the current item
    pub cooking_total_time: u32,
    /// Cumulative experience accumulated from completed smelting
    pub stored_experience: f32,
}

impl FurnaceTileEntity {
    pub fn new(furnace_type: FurnaceType) -> Self {
        let cook_time = furnace_type.default_cook_time();
        Self {
            furnace_type,
            input: None,
            fuel: None,
            output: None,
            lit_time: 0,
            lit_duration: 0,
            cooking_progress: 0,
            cooking_total_time: cook_time,
            stored_experience: 0.0,
        }
    }

    /// Whether the furnace is actively burning fuel.
    #[inline]
    pub fn is_lit(&self) -> bool {
        self.lit_time > 0
    }

    /// Returns the normalized burn progress ratio for rendering the flame icon (0.0 to 1.0).
    pub fn lit_ratio(&self) -> f32 {
        if self.lit_duration > 0 && self.lit_time > 0 {
            self.lit_time as f32 / self.lit_duration as f32
        } else {
            0.0
        }
    }

    /// Returns the normalized cook progress ratio for rendering the arrow progress bar (0.0 to 1.0).
    pub fn cook_ratio(&self) -> f32 {
        if self.cooking_total_time > 0 {
            (self.cooking_progress as f32 / self.cooking_total_time as f32).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Primary 20 Hz tick function matching vanilla AbstractFurnaceBlockEntity.serverTick (dgk.a).
    /// Returns true if furnace lit state or inventory contents changed.
    pub fn tick(&mut self) -> bool {
        let mut state_changed = false;
        let was_lit = self.is_lit();

        if self.lit_time > 0 {
            self.lit_time -= 1;
        }

        let recipe = self.input.and_then(|input_stack| {
            get_smelting_recipe(input_stack.item, self.furnace_type)
        });

        let can_smelt = if let Some(rec) = recipe {
            self.can_output_accept(rec.output)
        } else {
            false
        };

        // If not currently lit but can smelt, ignite fuel if available
        if !self.is_lit() && can_smelt {
            if let Some(fuel_stack) = self.fuel {
                let burn_ticks = get_fuel_burn_ticks(fuel_stack.item);
                if burn_ticks > 0 {
                    self.lit_time = burn_ticks;
                    self.lit_duration = burn_ticks;

                    // Consume fuel: special case for Lava Bucket -> leaves empty Bucket
                    if fuel_stack.item == ItemId::LavaBucket {
                        self.fuel = Some(ItemStack::new(ItemId::Bucket, 1));
                    } else if fuel_stack.count <= 1 {
                        self.fuel = None;
                    } else {
                        self.fuel = Some(ItemStack::new(fuel_stack.item, fuel_stack.count - 1));
                    }
                    state_changed = true;
                }
            }
        }

        // Cooking progression
        if self.is_lit() && can_smelt {
            let rec = recipe.unwrap();
            self.cooking_total_time = rec.cook_time;
            self.cooking_progress += 1;

            if self.cooking_progress >= self.cooking_total_time {
                self.cooking_progress = 0;
                self.smelt_item(rec);
                state_changed = true;
            }
        } else {
            // When not actively smelting, cook progress decays by 2 per tick
            if self.cooking_progress > 0 {
                self.cooking_progress = self.cooking_progress.saturating_sub(2);
            }
        }

        if was_lit != self.is_lit() {
            state_changed = true;
        }

        state_changed
    }

    fn can_output_accept(&self, result_item: ItemId) -> bool {
        match self.output {
            None => true,
            Some(out) => {
                out.item == result_item && out.count < result_item.max_stack_size()
            }
        }
    }

    fn smelt_item(&mut self, recipe: SmeltingRecipe) {
        // 1. Decrement input item
        if let Some(input_stack) = &mut self.input {
            if input_stack.count <= 1 {
                self.input = None;
            } else {
                input_stack.count -= 1;
            }
        }

        // 2. Increment output slot
        match &mut self.output {
            None => {
                self.output = Some(ItemStack::new(recipe.output, 1));
            }
            Some(out_stack) => {
                out_stack.count = (out_stack.count + 1).min(recipe.output.max_stack_size());
            }
        }

        // 3. Accumulate experience
        self.stored_experience += recipe.experience;
    }

    /// Claims and resets stored experience when player retrieves items from output slot.
    pub fn take_experience(&mut self) -> f32 {
        let xp = self.stored_experience;
        self.stored_experience = 0.0;
        xp
    }
}
