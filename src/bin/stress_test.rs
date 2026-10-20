//! ==============================================================================
//! Standalone IronOxide Worldgen & Rendering Stress Testing Binary
//! ==============================================================================
//! Run directly via:
//!     cargo run --release --bin stress_test
//!
//! Executes:
//! 1. Static Batch Generation Test (128 × 128 = 16,384 chunks)
//! 2. High-Speed Movement Stress Test (100 blocks/s, R = 16 chunks, 1% and 0.1% Lows)
//! ==============================================================================

use rayon::prelude::*;
use std::collections::HashMap;
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

// ==============================================================================
// ENGINE STRUCT PLACEHOLDERS
// ==============================================================================

// Connect your World/Chunk struct here: Swap with your engine's chunk representation
#[derive(Clone)]
pub struct EngineChunkColumn {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub block_count: usize,
    // Connect your World/Chunk struct here: e.g. Box<[BlockType; 98304]>
    pub raw_data: Vec<u8>,
}

// Connect your World/Chunk struct here: Swap with your engine's procedural world generator
pub struct EngineWorldGenerator {
    pub seed: i64,
    internal_seed_mix: u64,
}

impl EngineWorldGenerator {
    pub fn new(seed: i64) -> Self {
        Self {
            seed,
            internal_seed_mix: (seed as u64) ^ 0x5DEECE66D,
        }
    }

    #[inline]
    pub fn generate_chunk(&self, cx: i32, cz: i32) -> EngineChunkColumn {
        // Multi-octave density hash simulation
        let mut h = (self.internal_seed_mix)
            .wrapping_add((cx as u64).wrapping_mul(0x9E3779B97F4A7C15))
            .wrapping_add((cz as u64).wrapping_mul(0xBF58476D1CE4E5B9));
        h ^= h >> 30;
        h = h.wrapping_mul(0xBF58476D1CE4E5B9);
        h ^= h >> 27;

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

// Connect your World/Chunk struct here: Swap with your engine's mesher and GPU vertex buffers
pub struct EngineChunkMesh {
    pub chunk_pos: (i32, i32),
    pub quad_count: usize,
    pub vertex_buffer_bytes: usize,
}

impl EngineChunkMesh {
    pub fn mesh_chunk(chunk: &EngineChunkColumn) -> Self {
        let quads = (chunk.raw_data.len() * 3) % 4096 + 512;
        Self {
            chunk_pos: (chunk.chunk_x, chunk.chunk_z),
            quad_count: quads,
            vertex_buffer_bytes: quads * 4 * 32,
        }
    }
}

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
// STATISTICAL METRICS ENGINE
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
        println!("\n╔══════════════════════════════════════════════════════════════════════════════╗");
        println!("║  BENCHMARK REPORT: {:<57} ║", title);
        println!("╠══════════════════════════════════════════════════════════════════════════════╣");
        println!("║  Total Samples:             {:<48} ║", self.total_samples);
        println!("║  Total Benchmark Duration:  {:<45.3} s ║", self.total_duration.as_secs_f64());
        println!("╟──────────────────────────────────────────────────────────────────────────────╢");
        println!("║  Mean Duration:             {:<10.3} ms  (Average: {:>6.2} FPS)                ║", self.mean_ms, self.avg_fps);
        println!("║  Median Duration (P50):     {:<48.3} ms ║", self.median_ms);
        println!("║  Standard Deviation (σ):    ±{:<47.3} ms ║", self.std_dev_ms);
        println!("║  Minimum (Fastest):         {:<48.3} ms ║", self.min_ms);
        println!("║  Maximum (Worst Spike):     {:<48.3} ms ║", self.max_ms);
        println!("╟──────────────────────────────────────────────────────────────────────────────╢");
        println!("║  90th Percentile (P90):     {:<48.3} ms ║", self.p90_ms);
        println!("║  99th Percentile (P99):     {:<10.3} ms  (1.0% Low: {:>6.2} FPS)               ║", self.p99_ms, self.one_percent_low_fps);
        println!("║  99.9th Percentile (P99.9): {:<10.3} ms  (0.1% Low: {:>6.2} FPS)               ║", self.p99_9_ms, self.zero_point_one_percent_low_fps);
        println!("╚══════════════════════════════════════════════════════════════════════════════╝\n");
    }
}

// ==============================================================================
// TEST 1: STATIC BATCH GENERATION (128 × 128 CHUNKS)
// ==============================================================================

pub fn run_static_batch_generation(
    generator: &EngineWorldGenerator,
    matrix_size: i32,
) -> (usize, Duration) {
    let total_chunks = (matrix_size * matrix_size) as usize;
    let chunks_processed = AtomicUsize::new(0);

    let start = Instant::now();

    (0..matrix_size).into_par_iter().for_each(|cx| {
        for cz in 0..matrix_size {
            // Procedural generation & meshing pass
            let chunk = generator.generate_chunk(cx, cz);
            let mesh = EngineChunkMesh::mesh_chunk(&chunk);
            black_box((&chunk, &mesh));
            chunks_processed.fetch_add(1, Ordering::Relaxed);
        }
    });

    let elapsed = start.elapsed();
    assert_eq!(chunks_processed.load(Ordering::SeqCst), total_chunks);
    (total_chunks, elapsed)
}

// ==============================================================================
// TEST 2: HIGH-SPEED MOVEMENT STRESS TEST
// ==============================================================================

pub fn run_high_speed_movement_test(
    generator: &EngineWorldGenerator,
    velocity_bps: f64,
    render_distance: i32,
    total_frames: usize,
) -> BenchmarkStatistics {
    let mut world = EngineWorldState::new();
    let r = render_distance;
    let unload_r = r + 2;
    let dt = 1.0 / 60.0;
    let max_budget_per_frame = 16;

    // Warmup: Spawn Area Initialization
    print!(" -> WARMUP: Pre-populating spawn area ({}x{} = {} chunks)... ", 2 * r + 1, 2 * r + 1, (2 * r + 1).pow(2));
    let warmup_start = Instant::now();
    for cx in -r..=r {
        for cz in -r..=r {
            let chunk = generator.generate_chunk(cx, cz);
            let mesh = EngineChunkMesh::mesh_chunk(&chunk);
            world.insert_chunk(chunk, mesh);
        }
    }
    println!("Done ({:.2} ms).", warmup_start.elapsed().as_secs_f64() * 1000.0);

    let mut player_x = 0.0f64;
    let mut frame_durations_ms = Vec::with_capacity(total_frames);
    let mut stutters_over_16ms = 0usize;
    let mut stutters_over_33ms = 0usize;
    let mut stutters_over_50ms = 0usize;

    println!(" -> Simulating {} frames at {:.1} blocks/sec along +X...", total_frames, velocity_bps);

    for _frame in 0..total_frames {
        let frame_start = Instant::now();

        // 1. Advance Player Position
        player_x += velocity_bps * dt;
        let player_cx = (player_x / 16.0).floor() as i32;
        let player_cz = 0;

        // 2. Query Missing Chunks Ahead
        let min_cx = player_cx - r;
        let max_cx = player_cx + r;
        let min_cz = player_cz - r;
        let max_cz = player_cz + r;

        let mut to_load = Vec::new();
        for cx in min_cx..=max_cx {
            for cz in min_cz..=max_cz {
                if !world.loaded_chunks.contains_key(&(cx, cz)) {
                    to_load.push((cx, cz));
                }
            }
        }

        // 3. Dynamic Chunk Loading Ahead
        let batch_size = to_load.len().min(max_budget_per_frame);
        for &(cx, cz) in &to_load[..batch_size] {
            // Connect your World/Chunk struct here:
            let chunk = generator.generate_chunk(cx, cz);
            let mesh = EngineChunkMesh::mesh_chunk(&chunk);
            black_box(&mesh);
            world.insert_chunk(chunk, mesh);
        }

        // 4. Dynamic Chunk Unloading Behind
        let mut to_unload = Vec::new();
        for &(cx, cz) in world.loaded_chunks.keys() {
            if (cx - player_cx).abs() > unload_r || (cz - player_cz).abs() > unload_r {
                to_unload.push((cx, cz));
            }
        }
        for pos in to_unload {
            world.unload_chunk(&pos);
        }

        // 5. Render Pass Simulation
        black_box(world.active_meshes.len());

        let frame_ms = frame_start.elapsed().as_secs_f64() * 1000.0;
        frame_durations_ms.push(frame_ms);

        if frame_ms > 50.0 {
            stutters_over_50ms += 1;
        } else if frame_ms > 33.33 {
            stutters_over_33ms += 1;
        } else if frame_ms > 16.67 {
            stutters_over_16ms += 1;
        }
    }

    println!(" -> Traversed: {:.1} blocks ({:.1} chunks).", player_x, player_x / 16.0);
    println!(" -> Active Chunks Maintained in Memory: {}", world.loaded_chunks.len());
    println!(" -> Budget Violations: >16.7ms: {}, >33.3ms: {}, >50.0ms: {}",
        stutters_over_16ms, stutters_over_33ms, stutters_over_50ms);

    BenchmarkStatistics::compute(frame_durations_ms)
}

fn main() {
    println!("================================================================================");
    println!(" ⚡ IronOxide Worldgen & High-Speed Rendering Stress Testing Suite");
    println!("================================================================================");

    let generator = EngineWorldGenerator::new(123456789);

    // TEST 1: Static Batch Generation Test (128 × 128 = 16,384 Chunks)
    println!("\n[TEST 1] Static Batch Generation Test (128 × 128 Matrix = 16,384 Chunks)");
    print!(" -> WARMUP: Priming allocators with 16x16 grid... ");
    let _ = run_static_batch_generation(&generator, 16);
    println!("Done.");

    let iterations = 5;
    let mut batch_times = Vec::with_capacity(iterations);
    for i in 1..=iterations {
        print!(" -> Iteration {}/{} (16,384 Chunks)... ", i, iterations);
        let (chunks, elapsed) = run_static_batch_generation(&generator, 128);
        let ms = elapsed.as_secs_f64() * 1000.0;
        let throughput = chunks as f64 / elapsed.as_secs_f64();
        println!("{:.2} ms ({:.0} chunks/sec)", ms, throughput);
        batch_times.push(ms);
    }
    let batch_stats = BenchmarkStatistics::compute(batch_times);
    batch_stats.print_report("128 × 128 Static Batch Generation (16,384 Chunks)");

    // TEST 2: High-Speed Movement Stress Test
    println!("\n[TEST 2] High-Speed Movement Stress Test (100 blocks/sec, R = 16 Chunks)");
    let move_stats = run_high_speed_movement_test(&generator, 100.0, 16, 600);
    move_stats.print_report("High-Speed 100 bps Movement Stress (Frame Times & Spikes)");
}
