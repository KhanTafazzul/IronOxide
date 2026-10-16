// REA Verified: net.minecraft.client.renderer.entity.ItemEntityRenderer (mapped: fzj.class)
//
// Minecraft 1.20.4 Dropped Item Entity 3D Renderer:
// - Renders dropped items as 1/4-scale rotating textured voxel cubes.
// - Authentic sinusoidal bobbing elevation: y = y0 + sin(ticks * 0.1) * 0.1.
// - Continuous Y-axis yaw rotation matching vanilla ItemEntity spin.
// - Sub-pixel UV coordinates sampled directly from TextureAtlas for each cube face.
// - Vanilla directional face shading (Up: 1.0, Down: 0.5, North/South: 0.8, East/West: 0.6).

use glam::Vec3;
use crate::game::entities::ItemEntity;
use crate::render::atlas::{BlockType, Direction, TextureAtlas};
use crate::render::effects::EffectVertex;

pub const ENTITY_WGSL: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

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
    if (tex_color.a < 0.1) {
        discard;
    }
    return tex_color * in.color;
}
"#;

pub struct EntityRenderer {
    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl EntityRenderer {
    pub const MAX_VERTICES: usize = 4096;
    pub const MAX_INDICES: usize = 6144;
    pub const CUBE_HALF_SCALE: f32 = 0.125; // 1/4 of 1.0 block = 0.25 total width (0.125 half)

    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        atlas_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Entity Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(ENTITY_WGSL)),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Entity Pipeline Layout"),
            bind_group_layouts: &[Some(camera_bind_group_layout), Some(atlas_bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Entity Render Pipeline"),
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
            label: Some("Entity Dynamic Vertex Buffer"),
            size: (Self::MAX_VERTICES * std::mem::size_of::<EffectVertex>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entity Dynamic Index Buffer"),
            size: (Self::MAX_INDICES * std::mem::size_of::<u32>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
        }
    }

    /// Builds 1/4-scale rotating textured voxel cubes for all active dropped items
    pub fn build_item_quads(
        entities: &[ItemEntity],
        atlas: &TextureAtlas,
        vertices: &mut Vec<EffectVertex>,
        indices: &mut Vec<u32>,
    ) {
        let h = Self::CUBE_HALF_SCALE;

        for entity in entities {
            if entity.is_dead {
                continue;
            }

            let block = entity.item.to_block().unwrap_or(BlockType::Dirt);
            let center = Vec3::new(entity.pos.x, entity.visual_y(), entity.pos.z);
            let angle = entity.rotation_angle();
            let (cos_a, sin_a) = (angle.cos(), angle.sin());

            // Helper to rotate local (x, z) coordinates around Y-axis and translate to center
            let transform = |lx: f32, ly: f32, lz: f32| -> [f32; 3] {
                let rx = lx * cos_a - lz * sin_a;
                let rz = lx * sin_a + lz * cos_a;
                [center.x + rx, center.y + ly, center.z + rz]
            };

            // 6 Faces of the 1/4-scale cube:
            let faces = [
                // Direction, local corners [c0, c1, c2, c3], directional shade
                (
                    Direction::Up,
                    [[-h, h, h], [h, h, h], [h, h, -h], [-h, h, -h]],
                    1.0f32,
                ),
                (
                    Direction::Down,
                    [[-h, -h, -h], [h, -h, -h], [h, -h, h], [-h, -h, h]],
                    0.5f32,
                ),
                (
                    Direction::North,
                    [[h, -h, -h], [-h, -h, -h], [-h, h, -h], [h, h, -h]],
                    0.8f32,
                ),
                (
                    Direction::South,
                    [[-h, -h, h], [h, -h, h], [h, h, h], [-h, h, h]],
                    0.8f32,
                ),
                (
                    Direction::West,
                    [[-h, -h, -h], [-h, -h, h], [-h, h, h], [-h, h, -h]],
                    0.6f32,
                ),
                (
                    Direction::East,
                    [[h, -h, h], [h, -h, -h], [h, h, -h], [h, h, h]],
                    0.6f32,
                ),
            ];

            for (face_dir, corners, shade) in faces {
                let uv = atlas.get_face_uv(block, face_dir);
                let color = [shade, shade, shade, 1.0];

                let p0 = transform(corners[0][0], corners[0][1], corners[0][2]);
                let p1 = transform(corners[1][0], corners[1][1], corners[1][2]);
                let p2 = transform(corners[2][0], corners[2][1], corners[2][2]);
                let p3 = transform(corners[3][0], corners[3][1], corners[3][2]);

                let base = vertices.len() as u32;
                vertices.push(EffectVertex::new(p0, [uv[0], uv[3]], color));
                vertices.push(EffectVertex::new(p1, [uv[2], uv[3]], color));
                vertices.push(EffectVertex::new(p2, [uv[2], uv[1]], color));
                vertices.push(EffectVertex::new(p3, [uv[0], uv[1]], color));

                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }
        }
    }

    /// Renders all dropped items
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        target_view: &wgpu::TextureView,
        camera_bind_group: &wgpu::BindGroup,
        atlas_bind_group: &wgpu::BindGroup,
        atlas: &TextureAtlas,
        entities: &[ItemEntity],
    ) {
        if entities.is_empty() {
            return;
        }

        let mut vertices = Vec::with_capacity(entities.len() * 24);
        let mut indices = Vec::with_capacity(entities.len() * 36);

        Self::build_item_quads(entities, atlas, &mut vertices, &mut indices);

        let index_count = indices.len() as u32;
        if index_count == 0 {
            return;
        }

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Entity Render Pass"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::inventory::{ItemStack, ITEM_DIRT};

    #[test]
    fn test_build_item_quads_counts() {
        let atlas = TextureAtlas::load_or_create(None);
        let mut entities = Vec::new();

        let mut item = ItemEntity::new(
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::ZERO,
            ItemStack::new(ITEM_DIRT, 1),
        );
        item.age = 20;
        entities.push(item);

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        EntityRenderer::build_item_quads(&entities, &atlas, &mut vertices, &mut indices);

        // 1 item cube = 6 faces * 4 vertices = 24 vertices
        assert_eq!(vertices.len(), 24);
        // 1 item cube = 6 faces * 6 indices = 36 indices
        assert_eq!(indices.len(), 36);

        // Verify sinusoidal elevation formula:
        // y = y0 + sin(ticks * 0.1) * 0.1
        let expected_y = 5.0 + 0.125 + (20.0f32 * 0.1).sin() * 0.1;
        let avg_y: f32 = vertices.iter().map(|v| v.pos[1]).sum::<f32>() / 24.0;
        assert!((avg_y - expected_y).abs() < 1e-4, "Vertices should be centered at sinusoidal elevation");
    }

    #[test]
    fn test_cube_dimension_quarter_scale() {
        let atlas = TextureAtlas::load_or_create(None);
        let item = ItemEntity::new(
            Vec3::new(10.0, 10.0, 10.0),
            Vec3::ZERO,
            ItemStack::new(ITEM_DIRT, 1),
        );
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        EntityRenderer::build_item_quads(&[item], &atlas, &mut vertices, &mut indices);

        let min_y = vertices.iter().map(|v| v.pos[1]).fold(f32::INFINITY, f32::min);
        let max_y = vertices.iter().map(|v| v.pos[1]).fold(f32::NEG_INFINITY, f32::max);
        let height = max_y - min_y;

        assert!((height - 0.25).abs() < 1e-4, "Height of cube must be 0.25 (1/4 scale)");
    }
}
