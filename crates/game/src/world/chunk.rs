use crate::*;
use macroquad::prelude::*;
use std::hash::Hash;

/// Size of the chunk's side
pub const CHUNK_SIZE: usize = (u8::MAX as usize + 1) / 2;
/// Area of the chunk
pub const CHUNK_AREA: usize = CHUNK_SIZE * CHUNK_SIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
}

impl Hash for ChunkPos {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(((self.x as u64 & 0xffffffff) << 32) | (self.y as u64 & 0xffffffff));
    }
}

impl nohash_hasher::IsEnabled for ChunkPos {}

impl ChunkPos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    #[inline(always)]
    pub fn left(self) -> Self {
        Self::new(self.x - 1, self.y)
    }
    #[inline(always)]
    pub fn right(self) -> Self {
        Self::new(self.x + 1, self.y)
    }
    #[inline(always)]
    pub fn top(self) -> Self {
        Self::new(self.x, self.y + 1)
    }
    #[inline(always)]
    pub fn bottom(self) -> Self {
        Self::new(self.x, self.y - 1)
    }
    #[inline(always)]
    pub fn left_top(self) -> Self {
        Self::new(self.x - 1, self.y + 1)
    }
    #[inline(always)]
    pub fn right_top(self) -> Self {
        Self::new(self.x + 1, self.y + 1)
    }
    #[inline(always)]
    pub fn left_bottom(self) -> Self {
        Self::new(self.x - 1, self.y - 1)
    }
    #[inline(always)]
    pub fn right_bottom(self) -> Self {
        Self::new(self.x + 1, self.y - 1)
    }
}

#[derive(Debug)]
pub struct Chunk {
    data: Box<[Cell; CHUNK_AREA]>,
    next_random: Box<[u64; CHUNK_AREA]>,
    texture: Option<Texture2D>,
    image: Option<Image>,
    pub should_update: bool,
    pub should_redraw: bool,
}

impl Chunk {
    pub fn new(cells_template: &CellsTemplate) -> Self {
        let mut next_random = Box::new([0; CHUNK_AREA]);
        for i in 0..CHUNK_AREA {
            next_random[i] = ::rand::random();
        }

        Self {
            texture: None,
            image: None,
            data: Box::new([cells_template.cells[0].init(); CHUNK_AREA]),
            next_random,
            should_update: false,
            should_redraw: false,
        }
    }

    /// Get next random value for specific cell
    ///
    /// See [Xorshift](https://en.wikipedia.org/wiki/Xorshift) for more information
    pub fn get_random_value(&mut self, index: usize) -> u64 {
        let mut x = self.next_random[index];

        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;

        self.next_random[index] = x;
        x
    }

    #[inline(always)]
    pub fn get_cell(&self, pos: CellPos) -> Cell {
        self.data[pos.to_index()]
    }

    #[inline(always)]
    pub fn set_cell(&mut self, pos: CellPos, cell: Cell) {
        self.data[pos.to_index()] = cell;
    }

    #[inline(always)]
    pub fn set_by_index(&mut self, index: usize, cell: Cell) {
        let target = &mut self.data[index];
        if *target != cell {
            *target = cell;
            self.should_redraw = true;
            self.should_update = true;
        }
    }

    #[inline(always)]
    /// Swap two cells by their indices
    pub fn swap_by_index(&mut self, a_index: usize, b_index: usize) {
        self.data.swap(a_index, b_index);
    }

    #[inline(always)]
    pub fn get_by_index(&self, index: usize) -> Cell {
        self.data[index]
    }

    #[inline(always)]
    pub fn get_mut_by_index(&mut self, index: usize) -> &mut Cell {
        &mut self.data[index]
    }

    pub fn get_texture(&mut self, cells_template: &CellsTemplate) -> &Texture2D {
        if self.texture.is_none() || self.should_redraw {
            self.should_redraw = false;
            let mut image = Image::gen_image_color(
                CHUNK_SIZE as u16,
                CHUNK_SIZE as u16,
                Color::from_rgba(0, 0, 0, 0),
            );

            for cell_index in 0..CHUNK_AREA {
                let cell = self.get_by_index(cell_index);
                let cell_pos = CellPos::from_index(cell_index);

                // let color = cell.color.;
                // image.set_pixel(
                //     cell_pos.x as u32,
                //     (image.height - 1 - cell_pos.y) as u32,
                //     color,
                // );
                let pixel_x = cell_pos.x as usize;
                let pixel_y = CHUNK_SIZE - 1 - cell_pos.y as usize;

                let color = cell.color(cells_template).calculate(self, cell_index);
                image.get_image_data_mut()[pixel_y * CHUNK_SIZE + pixel_x] = color;
            }

            self.image = Some(image);
            let image = self.image.as_ref().unwrap();

            match &self.texture {
                Some(texture) => {
                    texture.update(image);
                }
                None => {
                    let texture = Texture2D::from_image(image);
                    texture.set_filter(FilterMode::Nearest);

                    self.texture = Some(texture);
                }
            }
        }

        self.texture.as_ref().unwrap()
    }
}
