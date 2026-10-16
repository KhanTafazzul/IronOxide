// REA Verified: net.minecraft.world.level.chunk.storage.RegionFileStorage (mapped: dmk.class)
// World Persistence & Asynchronous Region File Storage (.mca) Manager

pub mod nbt;
pub mod region;

pub use nbt::{NbtCompound, NbtTag};
pub use region::{
    deserialize_chunk_column, serialize_chunk_column, RegionFile, SECTOR_BYTES,
};

use glam::IVec3;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};

use crate::game::World;
use crate::render::mesher::ChunkSection;
use crate::world::lighting::SectionLight;

/// Background storage command sent from game loop to persistence thread
pub enum StorageCommand {
    SaveChunk {
        cx: i32,
        cz: i32,
        nbt_bytes: Vec<u8>,
    },
    FlushAll,
    Shutdown,
}

/// Asynchronous Anvil world persistence coordinator.
/// Manages background disk I/O to prevents game freezes during chunk autosaves.
pub struct WorldStorageManager {
    pub world_name: String,
    pub region_dir: PathBuf,
    pub dirty_chunks: HashSet<(i32, i32)>,
    sender: Option<Sender<StorageCommand>>,
    worker_handle: Option<JoinHandle<()>>,
}

impl WorldStorageManager {
    /// Initializes world storage in `saves/{world_name}/region` and starts background worker
    pub fn new(world_name: &str) -> Self {
        let region_dir = PathBuf::from("saves").join(world_name).join("region");
        if let Err(e) = fs::create_dir_all(&region_dir) {
            eprintln!("[Storage] Failed to create region directory {}: {e}", region_dir.display());
        }

        let (tx, rx) = channel::<StorageCommand>();
        let worker_dir = region_dir.clone();

        let handle = thread::Builder::new()
            .name("IronOxide-RegionSaver".to_string())
            .spawn(move || {
                Self::run_worker(rx, worker_dir);
            })
            .expect("Failed to spawn RegionSaver background thread");

        Self {
            world_name: world_name.to_string(),
            region_dir,
            dirty_chunks: HashSet::new(),
            sender: Some(tx),
            worker_handle: Some(handle),
        }
    }

    /// Background persistence worker processing queued chunk saves
    fn run_worker(rx: Receiver<StorageCommand>, region_dir: PathBuf) {
        let mut open_regions: HashMap<(i32, i32), RegionFile> = HashMap::new();

        while let Ok(cmd) = rx.recv() {
            match cmd {
                StorageCommand::SaveChunk { cx, cz, nbt_bytes } => {
                    let rx_coord = cx.div_euclid(32);
                    let rz_coord = cz.div_euclid(32);
                    let reg_key = (rx_coord, rz_coord);

                    let region = open_regions.entry(reg_key).or_insert_with(|| {
                        let path = region_dir.join(format!("r.{rx_coord}.{rz_coord}.mca"));
                        RegionFile::open_or_create(path).expect("Failed to open or create region file")
                    });

                    if let Err(e) = region.write_chunk(cx, cz, &nbt_bytes) {
                        eprintln!("[Storage] Error writing chunk ({cx}, {cz}) to region ({rx_coord}, {rz_coord}): {e}");
                    }
                }
                StorageCommand::FlushAll => {
                    open_regions.clear();
                }
                StorageCommand::Shutdown => {
                    open_regions.clear();
                    break;
                }
            }
        }
    }

    /// Mark a chunk (cx, cz) as modified
    pub fn mark_dirty(&mut self, cx: i32, cz: i32) {
        self.dirty_chunks.insert((cx, cz));
    }

    /// Mark chunk containing world voxel pos as modified
    pub fn mark_block_dirty(&mut self, world_pos: IVec3) {
        let cx = world_pos.x.div_euclid(16);
        let cz = world_pos.z.div_euclid(16);
        self.mark_dirty(cx, cz);
    }

    /// Mark chunks containing modified sections as dirty
    pub fn mark_sections_dirty(&mut self, section_coords: &[IVec3]) {
        for sec in section_coords {
            self.mark_dirty(sec.x, sec.z);
        }
    }

    /// Snapshots all dirty chunks and dispatches them asynchronously to the background saver
    pub fn flush_dirty(&mut self, world: &World) {
        if self.dirty_chunks.is_empty() {
            return;
        }

        let dirty: Vec<(i32, i32)> = self.dirty_chunks.drain().collect();
        for (cx, cz) in dirty {
            self.save_chunk_async(cx, cz, world);
        }
    }

    /// Forces all currently loaded chunks in the world to be queued for saving
    pub fn flush_all(&mut self, world: &World) {
        let mut all_chunks = HashSet::new();
        for pos in world.sections.keys() {
            all_chunks.insert((pos.x, pos.z));
        }

        for (cx, cz) in all_chunks {
            self.save_chunk_async(cx, cz, world);
        }
        self.dirty_chunks.clear();

        if let Some(tx) = &self.sender {
            let _ = tx.send(StorageCommand::FlushAll);
        }
    }

    /// Internal helper: snapshot chunk column sections and dispatch to background saver
    fn save_chunk_async(&self, cx: i32, cz: i32, world: &World) {
        let mut column_sections = Vec::new();

        for (&pos, section) in &world.sections {
            if pos.x == cx && pos.z == cz {
                let light_opt = world.lighting.sections.get(&pos);
                column_sections.push((pos.y, section, light_opt));
            }
        }

        column_sections.sort_by_key(|&(sec_y, _, _)| sec_y);

        if column_sections.is_empty() {
            return;
        }

        match serialize_chunk_column(cx, cz, &column_sections) {
            Ok(nbt_bytes) => {
                if let Some(tx) = &self.sender {
                    let _ = tx.send(StorageCommand::SaveChunk {
                        cx,
                        cz,
                        nbt_bytes,
                    });
                }
            }
            Err(e) => {
                eprintln!("[Storage] Serialization failed for chunk ({cx}, {cz}): {e}");
            }
        }
    }

    /// Loads chunk column from corresponding region file on disk
    pub fn load_chunk_column(
        &self,
        cx: i32,
        cz: i32,
    ) -> io::Result<Option<Vec<(i32, ChunkSection, Option<SectionLight>)>>> {
        let rx_coord = cx.div_euclid(32);
        let rz_coord = cz.div_euclid(32);
        let mca_path = self.region_dir.join(format!("r.{rx_coord}.{rz_coord}.mca"));

        if !mca_path.exists() {
            return Ok(None);
        }

        let mut region = RegionFile::open_or_create(mca_path)?;
        match region.read_chunk(cx, cz)? {
            Some(data) => {
                let sections = deserialize_chunk_column(&data)?;
                Ok(Some(sections))
            }
            None => Ok(None),
        }
    }

    /// Checks if a chunk exists saved in region storage
    pub fn has_chunk(&self, cx: i32, cz: i32) -> bool {
        let rx_coord = cx.div_euclid(32);
        let rz_coord = cz.div_euclid(32);
        let mca_path = self.region_dir.join(format!("r.{rx_coord}.{rz_coord}.mca"));
        if !mca_path.exists() {
            return false;
        }

        if let Ok(region) = RegionFile::open_or_create(mca_path) {
            return region.has_chunk(cx, cz);
        }
        false
    }
}

impl Drop for WorldStorageManager {
    fn drop(&mut self) {
        if let Some(tx) = self.sender.take() {
            let _ = tx.send(StorageCommand::Shutdown);
        }
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}
