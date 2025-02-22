use crate::*;

pub struct ChunkUpdateContext {
    pub current_tick: u32,
    pub update_variant: UpdateVariant,
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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UpdateVariant {
    #[default]
    A,
    B,
    C,
    D,
}

impl UpdateVariant {
    pub fn next(&self) -> UpdateVariant {
        match self {
            UpdateVariant::A => UpdateVariant::B,
            UpdateVariant::B => UpdateVariant::C,
            UpdateVariant::C => UpdateVariant::D,
            UpdateVariant::D => UpdateVariant::A,
        }
    }
}

impl UpdateVariant {
    fn x_offset(&self) -> CellCord {
        match self {
            UpdateVariant::A => 0,
            UpdateVariant::B => 1,
            UpdateVariant::C => 0,
            UpdateVariant::D => 1,
        }
    }

    fn y_offset(&self) -> CellCord {
        match self {
            UpdateVariant::A => 0,
            UpdateVariant::B => 0,
            UpdateVariant::C => 1,
            UpdateVariant::D => 1,
        }
    }
}

impl ChunkUpdateContext {
    #[inline(always)]
    pub fn get_chunk(&self, side: Side) -> &Chunk {
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
    pub fn get_chunk_mut(&mut self, side: Side) -> &mut Chunk {
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
    pub fn get_cell(&self, pos: AbsoluteCellPos) -> Cell {
        self.get_chunk(pos.side).get_by_index(pos.index)
    }

    #[inline(always)]
    pub fn set_cell(&mut self, pos: AbsoluteCellPos, mut cell: Cell) {
        // mark cells as updated
        cell.last_update = self.current_tick;
        self.get_chunk_mut(pos.side).set_by_index(pos.index, cell);
    }

    #[inline(always)]
    pub fn swap_cells(&mut self, a: AbsoluteCellPos, b: AbsoluteCellPos) {
        let cell_a = self.get_cell(a);
        let cell_b = self.get_cell(b);

        self.set_cell(a, cell_b);
        self.set_cell(b, cell_a);
    }

    #[inline(always)]
    fn get_cell_group(&mut self, index_left_bottom: usize) -> CellGroup {
        assert!(index_left_bottom < CHUNK_AREA, "Index out of bounds");

        let x = index_left_bottom % CHUNK_SIZE;
        let y = index_left_bottom / CHUNK_SIZE;

        let cell_a = self.center.get_by_index(index_left_bottom);
        let cell_b = if x + 1 < CHUNK_SIZE {
            self.center
                .get_by_index(CellPos::new(x as CellCord + 1, y as CellCord).to_index())
        } else {
            self.right
                .get_by_index(CellPos::new(0, y as CellCord).to_index())
        };
        let cell_c = if y + 1 < CHUNK_SIZE {
            self.center
                .get_by_index(CellPos::new(x as CellCord, y as CellCord + 1).to_index())
        } else {
            self.top
                .get_by_index(CellPos::new(x as CellCord, 0).to_index())
        };
        let cell_d = if x + 1 < CHUNK_SIZE && y + 1 < CHUNK_SIZE {
            self.center
                .get_by_index(CellPos::new(x as CellCord + 1, y as CellCord + 1).to_index())
        } else if x + 1 < CHUNK_SIZE {
            self.top
                .get_by_index(CellPos::new(x as CellCord + 1, 0).to_index())
        } else if y + 1 < CHUNK_SIZE {
            self.right
                .get_by_index(CellPos::new(0, y as CellCord + 1).to_index())
        } else {
            self.right_top.get_by_index(0)
        };

        CellGroup {
            cells: [cell_a, cell_b, cell_c, cell_d],
        }
    }

    #[inline(always)]
    fn set_cell_group(&mut self, index_left_bottom: usize, cell_group: CellGroup) {
        assert!(index_left_bottom < CHUNK_AREA, "Index out of bounds");

        let x = index_left_bottom % CHUNK_SIZE;
        let y = index_left_bottom / CHUNK_SIZE;

        self.center
            .set_by_index(index_left_bottom, cell_group.cells[0]);
        if x + 1 < CHUNK_SIZE {
            self.center.set_by_index(
                CellPos::new(x as CellCord + 1, y as CellCord).to_index(),
                cell_group.cells[1],
            );
        } else {
            self.right.set_by_index(
                CellPos::new(0, y as CellCord).to_index(),
                cell_group.cells[1],
            );
        }
        if y + 1 < CHUNK_SIZE {
            self.center.set_by_index(
                CellPos::new(x as CellCord, y as CellCord + 1).to_index(),
                cell_group.cells[2],
            );
        } else {
            self.top.set_by_index(
                CellPos::new(x as CellCord, 0).to_index(),
                cell_group.cells[2],
            );
        }
        if x + 1 < CHUNK_SIZE && y + 1 < CHUNK_SIZE {
            self.center.set_by_index(
                CellPos::new(x as CellCord + 1, y as CellCord + 1).to_index(),
                cell_group.cells[3],
            );
        } else if x + 1 < CHUNK_SIZE {
            self.top.set_by_index(
                CellPos::new(x as CellCord + 1, 0).to_index(),
                cell_group.cells[3],
            );
        } else if y + 1 < CHUNK_SIZE {
            self.right.set_by_index(
                CellPos::new(0, y as CellCord + 1).to_index(),
                cell_group.cells[3],
            );
        } else {
            self.right_top.set_by_index(0, cell_group.cells[3]);
        }
    }

    /// This function will process only central chunk, but it will also access the surrounding
    /// chunks and in some cases modify them (e.g. sand falling)
    pub fn process(&mut self) {
        self.center.should_update = false;

        for x in 0..CHUNK_SIZE / 2 {
            for y in 0..CHUNK_SIZE / 2 {
                let x_offset = x as CellCord * 2 + self.update_variant.x_offset();
                let y_offset = y as CellCord * 2 + self.update_variant.y_offset();
                let index = CellPos::new(x_offset, y_offset).to_index();

                let group = self.get_cell_group(index);

                let mut random_seed = self.center.get_random_seed(index);
                let prev_seed = random_seed;
                if let Some(processed) = group.process(RULES, &mut random_seed) {
                    self.center.should_update = true;
                    if random_seed != prev_seed {
                        self.center.set_random_seed(index, random_seed);
                    }

                    self.set_cell_group(index, processed);
                }
            }
        }

        // If chunk updated next frame we need to update surrounding chunks
        if self.center.should_update {
            self.bottom.should_update = true;
            self.top.should_update = true;
            self.left.should_update = true;
            self.right.should_update = true;
            self.left_top.should_update = true;
            self.right_top.should_update = true;
            self.left_bottom.should_update = true;
            self.right_bottom.should_update = true;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HorizontalSide {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VerticalSide {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Side {
    horizontal: HorizontalSide,
    vertical: VerticalSide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AbsoluteCellPos {
    index: usize,
    side: Side,
}

impl AbsoluteCellPos {
    pub fn central(index: usize) -> Self {
        Self {
            index,
            side: Side {
                horizontal: HorizontalSide::Center,
                vertical: VerticalSide::Center,
            },
        }
    }

    pub fn right(index: usize) -> Self {
        Self {
            index,
            side: Side {
                horizontal: HorizontalSide::Right,
                vertical: VerticalSide::Center,
            },
        }
    }

    pub fn top(index: usize) -> Self {
        Self {
            index,
            side: Side {
                horizontal: HorizontalSide::Center,
                vertical: VerticalSide::Top,
            },
        }
    }

    pub fn top_right(index: usize) -> Self {
        Self {
            index,
            side: Side {
                horizontal: HorizontalSide::Right,
                vertical: VerticalSide::Top,
            },
        }
    }
}

#[test]
fn test_get_cell_group() {
    let mut context = ChunkUpdateContext {
        current_tick: 0,
        update_variant: UpdateVariant::A,
        center: Chunk::default(),
        left: Chunk::default(),
        right: Chunk::default(),
        top: Chunk::default(),
        bottom: Chunk::default(),
        left_top: Chunk::default(),
        right_top: Chunk::default(),
        left_bottom: Chunk::default(),
        right_bottom: Chunk::default(),
    };

    context.set_cell_group(
        CellPos::new(10, CHUNK_SIZE as CellCord - 1).to_index(),
        CellGroup {
            cells: [
                CELL_STONE.init(),
                CELL_SAND.init(),
                CELL_SAND.init(),
                CELL_WATER.init(),
            ],
        },
    );

    let group = context.get_cell_group(0);
    assert_eq!(group.cells[0].last_update, 0);
    assert_eq!(group.cells[1].last_update, 0);
    assert_eq!(group.cells[2].last_update, 0);
    assert_eq!(group.cells[3].last_update, 0);

    {
        let group = context.get_cell_group(CellPos::new(10, CHUNK_SIZE as CellCord - 1).to_index());
        assert_eq!(group.cells[0].id, CELL_STONE.id);
        assert_eq!(group.cells[1].id, CELL_SAND.id);
        assert_eq!(group.cells[2].id, CELL_SAND.id);
        assert_eq!(group.cells[3].id, CELL_WATER.id);
    }

    context.set_cell(
        AbsoluteCellPos::central(CellPos::new((CHUNK_SIZE - 1) as CellCord, 2).to_index()),
        CELL_SAND.init(),
    );
    context.set_cell(
        AbsoluteCellPos::right(CellPos::new(0, 2).to_index()),
        CELL_BORDER.init(),
    );
    context.set_cell(
        AbsoluteCellPos::central(CellPos::new((CHUNK_SIZE - 1) as CellCord, 3).to_index()),
        CELL_STONE.init(),
    );
    context.set_cell(
        AbsoluteCellPos::right(CellPos::new(0, 3).to_index()),
        CELL_SAND.init(),
    );

    let group = context.get_cell_group(CellPos::new((CHUNK_SIZE - 1) as CellCord, 2).to_index());

    assert_eq!(group.cells[0].id, CELL_SAND.id);
    assert_eq!(group.cells[1].id, CELL_BORDER.id);
    assert_eq!(group.cells[2].id, CELL_STONE.id);
    assert_eq!(group.cells[3].id, CELL_SAND.id);
}
