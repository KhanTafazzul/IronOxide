# ⛏️ IronOxide: Reverse-Engineered Minecraft in Rust
## 📜 Master Project Progress & Implementation Log (`what i have done (what i have made till now).md`)

> **Project Goal**: Re-implement Minecraft 1.20.4 client and voxel engine from scratch in **Rust** using **WGPU** and **WGSL** shaders by reverse-engineering `client.jar` using decompilation and exact vanilla algorithms.
>
> **Status**: **128 / 128 Unit & Integration Tests Passing** ✅
> **Last Updated**: Tasks 1 through 25 Fully Completed & Verified.

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

## 📈 Test Suite & Verification Status

```text
running 128 tests
test result: ok. 128 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.10s
```

All subsystems have dedicated unit and integration tests covering:
- Physics collision resolution, gravity, jumping, and step-ups
- A* pathfinding and hostile mob targeting
- Anvil `.mca` chunk header serialization and zlib roundtrip
- Cellular automaton fluid flow and lava/water collision
- Breadth-first block & sky light propagation
- Handheld dynamic light shader uniform falloff
- Camera perspective switching and frustum culling
- Barycentric colormap UV sampling and 3x3 kernel smoothing
- Weather state machine transitions, temperature lapse rate, and rain/snow geometry generation

---

## 🧭 Instructions for Maintaining This Document
Whenever new tasks or systems are implemented:
1. Update the **Summary of Completed Milestones** table with the new task number, module path, status, and summary.
2. Add a detailed section under **Detailed Breakdown of Completed Tasks** listing the exact files modified, algorithms implemented, and REA decompiled references.
3. Update the **Test Suite & Verification Status** with the latest test counts and verification results.
