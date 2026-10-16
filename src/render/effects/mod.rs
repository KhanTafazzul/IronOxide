// REA Verified: net.minecraft.client.renderer.block.BlockModelShaper (mapped: fua.class)
// REA Verified: net.minecraft.client.particle.TerrainParticle (mapped: gbd.class)
//
// Block Hardness, Destroy Crack Overlay & Burst Particle Effects:
// - Renders 10 progressive destroy stages (destroy_stage_0 to destroy_stage_9).
// - Sub-pixel epsilon expansion ([x - eps, y - eps, z - eps] to [x + 1 + eps, y + 1 + eps, z + 1 + eps]) to prevent z-fighting.
// - Spawns 12-20 physics-affected voxel particles with random outward velocities and ground collisions.

use bytemuck::{Pod, Zeroable};
use glam::{IVec3, Vec3};

use crate::game::World;
use crate::render::atlas::{BlockType, TextureAtlas};

/// 3D Effect Vertex for crack overlays and particles
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct EffectVertex {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl EffectVertex {
    pub fn new(pos: [f32; 3], uv: [f32; 2], color: [f32; 4]) -> Self {
        Self { pos, uv, color }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<EffectVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 20,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// A physics-affected voxel particle spawned upon block destruction
#[derive(Debug, Clone)]
pub struct Particle {
    pub pos: Vec3,
    pub vel: Vec3,
    pub size: f32,
    pub color: [f32; 4],
    pub lifetime: f32,
    pub max_lifetime: f32,
}

impl Particle {
    pub fn new(pos: Vec3, vel: Vec3, size: f32, color: [f32; 4], max_lifetime: f32) -> Self {
        Self {
            pos,
            vel,
            size,
            color,
            lifetime: 0.0,
            max_lifetime,
        }
    }

    /// Ticks particle physics: gravity, air drag, and voxel floor collision
    pub fn tick(&mut self, dt: f32, world: &World) -> bool {
        self.lifetime += dt;
        if self.lifetime >= self.max_lifetime {
            return false; // Expired
        }

        // Gravity: -16.0 m/s^2 (Minecraft particle standard)
        self.vel.y -= 16.0 * dt;

        // Air drag: 0.98 per tick
        self.vel.x *= 0.98;
        self.vel.z *= 0.98;

        let next_pos = self.pos + self.vel * dt;

        // Voxel collision check against ground
        let block_coord = IVec3::new(
            next_pos.x.floor() as i32,
            next_pos.y.floor() as i32,
            next_pos.z.floor() as i32,
        );

        let solid = world.get_block(block_coord).is_opaque();
        if solid && next_pos.y <= (block_coord.y + 1) as f32 + 0.05 {
            // Bounce / land on block surface
            self.pos.y = (block_coord.y + 1) as f32 + self.size / 2.0;
            self.vel.y = (-self.vel.y * 0.3).max(0.0);
            self.vel.x *= 0.6;
            self.vel.z *= 0.6;
        } else {
            self.pos = next_pos;
        }

        true
    }
}

/// Particle System managing active block break particles
pub struct ParticleSystem {
    pub particles: Vec<Particle>,
    rng_state: u64,
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(128),
            rng_state: 88172645463325252,
        }
    }

    fn next_rand(&mut self) -> f32 {
        self.rng_state = self.rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((self.rng_state >> 33) as f32) / (u32::MAX as f32)
    }

    /// Spawns a burst of 12-20 physics-affected voxel particles with random outward velocities
    pub fn spawn_block_break(&mut self, pos: IVec3, block: BlockType) {
        if block == BlockType::Air {
            return;
        }

        let base_color = match block {
            BlockType::Dirt => [0.53, 0.37, 0.24, 1.0],
            BlockType::GrassBlock => [0.40, 0.62, 0.25, 1.0],
            BlockType::Stone => [0.55, 0.55, 0.55, 1.0],
            BlockType::Cobblestone => [0.46, 0.46, 0.46, 1.0],
            BlockType::Deepslate => [0.30, 0.30, 0.32, 1.0],
            BlockType::OakLog => [0.42, 0.30, 0.18, 1.0],
            BlockType::OakPlanks => [0.72, 0.58, 0.37, 1.0],
            BlockType::OakLeaves => [0.25, 0.55, 0.15, 0.9],
            BlockType::Glass => [0.85, 0.95, 1.0, 0.6],
            BlockType::Bricks => [0.65, 0.30, 0.25, 1.0],
            BlockType::Sand => [0.85, 0.80, 0.60, 1.0],
            _ => [0.50, 0.50, 0.50, 1.0],
        };

        // 16 particles (between 12 and 20)
        let count = 16;
        let center = pos.as_vec3() + Vec3::splat(0.5);

        for _ in 0..count {
            // Offset inside block: [-0.4 .. 0.4]
            let ox = (self.next_rand() - 0.5) * 0.8;
            let oy = (self.next_rand() - 0.5) * 0.8;
            let oz = (self.next_rand() - 0.5) * 0.8;
            let p_pos = center + Vec3::new(ox, oy, oz);

            // Outward burst velocity
            let vx = (self.next_rand() - 0.5) * 3.2;
            let vy = self.next_rand() * 2.8 + 1.2;
            let vz = (self.next_rand() - 0.5) * 3.2;

            // Random size: 0.08 .. 0.14
            let size = 0.08 + self.next_rand() * 0.06;

            // Subtle color tint variation
            let tint = (self.next_rand() - 0.5) * 0.1;
            let p_color = [
                (base_color[0] + tint).clamp(0.0, 1.0),
                (base_color[1] + tint).clamp(0.0, 1.0),
                (base_color[2] + tint).clamp(0.0, 1.0),
                base_color[3],
            ];

            let max_lifetime = 0.8 + self.next_rand() * 0.4; // 0.8s - 1.2s
            self.particles.push(Particle::new(
                p_pos,
                Vec3::new(vx, vy, vz),
                size,
                p_color,
                max_lifetime,
            ));
        }
    }

    /// Update all active particles and remove expired ones
    pub fn update(&mut self, dt: f32, world: &World) {
        self.particles.retain_mut(|p| p.tick(dt, world));
    }

    /// Builds camera-facing billboard quads for all particles
    pub fn build_billboards(
        &self,
        camera_right: Vec3,
        camera_up: Vec3,
        white_uv: [f32; 4],
        vertices: &mut Vec<EffectVertex>,
        indices: &mut Vec<u32>,
    ) {
        for p in &self.particles {
            let half = p.size / 2.0;
            let r = camera_right * half;
            let u = camera_up * half;

            // Compute alpha fade-out during the last 20% of lifetime
            let life_ratio = p.lifetime / p.max_lifetime;
            let alpha = if life_ratio > 0.8 {
                (1.0 - (life_ratio - 0.8) / 0.2) * p.color[3]
            } else {
                p.color[3]
            };
            let color = [p.color[0], p.color[1], p.color[2], alpha];

            let p0 = p.pos - r - u;
            let p1 = p.pos + r - u;
            let p2 = p.pos + r + u;
            let p3 = p.pos - r + u;

            let base = vertices.len() as u32;
            vertices.push(EffectVertex::new([p0.x, p0.y, p0.z], [white_uv[0], white_uv[1]], color));
            vertices.push(EffectVertex::new([p1.x, p1.y, p1.z], [white_uv[2], white_uv[1]], color));
            vertices.push(EffectVertex::new([p2.x, p2.y, p2.z], [white_uv[2], white_uv[3]], color));
            vertices.push(EffectVertex::new([p3.x, p3.y, p3.z], [white_uv[0], white_uv[3]], color));

            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
}

/// Renderer for block destroy crack overlay and particle system
pub struct EffectsRenderer {
    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub particles: ParticleSystem,
}

impl EffectsRenderer {
    const MAX_VERTICES: usize = 4096;
    const MAX_INDICES: usize = 6144;

    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        atlas_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Effects Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(EFFECTS_WGSL)),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Effects Pipeline Layout"),
            bind_group_layouts: &[Some(camera_bind_group_layout), Some(atlas_bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Effects Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(EffectVertex::desc())],
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

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Effects Dynamic Vertex Buffer"),
            size: (Self::MAX_VERTICES * std::mem::size_of::<EffectVertex>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Effects Dynamic Index Buffer"),
            size: (Self::MAX_INDICES * std::mem::size_of::<u32>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            particles: ParticleSystem::new(),
        }
    }

    /// Builds slightly expanded cube quads for the active destroy crack stage
    pub fn build_destroy_crack_quads(
        block_pos: IVec3,
        stage: usize,
        atlas: &TextureAtlas,
        vertices: &mut Vec<EffectVertex>,
        indices: &mut Vec<u32>,
    ) {
        let uv = atlas.get_destroy_stage_uv(stage);
        let eps = 0.002f32; // Slight epsilon expansion to prevent z-fighting
        let min_x = block_pos.x as f32 - eps;
        let min_y = block_pos.y as f32 - eps;
        let min_z = block_pos.z as f32 - eps;
        let max_x = (block_pos.x + 1) as f32 + eps;
        let max_y = (block_pos.y + 1) as f32 + eps;
        let max_z = (block_pos.z + 1) as f32 + eps;

        let color = [1.0, 1.0, 1.0, 1.0];

        // 6 Cube faces: UP, DOWN, NORTH, SOUTH, WEST, EAST
        // Up face (+Y)
        Self::add_quad(
            vertices,
            indices,
            [min_x, max_y, max_z],
            [max_x, max_y, max_z],
            [max_x, max_y, min_z],
            [min_x, max_y, min_z],
            uv,
            color,
        );

        // Down face (-Y)
        Self::add_quad(
            vertices,
            indices,
            [min_x, min_y, min_z],
            [max_x, min_y, min_z],
            [max_x, min_y, max_z],
            [min_x, min_y, max_z],
            uv,
            color,
        );

        // North face (-Z)
        Self::add_quad(
            vertices,
            indices,
            [max_x, min_y, min_z],
            [min_x, min_y, min_z],
            [min_x, max_y, min_z],
            [max_x, max_y, min_z],
            uv,
            color,
        );

        // South face (+Z)
        Self::add_quad(
            vertices,
            indices,
            [min_x, min_y, max_z],
            [max_x, min_y, max_z],
            [max_x, max_y, max_z],
            [min_x, max_y, max_z],
            uv,
            color,
        );

        // West face (-X)
        Self::add_quad(
            vertices,
            indices,
            [min_x, min_y, min_z],
            [min_x, min_y, max_z],
            [min_x, max_y, max_z],
            [min_x, max_y, min_z],
            uv,
            color,
        );

        // East face (+X)
        Self::add_quad(
            vertices,
            indices,
            [max_x, min_y, max_z],
            [max_x, min_y, min_z],
            [max_x, max_y, min_z],
            [max_x, max_y, max_z],
            uv,
            color,
        );
    }

    fn add_quad(
        vertices: &mut Vec<EffectVertex>,
        indices: &mut Vec<u32>,
        p0: [f32; 3],
        p1: [f32; 3],
        p2: [f32; 3],
        p3: [f32; 3],
        uv: [f32; 4],
        color: [f32; 4],
    ) {
        let base = vertices.len() as u32;
        vertices.push(EffectVertex::new(p0, [uv[0], uv[3]], color));
        vertices.push(EffectVertex::new(p1, [uv[2], uv[3]], color));
        vertices.push(EffectVertex::new(p2, [uv[2], uv[1]], color));
        vertices.push(EffectVertex::new(p3, [uv[0], uv[1]], color));

        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// Renders destroy crack overlay and burst particles
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        target_view: &wgpu::TextureView,
        camera_bind_group: &wgpu::BindGroup,
        atlas_bind_group: &wgpu::BindGroup,
        atlas: &TextureAtlas,
        camera_right: Vec3,
        camera_up: Vec3,
        mining_target: Option<(IVec3, usize)>,
    ) {
        let mut vertices = Vec::with_capacity(512);
        let mut indices = Vec::with_capacity(768);

        // 1. Build destroy crack overlay if a block is actively being broken
        if let Some((pos, stage)) = mining_target {
            Self::build_destroy_crack_quads(pos, stage, atlas, &mut vertices, &mut indices);
        }

        // 2. Build particle billboard quads
        let white_rect = atlas.get_sprite("dirt");
        let white_uv = [white_rect.u_min, white_rect.v_min, white_rect.u_max, white_rect.v_max];
        self.particles
            .build_billboards(camera_right, camera_up, white_uv, &mut vertices, &mut indices);

        let index_count = indices.len() as u32;
        if index_count == 0 {
            return;
        }

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Effects Render Pass"),
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

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, camera_bind_group, &[]);
        rpass.set_bind_group(1, atlas_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        rpass.draw_indexed(0..index_count, 0, 0..1);
    }
}

const EFFECTS_WGSL: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

struct VertexInput {
    @location(0) pos: vec3<f32>,
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
    out.clip_position = camera.view_proj * vec4<f32>(in.pos, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    return out;
}

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
    fn test_effect_vertex_layout() {
        assert_eq!(std::mem::size_of::<EffectVertex>(), 36);
        let desc = EffectVertex::desc();
        assert_eq!(desc.array_stride, 36);
        assert_eq!(desc.attributes.len(), 3);
    }

    #[test]
    fn test_particle_physics_gravity_and_expiration() {
        let world = World::new();
        let mut p = Particle::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
            0.1,
            [1.0, 1.0, 1.0, 1.0],
            1.0,
        );

        // Tick with 0.1s dt
        let alive = p.tick(0.1, &world);
        assert!(alive);
        assert!(p.vel.y < 2.0); // Gravity reduced upward velocity

        // Tick beyond lifetime
        let alive_after = p.tick(1.0, &world);
        assert!(!alive_after); // Expired
    }

    #[test]
    fn test_particle_spawn_burst_count() {
        let mut sys = ParticleSystem::new();
        assert_eq!(sys.particles.len(), 0);

        sys.spawn_block_break(IVec3::new(5, 1, 5), BlockType::Stone);
        assert_eq!(sys.particles.len(), 16); // 16 particles in 12-20 burst range

        for p in &sys.particles {
            assert!(p.vel.y > 0.0); // Outward upward burst
            assert!(p.lifetime == 0.0);
            assert!(p.max_lifetime >= 0.8 && p.max_lifetime <= 1.2);
        }
    }

    #[test]
    fn test_destroy_crack_quads_count() {
        let atlas = TextureAtlas::load_or_create(None);
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        EffectsRenderer::build_destroy_crack_quads(
            IVec3::new(0, 0, 0),
            5,
            &atlas,
            &mut vertices,
            &mut indices,
        );

        // 6 faces * 4 vertices = 24 vertices
        assert_eq!(vertices.len(), 24);
        // 6 faces * 6 indices = 36 indices
        assert_eq!(indices.len(), 36);
    }
}
