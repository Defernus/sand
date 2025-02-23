use crate::*;
use macroquad::prelude::*;

pub struct GameState {
    pub world: WorldState,
    pub selected_cell: usize,
    pub ticks_per_frame: u16,
    pub spawn_mode: SpawnMode,
    pub cell_variants: Vec<&'static str>,
    pub camera: WorldCamera,
    pub camera_speed: f32,
    pub camera_fast_speed: f32,

    pub last_chunks_drawn: usize,
    pub last_chunks_updated: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, strum::Display)]
pub enum SpawnMode {
    Single,
    Circle,
}

macro_rules! is_pressed {
    ($($key:ident),*) => {
        $(is_key_pressed(KeyCode:: $key) ||)* false
    };
}

impl GameState {
    pub fn new() -> Self {
        Self {
            world: WorldState::default(),

            cell_variants: CELLS.iter().map(|cell| cell.name).collect(),

            spawn_mode: SpawnMode::Circle,

            camera: WorldCamera::new(Vec2::ZERO, 2.0),

            selected_cell: 1,
            ticks_per_frame: 1,

            camera_speed: 100.0,
            camera_fast_speed: 500.0,

            last_chunks_drawn: 0,
            last_chunks_updated: 0,
        }
    }

    pub fn draw_to_screen(&mut self) {
        let (min, max) = self.camera.get_screen_chunks_area();
        self.last_chunks_drawn = 0;

        for chunk_x in min.x..=max.x {
            for chunk_y in min.y..=max.y {
                let chunk_pos = ChunkPos::new(chunk_x, chunk_y);

                self.draw_chunk_to_screen(chunk_pos);

                self.last_chunks_drawn += 1;
            }
        }
    }

    pub fn draw_chunk_to_screen(&mut self, chunk_pos: ChunkPos) {
        let offset = self.camera.chunk_pos_to_screen_cord(chunk_pos);
        let chunk_size = self.camera.chunk_screen_size();

        let chunk = self.world.ensure_chunk(chunk_pos);

        let texture = chunk.get_texture();

        const BG_COLOR_1: Color = Color::new(0.4, 0.4, 0.4, 1.0);
        const BG_COLOR_2: Color = Color::new(0.7, 0.7, 0.7, 1.0);
        draw_rectangle(
            offset.x,
            offset.y,
            chunk_size.x * 0.5,
            chunk_size.y * 0.5,
            BG_COLOR_1,
        );
        draw_rectangle(
            offset.x,
            offset.y + chunk_size.y * 0.5,
            chunk_size.x * 0.5,
            chunk_size.y * 0.5,
            BG_COLOR_2,
        );
        draw_rectangle(
            offset.x + chunk_size.x * 0.5,
            offset.y,
            chunk_size.x * 0.5,
            chunk_size.y * 0.5,
            BG_COLOR_2,
        );
        draw_rectangle(
            offset.x + chunk_size.x * 0.5,
            offset.y + chunk_size.x * 0.5,
            chunk_size.x * 0.5,
            chunk_size.y * 0.5,
            BG_COLOR_1,
        );

        draw_texture_ex(
            texture,
            offset.x,
            offset.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(chunk_size),

                ..Default::default()
            },
        );

        let text_color = if chunk.should_update { RED } else { WHITE };
        draw_text(
            &format!("{} {}", chunk_pos.x, chunk_pos.y),
            offset.x + chunk_size.x / 2.0,
            offset.y + chunk_size.x / 2.0,
            10.0,
            text_color,
        );
    }

    pub fn on_frame(&mut self) {
        for _ in 0..self.ticks_per_frame {
            self.last_chunks_updated = self.world.update_state();
        }

        self.camera.resize(vec2(screen_width(), screen_height()));

        let dt = get_frame_time();

        self.handle_change_scale();
        self.handle_tick_speed_selection();
        self.handle_cell_selection();
        self.handle_spawn_cells();
        self.handle_spawn_mode_selection();
        self.handle_move_camera(dt);
    }

    pub fn handle_move_camera(&mut self, dt: f32) {
        let mut camera_move = Vec2::ZERO;

        if is_key_down(KeyCode::W) {
            camera_move.y += 1.0;
        }
        if is_key_down(KeyCode::S) {
            camera_move.y -= 1.0;
        }
        if is_key_down(KeyCode::D) {
            camera_move.x += 1.0;
        }
        if is_key_down(KeyCode::A) {
            camera_move.x -= 1.0;
        }

        if camera_move != Vec2::ZERO {
            let speed = if is_key_down(KeyCode::LeftShift) {
                self.camera_fast_speed
            } else {
                self.camera_speed
            };

            let camera_move = camera_move.normalize() * speed * dt;
            self.camera.position += camera_move;
        }
    }

    pub fn handle_tick_speed_selection(&mut self) {
        if is_pressed!(Up) {
            self.ticks_per_frame += 1;
        }

        if is_pressed!(Down) {
            self.ticks_per_frame = self.ticks_per_frame.saturating_sub(1);
        }
    }

    pub fn handle_cell_selection(&mut self) {
        if is_pressed!(Left) {
            if self.selected_cell == 0 {
                self.selected_cell = self.cell_variants.len() - 1;
            } else {
                self.selected_cell -= 1;
            }
        }
        if is_pressed!(Right) {
            self.selected_cell += 1;
            if self.selected_cell >= self.cell_variants.len() {
                self.selected_cell = 0;
            }
        }
    }

    pub fn handle_spawn_mode_selection(&mut self) {
        if is_pressed!(Space) {
            self.spawn_mode = match self.spawn_mode {
                SpawnMode::Single => SpawnMode::Circle,
                SpawnMode::Circle => SpawnMode::Single,
            };
        }
    }

    pub fn world_mouse_position(&self) -> GlobalCellPos {
        let (x, y) = mouse_position();

        self.camera.screen_cord_to_global_pos(vec2(x, y))
    }

    pub fn handle_spawn_cells(&mut self) {
        let condition = match self.spawn_mode {
            SpawnMode::Single => is_mouse_button_pressed(MouseButton::Left),
            SpawnMode::Circle => is_mouse_button_down(MouseButton::Left),
        };
        if !condition {
            return;
        }

        let position = self.world_mouse_position();

        let selected_cell_name = self.cell_variants[self.selected_cell];
        let cell = CELLS
            .iter()
            .find(|cell| cell.name == selected_cell_name)
            .expect("Cell not found");

        match self.spawn_mode {
            SpawnMode::Single => {
                println!(
                    "spawn at: {:?}\n({}, {})\n\t{:?}\n\t{:?}",
                    position,
                    position.x(),
                    position.y(),
                    self.camera.screen_pos_to_world_pos(mouse_position().into()),
                    self.camera
                        .screen_pos_to_world_pos(mouse_position().into())
                        .floor()
                );
                self.world.set_cell(position, cell.init());
            }
            SpawnMode::Circle => {
                let radius = 1;
                for y in -radius..=radius {
                    for x in -radius..=radius {
                        let position = position + RelativePos::new(x, y);
                        self.world.set_cell(position, cell.init());
                    }
                }
            }
        }
    }

    pub fn draw_debug_text(&self) {
        let x = 10.0;
        let regular_font_size = 16.0;

        let mut y = 00.0;
        macro_rules! next_y {
            () => {{
                y += 20.0;
                y
            }};
        }

        macro_rules! draw_debug_line(
            ($($arg:tt)*) => {
                draw_text_shadow(&format!($( $arg )*), x, next_y!(), regular_font_size, WHITE);
            };
        );

        let fps = get_fps();
        let fps = format!("FPS: {fps}");
        // draw_text(&fps, x, next_y!(), 16.0, WHITE);
        draw_debug_line!("FPS: {fps}");

        let selected_cell = self.cell_variants[self.selected_cell];
        draw_debug_line!("Cell to spawn (left/right to change): {selected_cell}");

        draw_debug_line!(
            "Ticks per frame (up/down to change): {}",
            self.ticks_per_frame
        );

        draw_debug_line!("Chunks drawn: {}", self.last_chunks_drawn);

        draw_debug_line!("Chunks updated: {}", self.last_chunks_updated);

        draw_debug_line!("Chunks loaded: {}", self.world.len());

        let (min, max) = self.camera.get_screen_chunks_area();
        draw_debug_line!(
            "Screen chunks: ({}, {}) - ({}, {})",
            min.x,
            min.y,
            max.x,
            max.y
        );

        let mouse_pos = self.world_mouse_position();
        draw_debug_line!("Mouse pos: ({}, {})", mouse_pos.x(), mouse_pos.y());

        // chunk and cell position
        draw_debug_line!("Chunk pos: ({}, {})", mouse_pos.chunk.x, mouse_pos.chunk.y);
        draw_debug_line!("Cell pos: ({}, {})", mouse_pos.cell.x, mouse_pos.cell.y);

        draw_debug_line!("Spawn mode: {:?}", self.spawn_mode);
    }

    pub fn handle_change_scale(&mut self) {
        let mouse_wheel = mouse_wheel().1;

        if mouse_wheel != 0.0 {
            let scale = self.camera.cell_size.powi(2);
            let scale = scale + (mouse_wheel * 0.1);
            let new_size = scale.sqrt();
            self.camera.cell_size = new_size.max(0.1).min(10.0);
        }
    }
}
