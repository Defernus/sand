use crate::*;
use macroquad::prelude::*;

pub struct GameState {
    pub world: World,
    pub cell_variants: Vec<CellType>,
    pub selected_cell: usize,
    pub ticks_per_frame: u16,
    pub spawn_mode: SpawnMode,
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
            world: World::default(),

            cell_variants: vec![
                CellType::Empty,
                CellType::Wall,
                CellType::Sand,
                CellType::Water,
                CellType::Gas,
            ],

            spawn_mode: SpawnMode::Circle,

            selected_cell: 2,
            ticks_per_frame: 4,
        }
    }

    pub fn draw_to_image(&self, image: &mut Image) {
        self.world.draw_to_image(image, CellPosition::default());
    }

    pub fn on_frame(&mut self) {
        for _ in 0..self.ticks_per_frame {
            self.world.update_state();
        }

        self.handle_tick_speed_selection();
        self.handle_cell_selection();
        self.handle_spawn_cells();
        self.handle_spawn_mode_selection();
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

    /// Returns cell position at the screen coordinates
    fn screen_cord_to_position(&self, x: f32, y: f32) -> CellPosition {
        let dpi_scale = screen_dpi_scale();

        let x = x * dpi_scale;
        let y = (screen_height() - y) * dpi_scale;

        Position {
            x: x as i32,
            y: y as i32,
        }
        .into()
    }

    pub fn handle_spawn_cells(&mut self) {
        let condition = match self.spawn_mode {
            SpawnMode::Single => is_mouse_button_pressed(MouseButton::Left),
            SpawnMode::Circle => is_mouse_button_down(MouseButton::Left),
        };
        if !condition {
            return;
        }

        let (x, y) = mouse_position();

        let position = self.screen_cord_to_position(x, y);

        let cell = self.cell_variants[self.selected_cell];

        match self.spawn_mode {
            SpawnMode::Single => {
                self.world.set(position, cell);
            }
            SpawnMode::Circle => {
                self.world.spawn_cells(cell, position, 32, 0.5);
            }
        }
    }

    pub fn draw_debug_text(&self) {
        let x = 10.0;

        let mut y = 00.0;
        macro_rules! next_y {
            () => {{
                y += 20.0;
                y
            }};
        }

        let fps = get_fps();
        let fps = format!("FPS: {fps}");
        draw_text(&fps, x, next_y!(), 16.0, WHITE);

        let selected_cell = self.cell_variants[self.selected_cell];
        let selected_cell = format!("Cell to spawn (left/right to change): {selected_cell}");
        draw_text(&selected_cell, x, next_y!(), 16.0, WHITE);

        let ticks_per_frame = format!(
            "Ticks per frame (up/down to change): {}",
            self.ticks_per_frame
        );
        draw_text(&ticks_per_frame, x, next_y!(), 16.0, WHITE);
    }
}
