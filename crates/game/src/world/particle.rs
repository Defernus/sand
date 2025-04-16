use crate::*;
use macroquad::math::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Particle {
    pub vel: Vec2,
    /// Position of the particle inside the chunk
    pub in_chunk_pos: Vec2,
    pub age: u32,
    pub color: [u8; 4],
    pub gravity: Vec2,
    pub cell_id: CellId,
}

impl Particle {
    /// Return particle position in the chunk. If the particle is outside the chunk, return None
    #[inline(always)]
    pub fn get_cell_pos(&self) -> Option<CellPos> {
        if self.in_chunk_pos.x < 0.0
            || self.in_chunk_pos.x >= CHUNK_SIZE as f32
            || self.in_chunk_pos.y < 0.0
            || self.in_chunk_pos.y >= CHUNK_SIZE as f32
        {
            return None;
        }

        let x = self.in_chunk_pos.x as CellCord;
        let y = self.in_chunk_pos.y as CellCord;

        Some(CellPos::new(x, y))
    }

    /// Apply velocity and gravity to the particle
    pub fn update_pos(&mut self, dt: f32) {
        self.age += 1;
        self.in_chunk_pos += self.vel * dt;
        self.vel += self.gravity * dt;

        self.validate_pos_in_update_region();
    }

    pub fn validate_pos_in_update_region(&self) {
        let size_f = CHUNK_SIZE as f32;
        assert!(
            self.in_chunk_pos.x > -size_f * 0.5,
            "Particle is moving too fast! x = {:?}",
            self.in_chunk_pos.x
        );
        assert!(
            self.in_chunk_pos.x < size_f * 1.5,
            "Particle is moving too fast! x = {:?}",
            self.in_chunk_pos.x
        );
        assert!(
            self.in_chunk_pos.y > -size_f * 0.5,
            "Particle is moving too fast! y = {:?}",
            self.in_chunk_pos.y
        );
        assert!(
            self.in_chunk_pos.y < size_f * 1.5,
            "Particle is moving too fast! y = {:?}",
            self.in_chunk_pos.y
        );
    }
}
