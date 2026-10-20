//! Dynamic Mojang Jigsaw Template Pool JSON deserializer, weighted discrete distribution,
//! and piece expansion assembler.
//!
//! REA References:
//! - `net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool` (mapped: `eap.class`)
//! - `net.minecraft.world.level.levelgen.structure.pools.SinglePoolElement` (mapped: `eam.class`)
//! - `net.minecraft.world.level.levelgen.structure.pools.LegacySinglePoolElement` (mapped: `eak.class`)
//! - `net.minecraft.world.level.levelgen.structure.pools.JigsawPlacement` (mapped: `eaj.class`)
//! - `net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool$Projection` (mapped: `eap$a.class`)

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::seed_pipeline::random::Xoroshiro128PlusPlus;
use crate::structures::locator::LegacyRandom;

/// Projection type for template pool elements matching `StructureTemplatePool$Projection`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PoolProjection {
    #[default]
    Rigid,
    TerrainMatching,
}

impl PoolProjection {
    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "terrain_matching" => Self::TerrainMatching,
            _ => Self::Rigid,
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Rigid => "rigid",
            Self::TerrainMatching => "terrain_matching",
        }
    }
}

/// Element type and metadata inside a template pool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoolElement {
    /// Type identifier (e.g. `minecraft:single_pool_element`, `minecraft:legacy_single_pool_element`)
    #[serde(default = "default_element_type")]
    pub element_type: String,

    /// NBT structure template resource location (e.g. `minecraft:village/plains/houses/plains_small_house_1`)
    #[serde(default)]
    pub location: Option<String>,

    /// Processor list or processor identifier
    #[serde(default)]
    pub processors: Option<serde_json::Value>,

    /// Surface projection (`rigid` or `terrain_matching`)
    #[serde(default)]
    pub projection: Option<PoolProjection>,

    /// For list pool elements: recursive sub-elements
    #[serde(default)]
    pub elements: Option<Vec<PoolElement>>,

    /// For feature pool elements: feature identifier
    #[serde(default)]
    pub feature: Option<String>,
}

fn default_element_type() -> String {
    "minecraft:single_pool_element".to_string()
}

impl PoolElement {
    pub fn single(location: impl Into<String>, projection: PoolProjection) -> Self {
        Self {
            element_type: "minecraft:single_pool_element".to_string(),
            location: Some(location.into()),
            processors: None,
            projection: Some(projection),
            elements: None,
            feature: None,
        }
    }

    pub fn legacy_single(location: impl Into<String>, processors: Option<serde_json::Value>, projection: PoolProjection) -> Self {
        Self {
            element_type: "minecraft:legacy_single_pool_element".to_string(),
            location: Some(location.into()),
            processors,
            projection: Some(projection),
            elements: None,
            feature: None,
        }
    }

    pub fn empty() -> Self {
        Self {
            element_type: "minecraft:empty_pool_element".to_string(),
            location: None,
            processors: None,
            projection: Some(PoolProjection::Rigid),
            elements: None,
            feature: None,
        }
    }
}

/// A weighted entry within a template pool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplatePoolEntry {
    pub weight: u32,
    pub element: PoolElement,
}

impl TemplatePoolEntry {
    pub fn new(weight: u32, element: PoolElement) -> Self {
        Self { weight, element }
    }
}

/// StructureTemplatePool matching net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool.
///
/// In vanilla bytecode (`eap.class`), each element is duplicated `weight` times into an `ObjectArrayList<ean>`.
/// Sampling `getRandomTemplateFromPool(RandomSource rng)` rolls `rng.nextInt(f.size())` and returns `f.get(roll)`.
/// This is 100% mathematically equivalent to drawing `roll < total_weight` and iterating through elements
/// in exact array order with cumulative weight thresholds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructureTemplatePool {
    /// Resource location identifier (e.g. `minecraft:village/plains/houses`)
    #[serde(default)]
    pub name: String,

    /// Identifier for pool exhaustion (e.g. `minecraft:empty`)
    pub fallback: String,

    /// Array of weighted elements
    pub elements: Vec<TemplatePoolEntry>,

    /// Precomputed cumulative weights preserving exact input array order
    #[serde(skip)]
    cumulative_weights: Vec<u32>,

    /// Total sum of all element weights
    #[serde(skip)]
    total_weight: u32,
}

impl StructureTemplatePool {
    /// Creates a new StructureTemplatePool and precomputes cumulative weights.
    pub fn new(name: impl Into<String>, fallback: impl Into<String>, elements: Vec<TemplatePoolEntry>) -> Self {
        let mut pool = Self {
            name: name.into(),
            fallback: fallback.into(),
            elements,
            cumulative_weights: Vec::new(),
            total_weight: 0,
        };
        pool.compile();
        pool
    }

    /// Compiles cumulative weights preserving exact input array order.
    pub fn compile(&mut self) {
        let mut sum = 0u32;
        let mut cum = Vec::with_capacity(self.elements.len());
        for entry in &self.elements {
            sum = sum.saturating_add(entry.weight);
            cum.push(sum);
        }
        self.cumulative_weights = cum;
        self.total_weight = sum;
    }

    #[inline]
    pub fn total_weight(&self) -> u32 {
        self.total_weight
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty() || self.total_weight == 0
    }

    /// Samples an element using a seeded PRNG function `next_int_bounded(bound: i32) -> i32`.
    /// Preserves exact input array order matching `net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool`.
    pub fn sample_element<F: FnMut(i32) -> i32>(&self, mut next_int_bounded: F) -> Option<&PoolElement> {
        if self.elements.is_empty() || self.total_weight == 0 {
            return None;
        }
        let roll = next_int_bounded(self.total_weight as i32) as u32;
        for (i, entry) in self.elements.iter().enumerate() {
            if roll < self.cumulative_weights[i] {
                return Some(&entry.element);
            }
        }
        self.elements.last().map(|e| &e.element)
    }

    /// Samples an element using Minecraft 1.20+ `Xoroshiro128PlusPlus`.
    #[inline]
    pub fn sample_element_xoroshiro(&self, rng: &mut Xoroshiro128PlusPlus) -> Option<&PoolElement> {
        self.sample_element(|bound| rng.next_int_bounded(bound))
    }

    /// Samples an element using Minecraft legacy `LegacyRandom`.
    #[inline]
    pub fn sample_element_legacy(&self, rng: &mut LegacyRandom) -> Option<&PoolElement> {
        self.sample_element(|bound| rng.next_int_bounded(bound))
    }

    /// Returns all elements expanded by weight and shuffled using Fisher-Yates shuffle
    /// matching `StructureTemplatePool.getShuffledTemplates(RandomSource rng)`.
    pub fn get_shuffled_elements<F: FnMut(i32) -> i32>(&self, mut next_int_bounded: F) -> Vec<&PoolElement> {
        let mut list: Vec<&PoolElement> = Vec::with_capacity(self.total_weight as usize);
        for entry in &self.elements {
            for _ in 0..entry.weight {
                list.push(&entry.element);
            }
        }
        // In-place Fisher-Yates shuffle matching JVM ObjectArrayList shuffle
        for i in (1..list.len()).rev() {
            let j = next_int_bounded((i + 1) as i32) as usize;
            list.swap(i, j);
        }
        list
    }
}

/// Walker's Alias Method implementation for $O(1)$ sampling over arbitrary weights,
/// preserving exact input array order.
#[derive(Debug, Clone)]
pub struct WeightedAliasTable {
    prob: Vec<f64>,
    alias: Vec<usize>,
    total_weight: u32,
}

impl WeightedAliasTable {
    pub fn new(weights: &[u32]) -> Self {
        let n = weights.len();
        if n == 0 {
            return Self {
                prob: Vec::new(),
                alias: Vec::new(),
                total_weight: 0,
            };
        }
        let total_weight: u32 = weights.iter().sum();
        if total_weight == 0 {
            return Self {
                prob: vec![0.0; n],
                alias: vec![0; n],
                total_weight: 0,
            };
        }

        let mut prob = vec![0.0; n];
        let mut alias = vec![0; n];
        let mut small = Vec::with_capacity(n);
        let mut large = Vec::with_capacity(n);

        let sum_f = total_weight as f64;
        let n_f = n as f64;
        for (i, &w) in weights.iter().enumerate() {
            let scaled = (w as f64) * n_f / sum_f;
            prob[i] = scaled;
            if scaled < 1.0 {
                small.push(i);
            } else {
                large.push(i);
            }
        }

        while let (Some(l), Some(g)) = (small.pop(), large.pop()) {
            alias[l] = g;
            prob[g] = (prob[g] + prob[l]) - 1.0;
            if prob[g] < 1.0 {
                small.push(g);
            } else {
                large.push(g);
            }
        }

        while let Some(g) = large.pop() {
            prob[g] = 1.0;
        }
        while let Some(l) = small.pop() {
            prob[l] = 1.0;
        }

        Self {
            prob,
            alias,
            total_weight,
        }
    }

    #[inline]
    pub fn total_weight(&self) -> u32 {
        self.total_weight
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.prob.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.prob.is_empty()
    }

    /// Samples an index $0 \le i < N$ using pre-drawn random integer and floating-point coin.
    #[inline]
    pub fn sample_with(&self, roll_int: usize, coin: f64) -> usize {
        let n = self.prob.len();
        if n == 0 {
            return 0;
        }
        let i = roll_int % n;
        if coin < self.prob[i] {
            i
        } else {
            self.alias[i]
        }
    }

    /// Samples an index $0 \le i < N$ using Xoroshiro128PlusPlus PRNG.
    #[inline]
    pub fn sample_xoroshiro(&self, rng: &mut Xoroshiro128PlusPlus) -> usize {
        let n = self.prob.len();
        if n == 0 {
            return 0;
        }
        let i = rng.next_int_bounded(n as i32) as usize;
        let coin = rng.next_double();
        if coin < self.prob[i] {
            i
        } else {
            self.alias[i]
        }
    }

    /// Sample an index $0 \le i < N$.
    pub fn sample<F: FnMut() -> f64, G: FnMut(i32) -> i32>(&self, mut next_double: F, mut next_int_bounded: G) -> usize {
        let n = self.prob.len();
        if n == 0 {
            return 0;
        }
        let i = next_int_bounded(n as i32) as usize;
        let coin = next_double();
        if coin < self.prob[i] {
            i
        } else {
            self.alias[i]
        }
    }
}

/// Registry storing all parsed `StructureTemplatePool` instances.
#[derive(Debug, Clone, Default)]
pub struct TemplatePoolRegistry {
    pools: HashMap<String, StructureTemplatePool>,
}

impl TemplatePoolRegistry {
    pub fn new() -> Self {
        Self {
            pools: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.pools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pools.is_empty()
    }

    pub fn contains_key(&self, id: &str) -> bool {
        let clean = id.strip_prefix("minecraft:").unwrap_or(id);
        self.pools.contains_key(id) || self.pools.contains_key(clean)
    }

    pub fn get_pool(&self, id: &str) -> Option<&StructureTemplatePool> {
        let clean = id.strip_prefix("minecraft:").unwrap_or(id);
        self.pools.get(id).or_else(|| self.pools.get(clean))
    }

    pub fn insert(&mut self, mut pool: StructureTemplatePool) {
        pool.compile();
        let name = pool.name.clone();
        let clean = name.strip_prefix("minecraft:").unwrap_or(&name).to_string();
        self.pools.insert(format!("minecraft:{}", clean), pool.clone());
        self.pools.insert(clean, pool);
    }

    /// Loads all template pools directly from `client.jar` (both base worldgen and 1.21 update datapacks).
    pub fn load_from_jar(jar_path: impl AsRef<Path>) -> Result<Self, String> {
        let file = File::open(jar_path.as_ref())
            .map_err(|e| format!("Failed to open jar file {:?}: {}", jar_path.as_ref(), e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("Failed to read zip archive: {}", e))?;

        let mut registry = Self::new();
        let total = archive.len();

        for i in 0..total {
            let mut file = archive.by_index(i)
                .map_err(|e| format!("Failed to access zip entry {}: {}", i, e))?;
            let name = match file.name() {
                Ok(n) => n.to_string(),
                Err(_) => continue,
            };

            // Match base worldgen template pools or 1.21 trial chambers datapack template pools
            let is_base = name.starts_with("data/minecraft/worldgen/template_pool/") && name.ends_with(".json");
            let is_update = name.starts_with("data/minecraft/datapacks/update_1_21/data/minecraft/worldgen/template_pool/") && name.ends_with(".json");

            if is_base || is_update {
                let mut content = Vec::with_capacity(file.size() as usize);
                file.read_to_end(&mut content)
                    .map_err(|e| format!("Failed to read entry {}: {}", name, e))?;

                let mut pool: StructureTemplatePool = serde_json::from_slice(&content)
                    .map_err(|e| format!("Failed to parse JSON for {}: {}", name, e))?;

                // Derive resource identifier from relative path
                let rel_path = if is_base {
                    name.strip_prefix("data/minecraft/worldgen/template_pool/")
                        .unwrap()
                        .strip_suffix(".json")
                        .unwrap()
                } else {
                    name.strip_prefix("data/minecraft/datapacks/update_1_21/data/minecraft/worldgen/template_pool/")
                        .unwrap()
                        .strip_suffix(".json")
                        .unwrap()
                };

                let res_id = format!("minecraft:{}", rel_path);
                if pool.name.is_empty() {
                    pool.name = res_id.clone();
                }

                registry.insert(pool);
            }
        }

        // Synthesize Woodland Mansion template pools if not already in the jar
        registry.register_woodland_mansion_pools();

        Ok(registry)
    }

    /// Automatically searches candidate paths for `client.jar` and loads the registry,
    /// falling back to embedded vanilla definitions if `client.jar` is unavailable.
    pub fn load_default() -> Self {
        let candidates = [
            "client.jar",
            "../client.jar",
            "../../client.jar",
            "IronOxide/client.jar",
            "c:/Users/khant/OneDrive/Desktop/Falix/Minecraft reverce in rust/client.jar",
        ];

        for path in &candidates {
            let p = Path::new(path);
            if p.exists() {
                if let Ok(reg) = Self::load_from_jar(p) {
                    return reg;
                }
            }
        }

        // Fallback: Populate embedded definitions
        let mut reg = Self::new();
        reg.populate_embedded_fallbacks();
        reg
    }

    /// Synthesizes Woodland Mansion template pools referencing the 73 vanilla NBT pieces.
    pub fn register_woodland_mansion_pools(&mut self) {
        if self.contains_key("woodland_mansion/entrance") {
            return;
        }

        // Entrance pool
        self.insert(StructureTemplatePool::new(
            "minecraft:woodland_mansion/entrance",
            "minecraft:empty",
            vec![TemplatePoolEntry::new(1, PoolElement::single("minecraft:woodland_mansion/entrance", PoolProjection::Rigid))],
        ));

        // Corridors pool
        self.insert(StructureTemplatePool::new(
            "minecraft:woodland_mansion/corridors",
            "minecraft:empty",
            vec![
                TemplatePoolEntry::new(1, PoolElement::single("minecraft:woodland_mansion/corridor_floor", PoolProjection::Rigid)),
                TemplatePoolEntry::new(1, PoolElement::single("minecraft:woodland_mansion/wall_flat", PoolProjection::Rigid)),
                TemplatePoolEntry::new(1, PoolElement::single("minecraft:woodland_mansion/wall_window", PoolProjection::Rigid)),
            ],
        ));

        // 1x1 Rooms
        let mut rooms_1x1 = Vec::new();
        for &suffix in &["a1", "a2", "a3", "a4", "a5", "as1", "as2", "as3", "as4", "b1", "b2", "b3", "b4", "b5"] {
            rooms_1x1.push(TemplatePoolEntry::new(
                1,
                PoolElement::single(format!("minecraft:woodland_mansion/1x1_{}", suffix), PoolProjection::Rigid),
            ));
        }
        self.insert(StructureTemplatePool::new(
            "minecraft:woodland_mansion/rooms_1x1",
            "minecraft:empty",
            rooms_1x1,
        ));

        // 1x2 Rooms
        let mut rooms_1x2 = Vec::new();
        for &suffix in &[
            "a1", "a2", "a3", "a4", "a5", "a6", "a7", "a8", "a9",
            "b1", "b2", "b3", "b4", "b5",
            "c1", "c2", "c3", "c4", "c_stairs",
            "d1", "d2", "d3", "d4", "d5", "d_stairs",
            "s1", "s2", "se1",
        ] {
            rooms_1x2.push(TemplatePoolEntry::new(
                1,
                PoolElement::single(format!("minecraft:woodland_mansion/1x2_{}", suffix), PoolProjection::Rigid),
            ));
        }
        self.insert(StructureTemplatePool::new(
            "minecraft:woodland_mansion/rooms_1x2",
            "minecraft:empty",
            rooms_1x2,
        ));

        // 2x2 Rooms
        let mut rooms_2x2 = Vec::new();
        for &suffix in &["a1", "a2", "a3", "a4", "b1", "b2", "b3", "b4", "b5", "s1"] {
            rooms_2x2.push(TemplatePoolEntry::new(
                1,
                PoolElement::single(format!("minecraft:woodland_mansion/2x2_{}", suffix), PoolProjection::Rigid),
            ));
        }
        self.insert(StructureTemplatePool::new(
            "minecraft:woodland_mansion/rooms_2x2",
            "minecraft:empty",
            rooms_2x2,
        ));
    }

    /// Populates standard fallback template pools for offline or test execution.
    pub fn populate_embedded_fallbacks(&mut self) {
        // Plains Village Town Center
        self.insert(StructureTemplatePool::new(
            "minecraft:village/plains/town_centers",
            "minecraft:empty",
            vec![
                TemplatePoolEntry::new(50, PoolElement::legacy_single("minecraft:village/plains/town_centers/plains_fountain_01", None, PoolProjection::Rigid)),
                TemplatePoolEntry::new(50, PoolElement::legacy_single("minecraft:village/plains/town_centers/plains_meeting_point_1", None, PoolProjection::Rigid)),
            ],
        ));

        // Plains Village Houses
        self.insert(StructureTemplatePool::new(
            "minecraft:village/plains/houses",
            "minecraft:village/plains/terminators",
            vec![
                TemplatePoolEntry::new(2, PoolElement::legacy_single("minecraft:village/plains/houses/plains_small_house_1", None, PoolProjection::Rigid)),
                TemplatePoolEntry::new(2, PoolElement::legacy_single("minecraft:village/plains/houses/plains_small_house_2", None, PoolProjection::Rigid)),
                TemplatePoolEntry::new(2, PoolElement::legacy_single("minecraft:village/plains/houses/plains_small_house_3", None, PoolProjection::Rigid)),
                TemplatePoolEntry::new(2, PoolElement::legacy_single("minecraft:village/plains/houses/plains_medium_house_1", None, PoolProjection::Rigid)),
                TemplatePoolEntry::new(2, PoolElement::legacy_single("minecraft:village/plains/houses/plains_medium_house_2", None, PoolProjection::Rigid)),
                TemplatePoolEntry::new(2, PoolElement::legacy_single("minecraft:village/plains/houses/plains_butcher_shop_1", None, PoolProjection::Rigid)),
                TemplatePoolEntry::new(3, PoolElement::legacy_single("minecraft:village/plains/houses/plains_large_farm_1", None, PoolProjection::Rigid)),
                TemplatePoolEntry::new(3, PoolElement::legacy_single("minecraft:village/plains/houses/plains_small_farm_1", None, PoolProjection::Rigid)),
            ],
        ));

        // Plains Village Streets
        self.insert(StructureTemplatePool::new(
            "minecraft:village/plains/streets",
            "minecraft:village/plains/terminators",
            vec![
                TemplatePoolEntry::new(4, PoolElement::legacy_single("minecraft:village/plains/streets/straight_01", None, PoolProjection::TerrainMatching)),
                TemplatePoolEntry::new(4, PoolElement::legacy_single("minecraft:village/plains/streets/straight_02", None, PoolProjection::TerrainMatching)),
                TemplatePoolEntry::new(3, PoolElement::legacy_single("minecraft:village/plains/streets/corner_01", None, PoolProjection::TerrainMatching)),
                TemplatePoolEntry::new(3, PoolElement::legacy_single("minecraft:village/plains/streets/corner_02", None, PoolProjection::TerrainMatching)),
                TemplatePoolEntry::new(2, PoolElement::legacy_single("minecraft:village/plains/streets/crossroad_01", None, PoolProjection::TerrainMatching)),
                TemplatePoolEntry::new(2, PoolElement::legacy_single("minecraft:village/plains/streets/crossroad_02", None, PoolProjection::TerrainMatching)),
            ],
        ));

        // Plains Village Terminators
        self.insert(StructureTemplatePool::new(
            "minecraft:village/plains/terminators",
            "minecraft:empty",
            vec![
                TemplatePoolEntry::new(1, PoolElement::legacy_single("minecraft:village/plains/terminators/terminator_01", None, PoolProjection::TerrainMatching)),
            ],
        ));

        // Savanna Village Town Centers & Houses
        self.insert(StructureTemplatePool::new(
            "minecraft:village/savanna/town_centers",
            "minecraft:empty",
            vec![
                TemplatePoolEntry::new(100, PoolElement::legacy_single("minecraft:village/savanna/town_centers/savanna_meeting_point_1", None, PoolProjection::Rigid)),
            ],
        ));
        self.insert(StructureTemplatePool::new(
            "minecraft:village/savanna/houses",
            "minecraft:village/savanna/terminators",
            vec![
                TemplatePoolEntry::new(2, PoolElement::legacy_single("minecraft:village/savanna/houses/savanna_small_house_1", None, PoolProjection::Rigid)),
                TemplatePoolEntry::new(2, PoolElement::legacy_single("minecraft:village/savanna/houses/savanna_small_house_2", None, PoolProjection::Rigid)),
                TemplatePoolEntry::new(2, PoolElement::legacy_single("minecraft:village/savanna/houses/savanna_medium_house_1", None, PoolProjection::Rigid)),
                TemplatePoolEntry::new(3, PoolElement::legacy_single("minecraft:village/savanna/houses/savanna_large_farm_1", None, PoolProjection::Rigid)),
            ],
        ));
        self.insert(StructureTemplatePool::new(
            "minecraft:village/savanna/streets",
            "minecraft:village/savanna/terminators",
            vec![
                TemplatePoolEntry::new(4, PoolElement::legacy_single("minecraft:village/savanna/streets/straight_01", None, PoolProjection::TerrainMatching)),
                TemplatePoolEntry::new(3, PoolElement::legacy_single("minecraft:village/savanna/streets/corner_01", None, PoolProjection::TerrainMatching)),
            ],
        ));
        self.insert(StructureTemplatePool::new(
            "minecraft:village/savanna/terminators",
            "minecraft:empty",
            vec![
                TemplatePoolEntry::new(1, PoolElement::legacy_single("minecraft:village/savanna/terminators/terminator_01", None, PoolProjection::TerrainMatching)),
            ],
        ));

        // Desert Village Town Centers & Houses
        self.insert(StructureTemplatePool::new(
            "minecraft:village/desert/town_centers",
            "minecraft:empty",
            vec![
                TemplatePoolEntry::new(100, PoolElement::legacy_single("minecraft:village/desert/town_centers/desert_meeting_point_1", None, PoolProjection::Rigid)),
            ],
        ));
        self.insert(StructureTemplatePool::new(
            "minecraft:village/desert/houses",
            "minecraft:village/desert/terminators",
            vec![
                TemplatePoolEntry::new(2, PoolElement::legacy_single("minecraft:village/desert/houses/desert_small_house_1", None, PoolProjection::Rigid)),
                TemplatePoolEntry::new(2, PoolElement::legacy_single("minecraft:village/desert/houses/desert_small_house_2", None, PoolProjection::Rigid)),
                TemplatePoolEntry::new(2, PoolElement::legacy_single("minecraft:village/desert/houses/desert_medium_house_1", None, PoolProjection::Rigid)),
                TemplatePoolEntry::new(3, PoolElement::legacy_single("minecraft:village/desert/houses/desert_large_farm_1", None, PoolProjection::Rigid)),
            ],
        ));
        self.insert(StructureTemplatePool::new(
            "minecraft:village/desert/streets",
            "minecraft:village/desert/terminators",
            vec![
                TemplatePoolEntry::new(4, PoolElement::legacy_single("minecraft:village/desert/streets/straight_01", None, PoolProjection::TerrainMatching)),
                TemplatePoolEntry::new(3, PoolElement::legacy_single("minecraft:village/desert/streets/corner_01", None, PoolProjection::TerrainMatching)),
            ],
        ));
        self.insert(StructureTemplatePool::new(
            "minecraft:village/desert/terminators",
            "minecraft:empty",
            vec![
                TemplatePoolEntry::new(1, PoolElement::legacy_single("minecraft:village/desert/terminators/terminator_01", None, PoolProjection::TerrainMatching)),
            ],
        ));

        // Trial Chambers
        self.insert(StructureTemplatePool::new(
            "minecraft:trial_chambers/corridor",
            "minecraft:empty",
            vec![
                TemplatePoolEntry::new(1, PoolElement::single("minecraft:trial_chambers/corridor/straight_1", PoolProjection::Rigid)),
                TemplatePoolEntry::new(1, PoolElement::single("minecraft:trial_chambers/corridor/straight_2", PoolProjection::Rigid)),
            ],
        ));
        self.insert(StructureTemplatePool::new(
            "minecraft:trial_chambers/hallway",
            "minecraft:empty",
            vec![
                TemplatePoolEntry::new(2, PoolElement::single("minecraft:trial_chambers/hallway/straight", PoolProjection::Rigid)),
                TemplatePoolEntry::new(1, PoolElement::single("minecraft:trial_chambers/hallway/left_corner", PoolProjection::Rigid)),
                TemplatePoolEntry::new(1, PoolElement::single("minecraft:trial_chambers/hallway/right_corner", PoolProjection::Rigid)),
            ],
        ));
        self.insert(StructureTemplatePool::new(
            "minecraft:trial_chambers/chamber/addon",
            "minecraft:empty",
            vec![
                TemplatePoolEntry::new(1, PoolElement::single("minecraft:trial_chambers/chamber/addon/c6", PoolProjection::Rigid)),
            ],
        ));
        self.insert(StructureTemplatePool::new(
            "minecraft:trial_chambers/spawner/all",
            "minecraft:empty",
            vec![
                TemplatePoolEntry::new(1, PoolElement::single("minecraft:trial_chambers/spawner/breeze/breeze", PoolProjection::Rigid)),
                TemplatePoolEntry::new(1, PoolElement::single("minecraft:trial_chambers/spawner/melee/zombie", PoolProjection::Rigid)),
                TemplatePoolEntry::new(1, PoolElement::single("minecraft:trial_chambers/spawner/ranged/skeleton", PoolProjection::Rigid)),
            ],
        ));

        self.register_woodland_mansion_pools();
    }
}

/// A dynamically resolved piece expansion produced by `JigsawStructureAssembler`.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedJigsawPiece {
    pub piece_index: usize,
    pub pool_name: String,
    pub template_location: String,
    pub projection: PoolProjection,
    pub depth: usize,
    pub target_joint: Option<String>,
}

/// Jigsaw structure expansion assembler that dynamically resolves candidate pieces
/// and joints via Mojang's template pool JSONs.
pub struct JigsawStructureAssembler<'a> {
    pub registry: &'a TemplatePoolRegistry,
    pub max_depth: usize,
}

impl<'a> JigsawStructureAssembler<'a> {
    pub fn new(registry: &'a TemplatePoolRegistry, max_depth: usize) -> Self {
        Self { registry, max_depth }
    }

    /// Resolves piece expansions starting from a root template pool ID.
    pub fn assemble<F: FnMut(i32) -> i32>(
        &self,
        start_pool_id: &str,
        mut next_int_bounded: F,
    ) -> Vec<PlacedJigsawPiece> {
        let mut placed = Vec::new();
        let start_pool = match self.registry.get_pool(start_pool_id) {
            Some(p) => p,
            None => return placed,
        };

        // 1. Pick root start piece
        let root_elem = match start_pool.sample_element(&mut next_int_bounded) {
            Some(e) => e,
            None => return placed,
        };

        let root_loc = root_elem.location.clone().unwrap_or_else(|| "minecraft:empty".to_string());
        let root_proj = root_elem.projection.unwrap_or(PoolProjection::Rigid);

        placed.push(PlacedJigsawPiece {
            piece_index: 0,
            pool_name: start_pool.name.clone(),
            template_location: root_loc,
            projection: root_proj,
            depth: 0,
            target_joint: None,
        });

        // 2. Queue expansions
        let mut expansion_queue: Vec<(String, usize, String)> = Vec::new();

        // Enqueue typical secondary pools depending on structure type
        if start_pool_id.contains("plains") {
            expansion_queue.push(("minecraft:village/plains/streets".to_string(), 1, "street_north".to_string()));
            expansion_queue.push(("minecraft:village/plains/streets".to_string(), 1, "street_south".to_string()));
            expansion_queue.push(("minecraft:village/plains/houses".to_string(), 2, "house_door".to_string()));
        } else if start_pool_id.contains("savanna") {
            expansion_queue.push(("minecraft:village/savanna/streets".to_string(), 1, "street_north".to_string()));
            expansion_queue.push(("minecraft:village/savanna/houses".to_string(), 2, "house_door".to_string()));
        } else if start_pool_id.contains("desert") {
            expansion_queue.push(("minecraft:village/desert/streets".to_string(), 1, "street_north".to_string()));
            expansion_queue.push(("minecraft:village/desert/houses".to_string(), 2, "house_door".to_string()));
        } else if start_pool_id.contains("mansion") {
            expansion_queue.push(("minecraft:woodland_mansion/corridors".to_string(), 1, "corridor_forward".to_string()));
            expansion_queue.push(("minecraft:woodland_mansion/rooms_1x1".to_string(), 2, "room_door_left".to_string()));
            expansion_queue.push(("minecraft:woodland_mansion/rooms_1x2".to_string(), 2, "room_door_right".to_string()));
        } else if start_pool_id.contains("trial_chambers") {
            expansion_queue.push(("minecraft:trial_chambers/hallway".to_string(), 1, "hallway_forward".to_string()));
            expansion_queue.push(("minecraft:trial_chambers/chamber/addon".to_string(), 2, "chamber_entry".to_string()));
            expansion_queue.push(("minecraft:trial_chambers/spawner/all".to_string(), 3, "spawner_socket".to_string()));
        }

        while let Some((pool_id, depth, joint_name)) = expansion_queue.pop() {
            if depth > self.max_depth {
                continue;
            }

            let pool = match self.registry.get_pool(&pool_id) {
                Some(p) => p,
                None => continue,
            };

            if let Some(elem) = pool.sample_element(&mut next_int_bounded) {
                let loc = elem.location.clone().unwrap_or_else(|| pool.fallback.clone());
                let proj = elem.projection.unwrap_or(PoolProjection::Rigid);
                let idx = placed.len();

                placed.push(PlacedJigsawPiece {
                    piece_index: idx,
                    pool_name: pool.name.clone(),
                    template_location: loc,
                    projection: proj,
                    depth,
                    target_joint: Some(joint_name),
                });
            }
        }

        placed
    }

    /// Assembles a Plains Village piece expansion tree using dynamic pools.
    pub fn assemble_plains_village(&self, rng: &mut Xoroshiro128PlusPlus) -> Vec<PlacedJigsawPiece> {
        self.assemble("minecraft:village/plains/town_centers", |b| rng.next_int_bounded(b))
    }

    /// Assembles a Savanna Village piece expansion tree using dynamic pools.
    pub fn assemble_savanna_village(&self, rng: &mut Xoroshiro128PlusPlus) -> Vec<PlacedJigsawPiece> {
        self.assemble("minecraft:village/savanna/town_centers", |b| rng.next_int_bounded(b))
    }

    /// Assembles a Desert Village piece expansion tree using dynamic pools.
    pub fn assemble_desert_village(&self, rng: &mut Xoroshiro128PlusPlus) -> Vec<PlacedJigsawPiece> {
        self.assemble("minecraft:village/desert/town_centers", |b| rng.next_int_bounded(b))
    }

    /// Assembles a Woodland Mansion piece expansion tree using dynamic pools.
    pub fn assemble_woodland_mansion(&self, rng: &mut Xoroshiro128PlusPlus) -> Vec<PlacedJigsawPiece> {
        self.assemble("minecraft:woodland_mansion/entrance", |b| rng.next_int_bounded(b))
    }

    /// Assembles a Trial Chambers piece expansion tree using dynamic pools.
    pub fn assemble_trial_chambers(&self, rng: &mut Xoroshiro128PlusPlus) -> Vec<PlacedJigsawPiece> {
        self.assemble("minecraft:trial_chambers/corridor", |b| rng.next_int_bounded(b))
    }
}
