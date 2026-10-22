// REA Verified: net.minecraft.client.renderer.entity.LivingEntityRenderer (mapped: gen.class)
// REA Verified: net.minecraft.client.renderer.texture.OverlayTexture (mapped: ggs.class)
//
// WGPU Entity Rendering Pipeline:
// - Vertex Shader: Transforms 3D box model vertices using camera view_proj and computes directional lighting.
// - Fragment Shader: Samples entity skin atlas, discards transparent fragments, and applies hurt red tint flash.
// - Authentic Minecraft hurt flash: 70% saturated red blend (OverlayTexture.RED_OVERLAY_COLOR).

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::sync::Arc;
use super::animation::{EntityAnimationState, ProceduralAnimationEngine};
use super::atlas::{EntitySkin, EntityTextureAtlas};
use super::box_model::HierarchicalModel;
use super::models::{BatModel, BreezeModel, CreakingModel, GhastModel, ParrotModel, QuadrupedModel, ZombieModel};
use crate::game::mobs::{MobEntity, MobType};

pub const ENTITY_PIPELINE_WGSL: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct EntityUniform {
    // x: global hurt flash multiplier [0.0 .. 1.0], yzw: reserved
    hurt_flash: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@group(2) @binding(0)
var<uniform> entity_uniform: EntityUniform;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) color: vec4<f32>,
    @location(4) hurt_tint: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) hurt_tint: f32,
    @location(3) light: f32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.pos, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    out.hurt_tint = in.hurt_tint;

    // Authentic directional normal shading matching Minecraft:
    // Up: 1.0, Down: 0.5, North/South (Z): 0.8, East/West (X): 0.6
    let n = normalize(in.normal);
    var shade = 0.7;
    if (abs(n.y) > 0.5) {
        shade = select(0.5, 1.0, n.y > 0.0);
    } else if (abs(n.z) > 0.5) {
        shade = 0.8;
    } else if (abs(n.x) > 0.5) {
        shade = 0.6;
    }
    out.light = shade;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_color = textureSample(t_diffuse, s_diffuse, in.uv);
    if (tex_color.a < 0.1) {
        discard;
    }

    // Directional shaded base diffuse
    var final_rgb = tex_color.rgb * in.color.rgb * in.light;

    // Hurt red tint flash shader:
    // Blends with saturated hurt red when entity takes damage
    let hurt = max(in.hurt_tint, entity_uniform.hurt_flash.x);
    if (hurt > 0.0) {
        let hurt_red = vec3<f32>(1.0, 0.15, 0.15);
        final_rgb = mix(final_rgb, hurt_red, hurt * 0.7);
    }

    return vec4<f32>(final_rgb, tex_color.a * in.color.a);
}
"#;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct EntityVertex {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
    pub color: [f32; 4],
    pub hurt_tint: f32,
}

impl EntityVertex {
    pub fn new(
        pos: [f32; 3],
        uv: [f32; 2],
        normal: [f32; 3],
        color: [f32; 4],
        hurt_tint: f32,
    ) -> Self {
        Self {
            pos,
            uv,
            normal,
            color,
            hurt_tint,
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<EntityVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct EntityUniformBuffer {
    pub hurt_flash: [f32; 4],
}

/// Complete WGPU entity rendering pipeline
/// Entity Mesh Builder containing all hierarchical mob model instances
pub struct EntityMeshBuilder {
    pub zombie_model: ZombieModel,
    pub quadruped_model: QuadrupedModel,
    pub breeze_model: BreezeModel,
    pub creaking_model: CreakingModel,
    pub bat_model: BatModel,
    pub parrot_model: ParrotModel,
    pub ghast_model: GhastModel,
}

impl Default for EntityMeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityMeshBuilder {
    pub fn new() -> Self {
        Self {
            zombie_model: ZombieModel::new(),
            quadruped_model: QuadrupedModel::new(),
            breeze_model: BreezeModel::new(),
            creaking_model: CreakingModel::new(),
            bat_model: BatModel::new(),
            parrot_model: ParrotModel::new(),
            ghast_model: GhastModel::new(),
        }
    }

    /// Builds hierarchical vertices and indices for a slice of living mob entities
    pub fn build_mob_meshes(
        &mut self,
        atlas: &EntityTextureAtlas,
        mobs: &[MobEntity],
        camera_pos: Vec3,
        vertices: &mut Vec<EntityVertex>,
        indices: &mut Vec<u32>,
    ) {
        let vertex_builder = |pos: [f32; 3], uv: [f32; 2], normal: [f32; 3], color: [f32; 4], hurt: f32| {
            EntityVertex::new(pos, uv, normal, color, hurt)
        };

        for mob in mobs {
            if !mob.health.is_alive() {
                continue;
            }

            let pos = mob.transform.position;
            let yaw = mob.transform.rotation.x;

            // Head tracking relative to camera
            let (head_yaw, head_pitch) = ProceduralAnimationEngine::update_head_tracking(
                0.0,
                0.0,
                pos,
                yaw,
                camera_pos,
                1.0,
            );

            let horiz_speed = (mob.velocity.value.x * mob.velocity.value.x + mob.velocity.value.z * mob.velocity.value.z).sqrt();
            let limb_speed = (horiz_speed * 5.0).clamp(0.0, 1.0);

            // Hurt flash factor from mob component
            let hurt_flash = ProceduralAnimationEngine::calculate_hurt_flash(mob.hurt_time);

            let anim_state = EntityAnimationState {
                limb_pos: mob.ai.walk_distance * 4.0,
                limb_speed,
                age_in_ticks: (mob.id as f32 * 17.0) + (mob.ai.walk_distance * 20.0),
                head_yaw,
                head_pitch,
                hurt_time: hurt_flash,
                is_flying: matches!(mob.mob_type, MobType::Bat | MobType::Parrot | MobType::Ghast | MobType::Breeze),
                is_resting: matches!(mob.mob_type, MobType::Creaking),
            };

            // Root transform in world space
            let root_transform = Mat4::from_translation(pos) * Mat4::from_rotation_y(yaw);

            match mob.mob_type {
                MobType::Zombie => {
                    let uv_rect = atlas.get_skin_uv(EntitySkin::Zombie);
                    self.zombie_model.setup_anim(&anim_state);
                    self.zombie_model.root().compile_mesh(
                        root_transform,
                        Some(uv_rect),
                        [1.0, 1.0, 1.0, 1.0],
                        hurt_flash,
                        &vertex_builder,
                        vertices,
                        indices,
                    );
                }
                MobType::Pig => {
                    let uv_rect = atlas.get_skin_uv(EntitySkin::Pig);
                    self.quadruped_model.setup_anim(&anim_state);
                    self.quadruped_model.root().compile_mesh(
                        root_transform,
                        Some(uv_rect),
                        [1.0, 1.0, 1.0, 1.0],
                        hurt_flash,
                        &vertex_builder,
                        vertices,
                        indices,
                    );
                }
                MobType::Cow => {
                    let uv_rect = atlas.get_skin_uv(EntitySkin::Cow);
                    self.quadruped_model.setup_anim(&anim_state);
                    self.quadruped_model.root().compile_mesh(
                        root_transform,
                        Some(uv_rect),
                        [1.0, 1.0, 1.0, 1.0],
                        hurt_flash,
                        &vertex_builder,
                        vertices,
                        indices,
                    );
                }
                MobType::Sheep => {
                    let uv_rect = atlas.get_skin_uv(EntitySkin::Sheep);
                    self.quadruped_model.setup_anim(&anim_state);
                    self.quadruped_model.root().compile_mesh(
                        root_transform,
                        Some(uv_rect),
                        [1.0, 1.0, 1.0, 1.0],
                        hurt_flash,
                        &vertex_builder,
                        vertices,
                        indices,
                    );
                }
                MobType::Creeper => {
                    let uv_rect = atlas.get_skin_uv(EntitySkin::Creeper);
                    self.quadruped_model.setup_anim(&anim_state);
                    self.quadruped_model.root().compile_mesh(
                        root_transform,
                        Some(uv_rect),
                        [1.0, 1.0, 1.0, 1.0],
                        hurt_flash,
                        &vertex_builder,
                        vertices,
                        indices,
                    );
                }
                MobType::Breeze => {
                    let uv_rect = atlas.get_skin_uv(EntitySkin::Breeze);
                    self.breeze_model.setup_anim(&anim_state);
                    self.breeze_model.root().compile_mesh(
                        root_transform,
                        Some(uv_rect),
                        [1.0, 1.0, 1.0, 1.0],
                        hurt_flash,
                        &vertex_builder,
                        vertices,
                        indices,
                    );
                }
                MobType::Creaking => {
                    let uv_rect = atlas.get_skin_uv(EntitySkin::Creaking);
                    self.creaking_model.setup_anim(&anim_state);
                    self.creaking_model.root().compile_mesh(
                        root_transform,
                        Some(uv_rect),
                        [1.0, 1.0, 1.0, 1.0],
                        hurt_flash,
                        &vertex_builder,
                        vertices,
                        indices,
                    );
                }
                MobType::Bat => {
                    let uv_rect = atlas.get_skin_uv(EntitySkin::Bat);
                    self.bat_model.setup_anim(&anim_state);
                    self.bat_model.root().compile_mesh(
                        root_transform,
                        Some(uv_rect),
                        [1.0, 1.0, 1.0, 1.0],
                        hurt_flash,
                        &vertex_builder,
                        vertices,
                        indices,
                    );
                }
                MobType::Parrot => {
                    let uv_rect = atlas.get_skin_uv(EntitySkin::Parrot);
                    self.parrot_model.setup_anim(&anim_state);
                    self.parrot_model.root().compile_mesh(
                        root_transform,
                        Some(uv_rect),
                        [1.0, 1.0, 1.0, 1.0],
                        hurt_flash,
                        &vertex_builder,
                        vertices,
                        indices,
                    );
                }
                MobType::Ghast => {
                    let uv_rect = atlas.get_skin_uv(EntitySkin::Ghast);
                    self.ghast_model.setup_anim(&anim_state);
                    self.ghast_model.root().compile_mesh(
                        root_transform,
                        Some(uv_rect),
                        [1.0, 1.0, 1.0, 1.0],
                        hurt_flash,
                        &vertex_builder,
                        vertices,
                        indices,
                    );
                }
            }
        }
    }
}

/// Complete WGPU entity rendering pipeline
pub struct EntityPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub atlas: Arc<EntityTextureAtlas>,
    pub atlas_texture: wgpu::Texture,
    pub atlas_view: wgpu::TextureView,
    pub atlas_sampler: wgpu::Sampler,
    pub atlas_bind_group: wgpu::BindGroup,
    pub mesh_builder: EntityMeshBuilder,
}

impl EntityPipeline {
    pub const MAX_VERTICES: usize = 16384;
    pub const MAX_INDICES: usize = 24576;

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        atlas_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Entity Shader Module"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(ENTITY_PIPELINE_WGSL)),
        });

        // Uniform buffer for hurt flash
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entity Uniform Buffer"),
            size: std::mem::size_of::<EntityUniformBuffer>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Entity Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Entity Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Entity Pipeline Layout"),
            bind_group_layouts: &[
                Some(camera_bind_group_layout),
                Some(atlas_bind_group_layout),
                Some(&uniform_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Entity Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(EntityVertex::desc())],
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
            size: (Self::MAX_VERTICES * std::mem::size_of::<EntityVertex>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entity Dynamic Index Buffer"),
            size: (Self::MAX_INDICES * std::mem::size_of::<u32>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Load or create the Entity Skin Texture Atlas
        let atlas = Arc::new(EntityTextureAtlas::load_or_create(None, None));
        let (atlas_texture, atlas_view, atlas_sampler, atlas_bind_group) =
            atlas.create_gpu_resources(device, queue, atlas_bind_group_layout);

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            uniform_bind_group,
            atlas,
            atlas_texture,
            atlas_view,
            atlas_sampler,
            atlas_bind_group,
            mesh_builder: EntityMeshBuilder::new(),
        }
    }

    /// Builds hierarchical vertices and indices for a slice of living mob entities
    pub fn build_mob_meshes(
        &mut self,
        mobs: &[MobEntity],
        camera_pos: Vec3,
        vertices: &mut Vec<EntityVertex>,
        indices: &mut Vec<u32>,
    ) {
        self.mesh_builder
            .build_mob_meshes(&self.atlas, mobs, camera_pos, vertices, indices);
    }

    /// Renders mobs and entities with the pipeline
    pub fn render_mobs(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        target_view: &wgpu::TextureView,
        camera_bind_group: &wgpu::BindGroup,
        camera_pos: Vec3,
        mobs: &[MobEntity],
    ) {
        if mobs.is_empty() {
            return;
        }

        let mut vertices = Vec::with_capacity(mobs.len() * 144);
        let mut indices = Vec::with_capacity(mobs.len() * 216);

        self.build_mob_meshes(mobs, camera_pos, &mut vertices, &mut indices);

        let index_count = indices.len() as u32;
        if index_count == 0 {
            return;
        }

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));

        let uniform_data = EntityUniformBuffer {
            hurt_flash: [0.0, 0.0, 0.0, 0.0],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniform_data]));

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Hierarchical Entity Render Pass"),
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
        rpass.set_bind_group(1, &self.atlas_bind_group, &[]);
        rpass.set_bind_group(2, &self.uniform_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        rpass.draw_indexed(0..index_count, 0, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_vertex_size_and_layout() {
        assert_eq!(std::mem::size_of::<EntityVertex>(), 52); // 3*4 + 2*4 + 3*4 + 4*4 + 4 = 12 + 8 + 12 + 16 + 4 = 52
        let layout = EntityVertex::desc();
        assert_eq!(layout.attributes.len(), 5);
    }

    #[test]
    fn test_build_mob_meshes_hierarchical_box_counts() {
        let atlas = EntityTextureAtlas::load_or_create(None, None);
        let mut builder = EntityMeshBuilder::new();

        let mut zombie = MobEntity::new_zombie(1, Vec3::new(0.0, 0.0, 0.0));
        zombie.hurt_time = 0.4; // Damaged -> hurt red flash should be active

        let pig = MobEntity::new_quadruped(2, Vec3::new(5.0, 0.0, 0.0), MobType::Pig);
        let breeze = MobEntity::new_breeze(3, Vec3::new(10.0, 0.0, 0.0));
        let creaking = MobEntity::new_creaking(4, Vec3::new(15.0, 0.0, 0.0));

        let mobs = vec![zombie, pig, breeze, creaking];
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        builder.build_mob_meshes(&atlas, &mobs, Vec3::new(0.0, 1.8, -5.0), &mut vertices, &mut indices);

        assert!(!vertices.is_empty());
        assert!(!indices.is_empty());

        // Zombie has 6 parts * 6 faces * 4 vertices = 144 vertices
        // Verify hurt_tint on zombie vertices (first 144) is active (> 0)
        let zombie_hurt_tint = vertices[0].hurt_tint;
        assert!(zombie_hurt_tint > 0.5, "Zombie should have red flash hurt_tint > 0.5");

        // Verify pig hurt_tint is 0.0 (undamaged)
        let pig_hurt_tint = vertices[144].hurt_tint;
        assert_eq!(pig_hurt_tint, 0.0, "Undamaged mob should have hurt_tint = 0.0");
    }
}


