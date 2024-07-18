use crate::*;
use macroquad::texture::Image;

pub struct Chunk {
    pub cells: Box<[CellState; Self::SIZE.area()]>,
    pub position: ChunkPosition,
}

impl Chunk {
    pub const SIZE: AreaSize = AreaSize::splat(128);

    #[inline(always)]
    pub fn new(position: ChunkPosition) -> Self {
        Self {
            position,
            cells: Box::new([CellState::default(); Self::SIZE.area()]),
        }
    }

    #[inline(always)]
    pub fn set(&mut self, position: InChunkCellPosition, cell: CellState) {
        self.set_at_index(position.to_index(), cell);
    }

    #[inline(always)]
    pub fn set_ty(&mut self, position: InChunkCellPosition, cell: CellType) {
        self.set_ty_at_index(position.to_index(), cell);
    }

    #[inline(always)]
    pub fn set_at_index(&mut self, index: usize, cell: CellState) {
        self.cells[index] = cell;
    }

    #[inline(always)]
    pub fn set_ty_at_index(&mut self, index: usize, cell: CellType) {
        self.cells[index].ty = cell;
    }

    #[inline(always)]
    pub fn get(&self, position: InChunkCellPosition) -> CellState {
        self.get_at_index(position.to_index())
    }

    #[inline(always)]
    pub fn get_at_index(&self, index: usize) -> CellState {
        self.cells[index]
    }

    pub fn draw_to_image(&self, image: &mut Image, offset: CellPosition) {
        let image_size = image.get_area_size();
        if !image_size.is_intersects_with_rect(*offset, Self::SIZE) {
            return;
        }

        for (index, cell) in self.cells.iter().enumerate() {
            let position = offset + InChunkCellPosition::from_index(index);

            if position.x < 0 || position.y < 0 {
                continue;
            }

            if position.x >= image.width as i32 || position.y >= image.height as i32 {
                continue;
            }

            cell.draw_to_image(image, position.x as u16, position.y as u16);
        }
    }
}
