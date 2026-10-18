//! Biome registry mapping all vanilla Minecraft 1.20.4 Overworld biomes.
//! REA Reference: net.minecraft.world.level.biome.Biomes (cuv.class)
//! REA Reference: net.minecraft.world.level.biome.OverworldBiomeBuilder (cve.class)

/// Canonical enumeration of all vanilla Overworld biomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiomeId {
    // --- Snowy / Cold Biomes ---
    SnowyPlains,
    IceSpikes,
    SnowyTaiga,
    FrozenPeaks,
    JaggedPeaks,
    Grove,
    SnowySlopes,
    FrozenRiver,
    SnowyBeach,

    // --- Temperate / Wet Biomes ---
    Plains,
    SunflowerPlains,
    Forest,
    FlowerForest,
    BirchForest,
    OldGrowthBirchForest,
    DarkForest,
    Taiga,
    OldGrowthPineTaiga,
    OldGrowthSpruceTaiga,
    Swamp,
    MangroveSwamp,
    Jungle,
    SparseJungle,
    BambooJungle,
    River,
    Beach,
    StonyShore,
    Meadow,
    CherryGrove,

    // --- Arid / Hot Biomes ---
    Desert,
    Savanna,
    SavannaPlateau,
    WindsweptSavanna,
    WindsweptHills,
    WindsweptGravellyHills,
    WindsweptForest,
    Badlands,
    ErodedBadlands,
    WoodedBadlands,
    StonyPeaks,

    // --- Ocean Variants ---
    WarmOcean,
    LukewarmOcean,
    DeepLukewarmOcean,
    Ocean,
    DeepOcean,
    ColdOcean,
    DeepColdOcean,
    FrozenOcean,
    DeepFrozenOcean,

    // --- Rare / Underground ---
    MushroomFields,
    DripstoneCaves,
    LushCaves,
    DeepDark,
}

/// Broad ecological category classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiomeCategory {
    Snowy,
    Cold,
    Temperate,
    Wet,
    Arid,
    Ocean,
    Mountain,
    Underground,
}

impl BiomeId {
    /// Canonical Minecraft resource location identifier string.
    pub const fn name(&self) -> &'static str {
        match self {
            // Snowy / Cold
            BiomeId::SnowyPlains => "minecraft:snowy_plains",
            BiomeId::IceSpikes => "minecraft:ice_spikes",
            BiomeId::SnowyTaiga => "minecraft:snowy_taiga",
            BiomeId::FrozenPeaks => "minecraft:frozen_peaks",
            BiomeId::JaggedPeaks => "minecraft:jagged_peaks",
            BiomeId::Grove => "minecraft:grove",
            BiomeId::SnowySlopes => "minecraft:snowy_slopes",
            BiomeId::FrozenRiver => "minecraft:frozen_river",
            BiomeId::SnowyBeach => "minecraft:snowy_beach",

            // Temperate / Wet
            BiomeId::Plains => "minecraft:plains",
            BiomeId::SunflowerPlains => "minecraft:sunflower_plains",
            BiomeId::Forest => "minecraft:forest",
            BiomeId::FlowerForest => "minecraft:flower_forest",
            BiomeId::BirchForest => "minecraft:birch_forest",
            BiomeId::OldGrowthBirchForest => "minecraft:old_growth_birch_forest",
            BiomeId::DarkForest => "minecraft:dark_forest",
            BiomeId::Taiga => "minecraft:taiga",
            BiomeId::OldGrowthPineTaiga => "minecraft:old_growth_pine_taiga",
            BiomeId::OldGrowthSpruceTaiga => "minecraft:old_growth_spruce_taiga",
            BiomeId::Swamp => "minecraft:swamp",
            BiomeId::MangroveSwamp => "minecraft:mangrove_swamp",
            BiomeId::Jungle => "minecraft:jungle",
            BiomeId::SparseJungle => "minecraft:sparse_jungle",
            BiomeId::BambooJungle => "minecraft:bamboo_jungle",
            BiomeId::River => "minecraft:river",
            BiomeId::Beach => "minecraft:beach",
            BiomeId::StonyShore => "minecraft:stony_shore",
            BiomeId::Meadow => "minecraft:meadow",
            BiomeId::CherryGrove => "minecraft:cherry_grove",

            // Arid / Hot
            BiomeId::Desert => "minecraft:desert",
            BiomeId::Savanna => "minecraft:savanna",
            BiomeId::SavannaPlateau => "minecraft:savanna_plateau",
            BiomeId::WindsweptSavanna => "minecraft:windswept_savanna",
            BiomeId::WindsweptHills => "minecraft:windswept_hills",
            BiomeId::WindsweptGravellyHills => "minecraft:windswept_gravelly_hills",
            BiomeId::WindsweptForest => "minecraft:windswept_forest",
            BiomeId::Badlands => "minecraft:badlands",
            BiomeId::ErodedBadlands => "minecraft:eroded_badlands",
            BiomeId::WoodedBadlands => "minecraft:wooded_badlands",
            BiomeId::StonyPeaks => "minecraft:stony_peaks",

            // Ocean Variants
            BiomeId::WarmOcean => "minecraft:warm_ocean",
            BiomeId::LukewarmOcean => "minecraft:lukewarm_ocean",
            BiomeId::DeepLukewarmOcean => "minecraft:deep_lukewarm_ocean",
            BiomeId::Ocean => "minecraft:ocean",
            BiomeId::DeepOcean => "minecraft:deep_ocean",
            BiomeId::ColdOcean => "minecraft:cold_ocean",
            BiomeId::DeepColdOcean => "minecraft:deep_cold_ocean",
            BiomeId::FrozenOcean => "minecraft:frozen_ocean",
            BiomeId::DeepFrozenOcean => "minecraft:deep_frozen_ocean",

            // Rare / Underground
            BiomeId::MushroomFields => "minecraft:mushroom_fields",
            BiomeId::DripstoneCaves => "minecraft:dripstone_caves",
            BiomeId::LushCaves => "minecraft:lush_caves",
            BiomeId::DeepDark => "minecraft:deep_dark",
        }
    }

    /// Resolves a resource location name (with or without 'minecraft:' prefix) into a BiomeId.
    pub fn from_name(name: &str) -> Option<Self> {
        let clean = name.strip_prefix("minecraft:").unwrap_or(name);
        match clean {
            // Snowy / Cold
            "snowy_plains" => Some(BiomeId::SnowyPlains),
            "ice_spikes" => Some(BiomeId::IceSpikes),
            "snowy_taiga" => Some(BiomeId::SnowyTaiga),
            "frozen_peaks" => Some(BiomeId::FrozenPeaks),
            "jagged_peaks" => Some(BiomeId::JaggedPeaks),
            "grove" => Some(BiomeId::Grove),
            "snowy_slopes" => Some(BiomeId::SnowySlopes),
            "frozen_river" => Some(BiomeId::FrozenRiver),
            "snowy_beach" => Some(BiomeId::SnowyBeach),

            // Temperate / Wet
            "plains" => Some(BiomeId::Plains),
            "sunflower_plains" => Some(BiomeId::SunflowerPlains),
            "forest" => Some(BiomeId::Forest),
            "flower_forest" => Some(BiomeId::FlowerForest),
            "birch_forest" => Some(BiomeId::BirchForest),
            "old_growth_birch_forest" => Some(BiomeId::OldGrowthBirchForest),
            "dark_forest" => Some(BiomeId::DarkForest),
            "taiga" => Some(BiomeId::Taiga),
            "old_growth_pine_taiga" => Some(BiomeId::OldGrowthPineTaiga),
            "old_growth_spruce_taiga" => Some(BiomeId::OldGrowthSpruceTaiga),
            "swamp" => Some(BiomeId::Swamp),
            "mangrove_swamp" => Some(BiomeId::MangroveSwamp),
            "jungle" => Some(BiomeId::Jungle),
            "sparse_jungle" => Some(BiomeId::SparseJungle),
            "bamboo_jungle" => Some(BiomeId::BambooJungle),
            "river" => Some(BiomeId::River),
            "beach" => Some(BiomeId::Beach),
            "stony_shore" => Some(BiomeId::StonyShore),
            "meadow" => Some(BiomeId::Meadow),
            "cherry_grove" => Some(BiomeId::CherryGrove),

            // Arid / Hot
            "desert" => Some(BiomeId::Desert),
            "savanna" => Some(BiomeId::Savanna),
            "savanna_plateau" => Some(BiomeId::SavannaPlateau),
            "windswept_savanna" => Some(BiomeId::WindsweptSavanna),
            "windswept_hills" => Some(BiomeId::WindsweptHills),
            "windswept_gravelly_hills" => Some(BiomeId::WindsweptGravellyHills),
            "windswept_forest" => Some(BiomeId::WindsweptForest),
            "badlands" => Some(BiomeId::Badlands),
            "eroded_badlands" => Some(BiomeId::ErodedBadlands),
            "wooded_badlands" => Some(BiomeId::WoodedBadlands),
            "stony_peaks" => Some(BiomeId::StonyPeaks),

            // Ocean Variants
            "warm_ocean" => Some(BiomeId::WarmOcean),
            "lukewarm_ocean" => Some(BiomeId::LukewarmOcean),
            "deep_lukewarm_ocean" => Some(BiomeId::DeepLukewarmOcean),
            "ocean" => Some(BiomeId::Ocean),
            "deep_ocean" => Some(BiomeId::DeepOcean),
            "cold_ocean" => Some(BiomeId::ColdOcean),
            "deep_cold_ocean" => Some(BiomeId::DeepColdOcean),
            "frozen_ocean" => Some(BiomeId::FrozenOcean),
            "deep_frozen_ocean" => Some(BiomeId::DeepFrozenOcean),

            // Rare / Underground
            "mushroom_fields" => Some(BiomeId::MushroomFields),
            "dripstone_caves" => Some(BiomeId::DripstoneCaves),
            "lush_caves" => Some(BiomeId::LushCaves),
            "deep_dark" => Some(BiomeId::DeepDark),

            _ => None,
        }
    }

    /// Primary ecological category.
    pub const fn category(&self) -> BiomeCategory {
        match self {
            BiomeId::SnowyPlains
            | BiomeId::IceSpikes
            | BiomeId::SnowyTaiga
            | BiomeId::FrozenRiver
            | BiomeId::SnowyBeach => BiomeCategory::Snowy,

            BiomeId::FrozenPeaks
            | BiomeId::JaggedPeaks
            | BiomeId::Grove
            | BiomeId::SnowySlopes
            | BiomeId::ColdOcean
            | BiomeId::DeepColdOcean
            | BiomeId::FrozenOcean
            | BiomeId::DeepFrozenOcean => BiomeCategory::Cold,

            BiomeId::Plains
            | BiomeId::SunflowerPlains
            | BiomeId::Forest
            | BiomeId::FlowerForest
            | BiomeId::BirchForest
            | BiomeId::OldGrowthBirchForest
            | BiomeId::DarkForest
            | BiomeId::Taiga
            | BiomeId::OldGrowthPineTaiga
            | BiomeId::OldGrowthSpruceTaiga
            | BiomeId::River
            | BiomeId::Beach
            | BiomeId::Meadow
            | BiomeId::CherryGrove => BiomeCategory::Temperate,

            BiomeId::Swamp
            | BiomeId::MangroveSwamp
            | BiomeId::Jungle
            | BiomeId::SparseJungle
            | BiomeId::BambooJungle => BiomeCategory::Wet,

            BiomeId::Desert
            | BiomeId::Savanna
            | BiomeId::SavannaPlateau
            | BiomeId::WindsweptSavanna
            | BiomeId::Badlands
            | BiomeId::ErodedBadlands
            | BiomeId::WoodedBadlands => BiomeCategory::Arid,

            BiomeId::WarmOcean
            | BiomeId::LukewarmOcean
            | BiomeId::DeepLukewarmOcean
            | BiomeId::Ocean
            | BiomeId::DeepOcean => BiomeCategory::Ocean,

            BiomeId::WindsweptHills
            | BiomeId::WindsweptGravellyHills
            | BiomeId::WindsweptForest
            | BiomeId::StonyPeaks
            | BiomeId::StonyShore => BiomeCategory::Mountain,

            BiomeId::MushroomFields
            | BiomeId::DripstoneCaves
            | BiomeId::LushCaves
            | BiomeId::DeepDark => BiomeCategory::Underground,
        }
    }

    /// Vanilla base temperature used for weather and precipitation calculations.
    pub const fn temperature(&self) -> f32 {
        match self {
            BiomeId::FrozenPeaks | BiomeId::JaggedPeaks => -0.7,
            BiomeId::SnowySlopes => -0.3,
            BiomeId::Grove => -0.2,
            BiomeId::SnowyPlains | BiomeId::IceSpikes | BiomeId::FrozenRiver | BiomeId::SnowyBeach => 0.0,
            BiomeId::SnowyTaiga => -0.5,
            BiomeId::Taiga | BiomeId::OldGrowthPineTaiga | BiomeId::OldGrowthSpruceTaiga => 0.25,
            BiomeId::ColdOcean | BiomeId::DeepColdOcean => 0.5,
            BiomeId::FrozenOcean | BiomeId::DeepFrozenOcean => 0.0,
            BiomeId::Plains | BiomeId::SunflowerPlains | BiomeId::DarkForest => 0.8,
            BiomeId::Forest | BiomeId::FlowerForest => 0.7,
            BiomeId::BirchForest | BiomeId::OldGrowthBirchForest => 0.6,
            BiomeId::Meadow => 0.5,
            BiomeId::CherryGrove => 0.5,
            BiomeId::Swamp | BiomeId::MangroveSwamp => 0.8,
            BiomeId::Jungle | BiomeId::SparseJungle | BiomeId::BambooJungle => 0.95,
            BiomeId::Savanna | BiomeId::SavannaPlateau | BiomeId::WindsweptSavanna => 1.2,
            BiomeId::Desert | BiomeId::Badlands | BiomeId::ErodedBadlands | BiomeId::WoodedBadlands => 2.0,
            _ => 0.5,
        }
    }

    /// Whether this biome has precipitation (rain or snow).
    pub const fn has_precipitation(&self) -> bool {
        match self {
            BiomeId::Desert
            | BiomeId::Badlands
            | BiomeId::ErodedBadlands
            | BiomeId::WoodedBadlands
            | BiomeId::Savanna
            | BiomeId::SavannaPlateau
            | BiomeId::WindsweptSavanna => false,
            _ => true,
        }
    }

    /// Whether precipitation in this biome falls as snow at sea level.
    pub const fn is_snowy(&self) -> bool {
        match self {
            BiomeId::SnowyPlains
            | BiomeId::IceSpikes
            | BiomeId::SnowyTaiga
            | BiomeId::FrozenPeaks
            | BiomeId::JaggedPeaks
            | BiomeId::Grove
            | BiomeId::SnowySlopes
            | BiomeId::FrozenRiver
            | BiomeId::SnowyBeach => true,
            _ => false,
        }
    }

    /// Whether this biome is a Badlands variant.
    pub const fn is_badlands(&self) -> bool {
        matches!(
            self,
            BiomeId::Badlands | BiomeId::ErodedBadlands | BiomeId::WoodedBadlands
        )
    }

    /// Whether this biome is desert or sandy beach.
    pub const fn is_desert_or_beach(&self) -> bool {
        matches!(self, BiomeId::Desert | BiomeId::Beach | BiomeId::WarmOcean)
    }

    /// Whether this biome is an ocean.
    pub const fn is_ocean(&self) -> bool {
        matches!(
            self,
            BiomeId::WarmOcean
                | BiomeId::LukewarmOcean
                | BiomeId::DeepLukewarmOcean
                | BiomeId::Ocean
                | BiomeId::DeepOcean
                | BiomeId::ColdOcean
                | BiomeId::DeepColdOcean
                | BiomeId::FrozenOcean
                | BiomeId::DeepFrozenOcean
        )
    }
}
