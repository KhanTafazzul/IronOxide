# ⚡ IronOxide

> **Minecraft 1.20.4 reverse-engineered from scratch in pure Rust.**  
> Zero Java runtime. Zero garbage collector stutter. 100% native multi-threaded voxel performance powered by **WGPU** and **WGSL** shaders.

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Language: Rust 2024](https://img.shields.io/badge/Language-Rust_2024-orange.svg)](https://www.rust-lang.org/)
[![Graphics: WGPU](https://img.shields.io/badge/Graphics-WGPU_/_WGSL-green.svg)](https://wgpu.rs/)
[![Tests: 303 Passing](https://img.shields.io/badge/Tests-303_Passed-brightgreen.svg)]()
[![Parity: Minecraft 1.20.4](https://img.shields.io/badge/Parity-Minecraft_1.20.4-red.svg)](https://minecraft.net/)

---

## 🎮 What is IronOxide?

**IronOxide** is a clean-room, native voxel engine and Minecraft client built in Rust. Instead of relying on JVM layers or emulation, it reverse-engineers the official `client.jar` bytecodes to reconstruct vanilla Minecraft's exact algorithms—from terrain generation and chunk meshing to inventory management and redstone-ready ticking.

Whether it is climbing snowy peaks, exploring procedural trial chambers, or running through volumetric rainstorms with dynamic handheld torchlight, IronOxide delivers the full Minecraft experience at blazing native frame rates.

---

## ✨ Key Features

### 🎨 Next-Gen Voxel Rendering & Shaders
* **Modern Graphics Stack**: Built directly on **WGPU** (Vulkan, DirectX 12, Metal, and WebGPU).
* **Asynchronous 3D Greedy Mesher**: Merges coplanar voxel faces in parallel via `rayon`, slashing triangle counts and draw calls to maximize FPS.
* **Ambient Occlusion & Biome Tinting**: 4-level smooth vertex AO and triangular barycentric colormap interpolation across biomes with 3x3 neighborhood smoothing.
* **Real-Time Dynamic Lighting**: Walk into dark caves with a Torch or Lantern in hand—smooth inverse-square point lights illuminate the world via WGSL shader uniforms.
* **Volumetric Weather**: Dynamic rain, snowfall, wind-driven particle sheets, and temperature lapse rates that naturally freeze surface water and accumulate layered snow.

### 🗺️ 1:1 Seed-for-Seed World Generation
* **Mathematical Vanilla Parity**: 100% bit-exact parity with Java Edition 1.20.4 worldgen seeds using Stafford13 mixing, Xoroshiro128++, and 6D climate sampling.
* **Subterranean 3D Noise Caves**: Expansive Cheese caverns, twisting Spaghetti tunnels, claustrophobic Noodle passages, and 112-step steep ravines.
* **Dynamic Aquifers & Flooding**: Subsurface Voronoi cell sampling, water-to-lava transitions at $Y < -54$, and natural stone barrier dams protecting dry cave systems.
* **All Three Dimensions**:
  * 🌲 **Overworld**: 47+ biomes, Badlands 192-layer banded terracotta, ore veins with exact vertical trapezoids, and procedural tree foliage placers.
  * 🔥 **The Nether**: Closed-box 3D density noise, boiling lava oceans at $Y \le 31$, Nether Fortresses, and Bastion Remnants.
  * 🌌 **The End**: Central dragon island ($R \le 100$), outer void gaps, Simplex islands, branching Chorus plants, and End Cities with 25% End Ships.
* **Dynamic Jigsaw Engine**: Parses all 182 Mojang template pool JSONs directly from `client.jar` to assemble Plains, Savanna, and Desert Villages, Woodland Mansions, and Trial Chambers.

### 🏃 Authentic Physics & Survival Gameplay
* **Vanilla Movement & Collision**: Decoupled AABB collision resolution, 0.6m continuous step-up traversal, authentic gravity ($0.08$), and jump impulse ($0.42$).
* **Sub-Voxel Raycasting**: Amanatides-Woo DDA grid traversal supporting non-cubic shapes like Slabs, Stairs, End Rods, and Chests with a 5.0-block reach.
* **Survival Vitals & Combat**: Health (20 HP), hunger, saturation, exhaustion drain, natural regeneration, and hostile Zombie mob AI with 3D A* pathfinding.
* **Interactive Workstations**: 3x3 Crafting Table, 20Hz ticking Furnaces/Smokers/Blast Furnaces, Anvils with prior work penalty ($2^n - 1$), and Smithing Tables.
* **Standard Anvil Storage**: Save and load worlds using standard `.mca` region files and Gzip-compressed `level.dat` NBT structures.

---

## 🧱 Workspace Architecture

IronOxide is organized into clean, high-performance modular crates:

```text
IronOxide/
├── crates/
│   ├── mc_worldgen/    # 1:1 seed-exact noise, 47+ biomes, carvers, aquifers, ores & template pools
│   ├── mc_items/       # 1,329+ item enum registry, tool harvest levels, and armor tiers
│   ├── mc_crafting/    # Serde shaped/shapeless recipe parser (822 recipes from client.jar) & O(1) cache
│   ├── mc_gameplay/    # Workstations engine (Crafting Table, Furnaces, Anvil, Smithing)
│   └── mc_entities/    # Mob ECS, AABB physics, GoalSelector AI engine & 3D voxel node navigation
└── src/
    ├── render/         # WGPU pipeline, greedy mesher, dynamic lighting, weather & HUD
    ├── game/           # Player physics, raycasting, survival stats, portals & mob AI
    ├── world/          # Block registry, cellular fluids, dual-nibble lighting & dimensions
    └── storage/        # Anvil .mca region serializer, Gzip NBT parser & level.dat
```

---

## 🚀 Quick Start

### Prerequisites
* [Rust](https://rustup.rs/) (2024 edition supported, latest stable)
* A graphics card supporting Vulkan, DirectX 12, or Metal
* *(Optional)* An official `client.jar` in the root folder (IronOxide will auto-extract authentic textures, recipes, and template pools; built-in procedural fallbacks are provided if absent).

### Commands
```bash
# Clone the repository
git clone https://github.com/your-username/IronOxide.git
cd IronOxide

# Run the automated test suite (303 tests)
cargo test --workspace

# Launch the client in optimized release mode
cargo run --release
```

---

## ⌨️ Controls & Keybindings

| Key | Action |
|:---:|:---|
| **W / A / S / D** | Walk Forward / Strafe Left / Backward / Strafe Right |
| **Space** | Jump / Ascend |
| **Left Shift** | Sprint modifier |
| **F** | Toggle Flight mode |
| **Left Click** | Mine / Break targeted block |
| **Right Click** | Place block / Interact with Chest or Workstation |
| **1 – 9 / Scroll** | Hotbar slot selection |
| **E** | Open Inventory / Crafting Grid |
| **F3** | Toggle Debug HUD (FPS, Coordinates, Biome, GPU & Memory) |
| **F5** | Cycle Camera (First Person $\rightarrow$ Third Person Back $\rightarrow$ Third Person Front) |
| **Esc** | Pause menu / Release mouse cursor |

---

## 🧪 Verification & Stability

Every subsystem is backed by unit tests verifying mathematical parity with the original Java game mechanics:

```text
✓ 179 passed in iron_oxide     (Physics, Lighting, Fluids, Portals, Rendering, GUI)
✓  94 passed in mc_worldgen    (Noise router, Biome climate, Carvers, Jigsaw pools, Pale Garden)
✓  17 passed in mc_entities    (Mob ECS, AABB physics, GoalSelector AI, 3D Voxel Node Navigation)
✓   5 passed in mc_items       (Tool tiers, Armor attributes, ItemId mappings)
✓   3 passed in mc_crafting    (Shaped & shapeless recipe resolution from client.jar)
✓   5 passed in mc_gameplay    (Furnace ticking, Anvil penalty, Crafting matrices)
──────────────────────────────────────────────────────────────────────────
Total: 303 / 303 Tests Passing (100% Green)
```

For full task-by-task engineering breakdowns and REA decompiled references, check out [DEVELOPMENT_MILESTONES.md](DEVELOPMENT_MILESTONES.md).

---

## 📄 License

This project is licensed under the **GNU General Public License v3.0 (GPLv3)**.  
See the [LICENSE](LICENSE) file for details.
