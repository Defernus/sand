use crate::*;

pub type CellCord = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CellPos {
    pub x: CellCord,
    pub y: CellCord,
}

impl CellPos {
    pub fn new(x: CellCord, y: CellCord) -> Self {
        debug_assert!(
            (x as usize) < CHUNK_SIZE,
            "x out of bounds {x} >= {CHUNK_SIZE}"
        );
        debug_assert!(
            (y as usize) < CHUNK_SIZE,
            "y out of bounds {y} >= {CHUNK_SIZE}"
        );

        Self { x, y }
    }

    pub fn from_index(index: usize) -> Self {
        debug_assert!(
            index < CHUNK_AREA,
            "Index out of bounds {index} >= {CHUNK_AREA}"
        );

        Self {
            x: (index % CHUNK_SIZE) as CellCord,
            y: (index / CHUNK_SIZE) as CellCord,
        }
    }

    pub fn to_index(&self) -> usize {
        debug_assert!(
            (self.x as usize) < CHUNK_SIZE,
            "x out of bounds {x} >= {CHUNK_SIZE}",
            x = self.x
        );
        debug_assert!(
            (self.y as usize) < CHUNK_SIZE,
            "y out of bounds {y} >= {CHUNK_SIZE}",
            y = self.y
        );

        self.x as usize + self.y as usize * CHUNK_SIZE
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlobalCellPos {
    pub chunk: ChunkPos,
    pub cell: CellPos,
}

impl GlobalCellPos {
    pub fn new(x: i32, y: i32) -> Self {
        let mut chunk = ChunkPos::new(x / CHUNK_SIZE as i32, y / CHUNK_SIZE as i32);
        if x < 0 {
            chunk.x -= 1;
        }
        if y < 0 {
            chunk.y -= 1;
        }

        let cell = CellPos::new(
            (x.rem_euclid(CHUNK_SIZE as i32)) as CellCord,
            (y.rem_euclid(CHUNK_SIZE as i32)) as CellCord,
        );

        Self { chunk, cell }
    }

    #[inline(always)]
    pub fn x(&self) -> i32 {
        self.chunk.x * CHUNK_SIZE as i32 + self.cell.x as i32
    }

    #[inline(always)]
    pub fn y(&self) -> i32 {
        self.chunk.y * CHUNK_SIZE as i32 + self.cell.y as i32
    }

    pub fn add(&self, x: i32, y: i32) -> Self {
        Self::new(self.x() + x, self.y() + y)
    }
}

#[test]
fn test_chunk_pos_to_index() {
    let pos = CellPos::new(5, 13);
    let index = pos.to_index();

    assert_eq!(CellPos::from_index(index), pos);
}

#[test]
fn test_global_cell_pos() {
    let x = CHUNK_SIZE as i32 * 3 + 5;
    let y = CHUNK_SIZE as i32 * 2 + 13;

    let global_pos = GlobalCellPos::new(x, y);

    assert_eq!(global_pos.chunk, ChunkPos::new(3, 2));
    assert_eq!(global_pos.cell, CellPos::new(5, 13));
}
