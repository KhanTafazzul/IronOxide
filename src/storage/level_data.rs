// REA Verified: net.minecraft.world.level.storage.LevelSummary (mapped: egn.class)
// REA Verified: net.minecraft.world.level.storage.PlayerDataStorage (mapped: cfi.class & blv.class)
// REA Verified: net.minecraft.world.level.storage.LevelStorageSource (mapped: egm.class)
//
// Minecraft level.dat NBT Parser & World Metadata:
// - Gzip-compressed root compound with "Data" subcompound.
// - World attributes: LevelName, GameType, Time, DayTime, SpawnX, SpawnY, SpawnZ.
// - Player compound: Pos ([x, y, z] doubles), Rotation ([yaw, pitch] floats),
//   Motion ([vx, vy, vz] doubles), Inventory (list of item stack compounds).

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

use crate::game::inventory::{
    ItemStack, PlayerInventory, ITEM_AIR, ITEM_BEDROCK, ITEM_BRICKS, ITEM_COBBLESTONE,
    ITEM_CRAFTING_TABLE, ITEM_DEEPSLATE, ITEM_DIRT, ITEM_GLASS, ITEM_GLOWSTONE,
    ITEM_GRASS_BLOCK, ITEM_LANTERN, ITEM_LAVA, ITEM_OAK_LEAVES, ITEM_OAK_LOG,
    ITEM_OAK_PLANKS, ITEM_OBSIDIAN, ITEM_SAND, ITEM_STICK, ITEM_STONE, ITEM_TORCH,
    ITEM_WATER, ITEM_WOODEN_AXE, ITEM_WOODEN_PICKAXE,
};
use crate::game::PlayerPhysics;
use crate::render::Camera;
use crate::storage::nbt::{
    deserialize_compound, serialize_compound, NbtCompound, NbtTag, TAG_COMPOUND, TAG_DOUBLE,
    TAG_FLOAT,
};

/// Item ID translation to Minecraft canonical string identifiers
pub fn item_name_to_id(name: &str) -> u16 {
    let stripped = name.strip_prefix("minecraft:").unwrap_or(name);
    match stripped {
        "air" => ITEM_AIR,
        "grass_block" => ITEM_GRASS_BLOCK,
        "dirt" => ITEM_DIRT,
        "stone" => ITEM_STONE,
        "cobblestone" => ITEM_COBBLESTONE,
        "deepslate" => ITEM_DEEPSLATE,
        "oak_planks" | "planks" => ITEM_OAK_PLANKS,
        "oak_log" | "log" => ITEM_OAK_LOG,
        "oak_leaves" | "leaves" => ITEM_OAK_LEAVES,
        "bedrock" => ITEM_BEDROCK,
        "sand" => ITEM_SAND,
        "glass" => ITEM_GLASS,
        "bricks" => ITEM_BRICKS,
        "crafting_table" => ITEM_CRAFTING_TABLE,
        "stick" => ITEM_STICK,
        "wooden_pickaxe" => ITEM_WOODEN_PICKAXE,
        "wooden_axe" => ITEM_WOODEN_AXE,
        "torch" => ITEM_TORCH,
        "lava" | "lava_bucket" => ITEM_LAVA,
        "water" | "water_bucket" => ITEM_WATER,
        "obsidian" => ITEM_OBSIDIAN,
        "lantern" => ITEM_LANTERN,
        "glowstone" => ITEM_GLOWSTONE,
        _ => ITEM_AIR,
    }
}

/// Canonical Minecraft string identifier from Item ID
pub fn item_id_to_name(id: u16) -> &'static str {
    match id {
        ITEM_GRASS_BLOCK => "minecraft:grass_block",
        ITEM_DIRT => "minecraft:dirt",
        ITEM_STONE => "minecraft:stone",
        ITEM_COBBLESTONE => "minecraft:cobblestone",
        ITEM_DEEPSLATE => "minecraft:deepslate",
        ITEM_OAK_PLANKS => "minecraft:oak_planks",
        ITEM_OAK_LOG => "minecraft:oak_log",
        ITEM_OAK_LEAVES => "minecraft:oak_leaves",
        ITEM_BEDROCK => "minecraft:bedrock",
        ITEM_SAND => "minecraft:sand",
        ITEM_GLASS => "minecraft:glass",
        ITEM_BRICKS => "minecraft:bricks",
        ITEM_CRAFTING_TABLE => "minecraft:crafting_table",
        ITEM_STICK => "minecraft:stick",
        ITEM_WOODEN_PICKAXE => "minecraft:wooden_pickaxe",
        ITEM_WOODEN_AXE => "minecraft:wooden_axe",
        ITEM_TORCH => "minecraft:torch",
        ITEM_LAVA => "minecraft:lava_bucket",
        ITEM_WATER => "minecraft:water_bucket",
        ITEM_OBSIDIAN => "minecraft:obsidian",
        ITEM_LANTERN => "minecraft:lantern",
        ITEM_GLOWSTONE => "minecraft:glowstone",
        _ => "minecraft:air",
    }
}

/// Saved item stack inside Player.Inventory
#[derive(Debug, Clone, PartialEq)]
pub struct SavedItemStack {
    pub slot: i8,
    pub id: String,
    pub count: i8,
}

/// Player Entity data stored in `level.dat`
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerData {
    /// World position [x, y, z] in blocks
    pub pos: [f64; 3],
    /// Orientation angles [yaw, pitch] in degrees
    pub rotation: [f32; 2],
    /// Velocity vector [vx, vy, vz]
    pub motion: [f64; 3],
    /// Saved player inventory slots
    pub inventory: Vec<SavedItemStack>,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            pos: [8.5, 65.0, 8.5],
            rotation: [0.0, 0.0],
            motion: [0.0, 0.0, 0.0],
            inventory: Vec::new(),
        }
    }
}

/// Complete `level.dat` world metadata and player snapshot
#[derive(Debug, Clone, PartialEq)]
pub struct LevelData {
    /// Display name of the world
    pub level_name: String,
    /// Game mode: 0 = Survival, 1 = Creative, 2 = Adventure, 3 = Spectator
    pub game_type: u8,
    /// Total world age in ticks
    pub time: i64,
    /// Current time of day in ticks (0..24000)
    pub day_time: i64,
    /// World spawn coordinate X
    pub spawn_x: i32,
    /// World spawn coordinate Y
    pub spawn_y: i32,
    /// World spawn coordinate Z
    pub spawn_z: i32,
    /// Player state snapshot
    pub player: PlayerData,
    /// World generation seed (RandomSeed)
    pub seed: i64,
}

impl Default for LevelData {
    fn default() -> Self {
        Self {
            level_name: "WorldName".to_string(),
            game_type: 0, // Survival
            time: 0,
            day_time: 1000, // Morning
            spawn_x: 8,
            spawn_y: 64,
            spawn_z: 8,
            player: PlayerData::default(),
            seed: 123456789,
        }
    }
}

impl LevelData {
    /// Construct a new LevelData with default settings
    pub fn new(world_name: &str) -> Self {
        Self {
            level_name: world_name.to_string(),
            ..Default::default()
        }
    }

    /// Parse LevelData from an uncompressed NBT Compound
    pub fn from_compound(compound: &NbtCompound) -> io::Result<Self> {
        let data_map = if let Some(sub) = compound.get_compound("Data") {
            sub
        } else {
            &compound.map
        };

        let level_name = data_map
            .get("LevelName")
            .and_then(|t| t.as_string())
            .unwrap_or("WorldName")
            .to_string();

        let game_type = data_map
            .get("GameType")
            .and_then(|t| t.as_byte().map(|b| b as u8).or_else(|| t.as_int().map(|i| i as u8)))
            .unwrap_or(0);

        let time = data_map
            .get("Time")
            .and_then(|t| match t {
                NbtTag::Long(l) => Some(*l),
                NbtTag::Int(i) => Some(*i as i64),
                _ => None,
            })
            .unwrap_or(0);

        let day_time = data_map
            .get("DayTime")
            .and_then(|t| match t {
                NbtTag::Long(l) => Some(*l),
                NbtTag::Int(i) => Some(*i as i64),
                _ => None,
            })
            .unwrap_or(1000);

        let spawn_x = data_map.get("SpawnX").and_then(|t| t.as_int()).unwrap_or(8);
        let spawn_y = data_map.get("SpawnY").and_then(|t| t.as_int()).unwrap_or(64);
        let spawn_z = data_map.get("SpawnZ").and_then(|t| t.as_int()).unwrap_or(8);

        let player = if let Some(player_map) = data_map.get("Player").and_then(|t| t.as_compound()) {
            Self::parse_player_data(player_map, spawn_x, spawn_y, spawn_z)?
        } else {
            PlayerData {
                pos: [spawn_x as f64 + 0.5, spawn_y as f64 + 1.0, spawn_z as f64 + 0.5],
                rotation: [0.0, 0.0],
                motion: [0.0, 0.0, 0.0],
                inventory: Vec::new(),
            }
        };

        let seed = data_map
            .get("RandomSeed")
            .or_else(|| {
                data_map
                    .get("WorldGenSettings")
                    .and_then(|t| t.as_compound())
                    .and_then(|w| w.get("seed"))
            })
            .and_then(|t| match t {
                NbtTag::Long(l) => Some(*l),
                NbtTag::Int(i) => Some(*i as i64),
                _ => None,
            })
            .unwrap_or(123456789);

        Ok(Self {
            level_name,
            game_type,
            time,
            day_time,
            spawn_x,
            spawn_y,
            spawn_z,
            player,
            seed,
        })
    }

    fn parse_player_data(
        player_map: &HashMap<String, NbtTag>,
        fallback_x: i32,
        fallback_y: i32,
        fallback_z: i32,
    ) -> io::Result<PlayerData> {
        let pos = if let Some(pos_list) = player_map.get("Pos").and_then(|t| t.as_list()) {
            let mut coords = [
                fallback_x as f64 + 0.5,
                fallback_y as f64 + 1.0,
                fallback_z as f64 + 0.5,
            ];
            for (idx, tag) in pos_list.iter().enumerate().take(3) {
                if let NbtTag::Double(d) = tag {
                    coords[idx] = *d;
                } else if let NbtTag::Float(f) = tag {
                    coords[idx] = *f as f64;
                }
            }
            coords
        } else {
            [
                fallback_x as f64 + 0.5,
                fallback_y as f64 + 1.0,
                fallback_z as f64 + 0.5,
            ]
        };

        let rotation = if let Some(rot_list) = player_map.get("Rotation").and_then(|t| t.as_list()) {
            let mut angles = [0.0f32, 0.0f32];
            for (idx, tag) in rot_list.iter().enumerate().take(2) {
                if let NbtTag::Float(f) = tag {
                    angles[idx] = *f;
                } else if let NbtTag::Double(d) = tag {
                    angles[idx] = *d as f32;
                }
            }
            angles
        } else {
            [0.0, 0.0]
        };

        let motion = if let Some(mot_list) = player_map.get("Motion").and_then(|t| t.as_list()) {
            let mut velocities = [0.0f64, 0.0f64, 0.0f64];
            for (idx, tag) in mot_list.iter().enumerate().take(3) {
                if let NbtTag::Double(d) = tag {
                    velocities[idx] = *d;
                } else if let NbtTag::Float(f) = tag {
                    velocities[idx] = *f as f64;
                }
            }
            velocities
        } else {
            [0.0, 0.0, 0.0]
        };

        let mut inventory = Vec::new();
        if let Some(inv_list) = player_map.get("Inventory").and_then(|t| t.as_list()) {
            for item_tag in inv_list {
                if let Some(item_map) = item_tag.as_compound() {
                    let slot = item_map.get("Slot").and_then(|t| t.as_byte()).unwrap_or(0);
                    let id = item_map
                        .get("id")
                        .and_then(|t| t.as_string())
                        .unwrap_or("minecraft:air")
                        .to_string();
                    let count = item_map.get("Count").and_then(|t| t.as_byte()).unwrap_or(1);
                    inventory.push(SavedItemStack { slot, id, count });
                }
            }
        }

        Ok(PlayerData {
            pos,
            rotation,
            motion,
            inventory,
        })
    }

    /// Serialize LevelData to NBT Compound structure matching vanilla level.dat
    pub fn to_compound(&self) -> NbtCompound {
        let mut data_map = HashMap::new();
        data_map.insert("LevelName".to_string(), NbtTag::String(self.level_name.clone()));
        data_map.insert("GameType".to_string(), NbtTag::Byte(self.game_type as i8));
        data_map.insert("Time".to_string(), NbtTag::Long(self.time));
        data_map.insert("DayTime".to_string(), NbtTag::Long(self.day_time));
        data_map.insert("SpawnX".to_string(), NbtTag::Int(self.spawn_x));
        data_map.insert("SpawnY".to_string(), NbtTag::Int(self.spawn_y));
        data_map.insert("SpawnZ".to_string(), NbtTag::Int(self.spawn_z));
        data_map.insert("RandomSeed".to_string(), NbtTag::Long(self.seed));

        // Player compound
        let mut player_map = HashMap::new();
        let pos_tags = vec![
            NbtTag::Double(self.player.pos[0]),
            NbtTag::Double(self.player.pos[1]),
            NbtTag::Double(self.player.pos[2]),
        ];
        player_map.insert("Pos".to_string(), NbtTag::List(TAG_DOUBLE, pos_tags));

        let rot_tags = vec![
            NbtTag::Float(self.player.rotation[0]),
            NbtTag::Float(self.player.rotation[1]),
        ];
        player_map.insert("Rotation".to_string(), NbtTag::List(TAG_FLOAT, rot_tags));

        let mot_tags = vec![
            NbtTag::Double(self.player.motion[0]),
            NbtTag::Double(self.player.motion[1]),
            NbtTag::Double(self.player.motion[2]),
        ];
        player_map.insert("Motion".to_string(), NbtTag::List(TAG_DOUBLE, mot_tags));

        let mut inv_tags = Vec::new();
        for item in &self.player.inventory {
            let mut item_map = HashMap::new();
            item_map.insert("Slot".to_string(), NbtTag::Byte(item.slot));
            item_map.insert("id".to_string(), NbtTag::String(item.id.clone()));
            item_map.insert("Count".to_string(), NbtTag::Byte(item.count));
            inv_tags.push(NbtTag::Compound(item_map));
        }
        player_map.insert("Inventory".to_string(), NbtTag::List(TAG_COMPOUND, inv_tags));

        data_map.insert("Player".to_string(), NbtTag::Compound(player_map));

        let mut root = NbtCompound::new();
        root.insert("Data", NbtTag::Compound(data_map));
        root
    }

    /// Parse gzip-compressed level.dat byte slice
    pub fn from_gzip_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut decoder = GzDecoder::new(bytes);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        let compound = deserialize_compound(&decompressed)?;
        Self::from_compound(&compound)
    }

    /// Serialize to gzip-compressed level.dat bytes
    pub fn to_gzip_bytes(&self) -> io::Result<Vec<u8>> {
        let compound = self.to_compound();
        let uncompressed = serialize_compound(&compound)?;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&uncompressed)?;
        encoder.finish()
    }

    /// Load gzip-compressed level.dat from disk
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let bytes = fs::read(path)?;
        Self::from_gzip_bytes(&bytes)
    }

    /// Save gzip-compressed level.dat to disk
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let gz_bytes = self.to_gzip_bytes()?;
        fs::write(path, gz_bytes)
    }

    /// Synchronize state from player, camera, and inventory into LevelData
    pub fn update_from_state(
        &mut self,
        player: &PlayerPhysics,
        camera: &Camera,
        inventory: &PlayerInventory,
    ) {
        self.player.pos = [
            player.position.x as f64,
            player.position.y as f64,
            player.position.z as f64,
        ];
        self.player.rotation = [camera.yaw, camera.pitch];
        self.player.motion = [
            player.velocity.x as f64,
            player.velocity.y as f64,
            player.velocity.z as f64,
        ];
        self.player.inventory = inventory_to_saved_items(inventory);
    }
}

/// Convert PlayerInventory to vanilla NBT saved items
pub fn inventory_to_saved_items(inv: &PlayerInventory) -> Vec<SavedItemStack> {
    let mut items = Vec::new();
    // Hotbar: slots 0..9 -> NBT Slot 0..8
    for (i, opt) in inv.hotbar.iter().enumerate() {
        if let Some(stack) = opt {
            if !stack.is_empty() {
                items.push(SavedItemStack {
                    slot: i as i8,
                    id: item_id_to_name(stack.id).to_string(),
                    count: stack.count as i8,
                });
            }
        }
    }
    // Main inventory: slots 9..35 -> NBT Slot 9..35
    for (i, opt) in inv.main.iter().enumerate() {
        if let Some(stack) = opt {
            if !stack.is_empty() {
                items.push(SavedItemStack {
                    slot: (i + 9) as i8,
                    id: item_id_to_name(stack.id).to_string(),
                    count: stack.count as i8,
                });
            }
        }
    }
    // Offhand -> NBT Slot -106
    if let Some(stack) = inv.offhand {
        if !stack.is_empty() {
            items.push(SavedItemStack {
                slot: -106,
                id: item_id_to_name(stack.id).to_string(),
                count: stack.count as i8,
            });
        }
    }
    // Armor slots: 100=boots, 101=leggings, 102=chestplate, 103=helmet
    for (i, &slot_id) in [103i8, 102, 101, 100].iter().enumerate() {
        if let Some(stack) = inv.armor[i] {
            if !stack.is_empty() {
                items.push(SavedItemStack {
                    slot: slot_id,
                    id: item_id_to_name(stack.id).to_string(),
                    count: stack.count as i8,
                });
            }
        }
    }
    items
}

/// Restore player inventory from NBT saved items
pub fn load_saved_items_into_inventory(saved: &[SavedItemStack], inv: &mut PlayerInventory) {
    for item in saved {
        let stack = ItemStack::new(item_name_to_id(&item.id), item.count.max(1) as u8);
        if stack.is_empty() {
            continue;
        }
        match item.slot {
            0..=8 => inv.hotbar[item.slot as usize] = Some(stack),
            9..=35 => inv.main[(item.slot - 9) as usize] = Some(stack),
            -106 | 45 => inv.offhand = Some(stack),
            103 => inv.armor[0] = Some(stack), // Helmet
            102 => inv.armor[1] = Some(stack), // Chestplate
            101 => inv.armor[2] = Some(stack), // Leggings
            100 => inv.armor[3] = Some(stack), // Boots
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_data_gzip_roundtrip() {
        let mut original = LevelData::new("Survival World");
        original.game_type = 0; // Survival
        original.time = 45000;
        original.day_time = 12000;
        original.spawn_x = 100;
        original.spawn_y = 65;
        original.spawn_z = -200;

        original.player = PlayerData {
            pos: [102.5, 66.0, -198.5],
            rotation: [45.0, -10.0],
            motion: [0.05, -0.07, 0.0],
            inventory: vec![
                SavedItemStack {
                    slot: 0,
                    id: "minecraft:torch".to_string(),
                    count: 32,
                },
                SavedItemStack {
                    slot: 1,
                    id: "minecraft:cobblestone".to_string(),
                    count: 64,
                },
                SavedItemStack {
                    slot: -106,
                    id: "minecraft:lantern".to_string(),
                    count: 1,
                },
            ],
        };
        original.seed = 123456789;

        // Serialize to gzip bytes
        let gz_bytes = original.to_gzip_bytes().expect("Gzip compression failed");
        assert!(!gz_bytes.is_empty());

        // Parse from gzip bytes
        let decoded = LevelData::from_gzip_bytes(&gz_bytes).expect("Gzip decompression failed");

        assert_eq!(decoded.level_name, "Survival World");
        assert_eq!(decoded.game_type, 0);
        assert_eq!(decoded.time, 45000);
        assert_eq!(decoded.day_time, 12000);
        assert_eq!(decoded.spawn_x, 100);
        assert_eq!(decoded.spawn_y, 65);
        assert_eq!(decoded.spawn_z, -200);

        assert_eq!(decoded.player.pos, [102.5, 66.0, -198.5]);
        assert_eq!(decoded.player.rotation, [45.0, -10.0]);
        assert_eq!(decoded.player.motion, [0.05, -0.07, 0.0]);
        assert_eq!(decoded.player.inventory.len(), 3);
        assert_eq!(decoded.player.inventory[0].id, "minecraft:torch");
        assert_eq!(decoded.player.inventory[0].count, 32);
        assert_eq!(decoded.player.inventory[2].slot, -106);
    }

    #[test]
    fn test_inventory_nbt_mapping() {
        let mut inv = PlayerInventory::new();
        inv.hotbar = [None; 9];
        inv.hotbar[0] = Some(ItemStack::new(ITEM_TORCH, 16));
        inv.hotbar[1] = Some(ItemStack::new(ITEM_COBBLESTONE, 64));
        inv.offhand = Some(ItemStack::new(ITEM_LANTERN, 1));

        let saved = inventory_to_saved_items(&inv);
        assert_eq!(saved.len(), 3);

        let mut restored = PlayerInventory::new();
        load_saved_items_into_inventory(&saved, &mut restored);

        assert_eq!(restored.hotbar[0], Some(ItemStack::new(ITEM_TORCH, 16)));
        assert_eq!(restored.hotbar[1], Some(ItemStack::new(ITEM_COBBLESTONE, 64)));
        assert_eq!(restored.offhand, Some(ItemStack::new(ITEM_LANTERN, 1)));
    }
}
