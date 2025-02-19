use crate::*;

pub struct ChunkUpdateContext {
    pub current_update_switch: bool,
    pub center: Chunk,
    pub left: Chunk,
    pub right: Chunk,
    pub top: Chunk,
    pub bottom: Chunk,
    pub left_top: Chunk,
    pub right_top: Chunk,
    pub left_bottom: Chunk,
    pub right_bottom: Chunk,
}

/// Cell relative position
pub struct RelativePos {
    x: i8,
    y: i8,
}

impl ChunkUpdateContext {
    /// This function will process only central chunk, but it will also access the surrounding chunks and in some cases modify them (e.g. sand falling)
    pub fn process(&mut self) {
        self.center.should_update = false;

        for index in 0..CHUNK_AREA {
            // TODO update swapped cell also

            self.update_cell(index);
        }
    }

    #[inline(always)]
    fn update_cell(&mut self, index: usize) -> bool {
        let cell_pos = AbsoluteCellPos {
            index,
            side: Side {
                horizontal: HorizontalSide::Center,
                vertical: VerticalSide::Center,
            },
        };

        let cell = self.center.get_by_index(index);

        match cell.id {
            CELL_VACUUM => {
                // Do nothing
            }
            CELL_SAND => {
                let down_pos = get_absolute_cell_pos(index, RelativePos { x: 0, y: -1 });
                let down_cell = self.get_cell(down_pos);
                if down_cell.id == CELL_VACUUM || down_cell.id == CELL_WATER {
                    self.swap_cells(cell_pos, down_pos);
                    return true;
                }

                let down_left_pos = get_absolute_cell_pos(index, RelativePos { x: -1, y: -1 });
                let down_left_cell = self.get_cell(down_left_pos);
                if down_left_cell.id == CELL_VACUUM || down_left_cell.id == CELL_WATER {
                    self.swap_cells(cell_pos, down_left_pos);
                    return true;
                }

                let down_right_pos = get_absolute_cell_pos(index, RelativePos { x: 1, y: -1 });
                let down_right_cell = self.get_cell(down_right_pos);
                if down_right_cell.id == CELL_VACUUM || down_right_cell.id == CELL_WATER {
                    self.swap_cells(cell_pos, down_right_pos);
                    return true;
                }
            }
            CELL_WATER => {
                // TODO
            }
            CELL_STONE => {
                // TODO
            }
            _ => {
                // unknown cell, also do nothing
            }
        }

        false
    }

    #[inline(always)]
    fn get_chunk(&self, side: Side) -> &Chunk {
        match (side.horizontal, side.vertical) {
            (HorizontalSide::Left, VerticalSide::Top) => &self.left_top,
            (HorizontalSide::Right, VerticalSide::Top) => &self.right_top,
            (HorizontalSide::Left, VerticalSide::Bottom) => &self.left_bottom,
            (HorizontalSide::Right, VerticalSide::Bottom) => &self.right_bottom,
            (HorizontalSide::Left, VerticalSide::Center) => &self.left,
            (HorizontalSide::Right, VerticalSide::Center) => &self.right,
            (HorizontalSide::Center, VerticalSide::Top) => &self.top,
            (HorizontalSide::Center, VerticalSide::Bottom) => &self.bottom,
            (HorizontalSide::Center, VerticalSide::Center) => &self.center,
        }
    }

    #[inline(always)]
    fn get_chunk_mut(&mut self, side: Side) -> &mut Chunk {
        match (side.horizontal, side.vertical) {
            (HorizontalSide::Left, VerticalSide::Top) => &mut self.left_top,
            (HorizontalSide::Right, VerticalSide::Top) => &mut self.right_top,
            (HorizontalSide::Left, VerticalSide::Bottom) => &mut self.left_bottom,
            (HorizontalSide::Right, VerticalSide::Bottom) => &mut self.right_bottom,
            (HorizontalSide::Left, VerticalSide::Center) => &mut self.left,
            (HorizontalSide::Right, VerticalSide::Center) => &mut self.right,
            (HorizontalSide::Center, VerticalSide::Top) => &mut self.top,
            (HorizontalSide::Center, VerticalSide::Bottom) => &mut self.bottom,
            (HorizontalSide::Center, VerticalSide::Center) => &mut self.center,
        }
    }

    #[inline(always)]
    fn get_cell(&self, pos: AbsoluteCellPos) -> Cell {
        self.get_chunk(pos.side).get_by_index(pos.index)
    }

    #[inline(always)]
    fn set_cell(&mut self, pos: AbsoluteCellPos, cell: Cell) {
        self.get_chunk_mut(pos.side).set_by_index(pos.index, cell);
    }

    fn swap_cells(&mut self, a: AbsoluteCellPos, b: AbsoluteCellPos) {
        let mut cell_a = self.get_cell(a);
        let mut cell_b = self.get_cell(b);

        // mark cells as updated
        cell_a.updated_switch = self.current_update_switch;
        cell_b.updated_switch = self.current_update_switch;

        self.set_cell(a, cell_b);
        self.set_cell(b, cell_a);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum HorizontalSide {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum VerticalSide {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Side {
    horizontal: HorizontalSide,
    vertical: VerticalSide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct AbsoluteCellPos {
    index: usize,
    side: Side,
}

fn get_absolute_cell_pos(cell_index: usize, dst_relative_pos: RelativePos) -> AbsoluteCellPos {
    let cell_pos = CellPos::from_index(cell_index);

    let x = cell_pos.x as i32 + dst_relative_pos.x as i32;
    let y = cell_pos.y as i32 + dst_relative_pos.y as i32;

    let (x, horizontal) = if x < 0 {
        ((x + CHUNK_SIZE as i32) as CellCord, HorizontalSide::Left)
    } else if x < CHUNK_SIZE as i32 {
        (x as CellCord, HorizontalSide::Center)
    } else {
        ((x - CHUNK_SIZE as i32) as CellCord, HorizontalSide::Right)
    };

    let (y, vertical) = if y < 0 {
        ((y + CHUNK_SIZE as i32) as CellCord, VerticalSide::Bottom)
    } else if y < CHUNK_SIZE as i32 {
        (y as CellCord, VerticalSide::Center)
    } else {
        ((y - CHUNK_SIZE as i32) as CellCord, VerticalSide::Top)
    };

    let index = CellPos::new(x, y).to_index();
    let side = Side {
        horizontal,
        vertical,
    };

    AbsoluteCellPos { index, side }
}
