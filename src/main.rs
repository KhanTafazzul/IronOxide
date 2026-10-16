mod game;
mod render;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

use game::{raycast_voxel, Aabb, PlayerInput, PlayerPhysics, World, DEFAULT_MAX_REACH};
use render::atlas::{BlockType, TextureAtlas};
use render::mesher::{greedy_mesh_section, ChunkMesh};
use render::{Camera, RenderContext};

struct GameApp<'a> {
    render_ctx: Option<RenderContext<'a>>,
    camera: Camera,
    player: PlayerPhysics,
    world: World,
    atlas: Arc<TextureAtlas>,
    meshes: HashMap<glam::IVec3, ChunkMesh>,
    selected_block: BlockType,
    active_slot: usize,
    hotbar_slots: [BlockType; 9],
    mouse_captured: bool,
    // Movement key states
    key_w: bool,
    key_s: bool,
    key_a: bool,
    key_d: bool,
    key_space: bool,
    key_shift: bool,
    last_frame: Instant,
}

impl<'a> GameApp<'a> {
    fn new() -> Self {
        let atlas = Arc::new(TextureAtlas::load_or_create(None));
        let mut world = World::new();

        // Build a ground platform at y = 0
        for x in 0..16 {
            for z in 0..16 {
                world.set_block_at(glam::IVec3::new(x, 0, z), BlockType::GrassBlock);
            }
        }
        // Stepping blocks to test interaction and step-up
        world.set_block_at(glam::IVec3::new(8, 1, 8), BlockType::Stone);
        world.set_block_at(glam::IVec3::new(8, 1, 9), BlockType::Cobblestone);

        let mut meshes = HashMap::new();
        if let Some(sec) = world.sections.get(&glam::IVec3::ZERO) {
            let mesh = greedy_mesh_section(sec, &atlas, &world);
            meshes.insert(glam::IVec3::ZERO, mesh);
        }

        let spawn_pos = glam::Vec3::new(8.0, 1.0, 5.0);
        let player = PlayerPhysics::new(spawn_pos);
        let camera = Camera::new(player.eye_position(), 0.0, 0.0, 70.0);

        let hotbar_slots = [
            BlockType::Stone,
            BlockType::Dirt,
            BlockType::GrassBlock,
            BlockType::Cobblestone,
            BlockType::Deepslate,
            BlockType::OakPlanks,
            BlockType::OakLog,
            BlockType::Glass,
            BlockType::Bricks,
        ];
        let active_slot = 0;
        let selected_block = hotbar_slots[active_slot];

        Self {
            render_ctx: None,
            camera,
            player,
            world,
            atlas,
            meshes,
            selected_block,
            active_slot,
            hotbar_slots,
            mouse_captured: false,
            key_w: false,
            key_s: false,
            key_a: false,
            key_d: false,
            key_space: false,
            key_shift: false,
            last_frame: Instant::now(),
        }
    }

    /// Set active hotbar slot (0 to 8) and update active block
    fn set_active_slot(&mut self, slot: usize) {
        self.active_slot = slot.min(8);
        self.selected_block = self.hotbar_slots[self.active_slot];
        println!(
            "Active Hotbar Slot {}: {:?} (Block ID: {:?})",
            self.active_slot + 1,
            self.selected_block,
            self.selected_block
        );
    }

    fn update_physics(&mut self, _dt: f32) {
        let forward = if self.key_w { 1.0 } else if self.key_s { -1.0 } else { 0.0 };
        let strafe = if self.key_a { 1.0 } else if self.key_d { -1.0 } else { 0.0 };
        let input = PlayerInput {
            forward,
            strafe,
            jump: self.key_space,
            sprint: self.key_shift,
            yaw_degrees: self.camera.yaw,
        };

        self.player.tick(&input, &self.world);
        self.camera.eye = self.player.eye_position();
    }

    fn toggle_mouse_grab(&mut self, window: &Window) {
        self.mouse_captured = !self.mouse_captured;
        if self.mouse_captured {
            let _ = window.set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
            window.set_cursor_visible(false);
        } else {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            window.set_cursor_visible(true);
        }
    }

    /// Left-Click: Destroy targeted block and trigger immediate re-meshing
    fn break_targeted_block(&mut self) {
        if let Some(hit) = raycast_voxel(
            self.camera.eye,
            self.camera.forwards(),
            DEFAULT_MAX_REACH,
            &self.world,
        ) {
            println!("Left Click: Destroying block at {:?} (face: {:?})", hit.block_pos, hit.face);
            let affected = self.world.set_block_at(hit.block_pos, BlockType::Air);
            self.remesh_sections(&affected);
        }
    }

    /// Right-Click: Place selected block against the targeted face and trigger immediate re-meshing
    fn place_targeted_block(&mut self) {
        if let Some(hit) = raycast_voxel(
            self.camera.eye,
            self.camera.forwards(),
            DEFAULT_MAX_REACH,
            &self.world,
        ) {
            let place_pos = hit.place_pos();
            let block_box = Aabb::from_block(place_pos.x, place_pos.y, place_pos.z);
            if !block_box.intersects(&self.player.bounding_box()) {
                println!(
                    "Right Click: Placing {:?} at {:?} against {:?}",
                    self.selected_block, place_pos, hit.face
                );
                let affected = self.world.set_block_at(place_pos, self.selected_block);
                self.remesh_sections(&affected);
            } else {
                println!("Cannot place block inside player bounding box!");
            }
        }
    }

    /// Re-mesh affected chunk section and any boundary neighbor sections
    fn remesh_sections(&mut self, sections: &[glam::IVec3]) {
        for &sec_pos in sections {
            if let Some(section) = self.world.sections.get(&sec_pos) {
                let mesh = greedy_mesh_section(section, &self.atlas, &self.world);
                println!(
                    "Section {:?} re-meshed: {} quads, {} vertices",
                    sec_pos,
                    mesh.quad_count(),
                    mesh.vertex_count()
                );
                self.meshes.insert(sec_pos, mesh);
            }
        }
    }
}

impl<'a> ApplicationHandler for GameApp<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.render_ctx.is_some() {
            return;
        }

        let window_attrs = Window::default_attributes()
            .with_title("Minecraft Rust Client - WGPU Native Graphics")
            .with_inner_size(PhysicalSize::new(1280, 720));

        let window = Arc::new(event_loop.create_window(window_attrs).expect("Window creation failed"));

        // Capture cursor on start
        let _ = window.set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
        window.set_cursor_visible(false);
        self.mouse_captured = true;

        let ctx = pollster::block_on(RenderContext::new(Arc::clone(&window), &self.camera))
            .expect("RenderContext initialization failed");

        self.render_ctx = Some(ctx);
        self.last_frame = Instant::now();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(ctx) = &mut self.render_ctx {
                    ctx.resize(physical_size);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_x, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => (pos.y as f32).signum(),
                };
                if scroll > 0.0 {
                    // Scroll up: cycle left ((slot + 8) % 9)
                    let prev_slot = (self.active_slot + 8) % 9;
                    self.set_active_slot(prev_slot);
                } else if scroll < 0.0 {
                    // Scroll down: cycle right ((slot + 1) % 9)
                    let next_slot = (self.active_slot + 1) % 9;
                    self.set_active_slot(next_slot);
                }
            }
            WindowEvent::MouseInput { button, state, .. } => {
                if state == ElementState::Pressed {
                    if !self.mouse_captured {
                        if let Some(ctx) = &self.render_ctx {
                            let window = Arc::clone(&ctx.window);
                            self.toggle_mouse_grab(&window);
                        }
                    } else {
                        match button {
                            MouseButton::Left => self.break_targeted_block(),
                            MouseButton::Right => self.place_targeted_block(),
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state,
                        ..
                    },
                ..
            } => {
                let pressed = state == ElementState::Pressed;
                match key_code {
                    KeyCode::KeyW => self.key_w = pressed,
                    KeyCode::KeyS => self.key_s = pressed,
                    KeyCode::KeyA => self.key_a = pressed,
                    KeyCode::KeyD => self.key_d = pressed,
                    KeyCode::Space => self.key_space = pressed,
                    KeyCode::ShiftLeft | KeyCode::ShiftRight => self.key_shift = pressed,
                    // Hotbar slot switching (1 to 9)
                    KeyCode::Digit1 if pressed => self.set_active_slot(0),
                    KeyCode::Digit2 if pressed => self.set_active_slot(1),
                    KeyCode::Digit3 if pressed => self.set_active_slot(2),
                    KeyCode::Digit4 if pressed => self.set_active_slot(3),
                    KeyCode::Digit5 if pressed => self.set_active_slot(4),
                    KeyCode::Digit6 if pressed => self.set_active_slot(5),
                    KeyCode::Digit7 if pressed => self.set_active_slot(6),
                    KeyCode::Digit8 if pressed => self.set_active_slot(7),
                    KeyCode::Digit9 if pressed => self.set_active_slot(8),
                    KeyCode::Escape if pressed => {
                        if let Some(ctx) = &self.render_ctx {
                            let window = Arc::clone(&ctx.window);
                            self.toggle_mouse_grab(&window);
                        }
                    }
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last_frame).as_secs_f32().min(0.1);
                self.last_frame = now;

                self.update_physics(dt);

                if let Some(ctx) = &mut self.render_ctx {
                    ctx.update_camera(&self.camera);
                    if let Err(e) = ctx.render(self.active_slot) {
                        eprintln!("Render error: {e:?}");
                    }
                    ctx.window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        if !self.mouse_captured {
            return;
        }

        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            let sensitivity = 0.15;
            let new_yaw = self.camera.yaw + (dx as f32) * sensitivity;
            let new_pitch = self.camera.pitch - (dy as f32) * sensitivity;
            self.camera.set_rotation(new_yaw, new_pitch);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = GameApp::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
