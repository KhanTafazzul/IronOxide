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

use game::inventory::{ItemStack, PlayerInventory};
use game::{raycast_voxel, Aabb, PlayerInput, PlayerPhysics, World, DEFAULT_MAX_REACH};
use render::atlas::{BlockType, TextureAtlas};
use render::gui::inventory::InventoryGui;
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
    inventory: PlayerInventory,
    inventory_gui: InventoryGui,
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
            BlockType::OakLog,
            BlockType::Cobblestone,
            BlockType::Dirt,
            BlockType::Stone,
            BlockType::Deepslate,
            BlockType::OakPlanks,
            BlockType::OakLeaves,
            BlockType::Glass,
            BlockType::Bricks,
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
            inventory,
            inventory_gui,
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
    fn add_item_to_inventory(&mut self, mut stack: ItemStack) {
        if stack.is_empty() {
            return;
        }

        // 1. Try to merge with existing stacks in hotbar
        for slot in &mut self.inventory.hotbar {
            if let Some(mut existing) = *slot {
                if existing.can_stack_with(&stack) && existing.count < existing.max_stack {
                    let space = existing.max_stack - existing.count;
                    let to_add = stack.count.min(space);
                    existing.count += to_add;
                    stack.count -= to_add;
                    *slot = Some(existing);
                    if stack.count == 0 {
                        return;
                    }
                }
            }
        }

        // 2. Try to merge with existing stacks in main storage
        for slot in &mut self.inventory.main {
            if let Some(mut existing) = *slot {
                if existing.can_stack_with(&stack) && existing.count < existing.max_stack {
                    let space = existing.max_stack - existing.count;
                    let to_add = stack.count.min(space);
                    existing.count += to_add;
                    stack.count -= to_add;
                    *slot = Some(existing);
                    if stack.count == 0 {
                        return;
                    }
                }
            }
        }

        // 3. Find first empty slot in hotbar
        for slot in &mut self.inventory.hotbar {
            if slot.is_none() {
                *slot = Some(stack);
                return;
            }
        }

        // 4. Find first empty slot in main storage
        for slot in &mut self.inventory.main {
            if slot.is_none() {
                *slot = Some(stack);
                return;
            }
        }

        println!("Inventory full! Dropped item: {:?}", stack);
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

    /// Left-Click: Destroy targeted block, drop item into inventory, and trigger re-meshing
    fn break_targeted_block(&mut self) {
        if let Some(hit) = raycast_voxel(
            self.camera.eye,
            self.camera.forwards(),
            DEFAULT_MAX_REACH,
            &self.world,
        ) {
            let broken_block = self.world.get_block(hit.block_pos);
            println!("Left Click: Destroying {:?} at {:?}", broken_block, hit.block_pos);
            let affected = self.world.set_block_at(hit.block_pos, BlockType::Air);
            self.remesh_sections(&affected);

            // Vanilla-style block drop
            let drop = match broken_block {
                BlockType::GrassBlock => ItemStack::new(game::inventory::ITEM_DIRT, 1),
                BlockType::Stone => ItemStack::new(game::inventory::ITEM_COBBLESTONE, 1),
                other => ItemStack::from_block(other, 1)
                    .unwrap_or(ItemStack::new(game::inventory::ITEM_DIRT, 1)),
            };
            self.add_item_to_inventory(drop);
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
                if state == ElementState::Pressed {
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
                    // Open/close inventory on 'E'
                    KeyCode::KeyE if pressed => {
                        if let Some(ctx) = &self.render_ctx {
                            let window = Arc::clone(&ctx.window);
                            self.inventory_gui.toggle(&window);
                            self.mouse_captured = !self.inventory_gui.is_open;
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

                if let Some(ctx) = &mut self.render_ctx {
                    ctx.update_camera(&self.camera);
                    if let Err(e) = ctx.render(
                        self.active_slot,
                        Some((&self.inventory_gui, &self.inventory)),
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
