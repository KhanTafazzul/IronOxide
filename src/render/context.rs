use std::sync::Arc;
use winit::window::Window;
use wgpu::util::DeviceExt;
use crate::render::atlas::TextureAtlas;
use crate::render::camera::Camera;
use crate::render::effects::EffectsRenderer;
use crate::render::gui::{GuiAtlas, GuiRenderer};
use crate::render::uniforms::CameraUniform;

pub struct RenderContext<'w> {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'w>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group: wgpu::BindGroup,
    pub atlas: TextureAtlas,
    pub atlas_texture: wgpu::Texture,
    pub atlas_view: wgpu::TextureView,
    pub atlas_sampler: wgpu::Sampler,
    pub atlas_bind_group_layout: wgpu::BindGroupLayout,
    pub atlas_bind_group: wgpu::BindGroup,
    pub gui_atlas: Arc<GuiAtlas>,
    pub gui_renderer: GuiRenderer,
    pub effects_renderer: EffectsRenderer,
}

impl<'w> RenderContext<'w> {
    pub async fn new(window: Arc<Window>, camera: &Camera) -> Result<Self, Box<dyn std::error::Error>> {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(&window))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: true,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await?;

        let mut config = surface
            .get_default_config(&adapter, width, height)
            .ok_or("Failed to get default surface configuration")?;
        config.present_mode = wgpu::PresentMode::AutoVsync;
        surface.configure(&device, &config);

        // Uniform setup
        let aspect = width as f32 / height as f32;
        let mut camera_uniform = CameraUniform::default();
        camera_uniform.update(camera, aspect);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // REA Verified: TextureAtlas loading & GPU resource setup
        let atlas = TextureAtlas::load_or_create(None);
        let (atlas_texture, atlas_view, atlas_sampler, atlas_bind_group_layout, atlas_bind_group) =
            atlas.create_gpu_resources(&device, &queue);

        // REA Verified: 2D HUD Atlas and GuiRenderer setup
        let gui_atlas = Arc::new(GuiAtlas::load_or_create(None));
        let gui_renderer = GuiRenderer::new(&device, &queue, config.format, Arc::clone(&gui_atlas));

        // REA Verified: 3D Destroy crack overlay & Particle System setup
        let effects_renderer = EffectsRenderer::new(
            &device,
            config.format,
            &camera_bind_group_layout,
            &atlas_bind_group_layout,
        );

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            camera_uniform,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
            atlas,
            atlas_texture,
            atlas_view,
            atlas_sampler,
            atlas_bind_group_layout,
            atlas_bind_group,
            gui_atlas,
            gui_renderer,
            effects_renderer,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn update_camera(&mut self, camera: &Camera) {
        let aspect = self.config.width as f32 / self.config.height as f32;
        self.camera_uniform.update(camera, aspect);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn render(
        &mut self,
        active_slot: usize,
        inventory_gui: Option<(&crate::render::gui::InventoryGui, &crate::game::inventory::PlayerInventory)>,
        mining_target: Option<(glam::IVec3, usize)>,
        camera_right: glam::Vec3,
        camera_up: glam::Vec3,
        player_stats: Option<&crate::game::stats::PlayerStats>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex) | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => tex,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => return Ok(()),
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                let size = self.size;
                self.resize(size);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err("WGPU surface validation error".into());
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Frame Encoder"),
            });

        {
            // Clear screen with Minecraft atmospheric sky color: RGBA(0.47, 0.65, 1.0, 1.0)
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Sky Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.4705,
                            g: 0.6549,
                            b: 1.0000,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }

        // 3D Effects pass: Destroy crack overlay & Particle billboards
        self.effects_renderer.render(
            &mut encoder,
            &self.queue,
            &view,
            &self.camera_bind_group,
            &self.atlas_bind_group,
            &self.atlas,
            camera_right,
            camera_up,
            mining_target,
        );

        // 2D HUD and/or Inventory pass (executes after 3D pass with LoadOp::Load without clearing color attachment)
        self.gui_renderer.render(
            &mut encoder,
            &self.queue,
            &view,
            self.config.width,
            self.config.height,
            active_slot,
            inventory_gui,
            player_stats,
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(output);

        Ok(())
    }
}
