// REA Verified: net.minecraft.world.level.block.entity.ChestBlockEntity (mapped: dhc.class)
// REA Verified: net.minecraft.world.level.block.entity.RandomizableContainerBlockEntity (mapped: dhz.class)
// REA Verified: net.minecraft.world.level.block.entity.BlockEntity (mapped: dgv.class)
//
// Tile Entity System & Chest Storage:
// - Non-standard block state storage associated with chunk sections.
// - ChestTileEntity with 27 item slots ([Option<ItemStack>; 27]).
// - Vanilla-compliant Anvil NBT serialization & deserialization (id: "minecraft:chest", x, y, z, Items list).

use glam::IVec3;
use std::collections::HashMap;

use crate::game::inventory::{
    ItemStack, ITEM_AIR, ITEM_BEDROCK, ITEM_BRICKS, ITEM_CHEST, ITEM_COBBLESTONE,
    ITEM_CRAFTING_TABLE, ITEM_DEEPSLATE, ITEM_DIRT, ITEM_GLASS, ITEM_GLOWSTONE, ITEM_GRASS_BLOCK,
    ITEM_LANTERN, ITEM_LAVA, ITEM_OAK_LEAVES, ITEM_OAK_LOG, ITEM_OAK_PLANKS, ITEM_OBSIDIAN,
    ITEM_SAND, ITEM_STICK, ITEM_STONE, ITEM_TORCH, ITEM_VINE, ITEM_WATER, ITEM_WOODEN_AXE,
    ITEM_WOODEN_PICKAXE,
};
use crate::storage::nbt::{NbtCompound, NbtTag, TAG_COMPOUND};

pub const CHEST_SLOT_COUNT: usize = 27;

/// Converts engine Item ID to vanilla Minecraft resource identifier
pub fn item_id_to_vanilla_name(id: u16) -> &'static str {
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
        ITEM_TORCH => "minecraft:torch",
        ITEM_LAVA => "minecraft:lava_bucket",
        ITEM_WATER => "minecraft:water_bucket",
        ITEM_OBSIDIAN => "minecraft:obsidian",
        ITEM_LANTERN => "minecraft:lantern",
        ITEM_GLOWSTONE => "minecraft:glowstone",
        ITEM_CRAFTING_TABLE => "minecraft:crafting_table",
        ITEM_STICK => "minecraft:stick",
        ITEM_WOODEN_PICKAXE => "minecraft:wooden_pickaxe",
        ITEM_WOODEN_AXE => "minecraft:wooden_axe",
        ITEM_VINE => "minecraft:vine",
        ITEM_CHEST => "minecraft:chest",
        _ => "minecraft:air",
    }
}

/// Converts vanilla Minecraft resource identifier to engine Item ID
pub fn vanilla_name_to_item_id(name: &str) -> u16 {
    match name {
        "minecraft:grass_block" => ITEM_GRASS_BLOCK,
        "minecraft:dirt" => ITEM_DIRT,
        "minecraft:stone" => ITEM_STONE,
        "minecraft:cobblestone" => ITEM_COBBLESTONE,
        "minecraft:deepslate" => ITEM_DEEPSLATE,
        "minecraft:oak_planks" => ITEM_OAK_PLANKS,
        "minecraft:oak_log" => ITEM_OAK_LOG,
        "minecraft:oak_leaves" => ITEM_OAK_LEAVES,
        "minecraft:bedrock" => ITEM_BEDROCK,
        "minecraft:sand" => ITEM_SAND,
        "minecraft:glass" => ITEM_GLASS,
        "minecraft:bricks" => ITEM_BRICKS,
        "minecraft:torch" => ITEM_TORCH,
        "minecraft:lava" | "minecraft:lava_bucket" => ITEM_LAVA,
        "minecraft:water" | "minecraft:water_bucket" => ITEM_WATER,
        "minecraft:obsidian" => ITEM_OBSIDIAN,
        "minecraft:lantern" => ITEM_LANTERN,
        "minecraft:glowstone" => ITEM_GLOWSTONE,
        "minecraft:crafting_table" => ITEM_CRAFTING_TABLE,
        "minecraft:stick" => ITEM_STICK,
        "minecraft:wooden_pickaxe" => ITEM_WOODEN_PICKAXE,
        "minecraft:wooden_axe" => ITEM_WOODEN_AXE,
        "minecraft:vine" => ITEM_VINE,
        "minecraft:chest" => ITEM_CHEST,
        _ => ITEM_AIR,
    }
}

/// Tile Entity for single Chest container with 27 inventory slots (3 rows of 9)
#[derive(Debug, Clone, PartialEq)]
pub struct ChestTileEntity {
    pub position: IVec3,
    pub items: [Option<ItemStack>; CHEST_SLOT_COUNT],
    pub custom_name: Option<String>,
}

impl ChestTileEntity {
    pub fn new(position: IVec3) -> Self {
        Self {
            position,
            items: [None; CHEST_SLOT_COUNT],
            custom_name: None,
        }
    }

    /// Read item stack in chest slot (0..26)
    pub fn get_item(&self, slot: usize) -> Option<ItemStack> {
        if slot < CHEST_SLOT_COUNT {
            self.items[slot]
        } else {
            None
        }
    }

    /// Set or clear item stack in chest slot (0..26)
    pub fn set_item(&mut self, slot: usize, stack: Option<ItemStack>) {
        if slot < CHEST_SLOT_COUNT {
            self.items[slot] = stack.filter(|s| !s.is_empty());
        }
    }

    /// Returns true if all 27 slots are empty
    pub fn is_empty(&self) -> bool {
        self.items.iter().all(|s| s.is_none() || s.map_or(true, |i| i.is_empty()))
    }

    /// Clears all 27 slots
    pub fn clear(&mut self) {
        self.items = [None; CHEST_SLOT_COUNT];
    }

    /// Serializes this chest tile entity into a vanilla-compliant NBT Compound
    pub fn to_nbt(&self) -> NbtCompound {
        let mut comp = NbtCompound::new();
        comp.insert("id", NbtTag::String("minecraft:chest".to_string()));
        comp.insert("x", NbtTag::Int(self.position.x));
        comp.insert("y", NbtTag::Int(self.position.y));
        comp.insert("z", NbtTag::Int(self.position.z));
        comp.insert("keepPacked", NbtTag::Byte(0));

        if let Some(name) = &self.custom_name {
            comp.insert("CustomName", NbtTag::String(name.clone()));
        }

        let mut item_tags = Vec::new();
        for (slot, item_opt) in self.items.iter().enumerate() {
            if let Some(stack) = item_opt {
                if !stack.is_empty() {
                    let mut item_comp = HashMap::new();
                    item_comp.insert("Slot".to_string(), NbtTag::Byte(slot as i8));
                    item_comp.insert(
                        "id".to_string(),
                        NbtTag::String(item_id_to_vanilla_name(stack.id).to_string()),
                    );
                    item_comp.insert("Count".to_string(), NbtTag::Byte(stack.count as i8));
                    item_tags.push(NbtTag::Compound(item_comp));
                }
            }
        }

        comp.insert("Items", NbtTag::List(TAG_COMPOUND, item_tags));
        comp
    }

    /// Deserializes a chest tile entity from a vanilla-compliant NBT Compound
    pub fn from_nbt(comp: &NbtCompound) -> Option<Self> {
        let id = comp.get("id").and_then(|t| t.as_string())?;
        if id != "minecraft:chest" {
            return None;
        }

        let x = comp.get("x").and_then(|t| t.as_int()).unwrap_or(0);
        let y = comp.get("y").and_then(|t| t.as_int()).unwrap_or(0);
        let z = comp.get("z").and_then(|t| t.as_int()).unwrap_or(0);
        let pos = IVec3::new(x, y, z);

        let mut chest = Self::new(pos);
        if let Some(name) = comp.get("CustomName").and_then(|t| t.as_string()) {
            chest.custom_name = Some(name.to_string());
        }

        if let Some(items_list) = comp.get_list("Items") {
            for tag in items_list {
                if let Some(item_map) = tag.as_compound() {
                    let slot_opt = item_map.get("Slot").and_then(|t| t.as_byte());
                    let id_opt = item_map.get("id").and_then(|t| t.as_string());
                    let count_opt = item_map.get("Count").and_then(|t| t.as_byte());

                    if let (Some(slot_byte), Some(item_name), Some(count_byte)) =
                        (slot_opt, id_opt, count_opt)
                    {
                        let slot = slot_byte as usize;
                        let item_id = vanilla_name_to_item_id(item_name);
                        let count = count_byte.max(0) as u8;

                        if slot < CHEST_SLOT_COUNT && item_id != ITEM_AIR && count > 0 {
                            chest.items[slot] = Some(ItemStack::new(item_id, count));
                        }
                    }
                }
            }
        }

        Some(chest)
    }
}

/// Generic Tile Entity enum for non-standard chunk voxel states
#[derive(Debug, Clone, PartialEq)]
pub enum TileEntity {
    Chest(ChestTileEntity),
}

impl TileEntity {
    pub fn position(&self) -> IVec3 {
        match self {
            TileEntity::Chest(c) => c.position,
        }
    }

    pub fn to_nbt(&self) -> NbtCompound {
        match self {
            TileEntity::Chest(c) => c.to_nbt(),
        }
    }

    pub fn from_nbt(comp: &NbtCompound) -> Option<Self> {
        let id = comp.get("id").and_then(|t| t.as_string())?;
        match id {
            "minecraft:chest" => ChestTileEntity::from_nbt(comp).map(TileEntity::Chest),
            _ => None,
        }
    }
}

/// Serializes a collection of tile entities into an Anvil `block_entities` NbtTag::List
pub fn serialize_block_entities(entities: &[TileEntity]) -> NbtTag {
    let mut list = Vec::with_capacity(entities.len());
    for entity in entities {
        list.push(NbtTag::Compound(entity.to_nbt().map));
    }
    NbtTag::List(TAG_COMPOUND, list)
}

/// Deserializes a collection of tile entities from an Anvil `block_entities` NbtTag::List
pub fn deserialize_block_entities(tag: &NbtTag) -> Vec<TileEntity> {
    let mut entities = Vec::new();
    if let Some(list) = tag.as_list() {
        for item in list {
            if let Some(comp_map) = item.as_compound() {
                let mut comp = NbtCompound::new();
                comp.map = comp_map.clone();
                if let Some(entity) = TileEntity::from_nbt(&comp) {
                    entities.push(entity);
                }
            }
        }
    }
    entities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chest_tile_entity_initialization_and_slots() {
        let pos = IVec3::new(10, 64, -20);
        let mut chest = ChestTileEntity::new(pos);
        assert_eq!(chest.position, pos);
        assert!(chest.is_empty());
        assert_eq!(chest.get_item(0), None);

        // Place items in chest
        let stack1 = ItemStack::new(ITEM_OAK_PLANKS, 32);
        let stack2 = ItemStack::new(ITEM_STONE, 64);
        chest.set_item(0, Some(stack1));
        chest.set_item(26, Some(stack2));

        assert!(!chest.is_empty());
        assert_eq!(chest.get_item(0), Some(stack1));
        assert_eq!(chest.get_item(26), Some(stack2));
        assert_eq!(chest.get_item(1), None);

        // Clear chest
        chest.clear();
        assert!(chest.is_empty());
        assert_eq!(chest.get_item(0), None);
    }

    #[test]
    fn test_chest_nbt_serialization_and_roundtrip() {
        let pos = IVec3::new(42, 70, -105);
        let mut chest = ChestTileEntity::new(pos);
        chest.custom_name = Some("Treasure Stash".to_string());
        chest.set_item(4, Some(ItemStack::new(ITEM_TORCH, 16)));
        chest.set_item(18, Some(ItemStack::new(ITEM_COBBLESTONE, 64)));

        // Serialize to NBT
        let nbt = chest.to_nbt();
        assert_eq!(
            nbt.get("id").and_then(|t| t.as_string()),
            Some("minecraft:chest")
        );
        assert_eq!(nbt.get("x").and_then(|t| t.as_int()), Some(42));
        assert_eq!(nbt.get("y").and_then(|t| t.as_int()), Some(70));
        assert_eq!(nbt.get("z").and_then(|t| t.as_int()), Some(-105));
        assert_eq!(
            nbt.get("CustomName").and_then(|t| t.as_string()),
            Some("Treasure Stash")
        );

        // Verify Items list contains 2 entries
        let items_list = nbt.get_list("Items").expect("Items list should exist");
        assert_eq!(items_list.len(), 2);

        // Deserialize back
        let recovered = ChestTileEntity::from_nbt(&nbt).expect("Deserialization failed");
        assert_eq!(recovered.position, pos);
        assert_eq!(recovered.custom_name, Some("Treasure Stash".to_string()));
        assert_eq!(recovered.get_item(4), Some(ItemStack::new(ITEM_TORCH, 16)));
        assert_eq!(
            recovered.get_item(18),
            Some(ItemStack::new(ITEM_COBBLESTONE, 64))
        );
        assert_eq!(recovered.get_item(0), None);
    }

    #[test]
    fn test_tile_entity_batch_serialization_and_deserialization() {
        let chest1 = ChestTileEntity::new(IVec3::new(0, 60, 0));
        let mut chest2 = ChestTileEntity::new(IVec3::new(5, 60, 5));
        chest2.set_item(2, Some(ItemStack::new(ITEM_DIRT, 10)));

        let entities = vec![TileEntity::Chest(chest1), TileEntity::Chest(chest2)];
        let tag = serialize_block_entities(&entities);

        let recovered = deserialize_block_entities(&tag);
        assert_eq!(recovered.len(), 2);
        assert_eq!(recovered[0].position(), IVec3::new(0, 60, 0));
        assert_eq!(recovered[1].position(), IVec3::new(5, 60, 5));
    }
}
