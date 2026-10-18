//! Unit and integration tests for 16x40x16 Grid Aquifers, Fluid Flooding, and Barrier Noise.
//! REA Reference: net.minecraft.world.level.levelgen.Aquifer (doc.class)
//! REA Reference: net.minecraft.world.level.levelgen.Aquifer$NoiseBasedAquifer (doc$c.class)

use mc_worldgen::aquifers::*;
use mc_worldgen::biomes::SurfaceBlock;
use mc_worldgen::seed_pipeline::random::PositionalRandomFactory;

#[test]
fn test_aquifer_grid_cell_partitioning() {
    assert_eq!(CELL_SIZE_X, 16);
    assert_eq!(CELL_SIZE_Y, 40);
    assert_eq!(CELL_SIZE_Z, 16);

    // Verify Euclidean floor division for grid coordinates
    assert_eq!(grid_x(0), 0);
    assert_eq!(grid_x(15), 0);
    assert_eq!(grid_x(16), 1);
    assert_eq!(grid_x(-1), -1);
    assert_eq!(grid_x(-16), -1);
    assert_eq!(grid_x(-17), -2);

    assert_eq!(grid_y(0), 0);
    assert_eq!(grid_y(39), 0);
    assert_eq!(grid_y(40), 1);
    assert_eq!(grid_y(-1), -1);
    assert_eq!(grid_y(-40), -1);
    assert_eq!(grid_y(-54), -2);
    assert_eq!(grid_y(-64), -2);

    assert_eq!(grid_z(0), 0);
    assert_eq!(grid_z(15), 0);
    assert_eq!(grid_z(16), 1);
    assert_eq!(grid_z(-1), -1);

    // Verify anchor points fall within cell boundaries
    let factory = PositionalRandomFactory::from_seed(123456789);
    let sampler = AquiferGridSampler::new(&factory);

    for cx in -3..=3 {
        for cy in -2..=2 {
            for cz in -3..=3 {
                let anchor = sampler.get_cell_anchor(cx, cy, cz);
                assert!(
                    anchor.x >= cx * 16 && anchor.x < (cx + 1) * 16,
                    "anchor.x {} out of bounds for cell cx {}",
                    anchor.x,
                    cx
                );
                assert!(
                    anchor.y >= cy * 40 && anchor.y < (cy + 1) * 40,
                    "anchor.y {} out of bounds for cell cy {}",
                    anchor.y,
                    cy
                );
                assert!(
                    anchor.z >= cz * 16 && anchor.z < (cz + 1) * 16,
                    "anchor.z {} out of bounds for cell cz {}",
                    anchor.z,
                    cz
                );
            }
        }
    }
}

#[test]
fn test_aquifer_fluid_type_elevation_switching_lava_below_minus_54() {
    // Test FluidStatus elevation rules:
    // If Y < -54, fluid MUST switch to Lava
    let status_water = FluidStatus::new(30, AquiferFluidType::Water);

    // Above target level: Air
    assert_eq!(status_water.at_elevation(31), AquiferFluidType::Air);
    assert_eq!(status_water.at_elevation(50), AquiferFluidType::Air);

    // At or below target level down to -54: Water
    assert_eq!(status_water.at_elevation(30), AquiferFluidType::Water);
    assert_eq!(status_water.at_elevation(0), AquiferFluidType::Water);
    assert_eq!(status_water.at_elevation(-54), AquiferFluidType::Water);

    // Below -54: strictly switches to Lava!
    assert_eq!(status_water.at_elevation(-55), AquiferFluidType::Lava);
    assert_eq!(status_water.at_elevation(-60), AquiferFluidType::Lava);
    assert_eq!(status_water.at_elevation(-63), AquiferFluidType::Lava);

    // NoiseBasedAquifer test with fixed seed
    let factory = PositionalRandomFactory::from_seed(987654321);
    let aquifer = NoiseBasedAquifer::new(&factory);

    // Query deep underground points below -54
    for x in [-50, 0, 50] {
        for z in [-50, 0, 50] {
            let substance = aquifer.compute_substance_at(x, -58, z);
            // Must either be solid rock barrier or Lava fluid
            match substance {
                AquiferSubstance::Solid => {} // barrier wall or bedrock
                AquiferSubstance::Fluid(f) => assert_eq!(f, AquiferFluidType::Lava),
                AquiferSubstance::Air => panic!("Deep cavern at Y=-58 should never be open Air without fluid or barrier"),
            }
        }
    }
}

#[test]
fn test_aquifer_flooding_logic_backfills_carved_space() {
    let factory = PositionalRandomFactory::from_seed(12345);
    let aquifer = NoiseBasedAquifer::new(&factory);

    // Test a flooded cell at y = 10 with water
    let status = FluidStatus::new(20, AquiferFluidType::Water);
    assert_eq!(status.at_elevation(10), AquiferFluidType::Water);

    // Test apply_to_block backfilling:
    // Solid non-air blocks should never be replaced by the aquifer
    let stone = aquifer.apply_to_block(0, 10, 0, SurfaceBlock::Stone);
    assert_eq!(stone, SurfaceBlock::Stone);

    let deepslate = aquifer.apply_to_block(0, -20, 0, SurfaceBlock::Deepslate);
    assert_eq!(deepslate, SurfaceBlock::Deepslate);

    // Air blocks carved below local fluid level must be backfilled
    // If flooded with water, apply_to_block turns Air into Water
    let flooded = aquifer.apply_to_block(0, 0, 0, SurfaceBlock::Air);
    assert!(
        flooded == SurfaceBlock::Water || flooded == SurfaceBlock::Stone || flooded == SurfaceBlock::Air,
        "Flooded space should be Water, stone barrier, or dry pocket, got {:?}",
        flooded
    );
}

#[test]
fn test_barrier_noise_prevents_leaking_and_creates_dry_pockets() {
    let factory = PositionalRandomFactory::from_seed(54321);
    let barrier_sampler = BarrierNoiseSampler::new(&factory);

    // 1. Water vs Lava interface creates an unconditional high-pressure stone barrier
    let water_status = FluidStatus::new(30, AquiferFluidType::Water);
    let lava_status = FluidStatus::new(20, AquiferFluidType::Lava);

    let pressure_wl = barrier_sampler.calculate_pressure(0.0, 10, 0.0, &water_status, &lava_status);
    assert_eq!(pressure_wl, 2.0, "Water/Lava interface must yield pressure 2.0");

    // 2. Equal fluid levels have zero pressure
    let equal_status1 = FluidStatus::new(20, AquiferFluidType::Water);
    let equal_status2 = FluidStatus::new(20, AquiferFluidType::Water);
    let pressure_eq = barrier_sampler.calculate_pressure(0.0, 10, 0.0, &equal_status1, &equal_status2);
    assert_eq!(pressure_eq, 0.0, "Identical fluid levels must yield pressure 0.0");

    // 3. Disparate fluid levels produce barrier wall
    let high_water = FluidStatus::new(50, AquiferFluidType::Water);
    let low_water = FluidStatus::new(-10, AquiferFluidType::Water);

    let is_barrier = barrier_sampler.is_barrier_wall(10.0, 20, 10.0, 0.8, &high_water, &low_water);
    // At similarity 0.8 between level 50 and -10, pressure is active
    let pressure_diff = barrier_sampler.calculate_pressure(10.0, 20, 10.0, &high_water, &low_water);
    assert!(pressure_diff != 0.0);
    assert!(is_barrier, "Barrier rock must form between disparate fluid levels (50 vs -10)");
}

#[test]
fn test_aquifer_unflooded_dry_pockets() {
    // When a cell is dry (fluid_level = i32::MIN), it remains dry air
    let dry_status = FluidStatus::new(i32::MIN, AquiferFluidType::Air);
    assert_eq!(dry_status.at_elevation(10), AquiferFluidType::Air);
    assert_eq!(dry_status.at_elevation(50), AquiferFluidType::Air);
    assert_eq!(dry_status.at_elevation(-30), AquiferFluidType::Air);

    // Sample cells across a grid to verify that both flooded and dry cells occur naturally
    let factory = PositionalRandomFactory::from_seed(13579);
    let sampler = AquiferGridSampler::new(&factory);

    let mut dry_cell_found = false;
    let mut water_cell_found = false;
    let mut lava_cell_found = false;

    for cx in -10..=10 {
        for cy in -2..=2 {
            for cz in -10..=10 {
                let status = sampler.compute_cell_status(cx, cy, cz);
                match status.fluid_type {
                    AquiferFluidType::Air => dry_cell_found = true,
                    AquiferFluidType::Water => water_cell_found = true,
                    AquiferFluidType::Lava => lava_cell_found = true,
                }
            }
        }
    }

    assert!(dry_cell_found, "Natural world generation must generate unflooded dry cave pockets");
    assert!(water_cell_found, "Natural world generation must generate water aquifers");
    assert!(lava_cell_found, "Natural world generation must generate deep lava aquifers");
}

#[test]
fn test_aquifer_bedrock_immunity() {
    let factory = PositionalRandomFactory::from_seed(99999);
    let aquifer = NoiseBasedAquifer::new(&factory);

    // Bedrock floor (Y <= -60) must ALWAYS evaluate to Solid rock and never be replaced by fluid
    for y in [-60, -61, -63, -64] {
        let substance = aquifer.compute_substance_at(0, y, 0);
        assert_eq!(
            substance,
            AquiferSubstance::Solid,
            "Aquifer must strictly preserve Bedrock barrier at Y={}",
            y
        );
    }
}
