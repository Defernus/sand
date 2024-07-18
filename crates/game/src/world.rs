use crate::*;
use macroquad::prelude::*;
use nohash_hasher::IntMap;

#[derive(Default)]
pub struct World {
    pub chunks: IntMap<ChunkPosition, Chunk>,

    /// Current update id. Incremented every time world is updated.
    pub update_id: u64,
}

impl World {
    /// Spawn cells in area with given position, radius and density.
    pub fn spawn_cells(
        &mut self,
        cell: CellType,
        position: CellPosition,
        radius: usize,
        density: f32,
    ) {
        let radius = radius as i32;
        let r_squared = radius * radius;

        for x in -radius..radius {
            for y in -radius..radius {
                if x * x + y * y > r_squared {
                    continue;
                }

                if ::rand::random::<f32>() > density {
                    continue;
                }

                let position = position + CellPosition(Position { x, y });

                self.set(position, cell);
            }
        }
    }

    pub fn set_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.position, chunk);
    }

    /// Get or create chunk at given position.
    pub fn ensure_chunk(&mut self, position: ChunkPosition) -> &mut Chunk {
        self.chunks
            .entry(position.into())
            .or_insert_with(|| Chunk::new(position))
    }

    /// Remove chunk at given position and return it.
    pub fn take_chunk(&mut self, position: ChunkPosition) -> Option<Chunk> {
        self.chunks.remove(&position.into())
    }

    /// Set cell at given position. Load region if it's not loaded yet.
    #[inline(always)]
    pub fn set(&mut self, position: CellPosition, cell: CellType) {
        let last_update = self.update_id;

        let chunk = self.ensure_chunk(position.get_chunk_position());

        chunk.set(
            position.get_in_chunk_pos(),
            CellState {
                ty: cell,
                last_update,
            },
        );
    }

    #[inline(always)]
    pub fn draw_to_image(&self, image: &mut Image, offset: CellPosition) {
        for (chunk_position, chunk) in &self.chunks {
            let offset = offset + chunk_position.to_cell_offset();
            chunk.draw_to_image(image, offset);
        }
    }

    #[inline(always)]
    pub fn get(&self, position: CellPosition) -> Option<CellState> {
        let chunk = self.chunks.get(&position.get_chunk_position())?;

        Some(chunk.get(position.get_in_chunk_pos()))
    }
}

pub struct CellNeighbors {
    pub top: CellType,
    pub bottom: CellType,
    pub left: CellType,
    pub right: CellType,
    pub top_left: CellType,
    pub top_right: CellType,
    pub bottom_left: CellType,
    pub bottom_right: CellType,
}

impl CellNeighbors {
    #[inline(always)]
    pub fn get(&self, dir: Direction) -> CellType {
        match dir {
            Direction::Up => self.top,
            Direction::Down => self.bottom,
            Direction::Left => self.left,
            Direction::Right => self.right,
            Direction::UpLeft => self.top_left,
            Direction::UpRight => self.top_right,
            Direction::DownLeft => self.bottom_left,
            Direction::DownRight => self.bottom_right,
        }
    }
}
