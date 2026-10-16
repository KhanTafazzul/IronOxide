mod audio;
mod game;
mod render;
pub mod storage;
pub mod world;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

use audio::{AudioSystem, FootstepTracker, SoundEvent, SoundType};
use game::inventory::{ItemStack, PlayerInventory};
use game::stats::PlayerStats;
use game::{
    raycast_voxel, Aabb, BlockMiningTracker, ItemEntityManager, MobEcs, MobEvent, MiningUpdateResult, PlayerInput,
    PlayerPhysics, World, DEFAULT_MAX_REACH,
};
use render::atlas::{BlockType, TextureAtlas};
use render::gui::inventory::InventoryGui;
use render::mesher::{greedy_mesh_section, ChunkMesh};
use render::{Camera, RenderContext};
use storage::WorldStorageManager;

struct GameApp<'a> {
    render_ctx: Option<RenderContext<'a>>,
    camera: Camera,
    player: PlayerPhysics,
    world: World,
    storage: WorldStorageManager,
    last_save_time: Instant,
    atlas: Arc<TextureAtlas>,
    meshes: HashMap<glam::IVec3, ChunkMesh>,
    selected_block: BlockType,
    active_slot: usize,
    hotbar_slots: [BlockType; 9],
    inventory: PlayerInventory,
    inventory_gui: InventoryGui,
    stats: PlayerStats,
    mining_tracker: BlockMiningTracker,
    item_entities: ItemEntityManager,
    mobs: MobEcs,
    left_mouse_held: bool,
    mouse_captured: bool,
    audio: AudioSystem,
    footstep_tracker: FootstepTracker,
    last_player_pos: glam::Vec3,
    last_hit_sound_time: Instant,
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
        let mut storage = WorldStorageManager::new("WorldName");
        let mut world = World::new();

        // Check if spawn chunk column (0, 0) exists in persistent .mca region storage
        let mut loaded = false;
        if let Ok(Some(sections)) = storage.load_chunk_column(0, 0) {
            println!("[Storage] Loaded {} sections from Anvil region file for chunk (0, 0)", sections.len());
            for (sec_y, sec, light_opt) in sections {
                let sec_pos = glam::IVec3::new(0, sec_y, 0);
                world.sections.insert(sec_pos, sec);
                if let Some(light) = light_opt {
                    world.lighting.sections.insert(sec_pos, light);
                }
            }
            loaded = true;
        }

        if !loaded {
            // Build initial ground platform at y = 0
            for x in 0..16 {
                for z in 0..16 {
                    world.set_block_at(glam::IVec3::new(x, 0, z), BlockType::GrassBlock);
                }
            }
            // Stepping blocks to test interaction and step-up
            world.set_block_at(glam::IVec3::new(8, 1, 8), BlockType::Stone);
            world.set_block_at(glam::IVec3::new(8, 1, 9), BlockType::Cobblestone);
            // Point light source: Torch emits light level 14
            world.set_block_at(glam::IVec3::new(8, 2, 8), BlockType::Torch);

            // Initialize daylight / skylight across the spawn platform
            for x in 0..16 {
                for z in 0..16 {
                    world.lighting.update_skylight_column(x, z, 15, 0, &world.sections);
                }
            }

            // Mark spawn chunk dirty so it will be saved to saves/WorldName/region/r.0.0.mca
            storage.mark_dirty(0, 0);
        }

        let mut meshes = HashMap::new();
        for (&sec_pos, sec) in &world.sections {
            let mesh = greedy_mesh_section(sec, &atlas, &world);
            meshes.insert(sec_pos, mesh);
        }

        let spawn_pos = glam::Vec3::new(8.0, 1.0, 5.0);
        let player = PlayerPhysics::new(spawn_pos);
        let camera = Camera::new(player.eye_position(), 0.0, 0.0, 70.0);

        let hotbar_slots = [
            BlockType::Torch,
            BlockType::Cobblestone,
            BlockType::Dirt,
            BlockType::Stone,
            BlockType::Deepslate,
            BlockType::OakPlanks,
            BlockType::OakLeaves,
            BlockType::Glass,
            BlockType::Lava,
        ];
        let active_slot = 0;
        let selected_block = hotbar_slots[active_slot];

        let mut inventory = PlayerInventory::new();
        // Sync default hotbar item stacks
        for (i, &block) in hotbar_slots.iter().enumerate() {
            if let Some(stack) = ItemStack::from_block(block, 64) {
                inventory.hotbar[i] = Some(stack);
            }
        }
        let inventory_gui = InventoryGui::new();
        let mining_tracker = BlockMiningTracker::new();
        let audio = AudioSystem::new();
        let footstep_tracker = FootstepTracker::new();
        let last_player_pos = spawn_pos;
        let last_hit_sound_time = Instant::now();

        let mut mobs = MobEcs::new();
        mobs.spawn_zombie(glam::Vec3::new(12.0, 1.0, 12.0));

        Self {
            render_ctx: None,
            camera,
            player,
            world,
            storage,
            last_save_time: Instant::now(),
            atlas,
            meshes,
            selected_block,
            active_slot,
            hotbar_slots,
            inventory,
            inventory_gui,
            stats: PlayerStats::new(),
            mining_tracker,
            item_entities: ItemEntityManager::new(),
            mobs,
            left_mouse_held: false,
            mouse_captured: false,
            audio,
            footstep_tracker,
            last_player_pos,
            last_hit_sound_time,
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
        if let Some(stack) = self.inventory.hotbar[self.active_slot] {
            if let Some(block) = stack.to_block() {
                self.selected_block = block;
            }
        } else {
            self.selected_block = self.hotbar_slots[self.active_slot];
        }

        let item_name = self.inventory.hotbar[self.active_slot]
            .map(|s| s.name())
            .unwrap_or("Empty");
        println!(
            "Active Hotbar Slot {}: {:?} (Item: {})",
            self.active_slot + 1,
            self.selected_block,
            item_name
        );
    }

    /// Add an item stack to player inventory (hotbar first, then main storage)
    fn add_item_to_inventory(&mut self, stack: ItemStack) {
        if let Some(rem) = self.inventory.add_item(stack) {
            println!("Inventory full! Uncollected items: {:?}", rem);
        }
    }

    fn update_physics(&mut self, _dt: f32) {
        if self.inventory_gui.is_open {
            return;
        }

        let forward = if self.key_w { 1.0 } else if self.key_s { -1.0 } else { 0.0 };
        let strafe = if self.key_a { 1.0 } else if self.key_d { -1.0 } else { 0.0 };
        let input = PlayerInput {
            forward,
            strafe,
            jump: self.key_space,
            sprint: self.key_shift,
            yaw_degrees: self.camera.yaw,
        };

        // Physical activity triggers: Jumping adds 0.05 exhaustion (sprint-jump adds 0.2)
        if input.jump && self.player.on_ground {
            self.stats.on_jump(input.sprint);
        }

        self.player.tick(&input, &self.world);
        self.camera.eye = self.player.eye_position();

        // 3D Footstep sound trigger based on walking displacement
        let current_pos = self.player.position;
        let dx = current_pos.x - self.last_player_pos.x;
        let dz = current_pos.z - self.last_player_pos.z;
        let horizontal_disp = (dx * dx + dz * dz).sqrt();
        self.last_player_pos = current_pos;

        // Physical activity triggers: Sprinting adds 0.1 exhaustion per meter
        if input.sprint && horizontal_disp > 0.0 {
            self.stats.on_sprint_move(horizontal_disp);
        }

        if self.footstep_tracker.update(horizontal_disp, self.player.on_ground) {
            let block_under_coord = glam::IVec3::new(
                current_pos.x.floor() as i32,
                (current_pos.y - 0.2).floor() as i32,
                current_pos.z.floor() as i32,
            );
            let block_under = self.world.get_block(block_under_coord);
            let sound_type = SoundType::for_block(block_under);
            let pitch = self.audio.random_pitch(sound_type.pitch);
            self.audio.play_spatial(
                sound_type.step_sound,
                current_pos,
                self.camera.eye,
                self.camera.forwards(),
                self.camera.up(),
                0.35 * sound_type.volume,
                pitch,
            );
        }
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
            self.left_mouse_held = false;
            self.mining_tracker.reset();
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
                self.storage.mark_sections_dirty(&affected);

                // Consume placed block from hotbar slot
                if let Some(mut stack) = self.inventory.hotbar[self.active_slot] {
                    if stack.to_block() == Some(self.selected_block) {
                        stack.count = stack.count.saturating_sub(1);
                        if stack.count == 0 {
                            self.inventory.hotbar[self.active_slot] = None;
                        } else {
                            self.inventory.hotbar[self.active_slot] = Some(stack);
                        }
                    }
                }
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
            .with_title("IronOxide - Minecraft Rust Client (WGPU)")
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
                self.storage.flush_all(&self.world);
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(ctx) = &mut self.render_ctx {
                    ctx.resize(physical_size);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(ctx) = &self.render_ctx {
                    self.inventory_gui.update_mouse(
                        position.x as f32,
                        position.y as f32,
                        ctx.config.width as f32,
                        ctx.config.height as f32,
                    );
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if self.inventory_gui.is_open {
                    return;
                }
                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_x, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => (pos.y as f32).signum(),
                };
                if scroll > 0.0 {
                    let prev_slot = (self.active_slot + 8) % 9;
                    self.set_active_slot(prev_slot);
                } else if scroll < 0.0 {
                    let next_slot = (self.active_slot + 1) % 9;
                    self.set_active_slot(next_slot);
                }
            }
            WindowEvent::MouseInput { button, state, .. } => {
                match state {
                    ElementState::Pressed => {
                        if self.inventory_gui.is_open {
                            self.inventory_gui.handle_mouse_click(button, &mut self.inventory);
                            // Update active block from current hotbar slot
                            if let Some(stack) = self.inventory.hotbar[self.active_slot] {
                                if let Some(block) = stack.to_block() {
                                    self.selected_block = block;
                                }
                            }
                        } else if !self.mouse_captured {
                            if let Some(ctx) = &self.render_ctx {
                                let window = Arc::clone(&ctx.window);
                                self.toggle_mouse_grab(&window);
                            }
                        } else {
                            match button {
                                MouseButton::Left => {
                                    self.left_mouse_held = true;
                                    // Punch sound on air miss
                                    let hit = raycast_voxel(
                                        self.camera.eye,
                                        self.camera.forwards(),
                                        DEFAULT_MAX_REACH,
                                        &self.world,
                                    );
                                    if hit.is_none() {
                                        let pitch = self.audio.random_pitch(1.0);
                                        self.audio.play_local(SoundEvent::PlayerPunchAir, 0.45, pitch);
                                    }
                                }
                                MouseButton::Right => self.place_targeted_block(),
                                _ => {}
                            }
                        }
                    }
                    ElementState::Released => {
                        if button == MouseButton::Left {
                            self.left_mouse_held = false;
                            self.mining_tracker.reset();
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
                    // Open/close inventory on 'E'
                    KeyCode::KeyE if pressed => {
                        if let Some(ctx) = &self.render_ctx {
                            let window = Arc::clone(&ctx.window);
                            self.inventory_gui.toggle(&window);
                            self.mouse_captured = !self.inventory_gui.is_open;
                            self.left_mouse_held = false;
                            self.mining_tracker.reset();
                            if !self.inventory_gui.is_open {
                                if let Some(stack) = self.inventory.hotbar[self.active_slot] {
                                    if let Some(block) = stack.to_block() {
                                        self.selected_block = block;
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Escape if pressed => {
                        if let Some(ctx) = &self.render_ctx {
                            let window = Arc::clone(&ctx.window);
                            if self.inventory_gui.is_open {
                                self.inventory_gui.close(&window);
                                self.mouse_captured = true;
                                self.left_mouse_held = false;
                                self.mining_tracker.reset();
                                if let Some(stack) = self.inventory.hotbar[self.active_slot] {
                                    if let Some(block) = stack.to_block() {
                                        self.selected_block = block;
                                    }
                                }
                            } else {
                                self.toggle_mouse_grab(&window);
                            }
                        }
                    }
                    KeyCode::KeyW => self.key_w = pressed && !self.inventory_gui.is_open,
                    KeyCode::KeyS => self.key_s = pressed && !self.inventory_gui.is_open,
                    KeyCode::KeyA => self.key_a = pressed && !self.inventory_gui.is_open,
                    KeyCode::KeyD => self.key_d = pressed && !self.inventory_gui.is_open,
                    KeyCode::Space => self.key_space = pressed && !self.inventory_gui.is_open,
                    KeyCode::ShiftLeft | KeyCode::ShiftRight => self.key_shift = pressed && !self.inventory_gui.is_open,
                    // Hotbar slot switching (1 to 9)
                    KeyCode::Digit1 if pressed && !self.inventory_gui.is_open => self.set_active_slot(0),
                    KeyCode::Digit2 if pressed && !self.inventory_gui.is_open => self.set_active_slot(1),
                    KeyCode::Digit3 if pressed && !self.inventory_gui.is_open => self.set_active_slot(2),
                    KeyCode::Digit4 if pressed && !self.inventory_gui.is_open => self.set_active_slot(3),
                    KeyCode::Digit5 if pressed && !self.inventory_gui.is_open => self.set_active_slot(4),
                    KeyCode::Digit6 if pressed && !self.inventory_gui.is_open => self.set_active_slot(5),
                    KeyCode::Digit7 if pressed && !self.inventory_gui.is_open => self.set_active_slot(6),
                    KeyCode::Digit8 if pressed && !self.inventory_gui.is_open => self.set_active_slot(7),
                    KeyCode::Digit9 if pressed && !self.inventory_gui.is_open => self.set_active_slot(8),
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last_frame).as_secs_f32().min(0.1);
                self.last_frame = now;

                self.update_physics(dt);
                self.stats.update(dt);

                // REA Verified: Mining progress tracking & block break resolution
                let raycast_hit = if self.inventory_gui.is_open || !self.mouse_captured {
                    None
                } else {
                    raycast_voxel(
                        self.camera.eye,
                        self.camera.forwards(),
                        DEFAULT_MAX_REACH,
                        &self.world,
                    )
                };

                let held_item = self.inventory.hotbar[self.active_slot];
                let raycast_target = raycast_hit.map(|h| (h.block_pos, h.face, self.world.get_block(h.block_pos)));

                let mining_result = self.mining_tracker.update(dt, self.left_mouse_held, raycast_target, held_item);
                let mut active_mining_target = None;

                match mining_result {
                    MiningUpdateResult::Mining { pos, stage, .. } => {
                        active_mining_target = Some((pos, stage));
                        // 3D Spatial block hit sound every 220ms during mining
                        if now.duration_since(self.last_hit_sound_time) >= std::time::Duration::from_millis(220) {
                            self.last_hit_sound_time = now;
                            let target_block = self.world.get_block(pos);
                            let sound_type = SoundType::for_block(target_block);
                            let pitch = self.audio.random_pitch(sound_type.pitch);
                            self.audio.play_spatial(
                                sound_type.hit_sound,
                                pos.as_vec3() + glam::Vec3::splat(0.5),
                                self.camera.eye,
                                self.camera.forwards(),
                                self.camera.up(),
                                0.5 * sound_type.volume,
                                pitch,
                            );
                        }
                    }
                    MiningUpdateResult::Broken { pos, block, .. } => {
                        println!("Block broken: {:?} at {:?}", block, pos);
                        let affected = self.world.set_block_at(pos, BlockType::Air);
                        self.remesh_sections(&affected);
                        self.storage.mark_sections_dirty(&affected);

                        // Physical activity trigger: Breaking a block adds 0.005 exhaustion
                        self.stats.on_block_break();

                        // 3D Spatial block break sound
                        let sound_type = SoundType::for_block(block);
                        let pitch = self.audio.random_pitch(sound_type.pitch);
                        self.audio.play_spatial(
                            sound_type.break_sound,
                            pos.as_vec3() + glam::Vec3::splat(0.5),
                            self.camera.eye,
                            self.camera.forwards(),
                            self.camera.up(),
                            0.85 * sound_type.volume,
                            pitch,
                        );

                        // Spawn 12-20 physics-affected voxel particles with random outward velocities
                        if let Some(ctx) = &mut self.render_ctx {
                            ctx.effects_renderer.particles.spawn_block_break(pos, block);
                        }

                        // Spawn 1/4-scale dropped ItemEntity at block center with upward and lateral velocity
                        self.item_entities.spawn_block_drop(pos, block);
                    }
                    MiningUpdateResult::Idle => {}
                }

                // Asynchronously flush modified chunks every 2 seconds to prevent game freezes
                if now.duration_since(self.last_save_time) >= std::time::Duration::from_secs(2) {
                    self.last_save_time = now;
                    self.storage.flush_dirty(&self.world);
                }

                // Update dropped items: AABB collision, gravity, player magnetism, and pickup resolution
                let pickups = self.item_entities.update(
                    self.camera.eye,
                    &mut self.inventory,
                    &self.world,
                );
                for pickup_pos in pickups {
                    let pitch = self.audio.random_pitch(1.3);
                    self.audio.play_spatial(
                        SoundEvent::Pop,
                        pickup_pos,
                        self.camera.eye,
                        self.camera.forwards(),
                        self.camera.up(),
                        1.0,
                        pitch,
                    );
                }

                // Tick mob ECS: state machine (Idle, Targeting, Attack), A* pathfinding, and damage
                let mob_events = self.mobs.tick(self.camera.eye, &mut self.stats, &self.world);
                for event in mob_events {
                    match event {
                        MobEvent::ZombieAttack { pos, .. } => {
                            let pitch = self.audio.random_pitch(0.85);
                            self.audio.play_spatial(
                                SoundEvent::GrassHit,
                                pos,
                                self.camera.eye,
                                self.camera.forwards(),
                                self.camera.up(),
                                1.0,
                                pitch,
                            );
                        }
                    }
                }

                if let Some(ctx) = &mut self.render_ctx {
                    ctx.effects_renderer.particles.update(dt, &self.world);
                    ctx.update_camera(&self.camera);
                    let camera_right = self.camera.right();
                    let camera_up = self.camera.up();
                    if let Err(e) = ctx.render(
                        self.active_slot,
                        Some((&self.inventory_gui, &self.inventory)),
                        active_mining_target,
                        camera_right,
                        camera_up,
                        Some(&self.stats),
                        self.item_entities.active_items(),
                        self.mobs.mobs(),
                    ) {
                        eprintln!("Render error: {e:?}");
                    }
                    ctx.window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        if !self.mouse_captured || self.inventory_gui.is_open {
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
