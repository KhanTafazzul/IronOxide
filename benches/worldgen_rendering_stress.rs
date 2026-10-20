//! ==============================================================================
//! IronOxide Performance & Stress Benchmarking Suite
//! ==============================================================================
//! This benchmark suite performs two distinct stress tests:
//!
//! 1. Static Batch Generation Test:
//!    - Generates a fixed 128 × 128 chunk matrix (16,384 chunk columns from Y: -64 to 320).
//!    - Computes exact statistical metrics: Mean, Median, Std Dev, Min, Max, and Chunks/sec.
//!    - Inner loop wrapped in `criterion::black_box` to prevent dead-code elimination.
//!    - Dedicated warmup phase to eliminate allocator & thread-pool setup skew.
//!
//! 2. High-Speed Movement Stress Test (FPS, Chunk Streaming & Frame Spikes):
//!    - Simulates player moving along +X axis at 100 blocks/sec (Elytra / Super-speed flight).
//!    - Active render distance: R = 16 chunks (33 × 33 = 1,089 chunk active window).
//!    - Unload radius: R + 2 = 18 chunks.
//!    - Measures frame execution time for every tick/frame.
//!    - Detects micro-stutters by calculating 1% Low and 0.1% Low frame times / FPS.
//!
//! Placeholders for engine integration:
//! - Look for: `// Connect your World/Chunk struct here`
//! ==============================================================================

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// ==============================================================================
// ENGINE STRUCT PLACEHOLDERS & ENGINE BRIDGES
// ==============================================================================

// [PLACEHOLDER 1: CHUNK DATA STRUCTURE]
// Connect your World/Chunk struct here: Swap with your engine's chunk representation
#[derive(Clone)]
pub struct EngineChunkColumn {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub block_count: usize,
    // Connect your World/Chunk struct here: e.g. Box<[BlockType; 98304]> or PaletteSection
    pub raw_data: Vec<u8>,
}

impl EngineChunkColumn {
    pub fn new_empty(cx: i32, cz: i32) -> Self {
        Self {
            chunk_x: cx,
            chunk_z: cz,
            block_count: 16 * 384 * 16,
            raw_data: Vec::new(),
        }
    }
}

// [PLACEHOLDER 2: CHUNK GENERATOR]
// Connect your World/Chunk struct here: Swap with your engine's procedural world generator
pub struct EngineWorldGenerator {
    pub seed: i64,
    // Connect your World/Chunk struct here: e.g. NoiseRouter, BiomeRegistry, SurfaceSystem
    internal_seed_mix: u64,
}

impl EngineWorldGenerator {
    pub fn new(seed: i64) -> Self {
        Self {
            seed,
            internal_seed_mix: (seed as u64) ^ 0x5DEECE66D,
        }
    }

    /// Generates a single chunk column (Y = -64..=319, 16x384x16 voxels).
    /// Connect your World/Chunk struct here: Replace with your engine's full generation pipeline.
    #[inline]
    pub fn generate_chunk(&self, cx: i32, cz: i32) -> EngineChunkColumn {
        // Fast procedural hash mimicking multi-octave 3D density noise sampling
        let mut h = (self.internal_seed_mix)
            .wrapping_add((cx as u64).wrapping_mul(0x9E3779B97F4A7C15))
            .wrapping_add((cz as u64).wrapping_mul(0xBF58476D1CE4E5B9));
        h ^= h >> 30;
        h = h.wrapping_mul(0xBF58476D1CE4E5B9);
        h ^= h >> 27;

        // Simulate 16 sections of 16x16x16 voxels (384 height total)
        // In full engine, calls: mc_worldgen::pipeline::generate_chunk_column_pipeline
        let mut sample_bytes = Vec::with_capacity(1024);
        sample_bytes.extend_from_slice(&h.to_le_bytes());
        sample_bytes.resize(1024, (h & 0xFF) as u8);

        EngineChunkColumn {
            chunk_x: cx,
            chunk_z: cz,
            block_count: 16 * 384 * 16,
            raw_data: sample_bytes,
        }
    }
}

// [PLACEHOLDER 3: CHUNK MESH & RENDER ENGINE]
// Connect your World/Chunk struct here: Swap with your engine's mesher and GPU vertex buffers
pub struct EngineChunkMesh {
    pub chunk_pos: (i32, i32),
    pub quad_count: usize,
    pub vertex_buffer_bytes: usize,
}

impl EngineChunkMesh {
    /// Simulates greedy meshing face-culling and quad generation
    pub fn mesh_chunk(chunk: &EngineChunkColumn) -> Self {
        // Connect your World/Chunk struct here: e.g. greedy_mesh_section()
        let quads = (chunk.raw_data.len() * 3) % 4096 + 512;
        Self {
            chunk_pos: (chunk.chunk_x, chunk.chunk_z),
            quad_count: quads,
            vertex_buffer_bytes: quads * 4 * 32, // 4 vertices per quad * 32 bytes/vertex
        }
    }
}

// [PLACEHOLDER 4: ACTIVE WORLD CONTAINER]
// Connect your World/Chunk struct here: Swap with your engine's World / ChunkManager
pub struct EngineWorldState {
    pub loaded_chunks: HashMap<(i32, i32), EngineChunkColumn>,
    pub active_meshes: HashMap<(i32, i32), EngineChunkMesh>,
}

impl EngineWorldState {
    pub fn new() -> Self {
        Self {
            loaded_chunks: HashMap::with_capacity(2048),
            active_meshes: HashMap::with_capacity(2048),
        }
    }

    pub fn insert_chunk(&mut self, chunk: EngineChunkColumn, mesh: EngineChunkMesh) {
        let pos = (chunk.chunk_x, chunk.chunk_z);
        self.loaded_chunks.insert(pos, chunk);
        self.active_meshes.insert(pos, mesh);
    }

    pub fn unload_chunk(&mut self, pos: &(i32, i32)) {
        self.loaded_chunks.remove(pos);
        self.active_meshes.remove(pos);
    }
}

// ==============================================================================
// STATISTICAL METRICS ENGINE (Mean, Median, Std Dev, 1% Low, 0.1% Low)
// ==============================================================================

#[derive(Debug, Clone)]
pub struct BenchmarkStatistics {
    pub total_samples: usize,
    pub total_duration: Duration,
    pub mean_ms: f64,
    pub median_ms: f64,
    pub std_dev_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub p90_ms: f64,
    pub p99_ms: f64,       // 1% Low frame time
    pub p99_9_ms: f64,     // 0.1% Low frame time
    pub avg_fps: f64,
    pub one_percent_low_fps: f64,
    pub zero_point_one_percent_low_fps: f64,
}

impl BenchmarkStatistics {
    pub fn compute(mut durations_ms: Vec<f64>) -> Self {
        assert!(!durations_ms.is_empty(), "Cannot compute statistics on empty duration list");
        let count = durations_ms.len();

        // Sort ascending for percentile & median extraction
        durations_ms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let sum: f64 = durations_ms.iter().sum();
        let mean = sum / count as f64;

        let variance: f64 = durations_ms
            .iter()
            .map(|val| (val - mean).powi(2))
            .sum::<f64>()
            / count as f64;
        let std_dev = variance.sqrt();

        let median = if count % 2 == 1 {
            durations_ms[count / 2]
        } else {
            (durations_ms[count / 2 - 1] + durations_ms[count / 2]) / 2.0
        };

        let min = durations_ms[0];
        let max = durations_ms[count - 1];

        // Percentile Helper
        let percentile = |pct: f64| -> f64 {
            let rank = (pct / 100.0) * (count - 1) as f64;
            let lower = rank.floor() as usize;
            let upper = rank.ceil() as usize;
            if lower == upper {
                durations_ms[lower]
            } else {
                let weight = rank - lower as f64;
                durations_ms[lower] * (1.0 - weight) + durations_ms[upper] * weight
            }
        };

        let p90 = percentile(90.0);
        let p99 = percentile(99.0);
        let p99_9 = percentile(99.9);

        let avg_fps = if mean > 0.0 { 1000.0 / mean } else { 0.0 };
        let one_percent_low_fps = if p99 > 0.0 { 1000.0 / p99 } else { 0.0 };
        let zero_point_one_percent_low_fps = if p99_9 > 0.0 { 1000.0 / p99_9 } else { 0.0 };

        Self {
            total_samples: count,
            total_duration: Duration::from_secs_f64(sum / 1000.0),
            mean_ms: mean,
            median_ms: median,
            std_dev_ms: std_dev,
            min_ms: min,
            max_ms: max,
            p90_ms: p90,
            p99_ms: p99,
            p99_9_ms: p99_9,
            avg_fps,
            one_percent_low_fps,
            zero_point_one_percent_low_fps,
        }
    }

    pub fn print_report(&self, title: &str) {
        println!("\n================================================================================");
        println!("  BENCHMARK REPORT: {}", title);
        println!("================================================================================");
        println!("  Total Samples Recorded:     {}", self.total_samples);
        println!("  Total Benchmark Time:       {:.3} s", self.total_duration.as_secs_f64());
        println!("--------------------------------------------------------------------------------");
        println!("  Mean Frame/Batch Time:      {:.3} ms  (Avg: {:.2} FPS)", self.mean_ms, self.avg_fps);
        println!("  Median Time (P50):          {:.3} ms", self.median_ms);
        println!("  Std Deviation (σ):          ±{:.3} ms", self.std_dev_ms);
        println!("  Min Duration (Fastest):     {:.3} ms", self.min_ms);
        println!("  Max Duration (Worst Spike): {:.3} ms", self.max_ms);
        println!("--------------------------------------------------------------------------------");
        println!("  90th Percentile (P90):      {:.3} ms", self.p90_ms);
        println!("  99th Percentile (P99):      {:.3} ms  -> 1.0% Low: {:.2} FPS", self.p99_ms, self.one_percent_low_fps);
        println!("  99.9th Percentile (P99.9):  {:.3} ms  -> 0.1% Low: {:.2} FPS", self.p99_9_ms, self.zero_point_one_percent_low_fps);
        println!("================================================================================\n");
    }
}

// ==============================================================================
// TEST 1: STATIC BATCH GENERATION TEST (128 × 128 CHUNKS)
// ==============================================================================

/// Generates a fixed matrix of (matrix_size × matrix_size) chunks in parallel or serial,
/// wrapping inner chunk outputs in criterion::black_box to prevent dead-code elimination.
pub fn run_static_batch_generation(
    generator: &EngineWorldGenerator,
    matrix_size: i32,
    parallel: bool,
) -> (usize, Duration) {
    let total_chunks = (matrix_size * matrix_size) as usize;
    let chunks_processed = AtomicUsize::new(0);

    let start = Instant::now();

    if parallel {
        // Parallel multi-threaded batch generation across Rayon worker pool
        (0..matrix_size).into_par_iter().for_each(|cx| {
            for cz in 0..matrix_size {
                // Connect your World/Chunk struct here:
                let chunk = generator.generate_chunk(cx, cz);
                // Criterion black_box prevents compiler dead-code elimination
                black_box(&chunk);
                chunks_processed.fetch_add(1, Ordering::Relaxed);
            }
        });
    } else {
        // Serial single-threaded batch generation
        for cx in 0..matrix_size {
            for cz in 0..matrix_size {
                // Connect your World/Chunk struct here:
                let chunk = generator.generate_chunk(cx, cz);
                black_box(&chunk);
                chunks_processed.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    let elapsed = start.elapsed();
    assert_eq!(chunks_processed.load(Ordering::SeqCst), total_chunks);
    (total_chunks, elapsed)
}

/// Executes multiple iterations of the 128 × 128 batch generation test,
/// including a dedicated warmup phase, and calculates full statistical metrics.
pub fn execute_128x128_matrix_statistical_benchmark(
    generator: &EngineWorldGenerator,
    iterations: usize,
    warmup_iterations: usize,
) -> BenchmarkStatistics {
    const MATRIX_SIZE: i32 = 128; // 128 × 128 = 16,384 chunks

    println!("--------------------------------------------------------------------------------");
    println!(" [TEST 1] Static Batch Generation Test (128 × 128 Chunks = 16,384 Columns)");
    println!("--------------------------------------------------------------------------------");

    // 1. WARMUP PHASE: Pre-fault thread pool allocations & noise tables
    print!(" -> Executing Warmup Phase ({} cycles)... ", warmup_iterations);
    for _ in 0..warmup_iterations {
        // Warmup on a smaller 16x16 grid to prime memory allocators
        let (_, _) = run_static_batch_generation(generator, 16, true);
    }
    println!("Done (Allocators & Thread-pools primed).");

    // 2. MEASUREMENT PHASE
    let mut iteration_times_ms = Vec::with_capacity(iterations);
    for i in 1..=iterations {
        print!(" -> Running Iteration {}/{} (16,384 chunks)... ", i, iterations);
        let (chunks, elapsed) = run_static_batch_generation(generator, MATRIX_SIZE, true);
        let ms = elapsed.as_secs_f64() * 1000.0;
        let throughput = chunks as f64 / elapsed.as_secs_f64();
        println!("{:.2} ms ({:.0} chunks/sec)", ms, throughput);
        iteration_times_ms.push(ms);
    }

    let stats = BenchmarkStatistics::compute(iteration_times_ms);
    stats.print_report("128 × 128 Static Batch Generation (16,384 Chunks)");
    stats
}

// ==============================================================================
// TEST 2: HIGH-SPEED MOVEMENT STRESS TEST (FPS & CHUNK LOADING)
// ==============================================================================

/// High-Speed Player Traversal Configuration
pub struct MovementStressConfig {
    pub velocity_blocks_per_sec: f64, // e.g. 100.0 blocks/sec
    pub render_distance_chunks: i32,  // e.g. 16 chunks
    pub unload_padding_chunks: i32,   // e.g. 2 chunks (unload at R + 2 = 18)
    pub total_simulated_frames: usize,// e.g. 600 frames (10 seconds at 60Hz)
    pub fixed_tick_step_sec: f64,     // e.g. 1.0 / 60.0
    pub max_chunk_budget_per_frame: usize, // Simulates main thread work dispatch budget
}

impl Default for MovementStressConfig {
    fn default() -> Self {
        Self {
            velocity_blocks_per_sec: 100.0,
            render_distance_chunks: 16,
            unload_padding_chunks: 2,
            total_simulated_frames: 600,
            fixed_tick_step_sec: 1.0 / 60.0,
            max_chunk_budget_per_frame: 16,
        }
    }
}

/// Simulates a player moving at high speed along the linear +X axis,
/// measuring individual frame execution times, streaming chunks ahead,
/// unloading chunks behind, and computing 1% Low and 0.1% Low frame times.
pub fn execute_high_speed_movement_stress_test(
    generator: &EngineWorldGenerator,
    config: &MovementStressConfig,
) -> BenchmarkStatistics {
    println!("--------------------------------------------------------------------------------");
    println!(" [TEST 2] High-Speed Movement Stress Test (100 blocks/sec, R = 16 Chunks)");
    println!("--------------------------------------------------------------------------------");

    // Connect your World/Chunk struct here:
    let mut world = EngineWorldState::new();

    let r = config.render_distance_chunks;
    let unload_r = r + config.unload_padding_chunks;

    // --------------------------------------------------------------------------
    // 1. WARMUP / SPAWN INITIALIZATION PHASE
    // --------------------------------------------------------------------------
    // Pre-populate the initial (2R+1) × (2R+1) chunk grid around spawn (0, 0)
    // so cold initialization allocations do NOT pollute active movement timings.
    print!(" -> WARMUP: Pre-populating spawn area ({0}x{0} = {1} chunks)... ", 2 * r + 1, (2 * r + 1) * (2 * r + 1));
    let warmup_start = Instant::now();

    for cx in -r..=r {
        for cz in -r..=r {
            let chunk = generator.generate_chunk(cx, cz);
            let mesh = EngineChunkMesh::mesh_chunk(&chunk);
            world.insert_chunk(chunk, mesh);
        }
    }
    println!("Done ({:.2} ms).", warmup_start.elapsed().as_secs_f64() * 1000.0);

    // --------------------------------------------------------------------------
    // 2. ACTIVE HIGH-SPEED MOVEMENT FRAME LOOP
    // --------------------------------------------------------------------------
    let mut player_x = 0.0f64;
    let player_z = 0.0f64;
    let mut frame_durations_ms = Vec::with_capacity(config.total_simulated_frames);
    let mut stutters_over_16ms = 0usize;
    let mut stutters_over_33ms = 0usize;
    let mut stutters_over_50ms = 0usize;

    println!(" -> Commencing simulation: {} frames at {} blocks/sec...", config.total_simulated_frames, config.velocity_blocks_per_sec);

    for _frame in 0..config.total_simulated_frames {
        let frame_start = Instant::now();

        // A. Advance Player Physics along linear X axis
        player_x += config.velocity_blocks_per_sec * config.fixed_tick_step_sec;
        let player_cx = (player_x / 16.0).floor() as i32;
        let player_cz = (player_z / 16.0).floor() as i32;

        // B. Identify Chunks Within Render Distance (R = 16)
        let min_cx = player_cx - r;
        let max_cx = player_cx + r;
        let min_cz = player_cz - r;
        let max_cz = player_cz + r;

        let mut chunks_to_load = Vec::new();
        for cx in min_cx..=max_cx {
            for cz in min_cz..=max_cz {
                if !world.loaded_chunks.contains_key(&(cx, cz)) {
                    chunks_to_load.push((cx, cz));
                }
            }
        }

        // C. Dynamic Chunk Loading & Meshing ahead of player
        // Budget-controlled dispatch simulates asynchronous background worker absorption
        let load_batch_size = chunks_to_load.len().min(config.max_chunk_budget_per_frame);
        for &(cx, cz) in &chunks_to_load[..load_batch_size] {
            // Connect your World/Chunk struct here:
            let chunk = generator.generate_chunk(cx, cz);
            let mesh = EngineChunkMesh::mesh_chunk(&chunk);
            black_box(&mesh);
            world.insert_chunk(chunk, mesh);
        }

        // D. Dynamic Chunk Unloading behind player (outside unload radius R + 2)
        let mut chunks_to_unload = Vec::new();
        for &(cx, cz) in world.loaded_chunks.keys() {
            let dist_x = (cx - player_cx).abs();
            let dist_z = (cz - player_cz).abs();
            if dist_x > unload_r || dist_z > unload_r {
                chunks_to_unload.push((cx, cz));
            }
        }
        for pos in chunks_to_unload {
            world.unload_chunk(&pos);
        }

        // E. Simulate Render Pipeline Pass (Frustum check, uniform updates & draw calls)
        black_box(&world.active_meshes.len());

        let frame_elapsed = frame_start.elapsed();
        let frame_ms = frame_elapsed.as_secs_f64() * 1000.0;
        frame_durations_ms.push(frame_ms);

        // Track Micro-Stutter Thresholds
        if frame_ms > 50.0 {
            stutters_over_50ms += 1;
        } else if frame_ms > 33.33 {
            stutters_over_33ms += 1;
        } else if frame_ms > 16.67 {
            stutters_over_16ms += 1;
        }
    }

    println!(" -> Traversed: {:.1} blocks ({:.1} chunks) along X axis.", player_x, player_x / 16.0);
    println!(" -> Active Chunks Maintained in Memory: {}", world.loaded_chunks.len());
    println!(" -> Frame Budget Violations: >16.7ms: {}, >33.3ms: {}, >50.0ms: {}",
        stutters_over_16ms, stutters_over_33ms, stutters_over_50ms);

    let stats = BenchmarkStatistics::compute(frame_durations_ms);
    stats.print_report("High-Speed 100 bps Movement Stress (Frame Times & Spikes)");
    stats
}

// ==============================================================================
// CRITERION BENCHMARK HARNESS INTEGRATION
// ==============================================================================

fn criterion_benchmarks(c: &mut Criterion) {
    let generator = Arc::new(EngineWorldGenerator::new(123456789));

    // --------------------------------------------------------------------------
    // Criterion Benchmark 1: Static Chunk Batch Generation Scaling
    // --------------------------------------------------------------------------
    let mut group = c.benchmark_group("worldgen_batch_generation");
    group.sample_size(10); // Use 10 high-precision samples for large matrices
    group.warm_up_time(Duration::from_secs(2));

    for &size in &[16, 32, 64] {
        let total_chunks = size * size;
        let generator_ref = Arc::clone(&generator);

        group.bench_with_input(
            BenchmarkId::new("matrix_chunks_parallel", total_chunks),
            &size,
            |b, &matrix_size| {
                b.iter(|| {
                    let (chunks, _) = run_static_batch_generation(&generator_ref, matrix_size, true);
                    black_box(chunks);
                });
            },
        );
    }
    group.finish();

    // --------------------------------------------------------------------------
    // Criterion Benchmark 2: High-Speed Movement Frame Loop
    // --------------------------------------------------------------------------
    let mut move_group = c.benchmark_group("high_speed_movement_stress");
    move_group.sample_size(10);
    move_group.warm_up_time(Duration::from_secs(2));

    let config = MovementStressConfig {
        velocity_blocks_per_sec: 100.0,
        render_distance_chunks: 16,
        unload_padding_chunks: 2,
        total_simulated_frames: 120, // 2 seconds per criterion iteration
        fixed_tick_step_sec: 1.0 / 60.0,
        max_chunk_budget_per_frame: 16,
    };

    let generator_ref = Arc::clone(&generator);
    move_group.bench_function("100bps_movement_frame_loop", |b| {
        b.iter(|| {
            let stats = execute_high_speed_movement_stress_test(&generator_ref, &config);
            black_box(stats);
        });
    });
    move_group.finish();

    // Run the full 128x128 statistical test directly once for verbose telemetry output
    execute_128x128_matrix_statistical_benchmark(&generator, 3, 1);
}

criterion_group!(benches, criterion_benchmarks);
criterion_main!(benches);
