use crate::*;
use macroquad::prelude::*;

pub struct GameState {
    pub sandbox: Sandbox,
    pub cell_variants: Vec<Cell>,
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
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            sandbox: Sandbox::new(width, height).with_bottom_wall(),

            cell_variants: vec![
                Cell::Empty,
                Cell::Wall,
                Cell::Sand,
                Cell::Water,
                Cell::Gas,
                Cell::Seed,
            ],

            spawn_mode: SpawnMode::Circle,

            selected_cell: 2,
            ticks_per_frame: 4,
        }
    }

    pub fn on_frame(&mut self) {
        for _ in 0..self.ticks_per_frame {
            self.sandbox.tick();
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

    pub fn handle_spawn_cells(&mut self) {
        let condition = match self.spawn_mode {
            SpawnMode::Single => is_mouse_button_pressed(MouseButton::Left),
            SpawnMode::Circle => is_mouse_button_down(MouseButton::Left),
        };
        if condition {
            let (x, y) = mouse_position();

            let x = x / screen_width();
            let y = 1.0 - (y / screen_height());

            let x = (x * self.sandbox.size.width as f32) as i16;
            let y = (y * self.sandbox.size.height as f32) as i16;

            let cell = self.cell_variants[self.selected_cell];

            match self.spawn_mode {
                SpawnMode::Single => {
                    self.sandbox.set_wrapper(x, y, cell);
                }
                SpawnMode::Circle => {
                    self.sandbox.spawn_cells(cell, x, y, 4, 1.5);
                }
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
