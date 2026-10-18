# IronOxide

High-Performance Voxel Game Engine and Minecraft Client Architecture in Rust

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Language: Rust 2024](https://img.shields.io/badge/Language-Rust_2024-orange.svg)](https://www.rust-lang.org/)
[![Graphics: WGPU](https://img.shields.io/badge/Graphics-WGPU_/_WGSL-green.svg)](https://wgpu.rs/)
[![Tests: 216 Passing](https://img.shields.io/badge/Tests-216_Passed-brightgreen.svg)]()

---

## Executive Overview

IronOxide is a clean-room, native voxel game engine and client implementation of Minecraft (targeting version 1.20.4 protocol and mechanics) written in Rust. It utilizes modern systems-level programming paradigms, zero-cost abstractions, data-oriented design, and raw hardware access via the WGPU graphics API and WGSL shaders.

The engine achieves deterministic behavioral fidelity with the vanilla Java edition by directly referencing decompiled client bytecodes (`client.jar`) through reverse engineering analysis (REA). All critical submodules—including 3D chunk meshing, sub-voxel raycasting, decoupled AABB collision dynamics, multi-octave biome generation, breadth-first light propagation, and recursive structure assembly—are designed for maximum throughput, low memory footprint, and parallel execution.

---

## Architectural Highlights

### 1. Rendering Pipeline and Shaders
* **Hardware Abstraction Layer**: Built on top of WGPU, targeting Vulkan, DirectX 12, Metal, and WebGPU backends.
* **Pixel-Accurate Sampling**: Point-filtered texture sampling (`FilterMode::Nearest`) preserving the authentic pixelated visual identity.
* **Dynamic Texture Atlas**: Automated extraction and runtime stitching of vanilla block textures from `client.jar` (with algorithmic procedural fallback generation) providing sub-pixel UV coordinates for all cardinal directions.
* **Asynchronous 3D Greedy Mesher**:
  * Parallelized across CPU cores using `rayon`.
  * Merges coplanar, adjacent faces sharing identical textures into minimal rectangular geometry, drastically reducing draw call overhead and triangle counts.
  * Computes 4-level Ambient Occlusion (AO 0 to 3) per vertex based on neighbor occupancy.
  * Multi-section boundary updates to eliminate seam artifacts between neighboring chunks.
* **Biome Colormap Interpolation**:
  * Dual-layer colormap sampler (`grass.png` and `foliage.png`) utilizing triangular barycentric UV coordinate projection based on temperature and downfall metrics.
  * 3x3 block-kernel neighborhood smoothing yielding smooth biome color transitions across section boundaries.
* **View Frustum Culling**:
  * Extraction of 6 normalized frustum planes directly from the combined view-projection matrix.
  * Conservative axis-aligned bounding box (AABB) intersection tests for full 16x16x16 chunk sections.

### 2. Volumetric Weather, Dynamic Cycles & Accumulation
* **Dynamic Weather Cycle**:
  * Global timer state machine: `clear_weather_time` (12,000 to 168,000 ticks) and `rain_time` / `thunder_time` (12,000 to 24,000 / 168,000 ticks).
  * 50-tick smooth linear fade (`RAIN_FADE_PER_TICK = 0.02`) controlling visual rain/snow opacity, atmospheric sky light attenuation (up to 25%), and fog distance compression (up to 35%).
* **Precipitation & Accumulation Ticking**:
  * Elevation-dependent temperature lapse rate: $T_{\text{effective}} = T_{\text{base}} - (y - 64) \times 0.00166667$.
  * If raining and $T_{\text{effective}} < 0.15$:
    * Exposed still water sources freeze into `Ice` (if block light $< 11$).
    * Solid top surfaces accumulate `SnowLayer` ($h = 1$ placed onto air, or incremented $1 \rightarrow 8$).
  * When clear weather returns or when exposed to artificial block light $> 11$, snow layers thaw down and ice melts back to water.
* **Precipitation Meshing**:
  * Dynamic $32 \times 32 \times 32$ bounding cylinder centered on the camera.
  * Double-sided quad generation with vertical velocity translation and wind shear drift.
* **Handheld Dynamic Lighting**:
  * Dedicated WGSL uniform array supporting point light sources held in the main hand or off-hand (Torch, Lantern, Glowstone, Sea Lantern, Campfire).
  * Smooth quadratic inverse-square attenuation combined additively with precomputed vertex illumination.

### 3. Physics Engine and Spatial Queries
* **Decoupled AABB Dynamics**:
  * Direct port of vanilla `Entity.move(...)` mechanics.
  * Canonical player physical bounds ($0.6\text{m} \times 1.8\text{m}$ bounding box, $1.62\text{m}$ eye line).
  * Decoupled axis-by-axis collision resolution preventing clipping and tunneling.
  * 0.6m step-up traversal allowing automatic walking over slabs and single-block height differentials without jumping.
  * Exact movement constants: gravity ($0.08\text{ blocks/tick}^2$), jump impulse ($0.42\text{ blocks/tick}$), ground friction ($0.6$), and air drag ($0.91$).
* **Sub-Voxel Amanatides-Woo DDA Raycasting**:
  * Grid traversal supporting sub-voxel non-cubic bounding shapes (Slabs, Stairs, End Rods, Chests, and End Portal Frames).
  * 5.0-block reach radius with hit normal detection for precise block placement and breaking.

### 4. World Generation and Dimensions
* **Multi-Dimensional Block Registry**:
  * Full classification of blocks across Overworld, Nether, and The End.
  * Physical metadata flags: solid collision, transparency face-culling, light emission (0 to 15), hardness, and blast resistance.
* **The Nether Generator**:
  * Closed-box 3D density noise between $Y = 0$ and $Y = 128$.
  * Solid bedrock floor ($Y = 0..4$) and ceiling ($Y = 123..127$).
  * Global lava sea occupying all open caverns below $Y = 31$.
  * Climate noise model governing all 5 Nether Biomes: Nether Wastes, Soul Sand Valley, Crimson Forest, Warped Forest, and Basalt Deltas.
* **The End Dimension Generator**:
  * Radial distance noise engine isolating the Central Island ($R \le 100$).
  * Enforced absolute void gap between $100 < R < 1000$.
  * Outer island generation via 3D Simplex noise with parabolic underside falloff.
  * Stochastic cellular automaton for Chorus Plant and Chorus Flower branching growth.
* **Recursive Jigsaw Structure Engine**:
  * Authentic NBT structure schematic parser (`.nbt`) with Gzip decompression, size bounds, block palettes, and Jigsaw metadata parsing.
  * 3D cardinal rotation, alignment, and joint connection mechanics.
  * Axis-aligned bounding box (AABB) intersection detection preventing intersecting rooms.
  * Procedural Plains Village generation (town center well, branching streets, cottages, and farms).
  * Multi-story Woodland Mansion generator ($3 \times 3$ grid modules across 3 floors with foyer, grand library, master bedroom, and attic maze).
* **End City Piece Graph and End Ships**:
  * Outer island plateau qualification checks ($R \ge 1000$, $\Delta Y \le 4$, $Y \ge 55$, Highlands/Midlands biomes).
  * Recursive piece graph: Base Tower, cantilevered Tower Flanges, covered Bridge Corridors, and expanded Fat Towers.
  * 25% chance of spawning detached End Ships featuring Purpur hulls, bowsprit with Dragon Head, and treasure hold with Elytra.
* **1:1 Seed-for-Seed Terrain & Biome Parity (`mc_worldgen`)**:
  * Clean-room mathematical implementation of vanilla 1.20.4 `RandomState`, Stafford13 64-bit mixer, and Xoroshiro128++ PRNG with Lemire unsigned rejection sampling.
  * 6D climate target search across 7,593 vanilla Overworld `Climate.ParameterPoint` ranges.
  * Bit-for-bit parity for surface heightmap arrays (`OCEAN_FLOOR_WG` and `WORLD_SURFACE_WG`) and biome IDs for any seed.
* **Structure Grid Placement and Concentric Rings (`mc_worldgen::structures`)**:
  * Concentric rings placement algorithm reverse-engineered from `ConcentricRingsStructurePlacement` (`eaa.class`) and `ChunkGeneratorStructureState` (`dlf.class`):
    * Ring 0: 3 strongholds deterministically placed between 1,280 and 2,816 blocks from $(0,0)$.
    * Subsequent expanding concentric rings up to 128 strongholds total using exact $2\pi / \text{spread}$ angular progression.
  * Random spread placement engine reverse-engineered from `RandomSpreadStructurePlacement` (`eab.class`) using 48-bit Java LCG seed hashing and Linear/Triangular distribution types:
    * Trial Chambers: Spacing = 34, Separation = 12, Salt = 94251327 (Linear).
    * Villages: Spacing = 34, Separation = 8, Salt = 10387312 (Linear).
    * Woodland Mansions: Spacing = 80, Separation = 20, Salt = 10387313 (Triangular).
    * Nether Fortresses: Spacing = 27, Separation = 4, Salt = 30084232 (Linear).
    * Bastion Remnants: Spacing = 27, Separation = 4, Salt = 30084235 (Linear).
    * End Cities: Spacing = 20, Separation = 11, Salt = 10387313 (Triangular).
  * Direct chunk generator integration: chunk generation evaluates structure candidate positions and instantiates structure roots on matching coordinates.
* **Item Registry and Automated Recipe Engine (`mc_items`, `mc_crafting`)**:
  * Comprehensive `ItemId` registry encompassing 1,329+ items up to Minecraft 1.21 including weapons, tools, armor, materials, foods, and utility blocks (`Crafter`, `Mace`, `WindCharge`, `BreezeRod`, `HeavyCore`, `TrialKey`, `CopperBulbs`, `CopperGrates`).
  * Exact tool tiers (`Wood`, `Stone`, `Iron`, `Diamond`, `Netherite`, `Gold`) and armor materials reverse-engineered from `coo.class` and `ckk.class` with canonical durabilities, harvest levels, mining speed multipliers, attack damages, and defense points.
  * Automated recipe parser extracting all 822 crafting recipes and 99 tag definitions directly from `client.jar`.
  * Support for shaped matrix recipes (with character key parsing and horizontal mirror support) and shapeless recipes (with bipartite backtracking matching).
  * High-performance in-memory $O(1)$ lookup cache allowing instantaneous recipe resolution from any 3x3 crafting grid.
* **Interactive Workstations Engine (`mc_gameplay`)**:
  * **Crafting Table**: Full 3x3 crafting matrix with output stack preview and automated item return/drop on UI close.
  * **Furnace, Blast Furnace & Smoker**: 20 Hz ticking tile entities tracking `lit_time`, `lit_duration`, `cooking_progress`, and `cooking_total_time`, supporting all vanilla fuels (Coal, Charcoal, Wood, Lava Bucket, Blaze Rod).
  * **Anvil**: Repair calculations with 12% durability bonus, enchantment merging, renaming cost, exponential prior work penalty ($2^n - 1$), and survival 40-level cap.
  * **Smithing Table & Stonecutter**: Netherite equipment upgrades, armor trim attachments, and stone variant cutting.
* **Full Biome Registry & Surface Rules Engine (`mc_worldgen::biomes`)**:
  * Complete registry of all 47+ Overworld biomes across Snowy, Cold, Temperate, Wet, Arid, Ocean, and Cave categories with bidirectional name mapping and temperature/downfall parameters.
  * Deterministic 192-layer Badlands banded terracotta stratification matching vanilla `SurfaceSystem` (`dpi.class`) byte-for-byte with dynamic noise offsets.
  * Desert sand/sandstone subsurface rules, snowy mountain packed ice peaks, and deepslate/bedrock transitions.
* **3D Noise Cave Carvers & Canyons (`mc_worldgen::carvers`)**:
  * **Cheese Caves**: Expansive cavern voids generated where 3D multi-octave cheese noise (`cave_cheese`, firstOctave: -8) exceeds threshold in $Y \in [-54, 180]$.
  * **Spaghetti Caves**: Twisting tunnels formed by evaluating dual orthogonal 3D noise channels ($\sqrt{N_1^2 + N_2^2} < \text{radius}$) combined with 2D horizontal layer noise and roughness perturbation.
  * **Noodle Caves**: Narrow, claustrophobic high-frequency ridge passages evaluated at frequency multiplier $8/3 \approx 2.6667$.
  * **Canyons / Ravines**: Procedural 112-step gorge trajectories with vertical scaling ($yScale = 3.0$), steep sidewall cross-sections, and step-wise yaw/pitch path progression.
  * **Carving Rules & Bedrock Protection**: Substitutes solid terrain (`Stone`, `Deepslate`, `Dirt`, `Terracotta`, `Sand`, `Gravel`, `Sandstone`) with `Air` or `Water` (below `SEA_LEVEL = 63`), strictly guaranteeing **Bedrock immunity** ($Y \le -60$).
* **Local Aquifers & Fluid Flooding (`mc_worldgen::aquifers`)**:
  * **16x40x16 Grid Cell Sampler**: Discretizes subterranean coordinate space into discrete 3D interpolation cells with deterministic Voronoi cell anchor jittering.
  * **Target Fluid Levels & Floodedness**: Modulated by aquifer spread noise ($\pm 10$ blocks) and floodedness noise, naturally generating unflooded dry cave chambers (`fluid_level = MIN`) alongside subterranean lakes.
  * **Elevation Fluid Switching**: Below $Y < -54$, fluid strictly transitions from `Water` to `Lava`, generating deep molten cavern floors; secondary deep lava noise triggers lava pools below $Y \le -10$.
  * **Barrier Noise Stone Dams**: Asymmetric vertical pressure calculation and `aquifer_barrier` noise prevent fluids from leaking into adjacent dry air pockets, generating impermeable natural dividing stone and deepslate rock barriers.
  * **Bedrock Immunity**: Enforces absolute bedrock immunity at $Y \le -60$, ensuring bedrock boundaries are never replaced with fluid.

### 5. Portals and Dimensional Transit
* **Nether Portal Mechanics**:
  * Arbitrary rectangular obsidian frame validation (from $4 \times 5$ up to $23 \times 23$).
  * 8:1 horizontal coordinate translation between Overworld and Nether coordinates.
  * 128-block spherical radius search for existing active frames; automatic platform and frame construction if absent.
  * Dimension transition state machine (80 ticks in Survival, instant in Creative, 100 ticks cooldown).
* **End Portal Mechanics**:
  * 12-block circular frame validation with Eye of Ender verification.
  * 3x3 End Portal activation and fixed-coordinate spawn platform generation at $(100.5, 49.0, 0.5)$.

### 6. Storage Engine and Anvil Format
* **Anvil Region Format (`.mca`)**:
  * Low-level binary serialization and deserialization of 32x32 chunk region files.
  * 4096-byte header with sector offset allocations and timestamp arrays.
  * Zlib/Deflate compression for chunk column payloads.
* **World Metadata (`level.dat`)**:
  * Full Gzip-compressed NBT parser for world parameters (LevelName, GameType, Time, DayTime, SpawnX/Y/Z).
  * Player entity state persistence including position, rotation, inventory, and health.
* **Block Entities**:
  * Section-level tile entity registry.
  * 27-slot chest storage with NBT `Items` compound persistence and 176x168 double-layer container GUI with shift-click item transfers.

---

## Repository Structure

```
IronOxide/
├── Cargo.toml                     # Package manifest, workspace members, and dependencies
├── LICENSE                        # GNU General Public License v3.0 (GPLv3)
├── README.md                      # Executive overview and technical documentation
├── DEVELOPMENT_MILESTONES.md      # Detailed task-by-task engineering log and specifications
├── crates/                        # Modular engine workspace crates
│   ├── mc_items/                  # Complete ItemId registry (1,329+ items) and tool/armor tier attributes
│   ├── mc_crafting/               # Serde shaped/shapeless recipe parser (822 recipes from client.jar) & O(1) cache
│   ├── mc_gameplay/               # Interactive workstations (Crafting Table, Furnaces, Anvil, Smithing, Stonecutter)
│   └── mc_worldgen/               # 1:1 Seed-for-seed terrain, climate noise, 47+ biomes, surface rules & structures
└── src/
    ├── main.rs                    # Application entry point, event loop, and systems orchestration
    ├── audio/                     # Audio synthesis and event triggers
    ├── game/                      # Core game logic and mechanics
    │   ├── crafting/              # 2x2 and 3x3 crafting recipe evaluation
    │   ├── entities/              # Dropped item entities and physics
    │   ├── inventory/             # 36-slot player inventory and container abstractions
    │   ├── mining/                # Block hardness, breaking progress, and tool tiers
    │   ├── mobs/                  # Entity Component System (ECS), AI, and pathfinding
    │   ├── physics.rs             # Decoupled AABB collision resolution and step-up dynamics
    │   ├── portals.rs             # Nether and End portal detection, linking, and forcer
    │   ├── raycast.rs             # Sub-voxel Amanatides-Woo DDA spatial traversal
    │   ├── stats.rs               # Player vitals, hunger saturation, and exhaustion
    │   └── world.rs               # World coordinate management and multi-chunk boundaries
    ├── render/                    # Graphics pipeline and shaders
    │   ├── atlas.rs               # Dynamic texture atlas stitcher and UV coordinate map
    │   ├── camera.rs              # 3D view-projection camera with F5 perspective switching
    │   ├── colormap.rs            # Triangular barycentric colormap tinting with kernel smoothing
    │   ├── context.rs             # WGPU context, swapchain surface, and render passes
    │   ├── culling.rs             # 6-plane view frustum culling
    │   ├── dynamic_lights.rs      # Handheld real-time dynamic light uniform shader pipeline
    │   ├── effects.rs             # Particle systems and block breaking stage overlays
    │   ├── entity_renderer.rs     # Floating item drop and entity rendering
    │   ├── gui/                   # 2D orthographic HUD, crosshairs, and container screens
    │   ├── mesher.rs              # Asynchronous 3D greedy chunk mesher with ambient occlusion
    │   ├── models/                # Humanoid mob hierarchy and pendulum animation systems
    │   ├── uniforms.rs            # Camera and perspective uniform buffer allocations
    │   └── weather_renderer.rs    # Volumetric precipitation geometry and fog attenuation
    ├── storage/                   # File persistence and formats
    │   ├── level_data.rs          # level.dat Gzip NBT parser and world state
    │   ├── nbt.rs                 # Complete NBT binary specification serializer/deserializer
    │   └── region.rs              # Anvil MCA region format, sector allocation, and chunk columns
    └── world/                     # Environmental and generative systems
        ├── block_registry.rs      # Comprehensive multi-dimensional block registry and bounding boxes
        ├── fluids.rs              # Cellular automaton fluid simulation (lava and water)
        ├── lighting.rs            # Packed dual-nibble array breadth-first light propagation
        ├── structures/            # Procedural structure generators
        │   ├── end_city.rs        # End City recursive piece graph and End Ships
        │   └── jigsaw.rs          # Recursive Jigsaw engine, Plains Villages, and Mansions
        ├── tile_entity.rs         # Chunk section tile entity container and Anvil persistence
        ├── weather.rs             # Environmental weather state machine and temperature lapse rates
        └── generator/             # Procedural world generators
            ├── nether.rs          # 3D closed-box density noise and 5 Nether Biomes
            ├── the_end.rs         # Radial island noise, void gap, and Chorus plant growth
            └── overworld.rs       # 1:1 Seed-for-seed Overworld parity hookup to mc_worldgen
```

---

## Technical Specifications

| Subsystem | Specification / Standard |
|---|---|
| Language | Rust (2024 Edition, latest stable) |
| Target Protocol | Minecraft Java Edition 1.20.4 |
| Graphics API | WGPU (Vulkan / DX12 / Metal / WebGPU) |
| Shader Language | WGSL (WebGPU Shading Language) |
| Chunk Geometry | Greedy meshed axis-aligned quads with per-vertex AO and biome tinting |
| Raycasting | Sub-voxel Amanatides-Woo Fast Voxel Traversal algorithm |
| Physics | Decoupled 3-axis AABB collision with 0.6m continuous step-up traversal |
| Lighting | Breadth-First Search (BFS) flood-fill, 0 to 15 packed dual-nibble arrays |
| Chunk Storage | Standard Anvil `.mca` 32x32 region tables with Zlib/Deflate compression |
| License | GNU General Public License v3.0 (GPLv3) |

---

## Building and Running

### Prerequisites
1. **Rust Toolchain**: Install stable Rust via [rustup](https://rustup.rs/) (edition 2024 supported).
2. **Graphics Drivers**: Ensure Vulkan, DirectX 12, or Metal compatible drivers are installed.
3. **Optional Assets**: Place an authentic Minecraft 1.20.4 `client.jar` in the repository root or parent directory. If absent, the internal procedural generator will generate textures.

### Compilation Commands
```bash
# Verify compilation across all workspace crates
cargo check --workspace

# Execute the test suite (204 automated unit and integration tests)
cargo test --workspace

# Build and run the client in optimized release mode
cargo run --release
```

---

## Keybindings and Controls

| Input | Functional Action |
|---|---|
| W, A, S, D | Horizontal movement (Forward, Left, Backward, Right) |
| Space | Jump / Ascend |
| Left Shift | Sprint modifier |
| Mouse Motion | Camera rotation (Yaw and Pitch) |
| Mouse Left-Click | Mine / Break targeted block |
| Mouse Right-Click | Place active hotbar block / Interact with containers |
| Scroll Wheel / 1-9 | Hotbar slot selection (slots 1 through 9) |
| E | Toggle inventory and container screens |
| F3 | Toggle debug diagnostics overlay (FPS, coordinates, chunk, memory) |
| F5 | Cycle camera perspective (First Person, Third Person Back, Third Person Front) |
| Escape | Release / capture mouse cursor; close active GUI |

---

## Automated Verification Suite

The repository contains 204 automated unit and integration tests across the workspace verifying engine correctness, numerical stability, and format compliance:

```text
running 173 tests in IronOxide
test world::weather::tests::test_global_weather_cycle_timers_and_transitions ... ok
test world::weather::tests::test_chunk_precipitation_water_freezing_and_snow_accumulation ... ok
test world::weather::tests::test_snow_and_ice_thawing_under_light ... ok
...
test result: ok. 173 passed; 0 failed; 0 ignored; finished in 7.46s

running 30 tests in mc_worldgen
test test_surface_rules_badlands_banded_terracotta ... ok
test test_concentric_rings_stronghold_placement ... ok
test test_chunk_parity_seed_123456789 ... ok
test test_cheese_caves_cavern_density ... ok
test test_spaghetti_caves_3d_and_2d ... ok
test test_noodle_caves_claustrophobic_tunnels ... ok
test test_canyon_carver_generation_and_shape ... ok
test test_bedrock_never_carved ... ok
test test_carve_block_replacement_rules ... ok
test test_aquifer_grid_cell_partitioning ... ok
test test_aquifer_fluid_type_elevation_switching_lava_below_minus_54 ... ok
test test_aquifer_flooding_logic_backfills_carved_space ... ok
test test_barrier_noise_prevents_leaking_and_creates_dry_pockets ... ok
test test_aquifer_unflooded_dry_pockets ... ok
test test_aquifer_bedrock_immunity ... ok
...
test result: ok. 30 passed; 0 failed; 0 ignored; finished in 0.38s

running 5 tests in mc_gameplay
test test_crafting_table_preview_craft_and_auto_drop ... ok
test test_furnace_20hz_ticking_and_fuels ... ok
test test_anvil_prior_work_penalty_and_repairs ... ok
...
test result: ok. 5 passed; 0 failed; 0 ignored; finished in 0.00s

running 5 tests in mc_items
test test_tool_tier_attributes_parity ... ok
test test_item_id_name_roundtrip ... ok
...
test result: ok. 5 passed; 0 failed; 0 ignored; finished in 0.00s

running 3 tests in mc_crafting
test test_load_all_recipes_from_client_jar_and_lookup ... ok
...
test result: ok. 3 passed; 0 failed; 0 ignored; finished in 1.98s
```

For the comprehensive breakdown of completed milestones and upcoming development targets, consult [DEVELOPMENT_MILESTONES.md](file:///c:/Users/khant/OneDrive/Desktop/Falix/Minecraft%20reverce%20in%20rust/IronOxide/DEVELOPMENT_MILESTONES.md).

---

## License

This project is licensed under the **GNU General Public License v3.0 (GPLv3)**.

You are free to run, study, modify, and redistribute this software under the terms and conditions of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version. See the [LICENSE](file:///c:/Users/khant/OneDrive/Desktop/Falix/Minecraft%20reverce%20in%20rust/IronOxide/LICENSE) file for the complete license text.
