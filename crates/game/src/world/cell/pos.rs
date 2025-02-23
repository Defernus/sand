use crate::*;
use macroquad::math::vec2;
use std::ops::Rem;

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
    pub fn new(mut x: i32, mut y: i32) -> Self {
        let (chunk_x, cell_x) = if x < 0 {
            (
                (x + 1) / CHUNK_SIZE as i32 - 1,
                true_mod(x, CHUNK_SIZE as i32),
            )
        } else {
            (x / CHUNK_SIZE as i32, x % CHUNK_SIZE as i32)
        };
        let (chunk_y, cell_y) = if y < 0 {
            (
                (y + 1) / CHUNK_SIZE as i32 - 1,
                true_mod(y, CHUNK_SIZE as i32),
            )
        } else {
            (y / CHUNK_SIZE as i32, y % CHUNK_SIZE as i32)
        };

        let chunk = ChunkPos::new(chunk_x, chunk_y);

        let cell = CellPos::new(cell_x as CellCord, cell_y as CellCord);

        Self { chunk, cell }
    }

    #[inline(always)]
    pub fn x(&self) -> i32 {
        let mut res = self.chunk.x * CHUNK_SIZE as i32 + self.cell.x as i32;

        res
    }

    #[inline(always)]
    pub fn y(&self) -> i32 {
        let mut res = self.chunk.y * CHUNK_SIZE as i32 + self.cell.y as i32;

        res
    }
}

impl std::ops::Add<RelativePos> for GlobalCellPos {
    type Output = Self;

    fn add(self, rhs: RelativePos) -> Self::Output {
        GlobalCellPos::new(self.x() + rhs.x as i32, self.y() + rhs.y as i32)
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
    for x in -(CHUNK_SIZE as i32) * 3..(CHUNK_SIZE as i32) * 3 {
        for y in -(CHUNK_SIZE as i32) * 3..(CHUNK_SIZE as i32) * 3 {
            let pos = GlobalCellPos::new(x, y);

            assert_eq!(pos.x(), x);
            assert_eq!(pos.y(), y);

            assert_eq!(GlobalCellPos::new(pos.x(), pos.y()), pos);
        }
    }
}
