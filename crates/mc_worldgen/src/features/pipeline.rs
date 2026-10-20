//! Vanilla 11-step chunk decoration pipeline and feature dispatcher.
//! REA Reference: net.minecraft.world.level.levelgen.GenerationStep$Decoration (dom$b.class)
//! REA Reference: net.minecraft.world.level.biome.Biome.generate / ChunkGenerator.applyBiomeDecoration (dle.class)
//! REA Reference: net.minecraft.world.level.levelgen.placement.PlacedFeature (dyk.class)
//! REA Reference: net.minecraft.world.level.levelgen.placement.InSquarePlacement (dyh.class)
//! REA Reference: net.minecraft.world.level.levelgen.placement.CountPlacement (dyd.class)
//! REA Reference: net.minecraft.world.level.levelgen.placement.HeightRangePlacement (dyf.class)
//! REA Reference: net.minecraft.world.level.levelgen.placement.BiomeFilter (dxy.class)

use crate::biomes::{BiomeId, SurfaceBlock, MIN_Y, SEA_LEVEL};
use crate::features::ores::OreType;
use crate::features::trees::{TreeFeature, TreeSpecies};
use crate::pipeline::{ChunkColumnBuffer, CHUNK_HEIGHT, CHUNK_WIDTH};
use crate::seed_pipeline::random::{
    set_decoration_seed, set_feature_seed, Xoroshiro128PlusPlus,
};
use crate::structures::LegacyRandom;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use zip::ZipArchive;

// ----------------------------------------------------------------------------
// 1. VANILLA 11-STEP DECORATION STEP ENUM & CONSTANTS
// ----------------------------------------------------------------------------

pub const RAW_GENERATION: usize = 0;
pub const LAKES: usize = 1;
pub const LOCAL_MODIFICATIONS: usize = 2;
pub const UNDERGROUND_STRUCTURES: usize = 3;
pub const SURFACE_STRUCTURES: usize = 4;
pub const STRONGHOLDS: usize = 5;
pub const UNDERGROUND_ORES: usize = 6;
pub const UNDERGROUND_DECORATION: usize = 7;
pub const FLUID_SPRINGS: usize = 8;
pub const VEGETAL_DECORATION: usize = 9;
pub const TOP_LAYER_MODIFICATION: usize = 10;
pub const TOTAL_DECORATION_STEPS: usize = 11;

/// Exact vanilla Minecraft 1.20.4 GenerationStep$Decoration enumeration.
/// REA Reference: net.minecraft.world.level.levelgen.GenerationStep$Decoration (dom$b.class)
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DecorationStep {
    /// 0: Raw terrain carving, ravines, and baseline generation features.
    RawGeneration = RAW_GENERATION,
    /// 1: Surface and underground lava & water lakes.
    Lakes = LAKES,
    /// 2: Local geological anomalies (e.g. Amethyst geodes).
    LocalModifications = LOCAL_MODIFICATIONS,
    /// 3: Underground dungeons / monster rooms.
    UndergroundStructures = UNDERGROUND_STRUCTURES,
    /// 4: Surface structures (villages, desert pyramids, outposts).
    SurfaceStructures = SURFACE_STRUCTURES,
    /// 5: Strongholds and eye of ender targets.
    Strongholds = STRONGHOLDS,
    /// 6: Underground ore blobs (Coal, Iron, Copper, Gold, Redstone, Diamond, Lapis).
    UndergroundOres = UNDERGROUND_ORES,
    /// 7: Underground decorations (dripstone clusters, glow lichen underground).
    UndergroundDecoration = UNDERGROUND_DECORATION,
    /// 8: Water and lava springs in stone cliffs / ceilings.
    FluidSprings = FLUID_SPRINGS,
    /// 9: Foliage, trees, grass, flowers, mushrooms, aquatic plants.
    VegetalDecoration = VEGETAL_DECORATION,
    /// 10: Surface freezing (water -> ice) and top layer snow snowfall.
    TopLayerModification = TOP_LAYER_MODIFICATION,
}

impl DecorationStep {
    pub const ALL: [DecorationStep; TOTAL_DECORATION_STEPS] = [
        DecorationStep::RawGeneration,
        DecorationStep::Lakes,
        DecorationStep::LocalModifications,
        DecorationStep::UndergroundStructures,
        DecorationStep::SurfaceStructures,
        DecorationStep::Strongholds,
        DecorationStep::UndergroundOres,
        DecorationStep::UndergroundDecoration,
        DecorationStep::FluidSprings,
        DecorationStep::VegetalDecoration,
        DecorationStep::TopLayerModification,
    ];

    #[inline]
    pub const fn index(&self) -> usize {
        *self as usize
    }

    #[inline]
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(DecorationStep::RawGeneration),
            1 => Some(DecorationStep::Lakes),
            2 => Some(DecorationStep::LocalModifications),
            3 => Some(DecorationStep::UndergroundStructures),
            4 => Some(DecorationStep::SurfaceStructures),
            5 => Some(DecorationStep::Strongholds),
            6 => Some(DecorationStep::UndergroundOres),
            7 => Some(DecorationStep::UndergroundDecoration),
            8 => Some(DecorationStep::FluidSprings),
            9 => Some(DecorationStep::VegetalDecoration),
            10 => Some(DecorationStep::TopLayerModification),
            _ => None,
        }
    }

    #[inline]
    pub const fn name(&self) -> &'static str {
        match self {
            DecorationStep::RawGeneration => "raw_generation",
            DecorationStep::Lakes => "lakes",
            DecorationStep::LocalModifications => "local_modifications",
            DecorationStep::UndergroundStructures => "underground_structures",
            DecorationStep::SurfaceStructures => "surface_structures",
            DecorationStep::Strongholds => "strongholds",
            DecorationStep::UndergroundOres => "underground_ores",
            DecorationStep::UndergroundDecoration => "underground_decoration",
            DecorationStep::FluidSprings => "fluid_springs",
            DecorationStep::VegetalDecoration => "vegetal_decoration",
            DecorationStep::TopLayerModification => "top_layer_modification",
        }
    }
}

// ----------------------------------------------------------------------------
// 2. PLACED FEATURE DEFINITIONS & PLACEMENT MODIFIERS
// ----------------------------------------------------------------------------

/// Parsed representation of a placed feature in `client.jar`.
/// REA Reference: net.minecraft.world.level.levelgen.placement.PlacedFeature (dyk.class)
#[derive(Debug, Clone)]
pub struct PlacedFeatureDef {
    pub id: String,
    pub feature_id: String,
    pub modifiers: Vec<PlacementModifierDef>,
}

/// Parsed representation of placement modifiers.
/// REA Reference: net.minecraft.world.level.levelgen.placement.PlacementModifier (dyn.class)
#[derive(Debug, Clone)]
pub enum PlacementModifierDef {
    /// InSquare placement modifier (dyh.class).
    InSquare,
    /// Count placement modifier (dyd.class).
    Count(IntProviderDef),
    /// HeightRange placement modifier (dyf.class).
    HeightRange(HeightRangeDef),
    /// BiomeFilter placement modifier (dxy.class).
    BiomeFilter,
    /// RarityFilter (dyq.class).
    RarityFilter(i32),
    /// Heightmap placement (dyg.class).
    Heightmap(String),
    /// Surface water depth filter (dyt.class).
    SurfaceWaterDepthFilter(i32),
    /// Fallback generic / unsupported modifier.
    Other(String),
}

#[derive(Debug, Clone)]
pub enum IntProviderDef {
    Constant(i32),
    WeightedList(Vec<(i32, i32)>),
    Uniform(i32, i32),
}

impl IntProviderDef {
    pub fn sample(&self, rng: &mut Xoroshiro128PlusPlus) -> i32 {
        match self {
            IntProviderDef::Constant(c) => *c,
            IntProviderDef::WeightedList(entries) => {
                let total_weight: i32 = entries.iter().map(|(_, w)| *w).sum();
                if total_weight <= 0 {
                    return entries.first().map(|(d, _)| *d).unwrap_or(1);
                }
                let mut pick = rng.next_int_bounded(total_weight);
                for (data, weight) in entries {
                    if pick < *weight {
                        return *data;
                    }
                    pick -= *weight;
                }
                entries.last().map(|(d, _)| *d).unwrap_or(1)
            }
            IntProviderDef::Uniform(min_val, max_val) => {
                if min_val >= max_val {
                    *min_val
                } else {
                    min_val + rng.next_int_bounded(max_val - min_val + 1)
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum VerticalAnchorDef {
    Absolute(i32),
    AboveBottom(i32),
    BelowTop(i32),
}

impl VerticalAnchorDef {
    pub fn resolve(&self) -> i32 {
        match self {
            VerticalAnchorDef::Absolute(y) => *y,
            VerticalAnchorDef::AboveBottom(y) => MIN_Y + *y, // -64 + y
            VerticalAnchorDef::BelowTop(y) => (MIN_Y + CHUNK_HEIGHT as i32) - *y, // 320 - y
        }
    }
}

#[derive(Debug, Clone)]
pub enum HeightRangeDef {
    Uniform {
        min: VerticalAnchorDef,
        max: VerticalAnchorDef,
    },
    Trapezoid {
        min: VerticalAnchorDef,
        max: VerticalAnchorDef,
        plateau: i32,
    },
}

impl HeightRangeDef {
    pub fn sample(&self, rng: &mut Xoroshiro128PlusPlus) -> i32 {
        match self {
            HeightRangeDef::Uniform { min, max } => {
                let min_y = min.resolve();
                let max_y = max.resolve();
                if min_y >= max_y {
                    min_y
                } else {
                    min_y + rng.next_int_bounded(max_y - min_y + 1)
                }
            }
            HeightRangeDef::Trapezoid { min, max, plateau } => {
                let min_y = min.resolve();
                let max_y = max.resolve();
                if min_y >= max_y {
                    min_y
                } else {
                    let range = max_y - min_y;
                    if *plateau >= range {
                        min_y + rng.next_int_bounded(range + 1)
                    } else {
                        let half = (range - plateau) / 2;
                        let r1 = rng.next_int_bounded(half + 1);
                        let r2 = rng.next_int_bounded(half + 1);
                        min_y + r1 + r2 + (if *plateau > 0 { rng.next_int_bounded(plateau + 1) } else { 0 })
                    }
                }
            }
        }
    }
}

// ----------------------------------------------------------------------------
// 3. BIOME FEATURE REGISTRY (LOADED DIRECTLY FROM client.jar)
// ----------------------------------------------------------------------------

#[derive(Deserialize, Debug)]
struct BiomeJson {
    #[serde(default)]
    features: Vec<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct PlacedFeatureJson {
    feature: serde_json::Value,
    #[serde(default)]
    placement: Vec<serde_json::Value>,
}

/// Central registry holding all 11-step decoration features per biome directly parsed from `client.jar`.
#[derive(Debug, Clone)]
pub struct BiomeFeatureRegistry {
    /// Biome resource key -> ordered list of 11 decoration step feature ID arrays.
    pub biome_features: HashMap<String, [Vec<String>; TOTAL_DECORATION_STEPS]>,
    /// PlacedFeature resource key -> detailed definition (modifiers & feature target).
    pub placed_features: HashMap<String, PlacedFeatureDef>,
}

impl BiomeFeatureRegistry {
    /// Global lazily initialized singleton of the vanilla feature registry.
    pub fn global() -> &'static Self {
        static REGISTRY: OnceLock<BiomeFeatureRegistry> = OnceLock::new();
        REGISTRY.get_or_init(|| {
            Self::load_from_standard_paths().unwrap_or_else(Self::fallback_vanilla)
        })
    }

    /// Attempts to locate `client.jar` in standard paths and load all biome features.
    pub fn load_from_standard_paths() -> Option<Self> {
        let candidates = [
            "client.jar",
            "../client.jar",
            "../../client.jar",
            "../../../client.jar",
        ];
        for path in candidates {
            let p = PathBuf::from(path);
            if p.exists() {
                if let Ok(reg) = Self::load_from_jar(&p) {
                    return Some(reg);
                }
            }
        }
        None
    }

    /// Loads all biome features and placed features directly from `client.jar`.
    pub fn load_from_jar(jar_path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(jar_path)?;
        let reader = BufReader::new(file);
        let mut archive = ZipArchive::new(reader)?;
        Self::load_from_zip(&mut archive)
    }

    /// Reads biome and placed feature files from any open ZipArchive.
    pub fn load_from_zip<R: Read + Seek>(
        archive: &mut ZipArchive<R>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut biome_features = HashMap::new();
        let mut placed_features = HashMap::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = match file.name() {
                Ok(n) => n.to_string(),
                Err(_) => continue,
            };

            // 1. Biomes: data/minecraft/worldgen/biome/*.json
            if name.starts_with("data/minecraft/worldgen/biome/") && name.ends_with(".json") {
                let stem = &name["data/minecraft/worldgen/biome/".len()..name.len() - 5];
                let biome_key = format!("minecraft:{}", stem);

                let mut content = String::new();
                file.read_to_string(&mut content)?;

                if let Ok(biome_json) = serde_json::from_str::<BiomeJson>(&content) {
                    let mut steps: [Vec<String>; TOTAL_DECORATION_STEPS] = Default::default();
                    for (step_idx, step_features) in biome_json.features.into_iter().enumerate() {
                        if step_idx < TOTAL_DECORATION_STEPS {
                            steps[step_idx] = step_features;
                        }
                    }
                    biome_features.insert(biome_key, steps);
                }
            }
            // 2. Placed Features: data/minecraft/worldgen/placed_feature/*.json
            else if name.starts_with("data/minecraft/worldgen/placed_feature/")
                && name.ends_with(".json")
            {
                let stem = &name["data/minecraft/worldgen/placed_feature/".len()..name.len() - 5];
                let pf_key = format!("minecraft:{}", stem);

                let mut content = String::new();
                file.read_to_string(&mut content)?;

                if let Ok(pf_json) = serde_json::from_str::<PlacedFeatureJson>(&content) {
                    let target_feature = match pf_json.feature {
                        serde_json::Value::String(s) => s,
                        _ => pf_key.clone(),
                    };

                    let mut modifiers = Vec::new();
                    for mod_val in pf_json.placement {
                        if let Some(m) = Self::parse_modifier(&mod_val) {
                            modifiers.push(m);
                        }
                    }

                    placed_features.insert(
                        pf_key.clone(),
                        PlacedFeatureDef {
                            id: pf_key,
                            feature_id: target_feature,
                            modifiers,
                        },
                    );
                }
            }
        }

        Ok(Self {
            biome_features,
            placed_features,
        })
    }

    fn parse_modifier(v: &serde_json::Value) -> Option<PlacementModifierDef> {
        let mod_type = v.get("type")?.as_str()?;
        match mod_type {
            "minecraft:in_square" => Some(PlacementModifierDef::InSquare),
            "minecraft:biome" => Some(PlacementModifierDef::BiomeFilter),
            "minecraft:count" => {
                let count_val = v.get("count")?;
                let provider = if let Some(n) = count_val.as_i64() {
                    IntProviderDef::Constant(n as i32)
                } else if count_val.get("type").and_then(|t| t.as_str())
                    == Some("minecraft:weighted_list")
                {
                    let mut list = Vec::new();
                    if let Some(dist) = count_val.get("distribution").and_then(|d| d.as_array()) {
                        for item in dist {
                            let data = item.get("data").and_then(|d| d.as_i64()).unwrap_or(0) as i32;
                            let weight =
                                item.get("weight").and_then(|w| w.as_i64()).unwrap_or(1) as i32;
                            list.push((data, weight));
                        }
                    }
                    IntProviderDef::WeightedList(list)
                } else if count_val.get("type").and_then(|t| t.as_str())
                    == Some("minecraft:uniform")
                {
                    let min_v = count_val
                        .get("value")
                        .and_then(|v| v.get("min_inclusive"))
                        .and_then(|m| m.as_i64())
                        .unwrap_or(1) as i32;
                    let max_v = count_val
                        .get("value")
                        .and_then(|v| v.get("max_inclusive"))
                        .and_then(|m| m.as_i64())
                        .unwrap_or(1) as i32;
                    IntProviderDef::Uniform(min_v, max_v)
                } else {
                    IntProviderDef::Constant(1)
                };
                Some(PlacementModifierDef::Count(provider))
            }
            "minecraft:height_range" => {
                let h = v.get("height")?;
                let h_type = h.get("type")?.as_str()?;
                let parse_anchor = |val: Option<&serde_json::Value>| -> VerticalAnchorDef {
                    if let Some(a) = val {
                        if let Some(abs) = a.get("absolute").and_then(|x| x.as_i64()) {
                            VerticalAnchorDef::Absolute(abs as i32)
                        } else if let Some(ab) = a.get("above_bottom").and_then(|x| x.as_i64()) {
                            VerticalAnchorDef::AboveBottom(ab as i32)
                        } else if let Some(bt) = a.get("below_top").and_then(|x| x.as_i64()) {
                            VerticalAnchorDef::BelowTop(bt as i32)
                        } else if let Some(n) = a.as_i64() {
                            VerticalAnchorDef::Absolute(n as i32)
                        } else {
                            VerticalAnchorDef::Absolute(0)
                        }
                    } else {
                        VerticalAnchorDef::Absolute(0)
                    }
                };

                let min_anchor = parse_anchor(h.get("min_inclusive"));
                let max_anchor = parse_anchor(h.get("max_inclusive"));

                match h_type {
                    "minecraft:trapezoid" => {
                        let plateau = h.get("plateau").and_then(|p| p.as_i64()).unwrap_or(0) as i32;
                        Some(PlacementModifierDef::HeightRange(HeightRangeDef::Trapezoid {
                            min: min_anchor,
                            max: max_anchor,
                            plateau,
                        }))
                    }
                    _ => Some(PlacementModifierDef::HeightRange(HeightRangeDef::Uniform {
                        min: min_anchor,
                        max: max_anchor,
                    })),
                }
            }
            "minecraft:rarity_filter" => {
                let chance = v.get("chance").and_then(|c| c.as_i64()).unwrap_or(1) as i32;
                Some(PlacementModifierDef::RarityFilter(chance))
            }
            "minecraft:heightmap" => {
                let hm = v
                    .get("heightmap")
                    .and_then(|h| h.as_str())
                    .unwrap_or("OCEAN_FLOOR")
                    .to_string();
                Some(PlacementModifierDef::Heightmap(hm))
            }
            "minecraft:surface_water_depth_filter" => {
                let depth =
                    v.get("max_water_depth").and_then(|d| d.as_i64()).unwrap_or(0) as i32;
                Some(PlacementModifierDef::SurfaceWaterDepthFilter(depth))
            }
            other => Some(PlacementModifierDef::Other(other.to_string())),
        }
    }

    /// Fallback hardcoded vanilla Overworld biome features when client.jar is unavailable.
    pub fn fallback_vanilla() -> Self {
        let mut biome_features = HashMap::new();

        let standard_ores = vec![
            "minecraft:ore_dirt".to_string(),
            "minecraft:ore_gravel".to_string(),
            "minecraft:ore_granite_upper".to_string(),
            "minecraft:ore_granite_lower".to_string(),
            "minecraft:ore_diorite_upper".to_string(),
            "minecraft:ore_diorite_lower".to_string(),
            "minecraft:ore_andesite_upper".to_string(),
            "minecraft:ore_andesite_lower".to_string(),
            "minecraft:ore_tuff".to_string(),
            "minecraft:ore_coal_upper".to_string(),
            "minecraft:ore_coal_lower".to_string(),
            "minecraft:ore_iron_upper".to_string(),
            "minecraft:ore_iron_middle".to_string(),
            "minecraft:ore_iron_small".to_string(),
            "minecraft:ore_gold".to_string(),
            "minecraft:ore_gold_lower".to_string(),
            "minecraft:ore_redstone".to_string(),
            "minecraft:ore_redstone_lower".to_string(),
            "minecraft:ore_diamond".to_string(),
            "minecraft:ore_diamond_medium".to_string(),
            "minecraft:ore_diamond_large".to_string(),
            "minecraft:ore_diamond_buried".to_string(),
            "minecraft:ore_lapis".to_string(),
            "minecraft:ore_lapis_buried".to_string(),
            "minecraft:ore_copper".to_string(),
        ];

        let mut plains_steps: [Vec<String>; TOTAL_DECORATION_STEPS] = Default::default();
        plains_steps[LAKES] = vec!["minecraft:lake_lava_underground".into()];
        plains_steps[LOCAL_MODIFICATIONS] = vec!["minecraft:amethyst_geode".into()];
        plains_steps[UNDERGROUND_STRUCTURES] = vec!["minecraft:monster_room".into()];
        plains_steps[UNDERGROUND_ORES] = standard_ores.clone();
        plains_steps[FLUID_SPRINGS] = vec!["minecraft:spring_water".into(), "minecraft:spring_lava".into()];
        plains_steps[VEGETAL_DECORATION] = vec![
            "minecraft:trees_plains".into(),
            "minecraft:flower_plains".into(),
            "minecraft:patch_grass_plain".into(),
        ];
        plains_steps[TOP_LAYER_MODIFICATION] = vec!["minecraft:freeze_top_layer".into()];
        biome_features.insert("minecraft:plains".into(), plains_steps);

        Self {
            biome_features,
            placed_features: HashMap::new(),
        }
    }

    /// Retrieves the 11-step ordered features for a biome.
    pub fn get_features_for_biome(&self, biome: BiomeId) -> &[Vec<String>; TOTAL_DECORATION_STEPS] {
        static EMPTY_STEPS: [Vec<String>; TOTAL_DECORATION_STEPS] = [
            Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(),
            Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(),
            Vec::new(),
        ];

        let name = biome.name();
        if let Some(steps) = self.biome_features.get(name) {
            steps
        } else if let Some(steps) = self.biome_features.get("minecraft:plains") {
            steps
        } else {
            &EMPTY_STEPS
        }
    }

    /// Evaluates placement modifiers (InSquare, Count, HeightRange, BiomeFilter)
    /// using `feature_rng` to produce target placement world coordinates.
    pub fn evaluate_placement(
        &self,
        buffer: &ChunkColumnBuffer,
        chunk_x: i32,
        chunk_z: i32,
        feature_id: &str,
        feature_rng: &mut Xoroshiro128PlusPlus,
    ) -> Vec<(i32, i32, i32)> {
        let origin = (chunk_x * 16, 0, chunk_z * 16);
        let mut positions = vec![origin];

        if let Some(def) = self.placed_features.get(feature_id) {
            for modifier in &def.modifiers {
                match modifier {
                    PlacementModifierDef::InSquare => {
                        let mut next = Vec::with_capacity(positions.len());
                        for (x, y, z) in positions {
                            let dx = feature_rng.next_int_bounded(16);
                            let dz = feature_rng.next_int_bounded(16);
                            next.push((x + dx, y, z + dz));
                        }
                        positions = next;
                    }
                    PlacementModifierDef::Count(count_provider) => {
                        let count = count_provider.sample(feature_rng);
                        if count <= 0 {
                            positions.clear();
                            break;
                        }
                        let mut next = Vec::with_capacity(positions.len() * count as usize);
                        for pos in positions {
                            for _ in 0..count {
                                next.push(pos);
                            }
                        }
                        positions = next;
                    }
                    PlacementModifierDef::HeightRange(height_range) => {
                        let mut next = Vec::with_capacity(positions.len());
                        for (x, _, z) in positions {
                            let y = height_range.sample(feature_rng);
                            next.push((x, y, z));
                        }
                        positions = next;
                    }
                    PlacementModifierDef::RarityFilter(chance) => {
                        if *chance > 1 {
                            positions.retain(|_| feature_rng.next_int_bounded(*chance) == 0);
                        }
                    }
                    PlacementModifierDef::BiomeFilter => {
                        // Position must remain within current chunk generation scope
                        positions.retain(|&(x, _, z)| {
                            let lx = x - chunk_x * 16;
                            let lz = z - chunk_z * 16;
                            lx >= 0 && lx < 16 && lz >= 0 && lz < 16
                        });
                    }
                    PlacementModifierDef::Heightmap(_) => {
                        let mut next = Vec::with_capacity(positions.len());
                        for (x, _, z) in positions {
                            let lx = (x - chunk_x * 16).clamp(0, 15) as usize;
                            let lz = (z - chunk_z * 16).clamp(0, 15) as usize;
                            let mut surf_y = SEA_LEVEL;
                            for y in (-64..=319).rev() {
                                let b = buffer.get_local(lx, (y - MIN_Y) as usize, lz);
                                if b != SurfaceBlock::Air && b != SurfaceBlock::Water {
                                    surf_y = y;
                                    break;
                                }
                            }
                            next.push((x, surf_y, z));
                        }
                        positions = next;
                    }
                    _ => {}
                }
            }
        } else {
            // Default modifier fallback if definition was missing in json
            let dx = feature_rng.next_int_bounded(16);
            let dz = feature_rng.next_int_bounded(16);
            let y = -64 + feature_rng.next_int_bounded(384);
            positions = vec![(chunk_x * 16 + dx, y, chunk_z * 16 + dz)];
        }

        positions
    }

    /// Places feature blocks at evaluated positions in the chunk column buffer.
    pub fn place_feature_blocks(
        &self,
        buffer: &mut ChunkColumnBuffer,
        chunk_x: i32,
        chunk_z: i32,
        step_index: usize,
        feature_id: &str,
        positions: &[(i32, i32, i32)],
        feature_rng: &mut Xoroshiro128PlusPlus,
    ) {
        match step_index {
            // STEP 1: LAKES
            LAKES => {
                for &(x, y, z) in positions {
                    if y > -54 && y < 200 {
                        place_lava_lake(buffer, x, y, z, feature_rng);
                    }
                }
            }
            // STEP 2: LOCAL_MODIFICATIONS (Geodes)
            LOCAL_MODIFICATIONS => {
                for &(x, y, z) in positions {
                    if y < 30 {
                        place_amethyst_geode(buffer, x, y, z, feature_rng);
                    }
                }
            }
            // STEP 3: UNDERGROUND_STRUCTURES (Dungeons)
            UNDERGROUND_STRUCTURES => {
                for &(x, y, z) in positions {
                    if y < 40 {
                        place_monster_room(buffer, x, y, z, feature_rng);
                    }
                }
            }
            // STEP 6: UNDERGROUND_ORES
            UNDERGROUND_ORES => {
                let ore_target = if feature_id.contains("coal") {
                    Some((OreTarget::Ore(OreType::Coal), 17))
                } else if feature_id.contains("iron") {
                    Some((OreTarget::Ore(OreType::Iron), 9))
                } else if feature_id.contains("copper") {
                    Some((OreTarget::Ore(OreType::Copper), 10))
                } else if feature_id.contains("gold") {
                    Some((OreTarget::Ore(OreType::Gold), 9))
                } else if feature_id.contains("redstone") {
                    Some((OreTarget::Ore(OreType::Redstone), 8))
                } else if feature_id.contains("diamond") {
                    Some((OreTarget::Ore(OreType::Diamond), 8))
                } else if feature_id.contains("lapis") {
                    Some((OreTarget::Ore(OreType::Lapis), 7))
                } else if feature_id.contains("dirt") {
                    Some((OreTarget::Block(SurfaceBlock::Dirt), 33))
                } else if feature_id.contains("gravel") {
                    Some((OreTarget::Block(SurfaceBlock::Gravel), 33))
                } else {
                    None
                };

                if let Some((target, size)) = ore_target {
                    for &(x, y, z) in positions {
                        place_ore_blob(buffer, x, y, z, target, size, feature_rng);
                    }
                }
            }
            // STEP 8: FLUID_SPRINGS
            FLUID_SPRINGS => {
                let fluid = if feature_id.contains("lava") {
                    SurfaceBlock::Lava
                } else {
                    SurfaceBlock::Water
                };
                for &(x, y, z) in positions {
                    place_spring(buffer, x, y, z, fluid);
                }
            }
            // STEP 9: VEGETAL_DECORATION
            VEGETAL_DECORATION => {
                if feature_id.contains("tree") {
                    let species = if feature_id.contains("birch") {
                        TreeSpecies::Birch
                    } else if feature_id.contains("taiga") || feature_id.contains("snowy") {
                        TreeSpecies::Spruce
                    } else if feature_id.contains("jungle") {
                        TreeSpecies::MegaJungle
                    } else if feature_id.contains("savanna") {
                        TreeSpecies::Acacia
                    } else if feature_id.contains("dark_forest") {
                        TreeSpecies::DarkOak
                    } else if feature_id.contains("pale_garden") || feature_id.contains("pale_oak") {
                        TreeSpecies::PaleOak
                    } else {
                        TreeSpecies::Oak
                    };

                    for &(x, y, z) in positions {
                        let mut legacy_rng = LegacyRandom::new(feature_rng.next_long());
                        if species == TreeSpecies::PaleOak {
                            let place_heart = feature_rng.next_int_bounded(100) < 15;
                            crate::biomes::pale_garden::generate_pale_oak_tree(buffer, (x, y + 1, z), &mut legacy_rng, place_heart);
                        } else {
                            TreeFeature::place_tree(buffer, (x, y + 1, z), species, &mut legacy_rng);
                        }
                    }
                } else if feature_id.contains("pale_moss") {
                    for &(x, y, z) in positions {
                        let lx = (x - chunk_x * 16).clamp(0, 15) as usize;
                        let lz = (z - chunk_z * 16).clamp(0, 15) as usize;
                        let buf_y = (y + 1 - MIN_Y) as usize;
                        if buf_y < CHUNK_HEIGHT && buffer.get_local(lx, buf_y, lz) == SurfaceBlock::Air {
                            buffer.set_local(lx, buf_y, lz, SurfaceBlock::PaleMossCarpet);
                        }
                    }
                } else if feature_id.contains("flower") || feature_id.contains("grass") {
                    for &(x, y, z) in positions {
                        let lx = (x - chunk_x * 16).clamp(0, 15) as usize;
                        let lz = (z - chunk_z * 16).clamp(0, 15) as usize;
                        let buf_y = (y + 1 - MIN_Y) as usize;
                        if buf_y < CHUNK_HEIGHT && buffer.get_local(lx, buf_y, lz) == SurfaceBlock::Air {
                            let soil = buffer.get_local(lx, (y - MIN_Y) as usize, lz);
                            if soil == SurfaceBlock::GrassBlock || soil == SurfaceBlock::Dirt {
                                let plant = if feature_id.contains("tall_grass") {
                                    SurfaceBlock::TallGrass
                                } else if feature_id.contains("flower") {
                                    if feature_rng.next_boolean() {
                                        SurfaceBlock::Poppy
                                    } else {
                                        SurfaceBlock::Dandelion
                                    }
                                } else {
                                    SurfaceBlock::ShortGrass
                                };
                                buffer.set_local(lx, buf_y, lz, plant);
                            }
                        }
                    }
                }
            }
            // STEP 10: TOP_LAYER_MODIFICATION
            TOP_LAYER_MODIFICATION => {
                apply_freeze_top_layer(buffer, chunk_x, chunk_z);
            }
            _ => {}
        }
    }
}

// ----------------------------------------------------------------------------
// 4. CHUNK DECORATOR DISPATCHER
// ----------------------------------------------------------------------------

/// Dispatches Minecraft's exact 11 sequential decoration steps onto a chunk column buffer.
/// REA Reference: net.minecraft.world.level.chunk.ChunkGenerator.applyBiomeDecoration (dle.class)
pub fn decorate_chunk(
    buffer: &mut ChunkColumnBuffer,
    chunk_x: i32,
    chunk_z: i32,
    world_seed: u64,
    registry: &BiomeFeatureRegistry,
) {
    // Exact vanilla population seed derivation
    let chunk_pop_seed = set_decoration_seed(world_seed, chunk_x * 16, chunk_z * 16);
    let biome_features = registry.get_features_for_biome(buffer.biome);

    // Sequential 11-step decoration pipeline
    for step_index in 0..TOTAL_DECORATION_STEPS {
        let step_features = &biome_features[step_index];
        for (feature_index, feature_id) in step_features.iter().enumerate() {
            // Re-seed isolated PRNG instance for this feature
            let mut feature_rng = set_feature_seed(chunk_pop_seed, feature_index, step_index);

            // Evaluate placement modifiers (InSquare, Count, HeightRange, BiomeFilter)
            let positions = registry.evaluate_placement(
                buffer,
                chunk_x,
                chunk_z,
                feature_id,
                &mut feature_rng,
            );

            // Place the feature blocks into the chunk column
            registry.place_feature_blocks(
                buffer,
                chunk_x,
                chunk_z,
                step_index,
                feature_id,
                &positions,
                &mut feature_rng,
            );
        }
    }

    // 1.21.4 Pale Garden specialized vegetation pass if biome matches
    if buffer.biome == BiomeId::PaleGarden {
        let mut pale_rng = set_feature_seed(chunk_pop_seed, 0, VEGETAL_DECORATION);
        crate::features::pale_garden::decorate_pale_garden_vegetation(
            buffer,
            chunk_x,
            chunk_z,
            &mut pale_rng,
        );
    }
}

// ----------------------------------------------------------------------------
// 5. CONCRETE FEATURE PLACERS
// ----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
enum OreTarget {
    Ore(OreType),
    Block(SurfaceBlock),
}

fn place_ore_blob(
    buffer: &mut ChunkColumnBuffer,
    center_x: i32,
    center_y: i32,
    center_z: i32,
    target: OreTarget,
    vein_size: usize,
    _rng: &mut Xoroshiro128PlusPlus,
) {
    let radius = ((vein_size as f32).cbrt() * 0.8).max(1.0) as i32;
    for dz in -radius..=radius {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy + dz * dz <= radius * radius {
                    let wx = center_x + dx;
                    let wy = center_y + dy;
                    let wz = center_z + dz;

                    let lx = wx - buffer.chunk_x * 16;
                    let lz = wz - buffer.chunk_z * 16;
                    let ly = wy - MIN_Y;

                    if lx >= 0 && lx < 16 && lz >= 0 && lz < 16 && ly >= 0 && ly < CHUNK_HEIGHT as i32 {
                        let cur = buffer.get_local(lx as usize, ly as usize, lz as usize);
                        if cur == SurfaceBlock::Stone || cur == SurfaceBlock::Deepslate {
                            let ore_block = match target {
                                OreTarget::Ore(ore) => {
                                    if cur == SurfaceBlock::Deepslate {
                                        ore.deepslate_variant()
                                    } else {
                                        ore.stone_variant()
                                    }
                                }
                                OreTarget::Block(b) => b,
                            };
                            buffer.set_local(lx as usize, ly as usize, lz as usize, ore_block);
                        }
                    }
                }
            }
        }
    }
}

fn place_spring(
    buffer: &mut ChunkColumnBuffer,
    x: i32,
    y: i32,
    z: i32,
    fluid: SurfaceBlock,
) {
    let lx = x - buffer.chunk_x * 16;
    let lz = z - buffer.chunk_z * 16;
    let ly = y - MIN_Y;

    if lx > 0 && lx < 15 && lz > 0 && lz < 15 && ly > 0 && ly < (CHUNK_HEIGHT as i32 - 1) {
        let cur = buffer.get_local(lx as usize, ly as usize, lz as usize);
        if cur == SurfaceBlock::Stone || cur == SurfaceBlock::Deepslate {
            // Verify at least one side is exposed to air
            let neighbors = [
                buffer.get_local((lx - 1) as usize, ly as usize, lz as usize),
                buffer.get_local((lx + 1) as usize, ly as usize, lz as usize),
                buffer.get_local(lx as usize, (ly - 1) as usize, lz as usize),
                buffer.get_local(lx as usize, (ly + 1) as usize, lz as usize),
                buffer.get_local(lx as usize, ly as usize, (lz - 1) as usize),
                buffer.get_local(lx as usize, ly as usize, (lz + 1) as usize),
            ];
            let air_count = neighbors.iter().filter(|&&b| b == SurfaceBlock::Air).count();
            if air_count >= 1 && air_count <= 3 {
                buffer.set_local(lx as usize, ly as usize, lz as usize, fluid);
            }
        }
    }
}

fn place_lava_lake(
    buffer: &mut ChunkColumnBuffer,
    cx: i32,
    cy: i32,
    cz: i32,
    rng: &mut Xoroshiro128PlusPlus,
) {
    let r = 3 + rng.next_int_bounded(3);
    for dz in -r..=r {
        for dy in -2..=2 {
            for dx in -r..=r {
                if dx * dx + dz * dz <= r * r {
                    let lx = cx + dx - buffer.chunk_x * 16;
                    let lz = cz + dz - buffer.chunk_z * 16;
                    let ly = cy + dy - MIN_Y;
                    if lx >= 0 && lx < 16 && lz >= 0 && lz < 16 && ly >= 0 && ly < CHUNK_HEIGHT as i32 {
                        if dy <= 0 {
                            buffer.set_local(lx as usize, ly as usize, lz as usize, SurfaceBlock::Lava);
                        } else {
                            buffer.set_local(lx as usize, ly as usize, lz as usize, SurfaceBlock::Air);
                        }
                    }
                }
            }
        }
    }
}

fn place_amethyst_geode(
    buffer: &mut ChunkColumnBuffer,
    cx: i32,
    cy: i32,
    cz: i32,
    _rng: &mut Xoroshiro128PlusPlus,
) {
    let outer_r = 4;
    let inner_r = 2;
    for dz in -outer_r..=outer_r {
        for dy in -outer_r..=outer_r {
            for dx in -outer_r..=outer_r {
                let dist_sq = dx * dx + dy * dy + dz * dz;
                let lx = cx + dx - buffer.chunk_x * 16;
                let lz = cz + dz - buffer.chunk_z * 16;
                let ly = cy + dy - MIN_Y;
                if lx >= 0 && lx < 16 && lz >= 0 && lz < 16 && ly >= 0 && ly < CHUNK_HEIGHT as i32 {
                    if dist_sq <= inner_r * inner_r {
                        buffer.set_local(lx as usize, ly as usize, lz as usize, SurfaceBlock::Air);
                    } else if dist_sq <= outer_r * outer_r {
                        buffer.set_local(lx as usize, ly as usize, lz as usize, SurfaceBlock::Obsidian);
                    }
                }
            }
        }
    }
}

fn place_monster_room(
    buffer: &mut ChunkColumnBuffer,
    cx: i32,
    cy: i32,
    cz: i32,
    _rng: &mut Xoroshiro128PlusPlus,
) {
    for dz in -3..=3 {
        for dy in 0..=4 {
            for dx in -3..=3 {
                let lx = cx + dx - buffer.chunk_x * 16;
                let lz = cz + dz - buffer.chunk_z * 16;
                let ly = cy + dy - MIN_Y;
                if lx >= 0 && lx < 16 && lz >= 0 && lz < 16 && ly >= 0 && ly < CHUNK_HEIGHT as i32 {
                    if dx == -3 || dx == 3 || dz == -3 || dz == 3 || dy == 0 || dy == 4 {
                        buffer.set_local(lx as usize, ly as usize, lz as usize, SurfaceBlock::Cobblestone);
                    } else {
                        buffer.set_local(lx as usize, ly as usize, lz as usize, SurfaceBlock::Air);
                    }
                }
            }
        }
    }
    // Place central spawner
    let lx = cx - buffer.chunk_x * 16;
    let lz = cz - buffer.chunk_z * 16;
    let ly = cy + 1 - MIN_Y;
    if lx >= 0 && lx < 16 && lz >= 0 && lz < 16 && ly >= 0 && ly < CHUNK_HEIGHT as i32 {
        buffer.set_local(lx as usize, ly as usize, lz as usize, SurfaceBlock::Spawner);
    }
}

fn apply_freeze_top_layer(
    buffer: &mut ChunkColumnBuffer,
    _chunk_x: i32,
    _chunk_z: i32,
) {
    if !buffer.biome.is_snowy() && buffer.biome != BiomeId::FrozenPeaks && buffer.biome != BiomeId::JaggedPeaks {
        return;
    }

    for lz in 0..CHUNK_WIDTH {
        for lx in 0..CHUNK_WIDTH {
            for y in (-64..=319).rev() {
                let buf_y = (y - MIN_Y) as usize;
                let block = buffer.get_local(lx, buf_y, lz);
                if block == SurfaceBlock::Water && y >= SEA_LEVEL - 1 {
                    buffer.set_local(lx, buf_y, lz, SurfaceBlock::Ice);
                    break;
                } else if block != SurfaceBlock::Air && block != SurfaceBlock::Water {
                    let above = buf_y + 1;
                    if above < CHUNK_HEIGHT && buffer.get_local(lx, above, lz) == SurfaceBlock::Air {
                        buffer.set_local(lx, above, lz, SurfaceBlock::SnowLayer);
                    }
                    break;
                }
            }
        }
    }
}
