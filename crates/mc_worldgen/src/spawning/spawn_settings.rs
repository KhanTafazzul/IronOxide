//! REA Verified: net.minecraft.world.level.biome.MobSpawnSettings (mapped: cva.class)
//! REA Verified: net.minecraft.world.level.biome.MobSpawnSettings$SpawnerData (mapped: cva$c.class)
//!
//! Biome spawn weight tables, pack size configurations, and `client.jar` JSON deserialization.

use super::mob_category::MobCategory;
use mc_entities::EntityType;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

/// Individual mob entry within a biome's spawn list.
/// REA Reference: `cva$c.class` (SpawnerData)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnerData {
    /// Entity type to spawn.
    pub entity_type: EntityType,
    /// Relative probability weight (higher weight = higher spawn chance).
    pub weight: u32,
    /// Minimum pack size instantiated per spawn event.
    pub min_count: u32,
    /// Maximum pack size instantiated per spawn event.
    pub max_count: u32,
}

impl SpawnerData {
    pub fn new(entity_type: EntityType, weight: u32, min_count: u32, max_count: u32) -> Self {
        Self {
            entity_type,
            weight,
            min_count: min_count.max(1),
            max_count: max_count.max(min_count.max(1)),
        }
    }
}

/// Raw JSON representation of a spawner entry in `data/minecraft/worldgen/biome/*.json`.
#[derive(Debug, Deserialize)]
struct RawSpawnerEntry {
    #[serde(rename = "type")]
    entity_id: String,
    weight: u32,
    #[serde(rename = "minCount")]
    min_count: u32,
    #[serde(rename = "maxCount")]
    max_count: u32,
}

/// Raw JSON representation of the `spawners` dictionary in biome JSON.
#[derive(Debug, Deserialize)]
struct RawBiomeJson {
    #[serde(default)]
    spawners: HashMap<String, Vec<RawSpawnerEntry>>,
    #[serde(rename = "creature_spawn_probability", default = "default_creature_prob")]
    creature_probability: f32,
}

fn default_creature_prob() -> f32 {
    0.1
}

/// Converts a Minecraft entity resource location string (e.g. `"minecraft:zombie"`) to `EntityType`.
pub fn parse_entity_type(id: &str) -> Option<EntityType> {
    let stripped = id.strip_prefix("minecraft:").unwrap_or(id);
    match stripped {
        "player" => Some(EntityType::Player),
        // Passive & Farm
        "cow" => Some(EntityType::Cow),
        "sheep" => Some(EntityType::Sheep),
        "pig" => Some(EntityType::Pig),
        "chicken" => Some(EntityType::Chicken),
        "horse" => Some(EntityType::Horse),
        "donkey" => Some(EntityType::Donkey),
        "mule" => Some(EntityType::Mule),
        "llama" => Some(EntityType::Llama),
        "trader_llama" => Some(EntityType::TraderLlama),
        "bat" => Some(EntityType::Bat),
        "villager" => Some(EntityType::Villager),
        "wandering_trader" => Some(EntityType::WanderingTrader),
        "iron_golem" => Some(EntityType::IronGolem),
        "snow_golem" => Some(EntityType::SnowGolem),
        "cat" => Some(EntityType::Cat),
        "ocelot" => Some(EntityType::Ocelot),
        "wolf" => Some(EntityType::Wolf),
        "fox" => Some(EntityType::Fox),
        "rabbit" => Some(EntityType::Rabbit),
        "frog" => Some(EntityType::Frog),
        "turtle" => Some(EntityType::Turtle),
        "axolotl" => Some(EntityType::Axolotl),
        "parrot" => Some(EntityType::Parrot),
        "panda" => Some(EntityType::Panda),
        "bee" => Some(EntityType::Bee),
        "strider" => Some(EntityType::Strider),
        "camel" => Some(EntityType::Camel),
        "sniffer" => Some(EntityType::Sniffer),
        "armadillo" => Some(EntityType::Armadillo),
        "mooshroom" => Some(EntityType::Mooshroom),
        "goat" => Some(EntityType::Goat),
        "polar_bear" => Some(EntityType::PolarBear),
        "allay" => Some(EntityType::Allay),

        // Aquatic
        "cod" => Some(EntityType::Cod),
        "salmon" => Some(EntityType::Salmon),
        "pufferfish" => Some(EntityType::Pufferfish),
        "tropical_fish" => Some(EntityType::TropicalFish),
        "dolphin" => Some(EntityType::Dolphin),
        "squid" => Some(EntityType::Squid),
        "glow_squid" => Some(EntityType::GlowSquid),
        "tadpole" => Some(EntityType::Tadpole),

        // Hostile & Neutral
        "zombie" => Some(EntityType::Zombie),
        "zombie_villager" => Some(EntityType::ZombieVillager),
        "husk" => Some(EntityType::Husk),
        "drowned" => Some(EntityType::Drowned),
        "skeleton" => Some(EntityType::Skeleton),
        "stray" => Some(EntityType::Stray),
        "bogged" => Some(EntityType::Bogged),
        "wither_skeleton" => Some(EntityType::WitherSkeleton),
        "creeper" => Some(EntityType::Creeper),
        "spider" => Some(EntityType::Spider),
        "cave_spider" => Some(EntityType::CaveSpider),
        "enderman" => Some(EntityType::Enderman),
        "piglin" => Some(EntityType::Piglin),
        "piglin_brute" => Some(EntityType::PiglinBrute),
        "zombified_piglin" => Some(EntityType::ZombifiedPiglin),
        "hoglin" => Some(EntityType::Hoglin),
        "zoglin" => Some(EntityType::Zoglin),
        "blaze" => Some(EntityType::Blaze),
        "ghast" => Some(EntityType::Ghast),
        "magma_cube" => Some(EntityType::MagmaCube),
        "slime" => Some(EntityType::Slime),
        "witch" => Some(EntityType::Witch),
        "wither" => Some(EntityType::Wither),
        "ender_dragon" => Some(EntityType::EnderDragon),
        "shulker" => Some(EntityType::Shulker),
        "vindicator" => Some(EntityType::Vindicator),
        "evoker" => Some(EntityType::Evoker),
        "pillager" => Some(EntityType::Pillager),
        "ravager" => Some(EntityType::Ravager),
        "vex" => Some(EntityType::Vex),
        "phantom" => Some(EntityType::Phantom),
        "guardian" => Some(EntityType::Guardian),
        "elder_guardian" => Some(EntityType::ElderGuardian),
        "warden" => Some(EntityType::Warden),
        "breeze" => Some(EntityType::Breeze),
        "creaking" => Some(EntityType::Creaking),
        "silverfish" => Some(EntityType::Silverfish),
        "endermite" => Some(EntityType::Endermite),
        _ => None,
    }
}

/// Spawning settings and weighted lists for a single biome.
/// REA Reference: `cva.class` (MobSpawnSettings)
#[derive(Debug, Clone, PartialEq)]
pub struct BiomeSpawnSettings {
    pub spawners: HashMap<MobCategory, Vec<SpawnerData>>,
    pub creature_probability: f32,
}

impl Default for BiomeSpawnSettings {
    fn default() -> Self {
        Self {
            spawners: HashMap::new(),
            creature_probability: 0.1,
        }
    }
}

impl BiomeSpawnSettings {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a spawn entry to the specified category.
    pub fn add_spawn(&mut self, category: MobCategory, spawner: SpawnerData) -> &mut Self {
        self.spawners.entry(category).or_default().push(spawner);
        self
    }

    /// Retrieves all spawner entries for the category.
    pub fn get_mobs(&self, category: MobCategory) -> &[SpawnerData] {
        self.spawners.get(&category).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Performs weighted random selection of a mob entry given a uniform roll in `[0.0, 1.0)`.
    pub fn sample_mob(&self, category: MobCategory, roll: f32) -> Option<&SpawnerData> {
        let entries = self.get_mobs(category);
        if entries.is_empty() {
            return None;
        }

        let total_weight: u32 = entries.iter().map(|e| e.weight).sum();
        if total_weight == 0 {
            return None;
        }

        let target = (roll * total_weight as f32) as u32;
        let mut accum = 0u32;
        for entry in entries {
            accum += entry.weight;
            if target < accum {
                return Some(entry);
            }
        }

        entries.last()
    }
}

/// Central registry storing `BiomeSpawnSettings` for all registered biomes.
#[derive(Debug, Clone, Default)]
pub struct BiomeSpawnRegistry {
    biomes: HashMap<String, BiomeSpawnSettings>,
}

impl BiomeSpawnRegistry {
    pub fn new() -> Self {
        Self {
            biomes: HashMap::new(),
        }
    }

    /// Registers spawn settings for a biome key (e.g. `"minecraft:plains"`).
    pub fn register(&mut self, biome_key: impl Into<String>, settings: BiomeSpawnSettings) {
        self.biomes.insert(biome_key.into(), settings);
    }

    /// Looks up spawn settings by biome resource location.
    pub fn get_settings(&self, biome_key: &str) -> Option<&BiomeSpawnSettings> {
        self.biomes.get(biome_key)
    }

    /// Looks up spawn settings, returning default fallback if not found.
    pub fn get_or_default(&self, biome_key: &str) -> BiomeSpawnSettings {
        self.biomes.get(biome_key).cloned().unwrap_or_else(|| Self::default_overworld_settings())
    }

    /// Attempts to locate `client.jar` in standard relative paths and loads all biome spawn tables.
    pub fn load_from_client_jar() -> Option<Self> {
        let candidates = [
            "client.jar",
            "../client.jar",
            "../../client.jar",
            "../../../client.jar",
            "c:/Users/khant/OneDrive/Desktop/Falix/Minecraft reverce in rust/client.jar",
        ];

        for path in candidates {
            let p = PathBuf::from(path);
            if p.exists() {
                if let Ok(reg) = Self::load_from_jar(&p) {
                    if !reg.biomes.is_empty() {
                        return Some(reg);
                    }
                }
            }
        }

        None
    }

    /// Loads all biome spawn tables directly from a `client.jar` file.
    pub fn load_from_jar(jar_path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(jar_path)?;
        let reader = BufReader::new(file);
        let mut archive = ZipArchive::new(reader)?;
        Self::load_from_zip(&mut archive)
    }

    /// Parses biome spawn tables from any opened ZipArchive.
    pub fn load_from_zip<R: Read + Seek>(
        archive: &mut ZipArchive<R>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut registry = Self::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = match file.name() {
                Ok(n) => n.to_string(),
                Err(_) => continue,
            };

            if name.starts_with("data/minecraft/worldgen/biome/") && name.ends_with(".json") {
                let stem = &name["data/minecraft/worldgen/biome/".len()..name.len() - 5];
                let biome_key = format!("minecraft:{}", stem);

                let mut content = String::new();
                file.read_to_string(&mut content)?;

                if let Ok(raw) = serde_json::from_str::<RawBiomeJson>(&content) {
                    let mut settings = BiomeSpawnSettings {
                        spawners: HashMap::new(),
                        creature_probability: raw.creature_probability,
                    };

                    for (cat_name, entries) in raw.spawners {
                        if let Some(category) = MobCategory::from_name(&cat_name) {
                            for raw_entry in entries {
                                if let Some(entity_type) = parse_entity_type(&raw_entry.entity_id) {
                                    settings.add_spawn(
                                        category,
                                        SpawnerData::new(
                                            entity_type,
                                            raw_entry.weight,
                                            raw_entry.min_count,
                                            raw_entry.max_count,
                                        ),
                                    );
                                }
                            }
                        }
                    }

                    registry.register(biome_key, settings);
                }
            }
        }

        Ok(registry)
    }

    /// Returns a comprehensive fallback registry matching official vanilla Overworld, Nether, and End tables.
    pub fn fallback_registry() -> Self {
        let mut reg = Self::new();

        // 1. Plains / Sunflower Plains
        let mut plains = BiomeSpawnSettings::new();
        plains.creature_probability = 0.1;
        plains.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Sheep, 12, 4, 4));
        plains.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Pig, 10, 4, 4));
        plains.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Chicken, 10, 4, 4));
        plains.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Cow, 8, 4, 4));
        plains.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Horse, 5, 2, 6));
        plains.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Donkey, 1, 1, 3));
        plains.add_spawn(MobCategory::Ambient, SpawnerData::new(EntityType::Bat, 10, 8, 8));
        plains.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Spider, 100, 4, 4));
        plains.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Zombie, 95, 4, 4));
        plains.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::ZombieVillager, 5, 1, 1));
        plains.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Skeleton, 100, 4, 4));
        plains.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Creeper, 100, 4, 4));
        plains.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Slime, 100, 4, 4));
        plains.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Enderman, 10, 1, 4));
        plains.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Witch, 5, 1, 1));
        reg.register("minecraft:plains", plains.clone());
        reg.register("minecraft:sunflower_plains", plains);

        // 2. Forest / Flower Forest / Birch Forest
        let mut forest = BiomeSpawnSettings::new();
        forest.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Sheep, 12, 4, 4));
        forest.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Pig, 10, 4, 4));
        forest.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Chicken, 10, 4, 4));
        forest.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Cow, 8, 4, 4));
        forest.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Wolf, 5, 4, 4));
        forest.add_spawn(MobCategory::Ambient, SpawnerData::new(EntityType::Bat, 10, 8, 8));
        forest.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Spider, 100, 4, 4));
        forest.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Zombie, 95, 4, 4));
        forest.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Skeleton, 100, 4, 4));
        forest.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Creeper, 100, 4, 4));
        forest.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Slime, 100, 4, 4));
        forest.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Enderman, 10, 1, 4));
        forest.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Witch, 5, 1, 1));
        reg.register("minecraft:forest", forest.clone());
        reg.register("minecraft:flower_forest", forest.clone());
        reg.register("minecraft:birch_forest", forest);

        // 3. Desert
        let mut desert = BiomeSpawnSettings::new();
        desert.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Rabbit, 4, 2, 3));
        desert.add_spawn(MobCategory::Ambient, SpawnerData::new(EntityType::Bat, 10, 8, 8));
        desert.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Spider, 100, 4, 4));
        desert.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Husk, 80, 4, 4));
        desert.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Zombie, 19, 4, 4));
        desert.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::ZombieVillager, 1, 1, 1));
        desert.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Skeleton, 100, 4, 4));
        desert.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Creeper, 100, 4, 4));
        desert.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Slime, 100, 4, 4));
        desert.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Enderman, 10, 1, 4));
        desert.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Witch, 5, 1, 1));
        reg.register("minecraft:desert", desert);

        // 4. Taiga / Snowy Taiga
        let mut taiga = BiomeSpawnSettings::new();
        taiga.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Wolf, 8, 4, 4));
        taiga.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Fox, 8, 2, 4));
        taiga.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Rabbit, 4, 2, 3));
        taiga.add_spawn(MobCategory::Ambient, SpawnerData::new(EntityType::Bat, 10, 8, 8));
        taiga.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Spider, 100, 4, 4));
        taiga.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Zombie, 95, 4, 4));
        taiga.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Skeleton, 100, 4, 4));
        taiga.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Creeper, 100, 4, 4));
        taiga.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Slime, 100, 4, 4));
        taiga.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Enderman, 10, 1, 4));
        reg.register("minecraft:taiga", taiga.clone());
        reg.register("minecraft:snowy_taiga", taiga);

        // 5. Ocean / Deep Ocean
        let mut ocean = BiomeSpawnSettings::new();
        ocean.add_spawn(MobCategory::WaterAmbient, SpawnerData::new(EntityType::Cod, 10, 3, 6));
        ocean.add_spawn(MobCategory::WaterCreature, SpawnerData::new(EntityType::Squid, 1, 1, 4));
        ocean.add_spawn(MobCategory::WaterCreature, SpawnerData::new(EntityType::Dolphin, 1, 1, 2));
        ocean.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Drowned, 5, 1, 1));
        reg.register("minecraft:ocean", ocean.clone());
        reg.register("minecraft:deep_ocean", ocean);

        // 6. Warm Ocean
        let mut warm_ocean = BiomeSpawnSettings::new();
        warm_ocean.add_spawn(MobCategory::WaterAmbient, SpawnerData::new(EntityType::TropicalFish, 25, 8, 8));
        warm_ocean.add_spawn(MobCategory::WaterAmbient, SpawnerData::new(EntityType::Pufferfish, 15, 1, 3));
        warm_ocean.add_spawn(MobCategory::WaterCreature, SpawnerData::new(EntityType::Squid, 10, 1, 4));
        warm_ocean.add_spawn(MobCategory::WaterCreature, SpawnerData::new(EntityType::Dolphin, 2, 1, 2));
        reg.register("minecraft:warm_ocean", warm_ocean);

        // 7. Swamp / Mangrove Swamp
        let mut swamp = BiomeSpawnSettings::new();
        swamp.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Frog, 10, 2, 5));
        swamp.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Slime, 1, 1, 1));
        swamp.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Witch, 5, 1, 1));
        swamp.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Bogged, 30, 4, 4));
        reg.register("minecraft:swamp", swamp.clone());
        reg.register("minecraft:mangrove_swamp", swamp);

        // 8. Nether Wastes
        let mut nether = BiomeSpawnSettings::new();
        nether.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Ghast, 50, 4, 4));
        nether.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::ZombifiedPiglin, 100, 4, 4));
        nether.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::MagmaCube, 2, 4, 4));
        nether.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Enderman, 1, 4, 4));
        nether.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Strider, 60, 2, 4));
        reg.register("minecraft:nether_wastes", nether);

        // 9. Pale Garden (1.21.4)
        let mut pale_garden = BiomeSpawnSettings::new();
        pale_garden.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Creaking, 10, 1, 1));
        pale_garden.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Skeleton, 100, 4, 4));
        pale_garden.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Creeper, 100, 4, 4));
        reg.register("minecraft:pale_garden", pale_garden);

        reg
    }

    /// Default generic Overworld settings.
    pub fn default_overworld_settings() -> BiomeSpawnSettings {
        let mut settings = BiomeSpawnSettings::new();
        settings.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Zombie, 95, 4, 4));
        settings.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Skeleton, 100, 4, 4));
        settings.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Creeper, 100, 4, 4));
        settings.add_spawn(MobCategory::Monster, SpawnerData::new(EntityType::Spider, 100, 4, 4));
        settings.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Cow, 8, 4, 4));
        settings.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Sheep, 12, 4, 4));
        settings.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Pig, 10, 4, 4));
        settings.add_spawn(MobCategory::Creature, SpawnerData::new(EntityType::Chicken, 10, 4, 4));
        settings.add_spawn(MobCategory::Ambient, SpawnerData::new(EntityType::Bat, 10, 8, 8));
        settings
    }
}
