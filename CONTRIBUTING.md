# Contributing to IronOxide

Thank you for your interest in contributing to **IronOxide** (Iron Oxide), an independent, clean-room **Minecraft 1.20.4** client and **WGPU voxel engine** in pure Rust.

We welcome contributions from systems engineers, graphics programmers, game developers, and technical writers!

---

## 1. Clean-Room Engineering Principles

IronOxide adheres strictly to **Clean-Room Reverse Engineering**:
- **Independent Implementation**: All code in this repository is authored in safe, idiomatic Rust based on clean mathematical specifications, data-format specifications (NBT, Anvil `.mca`, JSON schemas), and behavioral observations.
- **Zero Proprietary Code Inclusion**: Do not paste, copy, or submit decompiled Mojang bytecode or proprietary source files.
- **Verification by Test**: New features should be verified with unit tests asserting deterministic behavioral parity.

---

## 2. Workspace Crates Overview

IronOxide is organized as a modular Cargo workspace:

- **[`iron_oxide`](src/)**: The primary client binary and renderer:
  - WGPU rendering pipeline, camera systems, and WGSL shader modules.
  - Multi-threaded greedy chunk mesher (`render/mesher.rs`).
  - Axis-aligned bounding box (AABB) player physics and sub-voxel raycasting (DDA).
- **[`crates/mc_worldgen`](crates/mc_worldgen/)**:
  - 1:1 mathematical seed generation, Stafford13 bit-mixing, and Xoroshiro128++ PRNG.
  - 6D climate sampling router across 47+ Overworld biomes.
  - 3D cave carvers (Cheese, Spaghetti, Noodle) and dynamic Voronoi aquifers.
  - Procedural Jigsaw structure placement from Mojang template pools.
- **[`crates/mc_entities`](crates/mc_entities/)**:
  - Entity Component System (ECS) for passive, hostile, and neutral mobs.
  - GoalSelector AI state machine and 3D A* pathfinding.
- **[`crates/mc_items`](crates/mc_items/)**:
  - Complete 1,329+ item enum registry, durability models, tool tiers, and armor attributes.
- **[`crates/mc_crafting`](crates/mc_crafting/)**:
  - Serde recipe loaders parsing standard JSON recipes (shaped and shapeless) from `client.jar` with $O(1)$ fast lookup caches.
- **[`crates/mc_gameplay`](crates/mc_gameplay/)**:
  - Workstation state machines: 20 Hz ticking Furnace, Blast Furnace, Smoker, Anvil with exponential prior work penalties ($2^n - 1$), and Smithing Tables.

---

## 3. Local Development & Testing Workflow

### Prerequisites
- [Rust Toolchain](https://rustup.rs/) (version 1.85+ recommended, 2024 edition compatible).
- A GPU or integrated processor with Vulkan, DirectX 12, or Metal driver support.

### Running Checks and Tests

Always verify the entire workspace before committing:

```bash
# 1. Check compilation across all crates and targets
cargo check --workspace --all-targets

# 2. Run the full test suite (373 tests)
cargo test --workspace

# 3. Test a specific modular crate (e.g. worldgen)
cargo test -p mc_worldgen

# 4. Run the release binary
cargo run --release
```

---

## 4. Submitting a Pull Request (PR)

1. **Fork the Repository**: Create your branch from `main` (`git checkout -b feature/my-enhancement`).
2. **Follow Rust Idioms**: Use standard Rust formatting (`cargo fmt`) and check for lints (`cargo clippy`).
3. **Include Unit Tests**: If you fix a worldgen feature, physics calculation, or recipe handler, write a unit test in the corresponding crate verifying the fix.
4. **Link Documentation**: Check [DEVELOPMENT_MILESTONES.md](DEVELOPMENT_MILESTONES.md) and [WORK_LEFT.md](WORK_LEFT.md) to ensure your PR aligns with existing roadmap milestones.
5. **Open a PR**: Write a clear PR description outlining the changes and attach relevant logs or gameplay screenshots.

---

## 5. Community & Conduct

- Be respectful, constructive, and collaborative.
- Open an issue using the [Bug Report](.github/ISSUE_TEMPLATE/bug_report.md) or [Feature Request](.github/ISSUE_TEMPLATE/feature_request.md) templates before undertaking massive refactors.
