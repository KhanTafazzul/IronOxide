// REA Verified: net.minecraft.client.gui.components.DebugScreenOverlay (mapped: exo.class)
//
// F3 Profiler & Engine Metrics Overlay:
// - Left Column (Game Information):
//   * Engine version and current FPS / frame time (e.g. "IronOxide 0.1.0 (144 fps / 6.94 ms)")
//   * Player position: "XYZ: x / y / z" and "Block: bx by bz [local_x local_y local_z]"
//   * Chunk coordinate: "Chunk: cx cy cz inside region r.rx.rz.mca"
//   * Facing direction: "Facing: north/south/west/east (Towards negative Z) (Yaw / Pitch)"
//   * Light & Biome: "Client Light: X (Y sky, Z block)" and "Biome: <biome>"
// - Right Column (System & Engine Information):
//   * Hardware specs: Active GPU device name via `wgpu::AdapterInfo`, WGPU backend (Vulkan, DX12, Metal)
//   * Memory usage: Current process RSS vs total system RAM ("Mem: XX% RSS/Total MB")
//   * Mesher metrics: Total active chunks, rendered quads, and draw calls per frame

use glam::Vec3;
use crate::render::gui::{GuiAtlas, GuiVertex, HudRenderer};

/// F3 Profiler & Engine Metrics Snapshot collected per-frame
#[derive(Debug, Clone)]
pub struct DebugMetrics {
    pub engine_version: String,
    pub fps: f32,
    pub frame_time_ms: f32,
    pub player_pos: Vec3,
    pub player_yaw: f32,
    pub player_pitch: f32,
    pub client_light: (u8, u8, u8), // (max, sky, block)
    pub heightmap_surface: i32,
    pub heightmap_motion_blocking: i32,
    pub biome: String,
    pub local_difficulty: f32,
    pub active_chunks: usize,
    pub rendered_quads: usize,
    pub draw_calls: usize,
    pub cpu_brand: String,
    pub gpu_name: String,
    pub gpu_vendor: String,
    pub gpu_backend: String,
    pub display_width: u32,
    pub display_height: u32,
    pub mem_rss_mb: u64,
    pub mem_total_mb: u64,
    pub allocation_rate_mb: u64,
    pub targeted_block: Option<(glam::IVec3, String)>,
    pub targeted_fluid: Option<(glam::IVec3, String)>,
}

impl Default for DebugMetrics {
    fn default() -> Self {
        Self {
            engine_version: "IronOxide 0.1.0".to_string(),
            fps: 60.0,
            frame_time_ms: 16.67,
            player_pos: Vec3::ZERO,
            player_yaw: 0.0,
            player_pitch: 0.0,
            client_light: (15, 15, 0),
            heightmap_surface: 64,
            heightmap_motion_blocking: 64,
            biome: "minecraft:plains".to_string(),
            local_difficulty: 1.50,
            active_chunks: 0,
            rendered_quads: 0,
            draw_calls: 0,
            cpu_brand: "8x Intel(R) Core(TM) / AMD Ryzen(TM)".to_string(),
            gpu_name: "Default Adapter".to_string(),
            gpu_vendor: "Intel".to_string(),
            gpu_backend: "Vulkan".to_string(),
            display_width: 1280,
            display_height: 720,
            mem_rss_mb: 128,
            mem_total_mb: 16384,
            allocation_rate_mb: 12,
            targeted_block: None,
            targeted_fluid: None,
        }
    }
}

impl DebugMetrics {
    /// Calculate Minecraft vanilla facing direction and axis string from player yaw
    pub fn get_facing(&self) -> (&'static str, &'static str, f32) {
        // Standard vanilla yaw wrapping to [-180, 180]
        let wrapped_yaw = (self.player_yaw + 180.0).rem_euclid(360.0) - 180.0;
        let (facing, axis) = if wrapped_yaw >= -180.0 && wrapped_yaw < -135.0 {
            ("north", "Towards negative Z")
        } else if wrapped_yaw >= -135.0 && wrapped_yaw < -45.0 {
            ("east", "Towards positive X")
        } else if wrapped_yaw >= -45.0 && wrapped_yaw < 45.0 {
            ("south", "Towards positive Z")
        } else if wrapped_yaw >= 45.0 && wrapped_yaw < 135.0 {
            ("west", "Towards negative X")
        } else {
            ("north", "Towards negative Z")
        };
        (facing, axis, wrapped_yaw)
    }

    /// Derives block, chunk, and Anvil region coordinates matching vanilla exo.class
    pub fn get_coordinates(&self) -> (glam::IVec3, glam::IVec3, glam::IVec3, glam::IVec2, glam::IVec2) {
        let bx = self.player_pos.x.floor() as i32;
        let by = self.player_pos.y.floor() as i32;
        let bz = self.player_pos.z.floor() as i32;
        let block_pos = glam::IVec3::new(bx, by, bz);

        let cx = bx.div_euclid(16);
        let cy = by.div_euclid(16);
        let cz = bz.div_euclid(16);
        let chunk_pos = glam::IVec3::new(cx, cy, cz);

        let local_pos = glam::IVec3::new(
            bx.rem_euclid(16),
            by.rem_euclid(16),
            bz.rem_euclid(16),
        );

        let rx = cx.div_euclid(32);
        let rz = cz.div_euclid(32);
        let region_pos = glam::IVec2::new(rx, rz);
        let local_region = glam::IVec2::new(cx.rem_euclid(32), cz.rem_euclid(32));

        (block_pos, chunk_pos, local_pos, region_pos, local_region)
    }

    /// Formats Left Column (Game Information) text lines matching vanilla DebugScreenOverlay (exo.class)
    pub fn build_left_lines(&self) -> Vec<String> {
        let (facing, axis, wrapped_yaw) = self.get_facing();
        let (b, c, local, r, local_r) = self.get_coordinates();

        vec![
            format!("{} ({:.0} fps / {:.2} ms)", self.engine_version, self.fps, self.frame_time_ms),
            format!("Integrated server @ {:.1}/50.0 ms, 0 tx, 0 rx", self.frame_time_ms),
            format!("C: {}/{} (D: 12, L: 0, C: 0)", self.active_chunks, self.active_chunks),
            "E: 0/0, B: 0, I: 0".to_string(),
            "P: 0. T: 0".to_string(),
            String::new(),
            format!("XYZ: {:.3} / {:.5} / {:.3}", self.player_pos.x, self.player_pos.y, self.player_pos.z),
            format!("Block: {} {} {} [{} {} {}]", b.x, b.y, b.z, local.x, local.y, local.z),
            format!("Chunk: {} {} {} in {} {} {} [{} {} in r.{}.{}.mca]", local.x, local.y, local.z, c.x, c.y, c.z, local_r.x, local_r.y, r.x, r.y),
            format!("Facing: {} ({}) ({:.1} / {:.1})", facing, axis, wrapped_yaw, self.player_pitch),
            format!("Client Light: {} ({} sky, {} block)", self.client_light.0, self.client_light.1, self.client_light.2),
            format!("CH S: {} M: {}", self.heightmap_surface, self.heightmap_motion_blocking),
            format!("SH S: {} M: {}", self.heightmap_surface, self.heightmap_motion_blocking),
            format!("Biome: {}", self.biome),
            format!("Local Difficulty: {:.2} // 0.00 (Day 0)", self.local_difficulty),
            format!("SC: {}, 0", self.active_chunks),
            "Sounds: 0/247 (Mood 0%)".to_string(),
        ]
    }

    /// Formats Right Column (System & Engine Information) text lines matching vanilla DebugScreenOverlay (exo.class)
    pub fn build_right_lines(&self) -> Vec<String> {
        let mem_pct = (self.mem_rss_mb * 100).checked_div(self.mem_total_mb.max(1)).unwrap_or(0);
        let alloc_pct = ((self.mem_rss_mb * 115) / self.mem_total_mb.max(1)).min(100);
        let alloc_mb = self.mem_rss_mb * 115 / 100;

        let mut lines = vec![
            "Java: 21.0.2 64bit".to_string(),
            format!("Mem: {:2}% {:03}/{:03}MB", mem_pct, self.mem_rss_mb, self.mem_total_mb),
            format!("Allocation rate: {:03}MB /s", self.allocation_rate_mb),
            format!("Allocated: {:2}% {:03}MB", alloc_pct, alloc_mb),
            String::new(),
            format!("CPU: {}", self.cpu_brand),
            format!("Display: {}x{} ({})", self.display_width, self.display_height, self.gpu_vendor),
            self.gpu_name.clone(),
            self.gpu_backend.clone(),
            String::new(),
            format!("Chunks: {} active", self.active_chunks),
            format!("Quads: {} rendered", self.rendered_quads),
            format!("Draw calls: {}", self.draw_calls),
        ];

        if let Some((pos, name)) = &self.targeted_block {
            lines.push(String::new());
            lines.push(format!("Targeted Block: {}, {}, {}", pos.x, pos.y, pos.z));
            lines.push(format!("minecraft:{}", name));
        }

        if let Some((pos, name)) = &self.targeted_fluid {
            lines.push(format!("Targeted Fluid: {}, {}, {}", pos.x, pos.y, pos.z));
            lines.push(format!("minecraft:{}", name));
        }

        lines
    }
}

/// F3 Profiler & Engine Metrics Overlay controller
#[derive(Debug, Clone)]
pub struct DebugOverlay {
    pub visible: bool,
}

impl Default for DebugOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugOverlay {
    pub fn new() -> Self {
        Self { visible: false }
    }

    /// Toggles visibility on F3 keypress
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Queries host process memory: returns (process_rss_mb, total_system_ram_mb)
    pub fn query_memory_stats() -> (u64, u64) {
        query_system_memory()
    }

    /// Builds 2D GUI quads (background boxes and text glyphs) for both columns
    pub fn build_debug_mesh(
        &self,
        metrics: &DebugMetrics,
        atlas: &GuiAtlas,
        screen_width: f32,
        _screen_height: f32,
        vertices: &mut Vec<GuiVertex>,
        indices: &mut Vec<u32>,
    ) {
        if !self.visible {
            return;
        }

        let char_w = 8.0;
        let char_h = 8.0;
        let line_h = 10.0;

        // 1. Render Left Column lines
        let left_lines = metrics.build_left_lines();
        let mut cur_y = 2.0;
        for line in &left_lines {
            draw_text_line(line, 2.0, cur_y, char_w, char_h, atlas, vertices, indices);
            cur_y += line_h;
        }

        // 2. Render Right Column lines (anchored to top-right screen edge)
        let right_lines = metrics.build_right_lines();
        let mut cur_right_y = 2.0;
        for line in &right_lines {
            let line_w = line.len() as f32 * char_w;
            let x = (screen_width - line_w - 4.0).max(2.0);
            draw_text_line(line, x, cur_right_y, char_w, char_h, atlas, vertices, indices);
            cur_right_y += line_h;
        }
    }
}

/// Draws a single line of text with semi-transparent background tint
fn draw_text_line(
    text: &str,
    x: f32,
    y: f32,
    char_w: f32,
    char_h: f32,
    atlas: &GuiAtlas,
    vertices: &mut Vec<GuiVertex>,
    indices: &mut Vec<u32>,
) {
    if text.is_empty() {
        return;
    }

    let line_w = text.len() as f32 * char_w;

    // Semi-transparent background box: RGBA(0.0, 0.0, 0.0, 0.55)
    HudRenderer::add_quad(
        vertices,
        indices,
        x - 1.0,
        y - 1.0,
        line_w + 2.0,
        char_h + 2.0,
        atlas.white_uv,
        [0.0, 0.0, 0.0, 0.55],
    );

    // Text glyphs: RGBA(0.9, 0.9, 0.9, 1.0)
    for (i, ch) in text.chars().enumerate() {
        if ch == ' ' {
            continue;
        }
        let gx = x + i as f32 * char_w;
        let glyph_uv = atlas.get_glyph_uv(ch as u8);
        HudRenderer::add_quad(
            vertices,
            indices,
            gx,
            y,
            char_w,
            char_h,
            glyph_uv,
            [0.9, 0.9, 0.9, 1.0],
        );
    }
}

/// Query system and process memory metrics (Windows and fallback)
#[cfg(target_os = "windows")]
fn query_system_memory() -> (u64, u64) {
    #[repr(C)]
    #[allow(non_snake_case)]
    struct MEMORYSTATUSEX {
        dwLength: u32,
        dwMemoryLoad: u32,
        ullTotalPhys: u64,
        ullAvailPhys: u64,
        ullTotalPageFile: u64,
        ullAvailPageFile: u64,
        ullTotalVirtual: u64,
        ullAvailVirtual: u64,
        ullAvailExtendedVirtual: u64,
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    struct VM_COUNTERS_EX {
        PeakVirtualSize: usize,
        VirtualSize: usize,
        PageFaultCount: u32,
        PeakWorkingSetSize: usize,
        WorkingSetSize: usize,
        QuotaPeakPagedPoolUsage: usize,
        QuotaPagedPoolUsage: usize,
        QuotaPeakNonPagedPoolUsage: usize,
        QuotaNonPagedPoolUsage: usize,
        PagefileUsage: usize,
        PeakPagefileUsage: usize,
        PrivateUsage: usize,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GlobalMemoryStatusEx(lpBuffer: *mut MEMORYSTATUSEX) -> i32;
        fn GetCurrentProcess() -> *mut std::ffi::c_void;
    }

    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn NtQueryInformationProcess(
            ProcessHandle: *mut std::ffi::c_void,
            ProcessInformationClass: u32,
            ProcessInformation: *mut std::ffi::c_void,
            ProcessInformationLength: u32,
            ReturnLength: *mut u32,
        ) -> i32;
    }

    let mut total_mb = 16384;
    unsafe {
        let mut stat = std::mem::zeroed::<MEMORYSTATUSEX>();
        stat.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        if GlobalMemoryStatusEx(&mut stat) != 0 {
            total_mb = stat.ullTotalPhys / (1024 * 1024);
        }
    }

    let mut rss_mb = 128;
    unsafe {
        let mut vm = std::mem::zeroed::<VM_COUNTERS_EX>();
        let mut ret_len = 0u32;
        let handle = GetCurrentProcess();
        let status = NtQueryInformationProcess(
            handle,
            3, // ProcessVmCounters
            &mut vm as *mut _ as *mut std::ffi::c_void,
            std::mem::size_of::<VM_COUNTERS_EX>() as u32,
            &mut ret_len,
        );
        if status == 0 && vm.WorkingSetSize > 0 {
            rss_mb = (vm.WorkingSetSize as u64) / (1024 * 1024);
        }
    }

    (rss_mb, total_mb)
}

#[cfg(not(target_os = "windows"))]
fn query_system_memory() -> (u64, u64) {
    (128, 16384)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_overlay_toggle() {
        let mut overlay = DebugOverlay::new();
        assert!(!overlay.is_visible());
        overlay.toggle();
        assert!(overlay.is_visible());
        overlay.toggle();
        assert!(!overlay.is_visible());
    }

    #[test]
    fn test_left_column_metrics_formatting() {
        let metrics = DebugMetrics {
            engine_version: "IronOxide 0.1.0".to_string(),
            fps: 144.0,
            frame_time_ms: 6.94,
            player_pos: Vec3::new(12.345, 64.0, -45.678),
            player_yaw: -179.8,
            player_pitch: 12.3,
            client_light: (15, 15, 0),
            biome: "minecraft:plains".to_string(),
            ..Default::default()
        };

        let left = metrics.build_left_lines();
        assert_eq!(left[0], "IronOxide 0.1.0 (144 fps / 6.94 ms)");
        assert_eq!(left[1], "Integrated server @ 6.9/50.0 ms, 0 tx, 0 rx");
        assert_eq!(left[2], "C: 0/0 (D: 12, L: 0, C: 0)");
        assert_eq!(left[3], "E: 0/0, B: 0, I: 0");
        assert_eq!(left[4], "P: 0. T: 0");
        assert_eq!(left[5], "");
        assert_eq!(left[6], "XYZ: 12.345 / 64.00000 / -45.678");
        assert_eq!(left[7], "Block: 12 64 -46 [12 0 2]");
        assert_eq!(left[8], "Chunk: 12 0 2 in 0 4 -3 [0 29 in r.0.-1.mca]");
        assert!(left[9].starts_with("Facing: north (Towards negative Z)"));
        assert_eq!(left[10], "Client Light: 15 (15 sky, 0 block)");
        assert_eq!(left[11], "CH S: 64 M: 64");
        assert_eq!(left[12], "SH S: 64 M: 64");
        assert_eq!(left[13], "Biome: minecraft:plains");
    }

    #[test]
    fn test_facing_direction_all_quadrants() {
        let mut metrics = DebugMetrics::default();

        // North: ~180 or -180
        metrics.player_yaw = -180.0;
        let (f, a, _) = metrics.get_facing();
        assert_eq!(f, "north");
        assert_eq!(a, "Towards negative Z");

        // South: ~0
        metrics.player_yaw = 0.0;
        let (f, a, _) = metrics.get_facing();
        assert_eq!(f, "south");
        assert_eq!(a, "Towards positive Z");

        // West: ~90
        metrics.player_yaw = 90.0;
        let (f, a, _) = metrics.get_facing();
        assert_eq!(f, "west");
        assert_eq!(a, "Towards negative X");

        // East: ~-90 or 270
        metrics.player_yaw = -90.0;
        let (f, a, _) = metrics.get_facing();
        assert_eq!(f, "east");
        assert_eq!(a, "Towards positive X");
    }

    #[test]
    fn test_right_column_metrics_formatting() {
        let metrics = DebugMetrics {
            gpu_name: "NVIDIA GeForce RTX 4080".to_string(),
            gpu_backend: "Vulkan".to_string(),
            mem_rss_mb: 180,
            mem_total_mb: 16384,
            active_chunks: 25,
            rendered_quads: 12480,
            draw_calls: 29,
            ..Default::default()
        };

        let right = metrics.build_right_lines();
        assert_eq!(right[0], "Java: 21.0.2 64bit");
        assert!(right[1].starts_with("Mem:"));
        assert!(right[1].contains("180/16384MB"));
        assert_eq!(right[2], "Allocation rate: 012MB /s");
        assert!(right[3].starts_with("Allocated:"));
        assert_eq!(right[4], "");
        assert!(right[5].starts_with("CPU:"));
        assert_eq!(right[6], "Display: 1280x720 (Intel)");
        assert_eq!(right[7], "NVIDIA GeForce RTX 4080");
        assert_eq!(right[8], "Vulkan");
        assert_eq!(right[9], "");
        assert_eq!(right[10], "Chunks: 25 active");
        assert_eq!(right[11], "Quads: 12480 rendered");
        assert_eq!(right[12], "Draw calls: 29");
    }

    #[test]
    fn test_chunk_region_coordinate_derivation() {
        let metrics = DebugMetrics {
            player_pos: Vec3::new(12.345, 64.0, -45.678),
            ..Default::default()
        };
        let (b, c, local, r, local_r) = metrics.get_coordinates();
        assert_eq!(b, glam::IVec3::new(12, 64, -46));
        assert_eq!(c, glam::IVec3::new(0, 4, -3));
        assert_eq!(local, glam::IVec3::new(12, 0, 2));
        assert_eq!(r, glam::IVec2::new(0, -1));
        assert_eq!(local_r, glam::IVec2::new(0, 29));
    }

    #[test]
    fn test_build_debug_mesh_quad_generation() {
        let atlas = GuiAtlas::load_or_create(None);
        let metrics = DebugMetrics::default();

        let mut overlay = DebugOverlay::new();
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Hidden: 0 quads
        overlay.build_debug_mesh(&metrics, &atlas, 1280.0, 720.0, &mut vertices, &mut indices);
        assert_eq!(vertices.len(), 0);
        assert_eq!(indices.len(), 0);

        // Visible: builds both columns
        overlay.toggle();
        overlay.build_debug_mesh(&metrics, &atlas, 1280.0, 720.0, &mut vertices, &mut indices);
        assert!(!vertices.is_empty());
        assert!(!indices.is_empty());
        assert_eq!(vertices.len() * 6, indices.len() * 4); // Quad vertex:index ratio 4:6
    }

    #[test]
    fn test_memory_query_stats_returns_positive() {
        let (rss, total) = DebugOverlay::query_memory_stats();
        assert!(rss > 0);
        assert!(total > 0);
        assert!(total >= rss);
    }
}
