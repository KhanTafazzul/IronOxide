# IronOxide ⛏️🦀

A high-performance Minecraft client and voxel engine reverse-engineered and built from scratch in **Rust** using **WGPU** and **WGSL** shaders.

---

## 🌟 Highlights & Features

- **WGPU Native Rendering**: Modern, cross-platform graphics backend using WGSL shaders and nearest-neighbor texture sampling (`FilterMode::Nearest`) for authentic Minecraft pixel aesthetics.
- **Runtime Texture Atlas Loader**: Dynamic extraction and stitching of vanilla block textures directly from `client.jar` (or procedural fallback generation) with sub-pixel UV coordinates for all face directions (`UP`, `DOWN`, `NORTH`, `SOUTH`, `EAST`, `WEST`).
- **3D Greedy Chunk Mesher**:
  - Axis-aligned greedy face merging algorithm that merges coplanar, adjacent faces sharing identical textures into minimal rectangular quads.
  - Per-vertex Ambient Occlusion (AO levels 0 to 3) calculated based on neighboring block occupancy.
  - Asynchronous background meshing powered by `rayon` to guarantee zero dropped frames during exploration.
- **Authentic Player Physics & Collision**:
  - Re-implemented from Minecraft 1.20.4 client source (`LocalPlayer` and `Entity.move(...)`).
  - Standard player dimensions: width 0.6m, height 1.8m, eye height 1.62m.
  - 3-axis decoupled AABB collision resolution with **0.6m step-up** (automatically walking up slabs and single block increments without jumping).
  - Exact movement constants: gravity (0.08 blocks/tick²), jump impulse (0.42 blocks/tick), air drag, and walking acceleration.
- **3D Fast Voxel Traversal (Amanatides-Woo DDA)**:
  - Sub-voxel raycasting with 5.0 blocks reach.
  - Left-Click: Destroy targeted block and trigger multi-section boundary re-meshing.
  - Right-Click: Place selected block against the targeted face with collision detection against the player AABB.
- **2D HUD, Crosshair & Hotbar**:
  - Orthographic 2D rendering pass executed after the 3D terrain pass with `wgpu::LoadOp::Load`.
  - Inverted blending crosshair (`BlendFactor::OneMinusDst`, `OneMinusSrc`) centered at exact screen center.
  - Vanilla $182 \times 22$ hotbar anchored at bottom center with active selection highlight box.
  - Input controls: Mouse scroll wheel and number keys (1–9) cycle active hotbar slots and update block placement.

---

## 🚀 Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/) (2024 edition / latest stable)
- Optional: Minecraft `client.jar` (1.20.4) placed in project root or parent directory for authentic textures.

### Build & Run
```bash
# Run unit tests
cargo test

# Run game in release mode
cargo run --release
```

### Controls
| Input | Action |
|---|---|
| **W, A, S, D** | Move Forward / Left / Backward / Right |
| **Space** | Jump |
| **Shift** | Sprint |
| **Mouse Motion** | Look Around (Yaw & Pitch) |
| **Mouse Left-Click** | Break Targeted Block |
| **Mouse Right-Click** | Place Selected Block |
| **Scroll Wheel / 1–9** | Cycle Active Hotbar Slot |
| **Escape** | Toggle Mouse Capture |

---

## 🛠️ Architecture Overview

```
mc_rust_game/
├── Cargo.toml
├── src/
│   ├── main.rs            # Application event loop, input handling & interaction
│   ├── game/
│   │   ├── mod.rs         # Game state exports
│   │   ├── physics.rs     # AABB collision, step-up & player movement
│   │   ├── raycast.rs     # Amanatides-Woo DDA voxel raycasting
│   │   └── world.rs       # Chunk section storage & multi-section boundary updates
│   └── render/
│       ├── mod.rs         # Renderer module re-exports
│       ├── atlas.rs       # Texture atlas stitcher & directional UV sampler
│       ├── camera.rs      # First-person 3D camera & projection matrix
│       ├── context.rs     # WGPU context, surface & render pass management
│       ├── gui/           # 2D HUD, inverted crosshair & hotbar renderer
│       ├── mesher.rs      # 3D Greedy mesher & vertex AO generation
│       └── uniforms.rs    # Camera uniform buffers
```

---

## 📄 License
MIT License.
