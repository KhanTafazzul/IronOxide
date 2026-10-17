# ⛏️ IronOxide: Reverse-Engineered Minecraft in Rust
## 📜 Master Project Progress & Implementation Log (`what i have done (what i have made till now).md`)

> **Project Goal**: Re-implement Minecraft 1.20.4 client and voxel engine from scratch in **Rust** using **WGPU** and **WGSL** shaders by reverse-engineering `client.jar` using decompilation and exact vanilla algorithms.
>
> **Status**: **162 / 162 Unit & Integration Tests Passing** ✅
> **Last Updated**: Tasks 1 through 30 & Task 32 Fully Completed & Verified.

---

## 📊 Summary of Completed Milestones

| Task / Feature | Module / Location | Status | Key Highlights |
|---|---|---|---|
| **Core Engine & Rendering** | `src/render/`, `src/main.rs` | ✅ Done | WGPU context, WGSL shaders, texture atlas stitching directly from `client.jar` |
| **Greedy Chunk Mesher** | `src/render/mesher.rs` | ✅ Done | 3D greedy face merging, per-vertex AO (0..3), Rayon async threadpool |
| **Player Physics & Collision** | `src/game/physics.rs` | ✅ Done | AABB collision, 0.6m step-up, vanilla gravity (0.08) & jump (0.42) |
| **Voxel Raycasting (DDA)** | `src/game/raycast.rs` | ✅ Done | Amanatides-Woo DDA, 5.0-block reach, multi-section boundary updates |
| **HUD, Crosshair & Hotbar** | `src/render/gui/mod.rs` | ✅ Done | Inverted blending crosshair, 182x22 hotbar, mouse wheel / 1-9 slots |
| **Task 7: Survival Inventory** | `src/game/inventory/`, `src/render/gui/inventory.rs` | ✅ Done | 27-slot main inventory, 9-slot hotbar, 2x2 crafting grid, 'E' key screen |
| **Task 8: Block Hardness & Mining** | `src/game/mining/`, `src/render/effects/` | ✅ Done | Vanilla hardness, tool tiers, 10 break crack decals, particle burst physics |
| **Task 9: Spatial 3D Audio** | `src/audio/` | ✅ Done | Rodio 3D spatial sound, distance attenuation, stereo panning, vanilla sound events |
| **Task 10: Survival Vitals & Hunger** | `src/game/stats.rs`, `src/render/gui/hud.rs` | ✅ Done | 20 HP health, 20 food, saturation, exhaustion, natural regen, HUD hearts/drumsticks |
| **Task 11: Voxel Light Engine** | `src/world/lighting.rs` | ✅ Done | Dual 4-bit packed NibbleArray, BFS block light & sky light flood fill, removal queues |
| **Task 12: Dropped Item Entities** | `src/game/entities.rs`, `src/render/entity_renderer.rs` | ✅ Done | Floating 3D item quads, spinning & bobbing, AABB physics, 1.5m player magnetism |
| **Task 13: ECS & Zombie Mob AI** | `src/game/mobs/`, `src/render/models/humanoid.rs` | ✅ Done | 3D A* grid pathfinding, hostile zombie state machine, hierarchical humanoid model |
| **Task 14: Anvil .MCA Region Storage** | `src/storage/region.rs`, `src/storage/nbt.rs` | ✅ Done | 4096-byte chunk location/timestamp tables, zlib NBT serialization, async disk saving |
| **Task 19: Fluid Dynamics Engine** | `src/world/fluids.rs` | ✅ Done | Water (7 blocks) & Lava (3 blocks) flow, falling level 8, obsidian/cobblestone collision |
| **Task 20: F3 Profiler & Metrics** | `src/render/gui/debug.rs` | ✅ Done | F3 toggle, coordinates, chunk/region, facing, light, GPU backend, memory RSS |
| **Task 21: Client Dynamic Lighting** | `src/render/dynamic_lights.rs` | ✅ Done | Handheld light sources (Torch/Lantern/Glowstone), WGSL terrain uniform, smooth falloff |
| **Task 22: Camera Modes & Culling** | `src/render/camera.rs`, `src/render/culling.rs` | ✅ Done | F5 cycling (1st, 3rd Back, 3rd Front), occlusion raycast clamp, 6-plane frustum culling |
| **Task 23: level.dat World Metadata** | `src/storage/level_data.rs` | ✅ Done | Gzip NBT parser, LevelName, GameType, Time, Spawn, Player pos/rot/inv persistence |
| **Task 24: Biome Colormap Tinting** | `src/render/colormap.rs` | ✅ Done | Triangular barycentric UV colormap sampling, 3x3 neighbor kernel, per-vertex tints |
| **Task 25: Volumetric Precipitation** | `src/world/weather.rs`, `src/render/weather_renderer.rs` | ✅ Done | Rain/Snow/Thunder state machine, lapse rate, 32x32x32 particle box, scrolling rain sheets |
| **Task 26: Tile Entities & Chest UI** | `src/world/tile_entity.rs`, `src/render/gui/chest_screen.rs` | ✅ Done | Chunk section tile entity storage, 27-slot chest NBT serialization, 176x168 container GUI, shift-click transfer |
| **Task 27: Block Registry & Voxel Models** | `src/world/block_registry.rs` | ✅ Done | 3 Dimensions (Overworld, Nether, End), Voxel shapes (Stairs, Slabs, Rods), properties |
| **Task 28: Nether Generator & 5 Biomes** | `src/world/generator/nether.rs` | ✅ Done | Closed-box 3D density (Y=0..128), Bedrock floor/ceiling, Lava ocean Y<=31, 5 Nether Biomes & features |
| **Task 29: The End Generator & Islands** | `src/world/generator/the_end.rs` | ✅ Done | Central island ($R \le 100$), void gap ($100 \le R < 1000$), outer islands, Simplex noise, Chorus plant cellular loop |
| **Task 30: Jigsaw Structure Engine** | `src/world/structures/jigsaw.rs` | ✅ Done | Recursive Jigsaw assembly, NBT schematic parser/serializer, AABB intersection check, Plains Village & Woodland Mansion |
| **Task 32: Nether & End Portals** | `src/game/portals.rs` | ✅ Done | 4x5 to 23x23 Obsidian frame detection, Flint & Steel lighting, (X/8, Y, Z/8) translation, 128-block search/forcing, 80/0 tick timers, 12-block End frame ring & fixed spawn platform |

---

## 🔍 Detailed Breakdown of Completed Tasks

### 1. Foundation & Engine Core (Tasks 1–6)
- **Graphics Pipeline**: Pure Rust using `wgpu` (Vulkan / DirectX 12 / Metal / WebGPU). Nearest-neighbor sampling preserving authentic pixel textures.
- **Dynamic Texture Atlas**:
  - Dynamically extracts and stitches block textures directly from `client.jar` (with high-quality procedural fallbacks).
  - Sub-pixel directional UV coordinate mapping for all 6 voxel faces (`UP`, `DOWN`, `NORTH`, `SOUTH`, `EAST`, `WEST`).
- **3D Greedy Mesher**:
  - Axis-aligned greedy face merging combining coplanar adjacent faces into minimal rectangular quads.
  - Per-vertex Ambient Occlusion (AO levels 0..3) calculated from neighbor block occupancy.
  - Background asynchronous section meshing powered by `rayon`.
- **Vanilla Physics & Collision**:
  - Re-implemented from Minecraft 1.20.4 `LocalPlayer` & `Entity.move(...)`.
  - Player dimensions: $0.6\text{m} \times 1.8\text{m}$ ($1.62\text{m}$ eye level).
  - 3-axis decoupled AABB collision resolution with **0.6m step-up** (auto-walk over single slabs/steps without jumping).
  - Constants: gravity $0.08\text{ blocks/tick}^2$, jump impulse $0.42\text{ blocks/tick}$, drag, air friction.
- **Sub-Voxel Amanatides-Woo DDA Raycasting**:
  - Precise 5.0-block reach.
  - Left-Click: block breaking with boundary chunk remeshing.
  - Right-Click: block placement against face normal with player AABB collision prevention.
- **2D HUD, Crosshair & Hotbar**:
  - Inverted blend mode crosshair (`BlendFactor::OneMinusDst`, `OneMinusSrc`) at exact screen center.
  - Vanilla $182 \times 22$ hotbar anchored to bottom center with active slot highlight.
  - Mouse scroll wheel and number keys (1–9) slot cycling.

---

### 2. Task 7: Survival Inventory, 2x2 Crafting & 'E' Key Screen
- **Module**: `src/game/inventory/`, `src/render/gui/inventory.rs`
- **Features**:
  - Complete 36-slot player inventory: 27 main inventory slots + 9 hotbar slots.
  - 2x2 crafting grid with 1 result slot and recipe matching (Planks from Wood Logs, Sticks from Planks, Crafting Table, Torches).
  - Mouse cursor slot manipulation: left-click to pick up/place entire stacks, right-click to split stacks or drop single items, shift-click auto-transfer.
  - `E` key toggle for opening/closing the inventory screen with mouse release/capture.

---

### 3. Task 8: Block Hardness, Destroy Cracks & Particles
- **Module**: `src/game/mining/`, `src/render/effects/`
- **Features**:
  - Block hardness database matching vanilla Minecraft 1.20.4 (Dirt = 0.5, Stone = 1.5, Obsidian = 50.0, Bedrock = -1.0 unbreakable).
  - Tool tier speed multipliers (Hand, Wood, Stone, Iron, Diamond, Netherite).
  - 10-stage destroy crack overlay decals (`destroy_stage_0..9.png`) mapped over targeted voxel face.
  - Voxel destruction particle burst: spawns 20–40 miniature colored cube fragments with randomized velocity, gravity, and expiration timers.

---

### 4. Task 9: 3D Spatial Audio System
- **Module**: `src/audio/`
- **Features**:
  - `rodio`-powered 3D spatial audio listener positioned at the player's head.
  - Inverse-distance sound attenuation and left/right stereo panning based on relative angle.
  - Sound event triggers: footstep sounds modulated by surface block type, block digging/break sounds, fluid splash sounds.

---

### 5. Task 10: Survival Vitals & Hunger Mechanics
- **Module**: `src/game/stats.rs`, `src/render/gui/hud.rs`
- **Features**:
  - Full vanilla `FoodData` simulation: Health (20.0), Food Level (20), Saturation (5.0), Exhaustion (0.0).
  - Natural health regeneration: fast regen when saturation > 0 and food = 20 (1 HP every 10 ticks); normal regen when food $\ge 18$ (1 HP every 80 ticks).
  - Starvation damage: 1 HP damage every 80 ticks when food reaches 0.
  - Activity exhaustion tracking: sprinting ($0.1\text{/m}$), jumping ($0.2\text{/jump}$), mining ($0.005\text{/block}$).
  - 2D HUD hearts and drumsticks rendering with damage shake animation when health changes.

---

### 6. Task 11: Voxel Breadth-First Light Engine
- **Module**: `src/world/lighting.rs`
- **Features**:
  - Dual 4-bit packed NibbleArray storage per $16 \times 16 \times 16$ `ChunkSection` (Block Light and Sky Light).
  - Multi-source BFS flood fill propagation: Torches (14), Glowstone / Lava (15), Fire (15).
  - Light removal BFS queue with neighbor repropagation to handle block removal and placement.
  - Direct integration into the 3D greedy mesher with smooth per-vertex lighting calculation.

---

### 7. Task 12: Dropped Item Entities & Magnetism
- **Module**: `src/game/entities.rs`, `src/render/entity_renderer.rs`
- **Features**:
  - Dropped item entities with AABB physics, gravity ($0.04\text{ blocks/tick}^2$), ground friction, and bounce dampening.
  - Continuous vertical bobbing ($y = y_0 + \sin(t) \times 0.1$) and $360^\circ$ yaw rotation animation.
  - 1.5-meter player magnetism attraction pulling floating items into player inventory.
  - 3D miniature item quad rendering with billboarding or isometric tilt.

---

### 8. Task 13: Entity Component System & Zombie Mob AI
- **Module**: `src/game/mobs/`, `src/render/models/humanoid.rs`
- **Features**:
  - ECS architecture with components: `Transform`, `Velocity`, `Aabb`, `Health`, `MobType`.
  - 3D A* grid pathfinding over voxel geometry: supports 1-block jump ascents, safe drops $\le 3$ blocks, and avoids lethal cliffs $> 3$ blocks.
  - Zombie AI state machine:
    - `IDLE`: Wanders randomly every 60–100 ticks.
    - `TARGETING`: Detects player within 16-block line-of-sight radius.
    - `ATTACK`: Pursues player and applies 3.0 melee damage on collision with a 20-tick cooldown.
  - Hierarchical humanoid box model (Head, Torso, Left Arm, Right Arm, Left Leg, Right Leg) with walking limb pendulum animations.

---

### 9. Task 14: Anvil .MCA Region Storage & World Persistence
- **Module**: `src/storage/region.rs`, `src/storage/nbt.rs`
- **Features**:
  - Full Anvil `.mca` format reader and writer:
    - 4096-byte chunk location table (4KiB sector offset + sector count).
    - 4096-byte chunk timestamp table.
  - Fast zlib/deflate NBT serialization and deserialization for chunk compounds.
  - Dirty chunk detection tracking modified blocks and lighting updates.
  - Asynchronous background worker thread persisting modified regions to `saves/<World>/region/r.X.Z.mca` without game stutter.

---

### 10. Task 19: Cellular Automaton Fluid Engine
- **Module**: `src/world/fluids.rs`
- **Features**:
  - Fluid state tracking: `level: 1..=8`, `falling: bool` for both Water and Lava.
  - Horizontal fluid propagation: drops by 1 level per block traveled (Water up to 7 blocks, Lava up to 3 blocks in Overworld).
  - Downward vertical flow: retains Level 8 with `falling = true` when dropping through air.
  - Collision reactions:
    - Water contacting stationary Lava source $\rightarrow$ converts to Obsidian (`BlockType::Obsidian`).
    - Water contacting flowing Lava $\rightarrow$ converts to Cobblestone (`BlockType::Cobblestone`).
  - Directional flow vectors applying swimming push forces to players and entities.
  - Proportional fluid height geometry meshing ($H = \text{level} / 8.0$).

---

### 11. Task 20: F3 Profiler & Engine Metrics Overlay
- **Module**: `src/render/gui/debug.rs`
- **Features**:
  - `F3` key toggle for HUD debug metrics overlay.
  - **Left Column**:
    - Engine version & current FPS / frame time (e.g., `144 fps / 6.94 ms`).
    - Player coordinates: `XYZ: x / y / z` and `Block: bx by bz`.
    - Chunk coordinates: `Chunk: cx cy cz inside region r.rx.rz.mca`.
    - Facing direction: `Facing: north/south/west/east (Towards negative Z) (Yaw, Pitch)`.
    - Client Light levels: `Client Light: X (Y sky, Z block)`.
  - **Right Column**:
    - Hardware GPU device name via `wgpu::AdapterInfo`, graphics backend (`Vulkan`, `DX12`, or `Metal`).
    - Memory usage: Current process RSS memory vs total physical RAM.
    - Mesher metrics: Total active chunk sections, rendered quads, and draw call counters.

---

### 12. Task 21: Client Dynamic Lighting Pipeline
- **Module**: `src/render/dynamic_lights.rs`
- **Features**:
  - Handheld dynamic light emission: Torch (14), Lantern (15), Glowstone (15) held in main hand or off-hand.
  - WGPU uniform buffer injection into terrain shader (`DynamicLight { pos: vec3<f32>, intensity: f32, radius: f32 }`).
  - Terrain fragment shader calculates inverse-square distance attenuation up to 15 blocks.
  - Dynamic light smoothly combined with static vertex light: `final_light = max(vertex_light, dynamic_light)`.
  - Zero CPU meshing overhead during player movement (1 uniform buffer update per frame).

---

### 13. Task 22: Multi-Perspective Camera & Frustum Culling
- **Module**: `src/render/camera.rs`, `src/render/culling.rs`
- **Features**:
  - `F5` key cycling through 3 camera modes:
    - `FirstPerson`: Camera locked to eye level ($y + 1.62$).
    - `ThirdPersonBack`: Camera offset 4.0 blocks behind player direction vector.
    - `ThirdPersonFront`: Camera offset 4.0 blocks ahead, rotated $180^\circ$ looking back at player.
  - **Camera Occlusion Raycast**: Raycasts backward from the player's head toward third-person target position; clamps camera distance immediately in front of hit voxels to prevent wall clipping.
  - **6-Plane Frustum Culling**: Extracts view frustum planes from combined View-Projection matrix and culls entire $16 \times 16 \times 16$ `ChunkSection` bounding boxes prior to submitting GPU draw commands.

---

### 14. Task 23: level.dat NBT Parser & World Metadata
- **Module**: `src/storage/level_data.rs`
- **Features**:
  - Gzip-compressed `level.dat` reader and writer.
  - Root `Data` compound extraction: `LevelName`, `GameType`, `Time`, `DayTime`, `SpawnX/Y/Z`.
  - Player compound tag extraction: `Pos` $[x, y, z]$, `Rotation` $[yaw, pitch]$, `Motion` $[vx, vy, vz]$, `Inventory` item stacks.
  - Engine launch hook: loads player coordinates directly from `level.dat` on world start.
  - Periodic background metadata saving to preserve world state across sessions.

---

### 15. Task 24: Biome Colormap Sampling & Foliage Tinting
- **Module**: `src/render/colormap.rs`
- **Features**:
  - Direct loading of $256 \times 256$ colormaps from `client.jar`:
    - `assets/minecraft/textures/colormap/grass.png`
    - `assets/minecraft/textures/colormap/foliage.png`
  - Triangular barycentric UV coordinate sampling:
    - Inputs clamped: Temperature $T \in [0.0, 1.0]$, Downfall/Humidity $H \in [0.0, 1.0]$.
    - Vanilla coordinate mapping: $U = 1.0 - T$, $V = 1.0 - (T \times H)$.
  - $3 \times 3$ block neighbor averaging kernel for seamless color transitions across biome borders.
  - Per-vertex RGB tint multiplier applied to Grass Blocks, Oak Leaves, and Vines in the terrain shader.

---

### 16. Task 25: Precipitation & Volumetric Rain/Snow Renderer
- **Module**: `src/world/weather.rs`, `src/render/weather_renderer.rs`
- **Features**:
  - Reverse-engineered from `net.minecraft.client.renderer.LevelRenderer.renderSnowAndRain(...)`.
  - Weather state machine: `Clear`, `Rain`, and `Thunder`.
  - Smooth 50-tick linear fade ($0.02\text{ per tick}$) for `rain_strength` and `thunder_strength`.
  - Temperature lapse rate with elevation: $T(y) = T_{\text{base}} - (y - 64.0) \times 0.00166667$.
  - Precipitation type: Snow if effective $T < 0.15$; otherwise Rain (dry biomes like Deserts do not precipitate).
  - Dynamic $32 \times 32 \times 32$ volumetric particle box centered around the player's camera.
  - Rain sheets scrolling downwards at $0.75\text{ blocks/tick}$ using `rain.png`.
  - Snow flakes falling at $0.10\text{ blocks/tick}$ with sinusoidal horizontal drift using `snow.png`.
  - Atmospheric effects: Sky light attenuated by $25\%$ during rain; fog distance pulled in by $35\%$ for realistic reduced visibility.

---

### 18. Task 26: Tile Entities & Chest UI Container
- **Module**: `src/world/tile_entity.rs`, `src/render/gui/chest_screen.rs`, `src/render/context.rs`, `src/main.rs`
- **REA Decompiled References**:
  - `net.minecraft.world.level.block.entity.ChestBlockEntity` (`dhc.class`)
  - `net.minecraft.client.gui.screens.inventory.ContainerScreen` (`fel.class`)
  - `net.minecraft.world.inventory.ChestMenu` (`cii.class`)
- **Features**:
  - **Chunk Section Tile Entity Storage**:
    - Chunk sections track non-standard block state tile entities via `HashMap<IVec3, TileEntity>`.
    - Lifecycle hooks in `set_block_at`: placing a Chest automatically instantiates `ChestTileEntity`; breaking removes it.
  - **ChestTileEntity & Anvil NBT Persistence**:
    - 27 inventory storage slots: `[Option<ItemStack>; 27]`.
    - Serializes and deserializes to/from Anvil chunk `block_entities` compound list with `id: "minecraft:chest"`, `x`, `y`, `z`, and `Items` compound list.
  - **Double-Layer Container GUI (176x168)**:
    - Extracted from `assets/minecraft/textures/gui/container/generic_54.png` in `client.jar` (stitched into `GuiAtlas` with procedural wood/iron fallback).
    - Top Section: 27 chest storage slots (3 rows $\times$ 9 columns, slot coordinates $x = 8 + c \times 18$, $y = 18 + r \times 18$).
    - Bottom Section: 36 player inventory slots (27 main storage $y = 84 + r \times 18$ + 9 hotbar $y = 142$).
  - **Shift-Click Quick Transfer (`ChestMenu.quickMoveStack`)**:
    - Shift-clicking chest slots merges items into player hotbar first, then main inventory.
    - Shift-clicking player hotbar or main inventory merges items into chest storage.
  - **Full Mouse Slot Manipulation**:
    - Left-click pickup, place, and merge with max stack clamping.
    - Right-click split half-stack into cursor or place single items.
    - Floating cursor stack with count badges, slot hover highlight, and movement halting while container is open.

---

### 19. Task 27: Comprehensive Block Registry & Voxel Models
- **Module**: `src/world/block_registry.rs`, `src/world/mod.rs`, `src/render/atlas.rs`, `src/game/physics.rs`, `src/game/raycast.rs`, `src/storage/region.rs`, `src/game/mining/mod.rs`, `src/audio/mod.rs`
- **REA Decompiled References**:
  - `net.minecraft.world.level.block.Blocks` (`cws.class`)
  - `net.minecraft.world.level.block.state.BlockBehaviour$Properties` (`djg$d.class`)
  - `net.minecraft.world.level.block.StairBlock` (`dej.class`)
  - `net.minecraft.world.level.block.SlabBlock` (`dds.class`)
  - `net.minecraft.world.level.block.EndRodBlock` (`czg.class` / `dcy.class`)
- **Features**:
  - **Comprehensive Multi-Dimensional BlockId Enum**:
    - **Overworld**: Wood variants (Oak, Birch, Spruce, Dark Oak logs, planks, leaves), Stone types (Stone, Cobblestone, Andesite, Granite, Diorite, Deepslate, Bricks), Ores (Coal, Iron, Copper, Gold, Redstone, Lapis, Diamond, Emerald), Glass, Obsidian, Sand, Gravel, Bedrock, Dirt, GrassBlock.
    - **Nether**: Netherrack, Soul Sand, Soul Soil, Basalt, Polished Basalt, Blackstone, Nether Bricks, Glowstone, Crimson/Warped Nylium, Crimson/Warped Stems, Nether Wart Block, Warped Wart Block, Crying Obsidian.
    - **The End**: End Stone, End Stone Bricks, Purpur Block, Purpur Pillar, End Rod, Purpur Stairs, Purpur Slab, End Portal Frame.
  - **Complete Property Metadata Flags (`BlockProperties`)**:
    - `is_solid`: Directs player and entity physics collision resolution.
    - `is_transparent`: Dictates face-culling against adjacent blocks (transparent neighbor faces remain rendered).
    - `light_emission`: Point and ambient emission from 0 to 15 (Lava/Glowstone = 15, Torch/End Rod = 14, Crying Obsidian = 10, End Portal Frame = 1).
    - `hardness` & `blast_resistance`: Exact vanilla properties (Obsidian = 50.0/1200.0, Bedrock = -1.0/3.6M, Stone = 1.5/6.0, Wood = 2.0/3.0, Dirt = 0.5/0.5).
    - `sound_family`, `tool_required`, and `min_harvest_tier` classifications.
  - **Compound Non-Cubic Bounding Boxes & Raycast DDA Support**:
    - Slabs: half-cube AABB $[0, 0, 0] \rightarrow [1, 0.5, 1]$.
    - Stairs: compound AABBs (base slab $[0, 0, 0] \rightarrow [1, 0.5, 1]$ + directional step $[0, 0.5, 0] \rightarrow [1, 1, 0.5]$).
    - End Rod: centered column $[0.375, 0, 0.375] \rightarrow [0.625, 1, 0.625]$.
    - Torches, Chests, and End Portal Frames ($13/16$ high).
    - Raycast traversal accurately clips against individual sub-boxes; rays pass cleanly through the empty upper half of slabs and around end rod columns.
    - Entity physics resolves collisions against compound bounding boxes with step-up support.
  - **Full Subsystem Synchronization**:
    - `TextureAtlas` stitches textures from `client.jar` with procedural color fallbacks for all dimensions.
    - Anvil `.mca` region reader and writer converts all blocks to/from vanilla namespaced IDs.
    - Audio sound events, mining speeds, and inventory conversions wired across the engine.

---

### 15. Task 28: The Nether Generator & 5 Biomes
- **Module**: `src/world/generator/nether.rs`, `src/world/generator/mod.rs`
- **REA Decompiled References**:
  - `net.minecraft.world.level.biome.MultiNoiseBiomeSourceParameterList$Preset` (`cvc$a$1.class`):
    Target climate points in $(T, H, W, \text{offset})$ parameter space:
    * `Nether Wastes`: $(0.0, 0.0, 0.0, 0.0)$
    * `Soul Sand Valley`: $(0.0, -0.5, 0.0, 0.0)$
    * `Crimson Forest`: $(0.4, 0.0, 0.0, 0.0)$
    * `Warped Forest`: $(0.0, 0.5, 0.0, 0.375)$
    * `Basalt Deltas`: $(-0.5, 0.0, 0.0, 0.175)$
  - `net.minecraft.data.worldgen.SurfaceRuleData` (`pr.class` - method `b()`):
    Nether surface rule sequences, bedrock roof ($Y \ge 123$) and floor ($Y \le 4$), lava ocean level ($Y \le 31$), and per-biome ground strata.
- **Features Implemented**:
  - **Closed-Box 3D Density Noise ($Y = 0..128$)**:
    * Solid Bedrock floor at $Y = 0$, stochastic dithering from $Y = 1$ to $Y = 4$.
    * Solid Bedrock roof at $Y = 127$, stochastic dithering from $Y = 123$ to $Y = 126$.
    * Lava ocean filling all open cavities below $Y \le 31$.
    * Mid-world open caverns ($Y = 32..114$) with 3D Perlin noise overhangs, floating ledges, and pillars.
  - **5 Nether Biomes & Environmental Features**:
    1. **Nether Wastes**: Netherrack expanse, soul sand shorelines near lava, ceiling glowstone stalactite clusters. Traditional dark red/brown ambient fog.
    2. **Soul Sand Valley**: Undulating Soul Sand and Soul Soil dunes, massive ribbed Bone Block fossils, ethereal cyan/blue ambient fog (`#1A3543`).
    3. **Crimson Forest**: Surface Crimson Nylium, giant Crimson Fungi (thick `CrimsonStem` trunks + `NetherWartBlock` dome canopies), weeping vines, rich fiery crimson fog (`#330808`).
    4. **Warped Forest**: Surface Warped Nylium, giant Warped Fungi (`WarpedStem` + `WarpedWartBlock`), twisting vines, deep teal/blue-green fog (`#1A2223`).
    5. **Basalt Deltas**: Rugged fragmented terrain of `Basalt`, `PolishedBasalt`, and `Blackstone`, boiling `MagmaBlock` pools, volcanic ash/smoke fog (`#686873`).
  - **Full Chunk Generation & World Integration**:
    * `NetherGenerator::generate_chunk_column(chunk_x, chunk_z)` builds all 8 vertical `ChunkSection`s ($Y = 0..128$).
    * `NetherGenerator::populate_world(world, chunk_x, chunk_z)` inserts generated sections directly into the multi-section `World`.

---

### 16. Task 29: The End Islands Generator & Biomes
- **Module**: `src/world/generator/the_end.rs`, `src/world/generator/mod.rs`, `src/world/block_registry.rs`, `src/render/atlas.rs`
- **REA Decompiled References**:
  - `net.minecraft.world.level.levelgen.DensityFunctions$EndIslandDensityFunction` (`dok$i.class`):
    * Central Island profile: $\text{height} = 100.0 - \sqrt{x^2 + z^2} \times 8.0$, clamped to $[-100.0, 80.0]$.
    * Void Gap enforcement: chunk cell squared distance check $cx^2 + cz^2 \le 4096$ suppresses island generation between radius 100 and 1000 blocks.
    * Outer Island sampling: sample chunk cell grid $(cx, cz)$ with continuous noise threshold $<-0.35$; variable island radius $R = (\text{abs}(cx) \times 3439 + \text{abs}(cz) \times 147) \pmod{13} + 9$.
  - `net.minecraft.world.level.biome.TheEndBiomeSource` (`cvf.class`):
    * Radial biome distribution: `TheEnd` ($R \le 100$), `TheVoid` ($100 < R < 1000$), `EndHighlands`, `EndMidlands`, `EndBarrens`, `SmallEndIslands` ($R \ge 1000$).
  - `net.minecraft.world.level.levelgen.synth.SimplexNoise` (`edx.class`):
    * 2D Simplex noise implementation with permutation table and skew constants $F_2 = 0.5(\sqrt{3} - 1)$, $G_2 = (3 - \sqrt{3})/6$.
- **Features Implemented**:
  - **Radial Island Density Generation**:
    * **Central Island**: Continuous convex landmass centered at $(0, 0)$ with radius $\sim 100$ blocks, towering peak at $Y \approx 55..65$, composed entirely of `EndStone`.
    * **The Void Gap**: Complete silence between $R = 100$ and $R = 1000$ blocks with strictly zero density.
    * **Outer Floating Islands**: Beyond $R \ge 1000$, islands emerge in high regions of Simplex noise space with smooth parabolic underside falloff: $\text{underside} = 1.0 - \frac{(y - \text{center})^2}{\text{thickness}^2}$.
  - **Stochastic Cellular Chorus Plant & Flower Growth**:
    * Outer island upper surfaces ($Y \ge 50$) spawn chorus plants.
    * Recursive branching cellular loop: shoots vertically 3–5 blocks, branches into horizontal elbows (cardinal directions with PRNG coordinates), climbs vertically, and caps endpoints with `BlockId::ChorusFlower`.
  - **Block Registry & Texture Additions**:
    * Added `ChorusPlant` and `ChorusFlower` to `BlockId`.
    * Procedural texture fallbacks and atlas UV registration for both chorus blocks.
  - **Full Chunk Generation & World Population**:
    * `TheEndGenerator::generate_chunk_column(chunk_x, chunk_z)` builds all 8 vertical `ChunkSection`s ($Y = 0..128$).
    * `TheEndGenerator::populate_world(world, chunk_x, chunk_z)` populates the multi-section `World`.

---

### 17. Task 32: Nether & End Portal Linking Engine
- **Module**: `src/game/portals.rs`, `src/world/block_registry.rs`, `src/render/atlas.rs`, `src/game/inventory/mod.rs`, `src/game/mod.rs`
- **REA Decompiled References**:
  - `net.minecraft.world.level.portal.PortalShape` (`efp.class`):
    * Minimum inner dimensions: 2 wide $\times$ 3 high ($4 \times 5$ total frame).
    * Maximum inner dimensions: 21 wide $\times$ 21 high ($23 \times 23$ total frame).
    * Frame verification: bottom, top, left, right obsidian columns; interior air/portal space.
  - `net.minecraft.world.level.portal.PortalForcer` (`efn.class`):
    * Search radius: 128 blocks horizontal search for existing active `NetherPortal` blocks.
    * Frame creation: dynamic 4x5 Obsidian frame with 2x3 portal opening, safe air exit clearance pockets front and back, and solid landing steps.
  - `net.minecraft.world.level.block.NetherPortalBlock` (`dbt.class`):
    * Axis orientation (X or Z), light level 11, translucent purple swirling render pass.
  - `net.minecraft.world.level.block.EndPortalBlock` (`cze.class`) & `EndPortalFrameBlock` (`czf.class`):
    * 12-block frame ring layout surrounding a $3 \times 3$ central void.
    * Fixed End spawn coordinates $(100.5, 49.0, 0.5)$ with $5 \times 5$ Obsidian platform at $Y = 48$ and air clearance above.
- **Features Implemented**:
  - **Nether Portal Shape Detection & Lighting**:
    * Scans and verifies any rectangular Obsidian frame from $4 \times 5$ up to $23 \times 23$ along either the X or Z axis.
    * Lighting with Flint & Steel turns internal air into `BlockId::NetherPortal`.
    * Breaking any frame block propagates and collapses inner portal blocks back into `BlockId::Air`.
  - **Coordinate Translation & Linking**:
    * Overworld to Nether: $(X / 8, Y, Z / 8)$ clamped to safe Nether bounds ($Y \in [10, 120]$).
    * Nether to Overworld: $(X \times 8, Y, Z \times 8)$ clamped to Overworld bounds ($Y \in [10, 250]$).
    * Radial search within 128 blocks for existing portal; if none exists, constructs a new 4x5 Obsidian frame with 2x3 portal interior.
  - **Teleportation State Machine & Visual Transition**:
    * Tracks ticks spent in portal: 80 ticks for Survival (4.0s), 0 ticks for Creative (instantaneous).
    * Dynamic screen distortion: swirling purple tint overlay, nausea camera wobble $\sin(t \times 0.25) \times \text{intensity} \times 0.35$.
    * 100-tick post-teleport cooldown to prevent infinite oscillation.
  - **End Portal Mechanics**:
    * Validates the 12-frame ring geometry.
    * Fills inner $3 \times 3$ with `BlockId::EndPortal`.
    * Stepping in triggers instant teleportation to $(100.5, 49.0, 0.5)$ in The End and generates the $5 \times 5$ Obsidian platform at $Y=48$.

---

### 18. Task 30: Jigsaw Structure Engine (Villages & Mansions)
- **Module**: `src/world/structures/jigsaw.rs`, `src/world/structures/mod.rs`, `src/world/mod.rs`
- **REA Decompiled References**:
  - `net.minecraft.world.level.levelgen.structure.structures.JigsawStructure` (`ebg.class`):
    * `startPool`: initial root template pool
    * `maxDepth`: recursion limit ($1 \le \text{depth} \le 7$ for villages, up to 20 for complex structures)
    * `maxDistanceFromCenter`: 128-block bounding radius from structure anchor
    * `projectStartToHeightmap`: terrain surface height snapping boolean
  - `net.minecraft.world.level.levelgen.structure.pools.JigsawPlacement`:
    * Face-to-face snap formula: $\text{origin}_{\text{cand}} = \text{joint}_{\text{world}} + \text{facing}_{\text{world}} - R(\text{joint}_{\text{local}})$
    * Joint rotation and alignment matching opposite cardinals: $R(\text{facing}_{\text{local}}) = \text{facing}_{\text{world}}.\text{opposite}()$
    * AABB non-overlap validation against all previously placed pieces
  - `net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate` (`ebt.class`):
    * Gzip / Uncompressed NBT schematic serialization: `size` [W, H, D], `palette` block names, `blocks` pos + state + jigsaw tile entity metadata.
- **Features Implemented**:
  - **Recursive Jigsaw Assembly Engine**:
    * Queue-based breadth-first / recursive joint matching.
    * Cardinal 90-degree step rotations (`None`, `Clockwise90`, `Clockwise180`, `CounterClockwise90`) for positions, sizes, and joint vectors.
    * AABB intersection prevention: candidate pieces are checked against all placed pieces (with 0.1 perimeter margin to allow adjacent wall contact without collision).
  - **Plains Village Generation**:
    * Central town well (size 5x7x5) with cobblestone rim, central water source, oak pillars, and 4 road connection joints.
    * Branching street path segments (size 3x1x6) with T-junctions and building connection joints.
    * Plains cottages (size 5x5x5) with cobblestone floor, oak log corners, oak planks, glass windows, and doorway.
    * 2-row wheat farms (size 7x2x7) with central irrigation canal and cobblestone borders.
    * Terrain surface height snapping: dynamically adjusts starting anchor to ground heightmap $Y(x, z)$.
  - **Woodland Mansion Generation**:
    * 3-story modular grid architecture ($3 \times 3$ rooms per floor, $8 \times 6 \times 8$ module dimensions).
    * Cobblestone foundation floor and Dark Oak exterior logs / planks with glass windows.
    * Floor 1: Grand entrance foyer, dining hall, corridors.
    * Floor 2: Dual grand library (bookshelves and study desks), master bedroom with bed and chest.
    * Floor 3: Attic wooden maze with partition walls and high ceiling.

---

## 📈 Test Suite & Verification Status

```text
running 162 tests
test result: ok. 162 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.85s
```

All subsystems have dedicated unit and integration tests covering:
- Physics collision resolution, gravity, jumping, step-ups, and non-cubic compound collision boxes
- A* pathfinding and hostile mob targeting
- Anvil `.mca` chunk header serialization, block ID mapping, and zlib roundtrip
- Cellular automaton fluid flow and lava/water collision
- Breadth-first block & sky light propagation
- Handheld dynamic light shader uniform falloff
- Camera perspective switching and frustum culling
- Barycentric colormap UV sampling and 3x3 kernel smoothing
- Weather state machine transitions, temperature lapse rate, and rain/snow geometry generation
- Tile entities (Chest 27 slots, shift-click transfer, NBT persistence)
- Block registry across 3 dimensions, vanilla ID roundtrips, non-cubic bounding boxes & raycasting
- Closed-box 3D density noise, Bedrock boundaries, Lava ocean filling, and 5 Nether biomes climate & feature generation
- End island radial generator (Central island, Void gap, Outer islands, Simplex noise, and Chorus plant cellular growth)
- Nether & End portal linking engine (shape detection, coordinate translation, 128-block search/forcing, 80/0 tick timers, and End portal fixed spawn)
- Recursive Jigsaw structure engine (NBT schematic parser/serializer, joint orientation matching, AABB collision check, Plains Village, and Woodland Mansion)

---

## 🚀 Upcoming Tasks & Tomorrow's Execution Roadmap

When resuming next, execution will start directly from **Task 31**:

---

### 📋 Task 31: Furnace & Smelting Block Entity with Burn Time & Recipe Book
- **Modules**: `src/world/tile_entity.rs`, `src/render/gui/furnace_screen.rs`, `src/game/crafting/smelting.rs`
- **Features**:
  - `FurnaceTileEntity`: 3 slots (Input, Fuel, Output), `burn_time`, `cook_time`, `total_cook_time` (200 ticks standard).
  - Vanilla fuel burn ticks: Coal/Charcoal = 1600 ticks, Wood = 300 ticks, Lava Bucket = 20,000 ticks.
  - Smelting recipes: Raw Iron $\rightarrow$ Iron Ingot, Cobblestone $\rightarrow$ Stone, Sand $\rightarrow$ Glass, Logs $\rightarrow$ Charcoal.
  - Flame progress & arrow progress GUI rendering with dynamic texture atlas UVs.

---

### 📋 Task 33: Redstone Wire, Repeater & Power Propagation Engine
- **Modules**: `src/world/redstone/`, `src/world/block_registry.rs`
- **Features**:
  - Signal strength simulation ($0 \le \text{power} \le 15$) with step-down attenuation.
  - Redstone Dust, Redstone Torch (inversion logic), Lever, Button, Pressure Plate.
  - Redstone Repeaters with 1 to 4 tick delay (2 to 8 game ticks) and signal booster.
  - Redstone wire dynamic connectivity cross/line meshing.

---

### 📋 Task 34: Overworld Procedural Multi-Biome Terrain World Generator
- **Modules**: `src/world/generator/overworld.rs`
- **Features**:
  - Multi-octave Perlin / Simplex noise generator with seed hashing.
  - Continentalness, Erosion, and Peaks/Valleys noise splines matching Minecraft 1.18+ terrain format.
  - Surface builder: Grass/Dirt for Plains/Forests, Sand/Sandstone for Deserts, Snow/Ice for Tundras.
  - 3D cave noise carvers (spaghetti & cheese caves) carving under $y = 64$.
  - Ore generation clusters (Coal, Iron, Gold, Redstone, Diamond at deepslate levels).

---

## 🧭 Instructions for Maintaining This Document
Whenever new tasks or systems are implemented:
1. Update the **Summary of Completed Milestones** table with the new task number, module path, status, and summary.
2. Add a detailed section under **Detailed Breakdown of Completed Tasks** listing the exact files modified, algorithms implemented, and REA decompiled references.
3. Update the **Test Suite & Verification Status** with the latest test counts and verification results.
