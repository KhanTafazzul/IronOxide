// REA Verified: net.minecraft.client.renderer.texture.TextureManager (mapped: gem.class)
// REA Verified: net.minecraft.client.renderer.entity.LivingEntityRenderer (mapped: gen.class)
//
// Entity Texture Atlas:
// - Loads skin textures from `assets/minecraft/textures/entity/**/*.png` or `client.jar`.
// - Dynamically packs multiple mob skins into a single high-performance GPU Texture Atlas.
// - Provides authentic procedural fallback skins for all mobs when disk/jar assets are absent.
// - Exposes normalized atlas sub-rect UVs: [u_min, v_min, u_max, v_max].

use std::collections::HashMap;
use std::path::Path;
use image::{Rgba, RgbaImage};

/// Supported mob skin keys in the Entity Texture Atlas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntitySkin {
    Zombie,
    Pig,
    Cow,
    Sheep,
    Creeper,
    Breeze,
    Creaking,
    Bat,
    Parrot,
    Ghast,
}

impl EntitySkin {
    pub fn default_path(&self) -> &'static str {
        match self {
            EntitySkin::Zombie => "assets/minecraft/textures/entity/zombie/zombie.png",
            EntitySkin::Pig => "assets/minecraft/textures/entity/pig/pig.png",
            EntitySkin::Cow => "assets/minecraft/textures/entity/cow/cow.png",
            EntitySkin::Sheep => "assets/minecraft/textures/entity/sheep/sheep.png",
            EntitySkin::Creeper => "assets/minecraft/textures/entity/creeper/creeper.png",
            EntitySkin::Breeze => "assets/minecraft/textures/entity/breeze/breeze.png",
            EntitySkin::Creaking => "assets/minecraft/textures/entity/creaking/creaking.png",
            EntitySkin::Bat => "assets/minecraft/textures/entity/bat.png",
            EntitySkin::Parrot => "assets/minecraft/textures/entity/parrot/parrot_red_blue.png",
            EntitySkin::Ghast => "assets/minecraft/textures/entity/ghast/ghast.png",
        }
    }
}

/// Entity Texture Atlas holding packed mob skins
pub struct EntityTextureAtlas {
    pub width: u32,
    pub height: u32,
    pub raw_image: RgbaImage,
    pub skin_rects: HashMap<EntitySkin, [f32; 4]>,
}

impl EntityTextureAtlas {
    /// Builds or loads the entity texture atlas from assets directory or jar
    pub fn load_or_create(assets_dir: Option<&Path>, jar_path: Option<&Path>) -> Self {
        // Collect skins
        let skins = [
            EntitySkin::Zombie,
            EntitySkin::Pig,
            EntitySkin::Cow,
            EntitySkin::Sheep,
            EntitySkin::Creeper,
            EntitySkin::Breeze,
            EntitySkin::Creaking,
            EntitySkin::Bat,
            EntitySkin::Parrot,
            EntitySkin::Ghast,
        ];

        let mut loaded_images: Vec<(EntitySkin, RgbaImage)> = Vec::new();

        for skin in &skins {
            let img = Self::load_skin(*skin, assets_dir, jar_path)
                .unwrap_or_else(|| Self::generate_fallback_skin(*skin));
            loaded_images.push((*skin, img));
        }

        // Pack images into a grid layout (e.g. 4 columns, dynamic rows)
        // Most entity skins are 64x64 or 64x32
        let cell_size = 64u32;
        let cols = 4u32;
        let rows = ((loaded_images.len() as u32) + cols - 1) / cols;

        let atlas_w = cols * cell_size;
        let atlas_h = rows * cell_size;

        let mut atlas_img = RgbaImage::from_pixel(atlas_w, atlas_h, Rgba([0, 0, 0, 0]));
        let mut skin_rects = HashMap::new();

        for (i, (skin, img)) in loaded_images.into_iter().enumerate() {
            let col = (i as u32) % cols;
            let row = (i as u32) / cols;

            let x_offset = col * cell_size;
            let y_offset = row * cell_size;

            // Copy pixels into atlas
            let src_w = img.width().min(cell_size);
            let src_h = img.height().min(cell_size);

            for y in 0..src_h {
                for x in 0..src_w {
                    let pixel = img.get_pixel(x, y);
                    atlas_img.put_pixel(x_offset + x, y_offset + y, *pixel);
                }
            }

            // UV rectangle in atlas: [u_min, v_min, u_max, v_max]
            let u_min = (x_offset as f32) / (atlas_w as f32);
            let v_min = (y_offset as f32) / (atlas_h as f32);
            let u_max = ((x_offset + src_w) as f32) / (atlas_w as f32);
            let v_max = ((y_offset + src_h) as f32) / (atlas_h as f32);

            skin_rects.insert(skin, [u_min, v_min, u_max, v_max]);
        }

        Self {
            width: atlas_w,
            height: atlas_h,
            raw_image: atlas_img,
            skin_rects,
        }
    }

    /// Retrieves UV bounds in the atlas for a specific mob skin
    pub fn get_skin_uv(&self, skin: EntitySkin) -> [f32; 4] {
        self.skin_rects
            .get(&skin)
            .copied()
            .unwrap_or([0.0, 0.0, 1.0, 1.0])
    }

    /// Attempts to load mob skin from disk or jar
    fn load_skin(
        skin: EntitySkin,
        assets_dir: Option<&Path>,
        jar_path: Option<&Path>,
    ) -> Option<RgbaImage> {
        let rel_path = skin.default_path();

        // 1. Try disk search
        let candidate_paths = [
            assets_dir.map(|d| d.join(rel_path)),
            Some(Path::new(rel_path).to_path_buf()),
            Some(Path::new("../").join(rel_path)),
            Some(Path::new("IronOxide").join(rel_path)),
        ];

        for opt in candidate_paths.into_iter().flatten() {
            if opt.exists() {
                if let Ok(img) = image::open(&opt) {
                    return Some(img.to_rgba8());
                }
            }
        }

        // 2. Try client.jar search
        let jar_candidates = [
            jar_path.map(|p| p.to_path_buf()),
            Some(Path::new("client.jar").to_path_buf()),
            Some(Path::new("../client.jar").to_path_buf()),
        ];

        for opt_jar in jar_candidates.into_iter().flatten() {
            if opt_jar.exists() {
                if let Ok(file) = std::fs::File::open(&opt_jar) {
                    if let Ok(mut archive) = zip::ZipArchive::new(file) {
                        if let Ok(mut entry) = archive.by_name(rel_path) {
                            use std::io::Read;
                            let mut buf = Vec::new();
                            if entry.read_to_end(&mut buf).is_ok() {
                                if let Ok(img) = image::load_from_memory(&buf) {
                                    return Some(img.to_rgba8());
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Procedurally generates authentic fallback textures for each mob
    pub fn generate_fallback_skin(skin: EntitySkin) -> RgbaImage {
        let (w, h, primary, secondary) = match skin {
            EntitySkin::Zombie => (64, 64, Rgba([79, 136, 61, 255]), Rgba([46, 120, 136, 255])),
            EntitySkin::Pig => (64, 32, Rgba([240, 162, 162, 255]), Rgba([220, 130, 130, 255])),
            EntitySkin::Cow => (64, 32, Rgba([96, 64, 32, 255]), Rgba([200, 200, 200, 255])),
            EntitySkin::Sheep => (64, 32, Rgba([230, 230, 230, 255]), Rgba([200, 180, 160, 255])),
            EntitySkin::Creeper => (64, 32, Rgba([16, 180, 32, 255]), Rgba([10, 100, 20, 255])),
            EntitySkin::Breeze => (32, 32, Rgba([140, 160, 240, 220]), Rgba([210, 230, 255, 180])),
            EntitySkin::Creaking => (64, 64, Rgba([180, 175, 160, 255]), Rgba([240, 140, 20, 255])),
            EntitySkin::Bat => (64, 64, Rgba([70, 50, 40, 255]), Rgba([40, 30, 25, 255])),
            EntitySkin::Parrot => (32, 32, Rgba([220, 30, 30, 255]), Rgba([30, 80, 220, 255])),
            EntitySkin::Ghast => (64, 64, Rgba([245, 245, 245, 255]), Rgba([180, 180, 180, 255])),
        };

        let mut img = RgbaImage::from_pixel(w, h, primary);

        // Add recognizable texture checker/patterns
        for y in 0..h {
            for x in 0..w {
                if (x / 4 + y / 4) % 2 == 1 {
                    img.put_pixel(x, y, secondary);
                }
            }
        }

        img
    }

    /// Uploads the packed atlas image into a WGPU texture, sampler, and bind group
    pub fn create_gpu_resources(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler, wgpu::BindGroup) {
        let size = wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Entity Texture Atlas"),
            size,
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
            &self.raw_image,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.width),
                rows_per_image: Some(self.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Entity Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Entity Atlas Bind Group"),
            layout: bind_group_layout,
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

        (texture, view, sampler, bind_group)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atlas_packing_and_uv_bounds() {
        let atlas = EntityTextureAtlas::load_or_create(None, None);

        assert!(atlas.width >= 128);
        assert!(atlas.height >= 128);

        // Verify UV bounds for all skins
        for skin in [
            EntitySkin::Zombie,
            EntitySkin::Pig,
            EntitySkin::Cow,
            EntitySkin::Creeper,
            EntitySkin::Breeze,
            EntitySkin::Ghast,
        ] {
            let uv = atlas.get_skin_uv(skin);
            assert!(uv[0] >= 0.0 && uv[0] <= 1.0);
            assert!(uv[1] >= 0.0 && uv[1] <= 1.0);
            assert!(uv[2] >= uv[0] && uv[2] <= 1.0);
            assert!(uv[3] >= uv[1] && uv[3] <= 1.0);
        }
    }
}
