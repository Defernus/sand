use crate::*;
use macroquad::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct WorldCamera {
    pub position: Vec2,
    /// How much space one cell takes on the screen (in pixels)
    pub cell_size: f32,

    pub screen_size: Vec2,
}

impl WorldCamera {
    pub fn new(position: Vec2, cell_size: f32) -> Self {
        Self {
            position,
            cell_size,
            screen_size: Vec2::new(1.0, 1.0),
        }
    }

    pub fn resize(&mut self, screen_size: Vec2) {
        self.screen_size = screen_size;
    }

    pub fn screen_pos_to_world_pos(&self, screen_pos: Vec2) -> Vec2 {
        vec2(
            screen_pos.x - self.screen_size.x / 2.0,
            self.screen_size.y / 2.0 - screen_pos.y,
        ) / self.cell_size
            + self.position
    }

    pub fn world_pos_to_screen_pos(&self, pos: Vec2) -> Vec2 {
        let pos = (pos - self.position) * self.cell_size;

        vec2(
            pos.x + self.screen_size.x / 2.0,
            self.screen_size.y / 2.0 - pos.y,
        )
    }

    /// Returns cell position at the screen coordinates
    pub fn screen_cord_to_global_pos(&self, pos: Vec2) -> GlobalCellPos {
        let pos = self.screen_pos_to_world_pos(pos);

        GlobalCellPos::new(pos.x.floor() as i32, pos.y.floor() as i32)
    }

    pub fn chunk_pos_to_screen_cord(&self, pos: ChunkPos) -> Vec2 {
        let pos = vec2(pos.x as f32, pos.y as f32 + 1.0) * CHUNK_SIZE as f32;

        self.world_pos_to_screen_pos(pos)
    }

    /// Returns min and max cell coordinates on the screen
    pub fn get_screen_chunks_area(&self) -> (ChunkPos, ChunkPos) {
        let min = self.min_world_pos();
        let max = self.max_world_pos();

        let min = ChunkPos::new(
            (min.x / CHUNK_SIZE as f32).floor() as i32,
            (min.y / CHUNK_SIZE as f32).floor() as i32,
        );
        let max = ChunkPos::new(
            (max.x / CHUNK_SIZE as f32).ceil() as i32,
            (max.y / CHUNK_SIZE as f32).ceil() as i32,
        );

        (min, max)
    }

    pub fn chunk_screen_size(&self) -> Vec2 {
        vec2(CHUNK_SIZE as f32, CHUNK_SIZE as f32) * self.cell_size
    }

    pub fn min_world_pos(&self) -> Vec2 {
        self.screen_pos_to_world_pos(vec2(0.0, self.screen_size.y))
    }

    pub fn max_world_pos(&self) -> Vec2 {
        self.screen_pos_to_world_pos(vec2(self.screen_size.x, 0.0))
    }
}
