// REA Verified: net.minecraft.world.level.chunk.storage.RegionFile (mapped: dmj.class)
// REA Verified: net.minecraft.world.level.chunk.storage.RegionFileStorage (mapped: dmk.class)
//
// Minecraft Anvil Region File (.mca) Reader & Writer:
// - 4096-byte chunk location table (3-byte sector offset + 1-byte sector count).
// - 4096-byte chunk timestamp table (unix epoch seconds).
// - Zlib/Deflate (type 2) compressed NBT chunk compounds.
// - Palette-based block states & dual nibble lighting serialization/deserialization.

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use glam::IVec3;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::render::atlas::BlockType;
use crate::render::mesher::ChunkSection;
use crate::storage::nbt::{
    deserialize_compound, serialize_compound, NbtCompound, NbtTag, TAG_COMPOUND,
};
use crate::world::lighting::SectionLight;

pub const SECTOR_BYTES: usize = 4096;
pub const SECTOR_INTS: usize = 1024;
pub const CHUNK_HEADER_BYTES: usize = 5; // 4 bytes length + 1 byte compression version
pub const VERSION_ZLIB: u8 = 2;

/// Anvil Region File (.mca) handling a 32x32 chunk area (1024 chunks total)
pub struct RegionFile {
    pub path: PathBuf,
    file: File,
    offsets: [u32; SECTOR_INTS],
    timestamps: [u32; SECTOR_INTS],
}

impl RegionFile {
    /// Opens an existing .mca file or initializes a new one with 8KiB headers
    pub fn open_or_create(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)?;

        let metadata = file.metadata()?;
        let mut offsets = [0u32; SECTOR_INTS];
        let mut timestamps = [0u32; SECTOR_INTS];

        if metadata.len() < (SECTOR_BYTES * 2) as u64 {
            // New or empty file: write 8192 zero bytes for location and timestamp tables
            file.seek(SeekFrom::Start(0))?;
            let empty_headers = vec![0u8; SECTOR_BYTES * 2];
            file.write_all(&empty_headers)?;
            file.flush()?;
        } else {
            // Read existing headers
            file.seek(SeekFrom::Start(0))?;
            let mut loc_buf = [0u8; SECTOR_BYTES];
            file.read_exact(&mut loc_buf)?;
            for i in 0..SECTOR_INTS {
                let off_b = &loc_buf[i * 4..(i + 1) * 4];
                offsets[i] = u32::from_be_bytes([off_b[0], off_b[1], off_b[2], off_b[3]]);
            }

            let mut time_buf = [0u8; SECTOR_BYTES];
            file.read_exact(&mut time_buf)?;
            for i in 0..SECTOR_INTS {
                let t_b = &time_buf[i * 4..(i + 1) * 4];
                timestamps[i] = u32::from_be_bytes([t_b[0], t_b[1], t_b[2], t_b[3]]);
            }
        }

        Ok(Self {
            path,
            file,
            offsets,
            timestamps,
        })
    }

    /// Returns reference to location offsets table
    #[inline]
    pub fn offsets(&self) -> &[u32; SECTOR_INTS] {
        &self.offsets
    }

    /// Checks if a chunk is present in the region file
    pub fn has_chunk(&self, cx: i32, cz: i32) -> bool {
        let idx = Self::get_chunk_index(cx, cz);
        let (offset, count) = Self::unpack_location(self.offsets[idx]);
        offset != 0 && count != 0
    }

    /// Computes table index for a chunk (cx, cz) within this 32x32 region
    #[inline]
    pub fn get_chunk_index(cx: i32, cz: i32) -> usize {
        let lx = (cx.rem_euclid(32)) as usize;
        let lz = (cz.rem_euclid(32)) as usize;
        lx + lz * 32
    }

    /// Unpacks sector offset and count from the packed location integer:
    /// (offset in 4KiB sectors, sector count)
    #[inline]
    pub fn unpack_location(packed: u32) -> (u32, u32) {
        let sector_offset = packed >> 8;
        let sector_count = packed & 0xFF;
        (sector_offset, sector_count)
    }

    /// Packs sector offset and count into standard 4-byte Anvil location entry
    #[inline]
    pub fn pack_location(sector_offset: u32, sector_count: u32) -> u32 {
        ((sector_offset & 0x00FF_FFFF) << 8) | (sector_count & 0xFF)
    }

    /// Reads and decompresses chunk NBT compound for (cx, cz)
    pub fn read_chunk(&mut self, cx: i32, cz: i32) -> io::Result<Option<Vec<u8>>> {
        let idx = Self::get_chunk_index(cx, cz);
        let packed = self.offsets[idx];
        let (sector_offset, sector_count) = Self::unpack_location(packed);

        if sector_offset == 0 || sector_count == 0 {
            return Ok(None); // Chunk not present in region
        }

        let file_offset = (sector_offset as u64) * (SECTOR_BYTES as u64);
        self.file.seek(SeekFrom::Start(file_offset))?;

        // Read 5-byte chunk header: [length: u32 BE, version: u8]
        let mut hdr = [0u8; CHUNK_HEADER_BYTES];
        self.file.read_exact(&mut hdr)?;

        let length = u32::from_be_bytes([hdr[0], hdr[1], hdr[2], hdr[3]]) as usize;
        let version = hdr[4];

        if length == 0 || length > (sector_count as usize) * SECTOR_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Corrupted chunk length header",
            ));
        }

        let compressed_len = length - 1;
        let mut compressed = vec![0u8; compressed_len];
        self.file.read_exact(&mut compressed)?;

        // Decompress payload
        let decompressed = match version {
            VERSION_ZLIB => {
                let mut decoder = ZlibDecoder::new(&compressed[..]);
                let mut data = Vec::new();
                decoder.read_to_end(&mut data)?;
                data
            }
            3 => compressed, // Uncompressed
            other => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unsupported compression scheme: {other}"),
                ))
            }
        };

        Ok(Some(decompressed))
    }

    /// Compresses and writes chunk NBT compound for (cx, cz)
    pub fn write_chunk(&mut self, cx: i32, cz: i32, uncompressed_nbt: &[u8]) -> io::Result<()> {
        let idx = Self::get_chunk_index(cx, cz);

        // Compress NBT data with Zlib
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(uncompressed_nbt)?;
        let compressed = encoder.finish()?;

        let length = (compressed.len() + 1) as u32; // +1 for version byte
        let total_packet_bytes = CHUNK_HEADER_BYTES - 1 + length as usize; // 4 + length
        let sectors_needed = ((total_packet_bytes + SECTOR_BYTES - 1) / SECTOR_BYTES) as u32;

        let (old_offset, old_count) = Self::unpack_location(self.offsets[idx]);

        // Allocate sector offset: reuse existing if sufficient, otherwise allocate at EOF
        let sector_offset = if old_offset >= 2 && sectors_needed <= old_count {
            old_offset
        } else {
            let file_len = self.file.metadata()?.len();
            let mut end_sector = ((file_len + SECTOR_BYTES as u64 - 1) / SECTOR_BYTES as u64) as u32;
            if end_sector < 2 {
                end_sector = 2;
            }
            end_sector
        };

        // Write chunk packet at sector offset
        let file_pos = (sector_offset as u64) * (SECTOR_BYTES as u64);
        self.file.seek(SeekFrom::Start(file_pos))?;

        // 4 bytes length + 1 byte version
        self.file.write_all(&length.to_be_bytes())?;
        self.file.write_all(&[VERSION_ZLIB])?;
        self.file.write_all(&compressed)?;

        // Pad remainder of allocated sectors with 0s
        let written_so_far = total_packet_bytes;
        let total_allocated = (sectors_needed as usize) * SECTOR_BYTES;
        if total_allocated > written_so_far {
            let padding = vec![0u8; total_allocated - written_so_far];
            self.file.write_all(&padding)?;
        }

        // Update location table
        self.offsets[idx] = Self::pack_location(sector_offset, sectors_needed);
        let loc_entry = self.offsets[idx].to_be_bytes();
        self.file.seek(SeekFrom::Start((idx * 4) as u64))?;
        self.file.write_all(&loc_entry)?;

        // Update timestamp table
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as u32)
            .unwrap_or(0);
        self.timestamps[idx] = timestamp;
        self.file.seek(SeekFrom::Start((SECTOR_BYTES + idx * 4) as u64))?;
        self.file.write_all(&timestamp.to_be_bytes())?;

        self.file.flush()?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Chunk Section & Palette NBT Serialization
// ---------------------------------------------------------------------------

pub fn block_to_vanilla_name(block: BlockType) -> &'static str {
    match block {
        BlockType::Air => "minecraft:air",
        BlockType::Stone => "minecraft:stone",
        BlockType::Dirt => "minecraft:dirt",
        BlockType::GrassBlock => "minecraft:grass_block",
        BlockType::Cobblestone => "minecraft:cobblestone",
        BlockType::Deepslate => "minecraft:deepslate",
        BlockType::OakLog => "minecraft:oak_log",
        BlockType::OakPlanks => "minecraft:oak_planks",
        BlockType::OakLeaves => "minecraft:oak_leaves",
        BlockType::Bedrock => "minecraft:bedrock",
        BlockType::Sand => "minecraft:sand",
        BlockType::Glass => "minecraft:glass",
        BlockType::Bricks => "minecraft:bricks",
        BlockType::Torch => "minecraft:torch",
        BlockType::Lava => "minecraft:lava",
        BlockType::Water => "minecraft:water",
        BlockType::Obsidian => "minecraft:obsidian",
        BlockType::Vine => "minecraft:vine",
        BlockType::Chest => "minecraft:chest",
    }
}

pub fn vanilla_name_to_block(name: &str) -> BlockType {
    match name {
        "minecraft:stone" => BlockType::Stone,
        "minecraft:dirt" => BlockType::Dirt,
        "minecraft:grass_block" => BlockType::GrassBlock,
        "minecraft:cobblestone" => BlockType::Cobblestone,
        "minecraft:deepslate" => BlockType::Deepslate,
        "minecraft:oak_log" => BlockType::OakLog,
        "minecraft:oak_planks" => BlockType::OakPlanks,
        "minecraft:oak_leaves" => BlockType::OakLeaves,
        "minecraft:bedrock" => BlockType::Bedrock,
        "minecraft:sand" => BlockType::Sand,
        "minecraft:glass" => BlockType::Glass,
        "minecraft:bricks" => BlockType::Bricks,
        "minecraft:torch" => BlockType::Torch,
        "minecraft:lava" => BlockType::Lava,
        "minecraft:water" => BlockType::Water,
        "minecraft:obsidian" => BlockType::Obsidian,
        "minecraft:vine" => BlockType::Vine,
        "minecraft:chest" => BlockType::Chest,
        _ => BlockType::Air,
    }
}

/// Serializes chunk column sections and light into a vanilla-compliant NBT Compound
pub fn serialize_chunk_column(
    cx: i32,
    cz: i32,
    sections: &[(i32, &ChunkSection, Option<&SectionLight>)],
) -> io::Result<Vec<u8>> {
    let mut root = NbtCompound::new();
    root.insert("DataVersion", NbtTag::Int(3700)); // 1.20.4
    root.insert("xPos", NbtTag::Int(cx));
    root.insert("zPos", NbtTag::Int(cz));
    root.insert("yPos", NbtTag::Int(0));
    root.insert("Status", NbtTag::String("minecraft:full".to_string()));

    let mut section_list = Vec::with_capacity(sections.len());

    for &(sec_y, section, light_opt) in sections {
        let mut sec_comp = HashMap::new();
        sec_comp.insert("Y".to_string(), NbtTag::Byte(sec_y as i8));

        // 1. Build palette & block states
        let mut palette = Vec::new();
        let mut palette_map = HashMap::new();

        // Always include Air as index 0
        palette.push(BlockType::Air);
        palette_map.insert(BlockType::Air, 0);

        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    let b = section.get(x, y, z);
                    if !palette_map.contains_key(&b) {
                        let idx = palette.len();
                        palette_map.insert(b, idx);
                        palette.push(b);
                    }
                }
            }
        }

        // Encode palette compounds
        let mut palette_tags = Vec::with_capacity(palette.len());
        for b in &palette {
            let mut pal_entry = HashMap::new();
            pal_entry.insert(
                "Name".to_string(),
                NbtTag::String(block_to_vanilla_name(*b).to_string()),
            );
            palette_tags.push(NbtTag::Compound(pal_entry));
        }

        // Pack voxel indices into LongArray
        let bits_per_block = (palette.len() as f32).log2().ceil().max(4.0) as usize;
        let blocks_per_long = 64 / bits_per_block;
        let long_count = (4096 + blocks_per_long - 1) / blocks_per_long;
        let mut packed_longs = vec![0i64; long_count];

        for i in 0..4096 {
            let x = i & 15;
            let z = (i >> 4) & 15;
            let y = (i >> 8) & 15;
            let block = section.get(x, y, z);
            let p_idx = *palette_map.get(&block).unwrap_or(&0) as i64;

            let long_idx = i / blocks_per_long;
            let bit_shift = (i % blocks_per_long) * bits_per_block;
            packed_longs[long_idx] |= (p_idx & ((1i64 << bits_per_block) - 1)) << bit_shift;
        }

        let mut block_states = HashMap::new();
        block_states.insert(
            "palette".to_string(),
            NbtTag::List(TAG_COMPOUND, palette_tags),
        );
        block_states.insert("data".to_string(), NbtTag::LongArray(packed_longs));
        sec_comp.insert("block_states".to_string(), NbtTag::Compound(block_states));

        // 2. Dual Nibble Lighting arrays (BlockLight & SkyLight, 2048 bytes each)
        if let Some(light) = light_opt {
            sec_comp.insert(
                "BlockLight".to_string(),
                NbtTag::ByteArray(light.block_light.raw_data().to_vec()),
            );
            sec_comp.insert(
                "SkyLight".to_string(),
                NbtTag::ByteArray(light.sky_light.raw_data().to_vec()),
            );
        }

        section_list.push(NbtTag::Compound(sec_comp));
    }

    root.insert("sections", NbtTag::List(TAG_COMPOUND, section_list));

    // 3. Block Entities list (id: "minecraft:chest", etc.)
    let mut block_entities = Vec::new();
    for &(_sec_y, section, _light_opt) in sections {
        for (_local_pos, te) in &section.tile_entities {
            block_entities.push(te.clone());
        }
    }
    root.insert(
        "block_entities",
        crate::world::tile_entity::serialize_block_entities(&block_entities),
    );

    serialize_compound(&root)
}

/// Deserializes chunk sections and light arrays from raw decompressed NBT bytes
pub fn deserialize_chunk_column(
    data: &[u8],
) -> io::Result<Vec<(i32, ChunkSection, Option<SectionLight>)>> {
    let compound = deserialize_compound(data)?;
    let sections_tag = match compound.get_list("sections") {
        Some(s) => s,
        None => return Ok(Vec::new()),
    };

    let mut result = Vec::new();

    for tag in sections_tag {
        let sec_map = match tag.as_compound() {
            Some(m) => m,
            None => continue,
        };

        let sec_y = match sec_map.get("Y").and_then(|t| t.as_byte()) {
            Some(y) => y as i32,
            None => continue,
        };

        let mut section = ChunkSection::new(IVec3::new(0, sec_y, 0));

        // Parse block_states
        if let Some(bs_map) = sec_map.get("block_states").and_then(|t| t.as_compound()) {
            let mut palette = Vec::new();
            if let Some(pal_list) = bs_map.get("palette").and_then(|t| t.as_list()) {
                for p_tag in pal_list {
                    if let Some(p_map) = p_tag.as_compound() {
                        if let Some(name) = p_map.get("Name").and_then(|t| t.as_string()) {
                            palette.push(vanilla_name_to_block(name));
                        }
                    }
                }
            }

            if palette.is_empty() {
                palette.push(BlockType::Air);
            }

            if let Some(NbtTag::LongArray(packed_longs)) = bs_map.get("data") {
                let bits_per_block = (palette.len() as f32).log2().ceil().max(4.0) as usize;
                let blocks_per_long = 64 / bits_per_block;
                let mask = (1i64 << bits_per_block) - 1;

                for i in 0..4096 {
                    let long_idx = i / blocks_per_long;
                    if long_idx < packed_longs.len() {
                        let bit_shift = (i % blocks_per_long) * bits_per_block;
                        let p_idx = ((packed_longs[long_idx] >> bit_shift) & mask) as usize;
                        let block = palette.get(p_idx).copied().unwrap_or(BlockType::Air);
                        let x = i & 15;
                        let z = (i >> 4) & 15;
                        let y = (i >> 8) & 15;
                        section.set(x, y, z, block);
                    }
                }
            } else if palette.len() == 1 {
                // Monolithic block section
                let fill_block = palette[0];
                for y in 0..16 {
                    for z in 0..16 {
                        for x in 0..16 {
                            section.set(x, y, z, fill_block);
                        }
                    }
                }
            }
        }

        // Parse light
        let mut light_opt = None;
        let block_light_raw = sec_map.get("BlockLight").and_then(|t| t.as_byte_array());
        let sky_light_raw = sec_map.get("SkyLight").and_then(|t| t.as_byte_array());

        if let (Some(bl), Some(sl)) = (block_light_raw, sky_light_raw) {
            let mut sec_light = SectionLight::new();
            if bl.len() == 2048 {
                sec_light.block_light.raw_data_mut().copy_from_slice(bl);
            }
            if sl.len() == 2048 {
                sec_light.sky_light.raw_data_mut().copy_from_slice(sl);
            }
            light_opt = Some(sec_light);
        }

        result.push((sec_y, section, light_opt));
    }

    if let Some(be_tag) = compound.get("block_entities") {
        let entities = crate::world::tile_entity::deserialize_block_entities(be_tag);
        for entity in entities {
            let pos = entity.position();
            let sec_y = pos.y.div_euclid(16);
            let local_pos = IVec3::new(
                pos.x.rem_euclid(16),
                pos.y.rem_euclid(16),
                pos.z.rem_euclid(16),
            );
            if let Some((_, section, _)) = result.iter_mut().find(|(y, _, _)| *y == sec_y) {
                section.set_tile_entity(local_pos, entity);
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_file_header_creation_and_offsets() {
        let temp_dir = std::env::temp_dir().join("iron_oxide_test_region");
        let mca_path = temp_dir.join("r.0.0.mca");
        let _ = std::fs::remove_file(&mca_path);

        let mut region = RegionFile::open_or_create(&mca_path).expect("Failed to create region file");

        // Verify initial empty tables (8192 bytes total header)
        assert_eq!(region.file.metadata().unwrap().len(), 8192);

        // Pack & unpack helper verification
        let packed = RegionFile::pack_location(12, 3);
        let (off, cnt) = RegionFile::unpack_location(packed);
        assert_eq!(off, 12);
        assert_eq!(cnt, 3);

        // Write test chunk (0, 0)
        let sample_nbt = b"Hello Anvil Region Storage!";
        region.write_chunk(0, 0, sample_nbt).expect("Write chunk failed");

        // Read chunk back
        let read_back = region.read_chunk(0, 0).expect("Read chunk failed").expect("Chunk missing");
        assert_eq!(read_back, sample_nbt);

        let _ = std::fs::remove_file(&mca_path);
    }

    #[test]
    fn test_chunk_column_nbt_serialization_and_roundtrip() {
        let mut section = ChunkSection::new(IVec3::ZERO);
        section.set(0, 0, 0, BlockType::Stone);
        section.set(1, 0, 0, BlockType::GrassBlock);
        section.set(2, 0, 0, BlockType::Torch);

        let mut light = SectionLight::new();
        light.set_block_light(2, 0, 0, 14); // Torch light

        let sections = vec![(0, &section, Some(&light))];
        let bytes = serialize_chunk_column(0, 0, &sections).expect("Serialization failed");

        let loaded = deserialize_chunk_column(&bytes).expect("Deserialization failed");
        assert_eq!(loaded.len(), 1);

        let (sec_y, loaded_sec, loaded_light) = &loaded[0];
        assert_eq!(*sec_y, 0);
        assert_eq!(loaded_sec.get(0, 0, 0), BlockType::Stone);
        assert_eq!(loaded_sec.get(1, 0, 0), BlockType::GrassBlock);
        assert_eq!(loaded_sec.get(2, 0, 0), BlockType::Torch);

        let l = loaded_light.as_ref().expect("Missing light");
        assert_eq!(l.get_block_light(2, 0, 0), 14);
    }
}
