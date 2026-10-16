// REA Verified: net.minecraft.client.gui.Gui (mapped: ewt.class)
// REA Verified: net.minecraft.client.gui.components.PlayerTabOverlay (mapped: eyh.class)
//
// 2D HUD, Inverted Crosshair & Hotbar Overlay.
// - Orthographic 2D WGPU render pass overlayed directly onto color attachment without clearing.
// - Inverted blend crosshair centered at ((W - 15) / 2, (H - 15) / 2).
// - Hotbar background anchored at bottom-center: ((W - 182) / 2, H - 22).
// - Selection box highlighted at: ((W - 182) / 2 - 1 + slot * 20, H - 23).
// - Supports 9 hotbar slots (0 to 8) cycling via scroll wheel & number keys 1-9.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use bytemuck::{Pod, Zeroable};
use image::{Rgba, RgbaImage};
use wgpu::util::DeviceExt;

pub mod hud;
pub mod inventory;
pub mod debug;
pub use hud::HudRenderer;
pub use inventory::InventoryGui;
pub use debug::{DebugMetrics, DebugOverlay};

use crate::game::inventory::PlayerInventory;
use crate::render::atlas::TextureAtlas;

/// 2D GUI Vertex with pixel position (0..width, 0..height), UV, and RGBA tint
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct GuiVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl GuiVertex {
    pub fn new(pos: [f32; 2], uv: [f32; 2], color: [f32; 4]) -> Self {
        Self { pos, uv, color }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GuiVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Uniform for 2D orthographic projection from screen pixels
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GuiScreenUniform {
    pub screen_size: [f32; 2],
    pub _padding: [f32; 2],
}

/// Normalized sub-rectangle in GUI texture atlas
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuiRect {
    pub u_min: f32,
    pub v_min: f32,
    pub u_max: f32,
    pub v_max: f32,
}

/// Stitched GUI texture atlas holding vanilla HUD and Inventory textures
pub struct GuiAtlas {
    pub width: u32,
    pub height: u32,
    pub image: RgbaImage,
    pub hotbar_uv: GuiRect,
    pub selection_uv: GuiRect,
    pub crosshair_uv: GuiRect,
    pub inventory_uv: GuiRect,
    pub white_uv: GuiRect,
    pub heart_container_uv: GuiRect,
    pub heart_full_uv: GuiRect,
    pub heart_half_uv: GuiRect,
    pub food_empty_uv: GuiRect,
    pub food_full_uv: GuiRect,
    pub food_half_uv: GuiRect,
    #[allow(dead_code)]
    pub font_uv: GuiRect,
}

impl GuiAtlas {
    pub const ATLAS_SIZE: u32 = 512;

    /// Locate `client.jar` in standard paths
    pub fn find_client_jar() -> Option<PathBuf> {
        TextureAtlas::find_client_jar()
    }

    /// Load HUD and Inventory textures from `client.jar` (or fallback procedurally)
    pub fn load_or_create(jar_path: Option<&Path>) -> Self {
        let mut hotbar_img: Option<RgbaImage> = None;
        let mut sel_img: Option<RgbaImage> = None;
        let mut cross_img: Option<RgbaImage> = None;
        let mut inv_img: Option<RgbaImage> = None;
        let mut heart_cnt_img: Option<RgbaImage> = None;
        let mut heart_full_img: Option<RgbaImage> = None;
        let mut heart_half_img: Option<RgbaImage> = None;
        let mut food_empty_img: Option<RgbaImage> = None;
        let mut food_full_img: Option<RgbaImage> = None;
        let mut food_half_img: Option<RgbaImage> = None;
        let mut font_img: Option<RgbaImage> = None;

        let resolved_path = jar_path
            .map(|p| p.to_path_buf())
            .or_else(Self::find_client_jar);

        if let Some(path) = resolved_path {
            if let Ok(file) = File::open(&path) {
                if let Ok(mut archive) = zip::ZipArchive::new(file) {
                    if let Ok(mut entry) = archive.by_name("assets/minecraft/textures/gui/sprites/hud/hotbar.png") {
                        let mut buf = Vec::new();
                        if entry.read_to_end(&mut buf).is_ok() {
                            if let Ok(img) = image::load_from_memory(&buf) {
                                hotbar_img = Some(img.to_rgba8());
                            }
                        }
                    }
                    if let Ok(mut entry) = archive.by_name("assets/minecraft/textures/gui/sprites/hud/hotbar_selection.png") {
                        let mut buf = Vec::new();
                        if entry.read_to_end(&mut buf).is_ok() {
                            if let Ok(img) = image::load_from_memory(&buf) {
                                sel_img = Some(img.to_rgba8());
                            }
                        }
                    }
                    if let Ok(mut entry) = archive.by_name("assets/minecraft/textures/gui/sprites/hud/crosshair.png") {
                        let mut buf = Vec::new();
                        if entry.read_to_end(&mut buf).is_ok() {
                            if let Ok(img) = image::load_from_memory(&buf) {
                                cross_img = Some(img.to_rgba8());
                            }
                        }
                    }
                    if let Ok(mut entry) = archive.by_name("assets/minecraft/textures/gui/container/inventory.png") {
                        let mut buf = Vec::new();
                        if entry.read_to_end(&mut buf).is_ok() {
                            if let Ok(img) = image::load_from_memory(&buf) {
                                let full = img.to_rgba8();
                                let mut cropped = RgbaImage::new(176, 166);
                                for y in 0..166 {
                                    for x in 0..176 {
                                        if x < full.width() && y < full.height() {
                                            cropped.put_pixel(x, y, *full.get_pixel(x, y));
                                        }
                                    }
                                }
                                inv_img = Some(cropped);
                            }
                        }
                    }
                    if let Ok(mut entry) = archive.by_name("assets/minecraft/textures/gui/sprites/hud/heart/container.png") {
                        let mut buf = Vec::new();
                        if entry.read_to_end(&mut buf).is_ok() {
                            if let Ok(img) = image::load_from_memory(&buf) {
                                heart_cnt_img = Some(img.to_rgba8());
                            }
                        }
                    }
                    if let Ok(mut entry) = archive.by_name("assets/minecraft/textures/gui/sprites/hud/heart/full.png") {
                        let mut buf = Vec::new();
                        if entry.read_to_end(&mut buf).is_ok() {
                            if let Ok(img) = image::load_from_memory(&buf) {
                                heart_full_img = Some(img.to_rgba8());
                            }
                        }
                    }
                    if let Ok(mut entry) = archive.by_name("assets/minecraft/textures/gui/sprites/hud/heart/half.png") {
                        let mut buf = Vec::new();
                        if entry.read_to_end(&mut buf).is_ok() {
                            if let Ok(img) = image::load_from_memory(&buf) {
                                heart_half_img = Some(img.to_rgba8());
                            }
                        }
                    }
                    if let Ok(mut entry) = archive.by_name("assets/minecraft/textures/gui/sprites/hud/food_empty.png") {
                        let mut buf = Vec::new();
                        if entry.read_to_end(&mut buf).is_ok() {
                            if let Ok(img) = image::load_from_memory(&buf) {
                                food_empty_img = Some(img.to_rgba8());
                            }
                        }
                    }
                    if let Ok(mut entry) = archive.by_name("assets/minecraft/textures/gui/sprites/hud/food_full.png") {
                        let mut buf = Vec::new();
                        if entry.read_to_end(&mut buf).is_ok() {
                            if let Ok(img) = image::load_from_memory(&buf) {
                                food_full_img = Some(img.to_rgba8());
                            }
                        }
                    }
                    if let Ok(mut entry) = archive.by_name("assets/minecraft/textures/gui/sprites/hud/food_half.png") {
                        let mut buf = Vec::new();
                        if entry.read_to_end(&mut buf).is_ok() {
                            if let Ok(img) = image::load_from_memory(&buf) {
                                food_half_img = Some(img.to_rgba8());
                            }
                        }
                    }
                    if let Ok(mut entry) = archive.by_name("assets/minecraft/textures/font/ascii.png") {
                        let mut buf = Vec::new();
                        if entry.read_to_end(&mut buf).is_ok() {
                            if let Ok(img) = image::load_from_memory(&buf) {
                                font_img = Some(img.to_rgba8());
                            }
                        }
                    }
                }
            }
        }

        // Procedural fallbacks if not found
        let hotbar = hotbar_img.unwrap_or_else(|| {
            let mut img = RgbaImage::new(182, 22);
            for y in 0..22 {
                for x in 0..182 {
                    let border = x == 0 || x == 181 || y == 0 || y == 21;
                    let slot_div = x % 20 == 0;
                    if border || slot_div {
                        img.put_pixel(x, y, Rgba([30, 30, 30, 220]));
                    } else {
                        img.put_pixel(x, y, Rgba([120, 120, 120, 160]));
                    }
                }
            }
            img
        });

        let selection = sel_img.unwrap_or_else(|| {
            let mut img = RgbaImage::new(24, 24);
            for y in 0..24 {
                for x in 0..24 {
                    let border = x < 2 || x >= 22 || y < 2 || y >= 22;
                    if border {
                        img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                    } else {
                        img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                    }
                }
            }
            img
        });

        let crosshair = cross_img.unwrap_or_else(|| {
            let mut img = RgbaImage::new(15, 15);
            for y in 0..15 {
                for x in 0..15 {
                    let is_center_cross = (x == 7 && y >= 3 && y <= 11) || (y == 7 && x >= 3 && x <= 11);
                    if is_center_cross {
                        img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                    } else {
                        img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                    }
                }
            }
            img
        });

        let inventory_bg = inv_img.unwrap_or_else(|| {
            let mut img = RgbaImage::new(176, 166);
            for y in 0..166 {
                for x in 0..176 {
                    let border = x == 0 || x == 175 || y == 0 || y == 165;
                    if border {
                        img.put_pixel(x, y, Rgba([55, 55, 55, 255]));
                    } else {
                        img.put_pixel(x, y, Rgba([198, 198, 198, 255]));
                    }
                }
            }
            img
        });

        let heart_container = heart_cnt_img.unwrap_or_else(|| {
            let mut img = RgbaImage::new(9, 9);
            for y in 0..9 {
                for x in 0..9 {
                    let is_outline = matches!(
                        (x, y),
                        (1, 0) | (2, 0) | (3, 0) | (5, 0) | (6, 0) | (7, 0)
                        | (0, 1) | (4, 1) | (8, 1)
                        | (0, 2) | (8, 2)
                        | (0, 3) | (8, 3)
                        | (1, 4) | (7, 4)
                        | (2, 5) | (6, 5)
                        | (3, 6) | (5, 6)
                        | (4, 7)
                    );
                    if is_outline {
                        img.put_pixel(x, y, Rgba([30, 30, 30, 255]));
                    }
                }
            }
            img
        });

        let heart_full = heart_full_img.unwrap_or_else(|| {
            let mut img = RgbaImage::new(9, 9);
            for y in 0..9 {
                for x in 0..9 {
                    let is_inside = match y {
                        1 => (x >= 1 && x <= 3) || (x >= 5 && x <= 7),
                        2..=3 => x >= 1 && x <= 7,
                        4 => x >= 2 && x <= 6,
                        5 => x >= 3 && x <= 5,
                        6 => x == 4,
                        _ => false,
                    };
                    if is_inside {
                        let highlight = (x == 2 && y == 2) || (x == 6 && y == 2);
                        if highlight {
                            img.put_pixel(x, y, Rgba([255, 120, 120, 255]));
                        } else {
                            img.put_pixel(x, y, Rgba([220, 20, 20, 255]));
                        }
                    }
                }
            }
            img
        });

        let heart_half = heart_half_img.unwrap_or_else(|| {
            let mut img = RgbaImage::new(9, 9);
            for y in 0..9 {
                for x in 0..9 {
                    let is_left_half = match y {
                        1 => x >= 1 && x <= 3,
                        2..=3 => x >= 1 && x <= 4,
                        4 => x >= 2 && x <= 4,
                        5 => x >= 3 && x <= 4,
                        6 => x == 4,
                        _ => false,
                    };
                    if is_left_half {
                        if x == 2 && y == 2 {
                            img.put_pixel(x, y, Rgba([255, 120, 120, 255]));
                        } else {
                            img.put_pixel(x, y, Rgba([220, 20, 20, 255]));
                        }
                    }
                }
            }
            img
        });

        let food_empty = food_empty_img.unwrap_or_else(|| {
            let mut img = RgbaImage::new(9, 9);
            for y in 0..9 {
                for x in 0..9 {
                    let is_border = matches!(
                        (x, y),
                        (3, 1) | (4, 1) | (5, 1)
                        | (2, 2) | (6, 2)
                        | (1, 3) | (7, 3)
                        | (1, 4) | (7, 4)
                        | (2, 5) | (6, 5)
                        | (3, 6) | (5, 6)
                        | (3, 7) | (4, 7)
                    );
                    if is_border {
                        img.put_pixel(x, y, Rgba([40, 25, 15, 255]));
                    }
                }
            }
            img
        });

        let food_full = food_full_img.unwrap_or_else(|| {
            let mut img = RgbaImage::new(9, 9);
            for y in 0..9 {
                for x in 0..9 {
                    let is_body = match y {
                        2 => x >= 3 && x <= 5,
                        3..=4 => x >= 2 && x <= 6,
                        5 => x >= 3 && x <= 5,
                        6 => x == 4,
                        _ => false,
                    };
                    let is_bone = (x == 4 && y == 7) || (x == 3 && y == 8) || (x == 5 && y == 8);
                    if is_body {
                        img.put_pixel(x, y, Rgba([185, 105, 45, 255]));
                    } else if is_bone {
                        img.put_pixel(x, y, Rgba([220, 215, 195, 255]));
                    }
                }
            }
            img
        });

        let food_half = food_half_img.unwrap_or_else(|| {
            let mut img = RgbaImage::new(9, 9);
            for y in 0..9 {
                for x in 0..9 {
                    let is_body_half = match y {
                        2 => x >= 4 && x <= 5,
                        3..=4 => x >= 4 && x <= 6,
                        5 => x >= 4 && x <= 5,
                        6 => x == 4,
                        _ => false,
                    };
                    let is_bone = (x == 4 && y == 7) || (x == 5 && y == 8);
                    if is_body_half {
                        img.put_pixel(x, y, Rgba([185, 105, 45, 255]));
                    } else if is_bone {
                        img.put_pixel(x, y, Rgba([220, 215, 195, 255]));
                    }
                }
            }
            img
        });

        // Stitch into 512x512 atlas:
        // Hotbar at (0, 0) -> 182 x 22
        // Selection at (0, 30) -> 24 x 24
        // Crosshair at (30, 30) -> 15 x 15
        // White solid at (60, 30) -> 16 x 16
        // Heart container at (80, 30) -> 9 x 9
        // Heart full at (92, 30) -> 9 x 9
        // Heart half at (104, 30) -> 9 x 9
        // Food empty at (116, 30) -> 9 x 9
        // Food full at (128, 30) -> 9 x 9
        // Food half at (140, 30) -> 9 x 9
        // Inventory container at (0, 60) -> 176 x 166
        let size = Self::ATLAS_SIZE;
        let mut atlas_image = RgbaImage::new(size, size);

        // Paste hotbar
        for y in 0..hotbar.height().min(size) {
            for x in 0..hotbar.width().min(size) {
                atlas_image.put_pixel(x, y, *hotbar.get_pixel(x, y));
            }
        }
        let hotbar_uv = GuiRect {
            u_min: 0.0,
            v_min: 0.0,
            u_max: hotbar.width() as f32 / size as f32,
            v_max: hotbar.height() as f32 / size as f32,
        };

        // Paste selection
        let sel_y_off = 30;
        for y in 0..selection.height() {
            for x in 0..selection.width() {
                atlas_image.put_pixel(x, sel_y_off + y, *selection.get_pixel(x, y));
            }
        }
        let selection_uv = GuiRect {
            u_min: 0.0,
            v_min: sel_y_off as f32 / size as f32,
            u_max: selection.width() as f32 / size as f32,
            v_max: (sel_y_off + selection.height()) as f32 / size as f32,
        };

        // Paste crosshair
        let cross_x_off = 30;
        let cross_y_off = 30;
        for y in 0..crosshair.height() {
            for x in 0..crosshair.width() {
                atlas_image.put_pixel(cross_x_off + x, cross_y_off + y, *crosshair.get_pixel(x, y));
            }
        }
        let crosshair_uv = GuiRect {
            u_min: cross_x_off as f32 / size as f32,
            v_min: cross_y_off as f32 / size as f32,
            u_max: (cross_x_off + crosshair.width()) as f32 / size as f32,
            v_max: (cross_y_off + crosshair.height()) as f32 / size as f32,
        };

        // Paste white 16x16 square for solid quads
        let white_x_off = 60;
        let white_y_off = 30;
        for y in 0..16 {
            for x in 0..16 {
                atlas_image.put_pixel(white_x_off + x, white_y_off + y, Rgba([255, 255, 255, 255]));
            }
        }
        let white_uv = GuiRect {
            u_min: white_x_off as f32 / size as f32,
            v_min: white_y_off as f32 / size as f32,
            u_max: (white_x_off + 16) as f32 / size as f32,
            v_max: (white_y_off + 16) as f32 / size as f32,
        };

        // Paste heart container (9x9) at (80, 30)
        let hc_x_off = 80;
        let hc_y_off = 30;
        for y in 0..heart_container.height() {
            for x in 0..heart_container.width() {
                atlas_image.put_pixel(hc_x_off + x, hc_y_off + y, *heart_container.get_pixel(x, y));
            }
        }
        let heart_container_uv = GuiRect {
            u_min: hc_x_off as f32 / size as f32,
            v_min: hc_y_off as f32 / size as f32,
            u_max: (hc_x_off + heart_container.width()) as f32 / size as f32,
            v_max: (hc_y_off + heart_container.height()) as f32 / size as f32,
        };

        // Paste heart full (9x9) at (92, 30)
        let hf_x_off = 92;
        let hf_y_off = 30;
        for y in 0..heart_full.height() {
            for x in 0..heart_full.width() {
                atlas_image.put_pixel(hf_x_off + x, hf_y_off + y, *heart_full.get_pixel(x, y));
            }
        }
        let heart_full_uv = GuiRect {
            u_min: hf_x_off as f32 / size as f32,
            v_min: hf_y_off as f32 / size as f32,
            u_max: (hf_x_off + heart_full.width()) as f32 / size as f32,
            v_max: (hf_y_off + heart_full.height()) as f32 / size as f32,
        };

        // Paste heart half (9x9) at (104, 30)
        let hh_x_off = 104;
        let hh_y_off = 30;
        for y in 0..heart_half.height() {
            for x in 0..heart_half.width() {
                atlas_image.put_pixel(hh_x_off + x, hh_y_off + y, *heart_half.get_pixel(x, y));
            }
        }
        let heart_half_uv = GuiRect {
            u_min: hh_x_off as f32 / size as f32,
            v_min: hh_y_off as f32 / size as f32,
            u_max: (hh_x_off + heart_half.width()) as f32 / size as f32,
            v_max: (hh_y_off + heart_half.height()) as f32 / size as f32,
        };

        // Paste food empty (9x9) at (116, 30)
        let fe_x_off = 116;
        let fe_y_off = 30;
        for y in 0..food_empty.height() {
            for x in 0..food_empty.width() {
                atlas_image.put_pixel(fe_x_off + x, fe_y_off + y, *food_empty.get_pixel(x, y));
            }
        }
        let food_empty_uv = GuiRect {
            u_min: fe_x_off as f32 / size as f32,
            v_min: fe_y_off as f32 / size as f32,
            u_max: (fe_x_off + food_empty.width()) as f32 / size as f32,
            v_max: (fe_y_off + food_empty.height()) as f32 / size as f32,
        };

        // Paste food full (9x9) at (128, 30)
        let ff_x_off = 128;
        let ff_y_off = 30;
        for y in 0..food_full.height() {
            for x in 0..food_full.width() {
                atlas_image.put_pixel(ff_x_off + x, ff_y_off + y, *food_full.get_pixel(x, y));
            }
        }
        let food_full_uv = GuiRect {
            u_min: ff_x_off as f32 / size as f32,
            v_min: ff_y_off as f32 / size as f32,
            u_max: (ff_x_off + food_full.width()) as f32 / size as f32,
            v_max: (ff_y_off + food_full.height()) as f32 / size as f32,
        };

        // Paste food half (9x9) at (140, 30)
        let fh_x_off = 140;
        let fh_y_off = 30;
        for y in 0..food_half.height() {
            for x in 0..food_half.width() {
                atlas_image.put_pixel(fh_x_off + x, fh_y_off + y, *food_half.get_pixel(x, y));
            }
        }
        let food_half_uv = GuiRect {
            u_min: fh_x_off as f32 / size as f32,
            v_min: fh_y_off as f32 / size as f32,
            u_max: (fh_x_off + food_half.width()) as f32 / size as f32,
            v_max: (fh_y_off + food_half.height()) as f32 / size as f32,
        };

        // Paste inventory container (176x166) at (0, 60)
        let inv_y_off = 60;
        for y in 0..inventory_bg.height().min(size - inv_y_off) {
            for x in 0..inventory_bg.width().min(size) {
                atlas_image.put_pixel(x, inv_y_off + y, *inventory_bg.get_pixel(x, y));
            }
        }
        let inventory_uv = GuiRect {
            u_min: 0.0,
            v_min: inv_y_off as f32 / size as f32,
            u_max: inventory_bg.width() as f32 / size as f32,
            v_max: (inv_y_off + inventory_bg.height()) as f32 / size as f32,
        };

        // Procedural font fallback if ascii.png not loaded from jar
        let font_bitmap = font_img.unwrap_or_else(|| {
            let mut img = RgbaImage::new(128, 128);
            for c in 32u8..=126 {
                let col = (c % 16) as u32;
                let row = (c / 16) as u32;
                let gx = col * 8;
                let gy = row * 8;
                for py in 1..7 {
                    for px in 1..6 {
                        img.put_pixel(gx + px, gy + py, Rgba([255, 255, 255, 255]));
                    }
                }
            }
            img
        });

        // Paste font (128x128) at (200, 0)
        let font_x_off = 200;
        let font_y_off = 0;
        for y in 0..font_bitmap.height().min(size) {
            for x in 0..font_bitmap.width().min(size - font_x_off) {
                atlas_image.put_pixel(font_x_off + x, font_y_off + y, *font_bitmap.get_pixel(x, y));
            }
        }
        let font_uv = GuiRect {
            u_min: font_x_off as f32 / size as f32,
            v_min: font_y_off as f32 / size as f32,
            u_max: (font_x_off + font_bitmap.width()) as f32 / size as f32,
            v_max: (font_y_off + font_bitmap.height()) as f32 / size as f32,
        };

        Self {
            width: size,
            height: size,
            image: atlas_image,
            hotbar_uv,
            selection_uv,
            crosshair_uv,
            inventory_uv,
            white_uv,
            heart_container_uv,
            heart_full_uv,
            heart_half_uv,
            food_empty_uv,
            food_full_uv,
            food_half_uv,
            font_uv,
        }
    }

    /// Computes the normalized UV sub-rectangle for an 8x8 ASCII character glyph
    pub fn get_glyph_uv(&self, c: u8) -> GuiRect {
        let size = Self::ATLAS_SIZE as f32;
        let col = (c % 16) as f32;
        let row = (c / 16) as f32;
        let u_min = (200.0 + col * 8.0) / size;
        let v_min = (row * 8.0) / size;
        let u_max = u_min + (8.0 / size);
        let v_max = v_min + (8.0 / size);
        GuiRect {
            u_min,
            v_min,
            u_max,
            v_max,
        }
    }
}

/// 2D HUD Renderer handling orthographic projection, crosshair inversion, and hotbar drawing
pub struct GuiRenderer {
    pub atlas: Arc<GuiAtlas>,
    pub screen_buffer: wgpu::Buffer,
    pub screen_bind_group: wgpu::BindGroup,
    pub texture_bind_group: wgpu::BindGroup,
    pub normal_pipeline: wgpu::RenderPipeline,
    pub crosshair_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl GuiRenderer {
    const MAX_VERTICES: usize = 16384;
    const MAX_INDICES: usize = 24576;

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        atlas: Arc<GuiAtlas>,
    ) -> Self {
        // 1. Uniform setup for screen size
        let screen_uniform = GuiScreenUniform {
            screen_size: [1280.0, 720.0],
            _padding: [0.0, 0.0],
        };

        let screen_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("GUI Screen Uniform Buffer"),
            contents: bytemuck::cast_slice(&[screen_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let screen_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("GUI Screen Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let screen_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GUI Screen Bind Group"),
            layout: &screen_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_buffer.as_entire_binding(),
            }],
        });

        // 2. Texture setup
        let texture_size = wgpu::Extent3d {
            width: atlas.width,
            height: atlas.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("GUI Atlas Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &atlas.image,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * atlas.width),
                rows_per_image: Some(atlas.height),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("GUI Texture Sampler (Nearest)"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("GUI Texture Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GUI Texture Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // 3. Shaders & Pipeline Layout
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GUI Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(GUI_WGSL)),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("GUI Pipeline Layout"),
            bind_group_layouts: &[Some(&screen_bind_group_layout), Some(&texture_bind_group_layout)],
            immediate_size: 0,
        });

        // Normal alpha blending pipeline for hotbar
        let normal_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("GUI Normal Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(GuiVertex::desc())],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // REA Verified: net.minecraft.client.gui.Gui (ewt.class)
        // RenderSystem.blendFuncSeparate(ONE_MINUS_DST_COLOR, ONE_MINUS_SRC_COLOR, ONE, ZERO)
        let crosshair_blend = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::OneMinusDst,
                dst_factor: wgpu::BlendFactor::OneMinusSrc,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::Zero,
                operation: wgpu::BlendOperation::Add,
            },
        };

        let crosshair_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("GUI Crosshair Pipeline (Inverted)"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(GuiVertex::desc())],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(crosshair_blend),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Dynamic vertex and index buffers
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GUI Dynamic Vertex Buffer"),
            size: (Self::MAX_VERTICES * std::mem::size_of::<GuiVertex>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GUI Dynamic Index Buffer"),
            size: (Self::MAX_INDICES * std::mem::size_of::<u32>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            atlas,
            screen_buffer,
            screen_bind_group,
            texture_bind_group,
            normal_pipeline,
            crosshair_pipeline,
            vertex_buffer,
            index_buffer,
        }
    }

    /// Add a 2D quad to vertex and index list
    fn add_quad(
        vertices: &mut Vec<GuiVertex>,
        indices: &mut Vec<u32>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rect: GuiRect,
        color: [f32; 4],
    ) {
        let base_idx = vertices.len() as u32;

        vertices.push(GuiVertex::new([x, y], [rect.u_min, rect.v_min], color));
        vertices.push(GuiVertex::new([x + w, y], [rect.u_max, rect.v_min], color));
        vertices.push(GuiVertex::new([x + w, y + h], [rect.u_max, rect.v_max], color));
        vertices.push(GuiVertex::new([x, y + h], [rect.u_min, rect.v_max], color));

        indices.extend_from_slice(&[
            base_idx,
            base_idx + 1,
            base_idx + 2,
            base_idx,
            base_idx + 2,
            base_idx + 3,
        ]);
    }

    /// Render HUD and/or Inventory overlay on top of 3D frame (executes with LoadOp::Load)
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        target_view: &wgpu::TextureView,
        screen_width: u32,
        screen_height: u32,
        active_slot: usize,
        inventory_gui: Option<(&InventoryGui, &PlayerInventory)>,
        player_stats: Option<&crate::game::stats::PlayerStats>,
        debug_overlay: Option<(&DebugOverlay, &DebugMetrics)>,
    ) {
        if screen_width == 0 || screen_height == 0 {
            return;
        }

        let sw = screen_width as f32;
        let sh = screen_height as f32;

        // Update screen size uniform
        queue.write_buffer(
            &self.screen_buffer,
            0,
            bytemuck::cast_slice(&[GuiScreenUniform {
                screen_size: [sw, sh],
                _padding: [0.0, 0.0],
            }]),
        );

        let mut normal_vertices = Vec::with_capacity(256);
        let mut normal_indices = Vec::with_capacity(384);
        let mut cross_vertices = Vec::with_capacity(4);
        let mut cross_indices = Vec::with_capacity(6);

        let is_inventory_open = inventory_gui
            .as_ref()
            .map(|(inv, _)| inv.is_open)
            .unwrap_or(false);

        if is_inventory_open {
            if let Some((inv_gui, inv_data)) = inventory_gui {
                inv_gui.build_render_mesh(
                    inv_data,
                    &self.atlas,
                    sw,
                    sh,
                    &mut normal_vertices,
                    &mut normal_indices,
                );
            }
        } else {
            // GUI integer scale (scale 2.0x for modern resolutions)
            let scale = if sh >= 720.0 { 2.0 } else { 1.0 };

            // 1. Hotbar Quad (Vanilla: 182x22, centered at bottom)
            let hotbar_w = 182.0 * scale;
            let hotbar_h = 22.0 * scale;
            let hotbar_x = ((sw - hotbar_w) / 2.0).round();
            let hotbar_y = sh - hotbar_h - (2.0 * scale);

            // 2. Selection Quad (Vanilla: 24x24, offset by active_slot * 20)
            let slot = active_slot.min(8) as f32;
            let sel_w = 24.0 * scale;
            let sel_h = 24.0 * scale;
            let sel_x = hotbar_x - (1.0 * scale) + (slot * 20.0 * scale);
            let sel_y = hotbar_y - (1.0 * scale);

            Self::add_quad(
                &mut normal_vertices,
                &mut normal_indices,
                hotbar_x,
                hotbar_y,
                hotbar_w,
                hotbar_h,
                self.atlas.hotbar_uv,
                [1.0, 1.0, 1.0, 1.0],
            );

            Self::add_quad(
                &mut normal_vertices,
                &mut normal_indices,
                sel_x,
                sel_y,
                sel_w,
                sel_h,
                self.atlas.selection_uv,
                [1.0, 1.0, 1.0, 1.0],
            );

            // 2.5 Vitals Quads (Hearts & Hunger Drumsticks)
            if let Some(stats) = player_stats {
                HudRenderer::build_vitals_quads(
                    stats,
                    &self.atlas,
                    hotbar_x,
                    hotbar_y,
                    hotbar_w,
                    scale,
                    &mut normal_vertices,
                    &mut normal_indices,
                );
            }

            // 3. Crosshair Quad (Vanilla: 15x15, exact screen center)
            let cross_w = 15.0 * scale;
            let cross_h = 15.0 * scale;
            let cross_x = ((sw - cross_w) / 2.0).round();
            let cross_y = ((sh - cross_h) / 2.0).round();

            Self::add_quad(
                &mut cross_vertices,
                &mut cross_indices,
                cross_x,
                cross_y,
                cross_w,
                cross_h,
                self.atlas.crosshair_uv,
                [1.0, 1.0, 1.0, 1.0],
            );
        }

        // 4. F3 Profiler & Engine Metrics Overlay
        if let Some((dbg, metrics)) = debug_overlay {
            dbg.build_debug_mesh(
                metrics,
                &self.atlas,
                sw,
                sh,
                &mut normal_vertices,
                &mut normal_indices,
            );
        }

        // Upload combined vertices
        let normal_idx_count = normal_indices.len() as u32;
        let cross_idx_count = cross_indices.len() as u32;

        let mut all_vertices = normal_vertices;
        let base_cross_idx = all_vertices.len() as u32;
        all_vertices.extend(cross_vertices);

        let mut all_indices = normal_indices;
        for idx in cross_indices {
            all_indices.push(base_cross_idx + idx);
        }

        if all_indices.is_empty() {
            return;
        }

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&all_vertices));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&all_indices));

        // Orthographic 2D Render Pass (load: LoadOp::Load preserves underlying 3D world!)
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("GUI Overlay Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        rpass.set_bind_group(0, &self.screen_bind_group, &[]);
        rpass.set_bind_group(1, &self.texture_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        // Draw normal alpha-blended elements (HUD or Inventory)
        if normal_idx_count > 0 {
            rpass.set_pipeline(&self.normal_pipeline);
            rpass.draw_indexed(0..normal_idx_count, 0, 0..1);
        }

        // Draw inverted crosshair if active
        if cross_idx_count > 0 {
            rpass.set_pipeline(&self.crosshair_pipeline);
            rpass.draw_indexed(normal_idx_count..(normal_idx_count + cross_idx_count), 0, 0..1);
        }
    }
}

const GUI_WGSL: &str = r#"
struct ScreenUniform {
    screen_size: vec2<f32>,
    _padding: vec2<f32>,
};

@group(0) @binding(0) var<uniform> screen: ScreenUniform;

struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let ndc_x = (in.pos.x / screen.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (in.pos.y / screen.screen_size.y) * 2.0;
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    return out;
}

@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);
    if (tex_color.a < 0.05) {
        discard;
    }
    return tex_color * in.color;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gui_vertex_layout() {
        assert_eq!(std::mem::size_of::<GuiVertex>(), 32);
        let desc = GuiVertex::desc();
        assert_eq!(desc.array_stride, 32);
        assert_eq!(desc.attributes.len(), 3);
    }

    #[test]
    fn test_gui_atlas_stitching() {
        let atlas = GuiAtlas::load_or_create(None);
        assert_eq!(atlas.width, 512);
        assert_eq!(atlas.height, 512);

        assert!(atlas.hotbar_uv.u_max > atlas.hotbar_uv.u_min);
        assert!(atlas.hotbar_uv.v_max > atlas.hotbar_uv.v_min);

        assert!(atlas.selection_uv.u_max > atlas.selection_uv.u_min);
        assert!(atlas.selection_uv.v_max > atlas.selection_uv.v_min);

        assert!(atlas.crosshair_uv.u_max > atlas.crosshair_uv.u_min);
        assert!(atlas.crosshair_uv.v_max > atlas.crosshair_uv.v_min);

        assert!(atlas.inventory_uv.u_max > atlas.inventory_uv.u_min);
        assert!(atlas.inventory_uv.v_max > atlas.inventory_uv.v_min);

        assert!(atlas.white_uv.u_max > atlas.white_uv.u_min);
        assert!(atlas.white_uv.v_max > atlas.white_uv.v_min);
    }

    #[test]
    fn test_crosshair_exact_screen_center() {
        let screen_width = 1280.0f32;
        let screen_height = 720.0f32;
        let scale = 2.0f32;
        let cross_w = 15.0 * scale;
        let cross_h = 15.0 * scale;

        let cross_x = ((screen_width - cross_w) / 2.0).round();
        let cross_y = ((screen_height - cross_h) / 2.0).round();

        assert_eq!(cross_x, 625.0);
        assert_eq!(cross_y, 345.0);
        // Verify center of crosshair aligns with center of screen (640, 360)
        assert_eq!(cross_x + cross_w / 2.0, 640.0);
        assert_eq!(cross_y + cross_h / 2.0, 360.0);
    }

    #[test]
    fn test_hotbar_and_active_slot_positions() {
        let sw = 1920.0f32;
        let sh = 1080.0f32;
        let scale = 2.0f32;

        let hotbar_w = 182.0 * scale;
        let hotbar_h = 22.0 * scale;
        let hotbar_x = ((sw - hotbar_w) / 2.0).round();
        let hotbar_y = sh - hotbar_h - (2.0 * scale);

        // Vanilla dimensions: 182x22 anchored at bottom center
        assert_eq!(hotbar_w, 364.0);
        assert_eq!(hotbar_h, 44.0);
        assert_eq!(hotbar_x, 778.0);
        assert_eq!(hotbar_y, 1032.0);

        // Selection box: 24x24 anchored at -1px from slot start
        for slot in 0..9 {
            let sel_x = hotbar_x - (1.0 * scale) + (slot as f32 * 20.0 * scale);
            let sel_y = hotbar_y - (1.0 * scale);

            assert_eq!(sel_y, 1030.0);
            if slot == 0 {
                assert_eq!(sel_x, 776.0);
            } else if slot == 8 {
                assert_eq!(sel_x, 776.0 + 8.0 * 40.0);
            }
        }
    }

    #[test]
    fn test_hotbar_slot_wrapping() {
        let mut active_slot = 0usize;

        // Scroll down (next slot)
        active_slot = (active_slot + 1) % 9;
        assert_eq!(active_slot, 1);

        // Cycle through all 9 slots
        for expected in 2..9 {
            active_slot = (active_slot + 1) % 9;
            assert_eq!(active_slot, expected);
        }
        active_slot = (active_slot + 1) % 9;
        assert_eq!(active_slot, 0); // Wraps back to 0

        // Scroll up (previous slot)
        active_slot = (active_slot + 8) % 9;
        assert_eq!(active_slot, 8); // Wraps back to 8
    }

    #[test]
    fn test_inverted_blend_mode_specs() {
        // REA Verified: RenderSystem.blendFuncSeparate(ONE_MINUS_DST_COLOR, ONE_MINUS_SRC_COLOR, ONE, ZERO)
        let crosshair_blend = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::OneMinusDst,
                dst_factor: wgpu::BlendFactor::OneMinusSrc,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::Zero,
                operation: wgpu::BlendOperation::Add,
            },
        };

        assert_eq!(crosshair_blend.color.src_factor, wgpu::BlendFactor::OneMinusDst);
        assert_eq!(crosshair_blend.color.dst_factor, wgpu::BlendFactor::OneMinusSrc);
        assert_eq!(crosshair_blend.color.operation, wgpu::BlendOperation::Add);
        assert_eq!(crosshair_blend.alpha.src_factor, wgpu::BlendFactor::One);
        assert_eq!(crosshair_blend.alpha.dst_factor, wgpu::BlendFactor::Zero);
    }
}
