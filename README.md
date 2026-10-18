# IronOxide

High-Performance Voxel Game Engine and Minecraft Client Architecture in Rust

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Language: Rust 2024](https://img.shields.io/badge/Language-Rust_2024-orange.svg)](https://www.rust-lang.org/)
[![Graphics: WGPU](https://img.shields.io/badge/Graphics-WGPU_/_WGSL-green.svg)](https://wgpu.rs/)
[![Tests: 184 Passing](https://img.shields.io/badge/Tests-184_Passed-brightgreen.svg)]()

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

### 2. Volumetric Weather and Real-Time Dynamic Lighting
* **Atmospheric Simulation**:
  * Deterministic weather state machine governing Clear, Rain, and Thunder states with exponential transition curves.
  * Elevation-dependent temperature lapse rate ($0.00166667^\circ\text{C}$ per block above sea level $Y = 64$) determining precipitation transitions between rain and snow.
* **Precipitation Meshing**:
  * Dynamic $32 \times 32 \times 32$ bounding cylinder centered on the camera.
  * Double-sided quad generation with vertical velocity translation and wind shear drift.
  * Real-time fog density escalation and skylight reduction during storms.
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
├── Cargo.toml                     # Package manifest, dependencies, and license metadata
├── LICENSE                        # GNU General Public License v3.0 (GPLv3)
├── README.md                      # Executive overview and technical documentation
├── DEVELOPMENT_MILESTONES.md      # Detailed task-by-task engineering log and specifications
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
            └── the_end.rs         # Radial island noise, void gap, and Chorus plant growth
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
# Verify compilation across all modules
cargo check

# Execute the test suite (167 automated unit and integration tests)
cargo test

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

The repository contains 167 automated unit and integration tests verifying engine correctness, numerical stability, and format compliance:

```text
running 167 tests
test game::physics::tests::test_aabb_intersection ... ok
test game::physics::tests::test_decoupled_axis_resolution ... ok
test game::physics::tests::test_step_up_traversal ... ok
test game::raycast::tests::test_raycast_subvoxel_precision ... ok
test render::mesher::tests::test_greedy_face_merging ... ok
test render::mesher::tests::test_ambient_occlusion_derivation ... ok
test render::colormap::tests::test_barycentric_uv_sampling ... ok
test storage::region::tests::test_region_file_header_offsets ... ok
test storage::nbt::tests::test_nbt_compound_roundtrip ... ok
test world::lighting::tests::test_dual_nibble_bfs_flood_fill ... ok
test world::fluids::tests::test_lava_water_conversion_obsidian_cobble ... ok
test world::generator::nether::tests::test_5_nether_biomes_climate ... ok
test world::generator::the_end::tests::test_outer_islands_generation ... ok
test world::structures::jigsaw::tests::test_structure_template_nbt_roundtrip ... ok
test world::structures::jigsaw::tests::test_plains_village_generation ... ok
test world::structures::end_city::tests::test_end_city_piece_tree ... ok
test world::structures::end_city::tests::test_end_ship_25_percent_spawn_probability ... ok
...
test result: ok. 167 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.89s
```

For the comprehensive breakdown of completed milestones and upcoming development targets, consult [DEVELOPMENT_MILESTONES.md](file:///c:/Users/khant/OneDrive/Desktop/Falix/Minecraft%20reverce%20in%20rust/IronOxide/DEVELOPMENT_MILESTONES.md).

---

## License

This project is licensed under the **GNU General Public License v3.0 (GPLv3)**.

You are free to run, study, modify, and redistribute this software under the terms and conditions of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version. See the [LICENSE](file:///c:/Users/khant/OneDrive/Desktop/Falix/Minecraft%20reverce%20in%20rust/IronOxide/LICENSE) file for the complete license text.
