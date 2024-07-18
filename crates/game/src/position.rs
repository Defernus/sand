use crate::*;
use std::ops::{Deref, DerefMut};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub y: i32,
    pub x: i32,
}

impl nohash_hasher::IsEnabled for Position {}

impl std::hash::Hash for Position {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // SAFETY: i32 and u32 have the same size and alignment.
        let (x, y) = unsafe {
            (
                std::mem::transmute::<i32, u32>(self.x),
                std::mem::transmute::<i32, u32>(self.y),
            )
        };

        let x = x as u64;
        let y = y as u64;

        let data = (y << 32) | x;

        data.hash(state);
    }
}

impl std::ops::Add<Position> for Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Self::Output {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub<Position> for Position {
    type Output = Position;

    fn sub(self, rhs: Position) -> Self::Output {
        Position {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Add<(i32, i32)> for Position {
    type Output = Position;

    fn add(self, rhs: (i32, i32)) -> Self::Output {
        Position {
            x: self.x + rhs.0,
            y: self.y + rhs.1,
        }
    }
}

impl std::ops::Add<Direction> for Position {
    type Output = Position;

    fn add(self, rhs: Direction) -> Self::Output {
        self + rhs.to_offset()
    }
}

impl std::ops::Add<AreaSize> for Position {
    type Output = Position;

    fn add(self, rhs: AreaSize) -> Self::Output {
        Position {
            x: self.x + rhs.width as i32,
            y: self.y + rhs.height as i32,
        }
    }
}

impl Position {
    #[inline(always)]
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelativePosition {
    pub y: u32,
    pub x: u32,
}

impl nohash_hasher::IsEnabled for RelativePosition {}

impl std::hash::Hash for RelativePosition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let x = self.x as u64;
        let y = self.y as u64;

        let data = (y << 32) | x;

        data.hash(state);
    }
}

impl std::ops::Add<RelativePosition> for Position {
    type Output = Position;

    fn add(self, rhs: RelativePosition) -> Self::Output {
        Position {
            x: self.x + rhs.x as i32,
            y: self.y + rhs.y as i32,
        }
    }
}

impl std::ops::Sub<RelativePosition> for Position {
    type Output = Position;

    fn sub(self, rhs: RelativePosition) -> Self::Output {
        Position {
            x: self.x - rhs.x as i32,
            y: self.y - rhs.y as i32,
        }
    }
}

impl From<RelativePosition> for Position {
    fn from(value: RelativePosition) -> Self {
        Position {
            x: value.x as i32,
            y: value.y as i32,
        }
    }
}

/// Absolute position of the chunk in the world
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkPosition(pub Position);

impl nohash_hasher::IsEnabled for ChunkPosition {}

impl From<Position> for ChunkPosition {
    fn from(pos: Position) -> Self {
        Self(pos)
    }
}

impl Deref for ChunkPosition {
    type Target = Position;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChunkPosition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Add<ChunkPosition> for ChunkPosition {
    type Output = ChunkPosition;

    fn add(self, rhs: ChunkPosition) -> Self::Output {
        ChunkPosition(self.0 + rhs.0)
    }
}

impl std::ops::Add<(i32, i32)> for ChunkPosition {
    type Output = ChunkPosition;

    fn add(self, rhs: (i32, i32)) -> Self::Output {
        ChunkPosition(self.0 + Position { x: rhs.0, y: rhs.1 })
    }
}

impl std::ops::Add<Position> for ChunkPosition {
    type Output = ChunkPosition;

    fn add(self, rhs: Position) -> Self::Output {
        ChunkPosition(self.0 + rhs)
    }
}

impl std::ops::Add<RelativePosition> for ChunkPosition {
    type Output = ChunkPosition;

    fn add(self, rhs: RelativePosition) -> Self::Output {
        ChunkPosition(self.0 + rhs)
    }
}

impl std::ops::Sub<RelativePosition> for ChunkPosition {
    type Output = ChunkPosition;

    fn sub(self, rhs: RelativePosition) -> Self::Output {
        ChunkPosition(self.0 - rhs)
    }
}

impl std::ops::Add<Direction> for ChunkPosition {
    type Output = ChunkPosition;

    fn add(self, rhs: Direction) -> Self::Output {
        ChunkPosition(self.0 + rhs.to_offset())
    }
}

impl ChunkPosition {
    #[inline(always)]
    pub fn new(x: i32, y: i32) -> Self {
        Self(Position { x, y })
    }

    #[inline(always)]
    pub fn to_cell_offset(self) -> CellPosition {
        Position {
            x: self.x * Chunk::SIZE.width as i32,
            y: self.y * Chunk::SIZE.height as i32,
        }
        .into()
    }

    /// Convert chunk position to update group offset.
    #[inline(always)]
    pub fn to_group_offset(self, group_shift: Position) -> Self {
        let pos = Position {
            x: downscale_cord(self.x, ChunkUpdateGroup::SIZE.width) as i32
                * ChunkUpdateGroup::SIZE.width as i32,
            y: downscale_cord(self.y, ChunkUpdateGroup::SIZE.height) as i32
                * ChunkUpdateGroup::SIZE.height as i32,
        };

        (pos + group_shift).into()
    }
}

/// Absolute cell position in the world
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InChunkCellPosition(pub RelativePosition);

impl nohash_hasher::IsEnabled for InChunkCellPosition {}

impl From<RelativePosition> for InChunkCellPosition {
    fn from(pos: RelativePosition) -> Self {
        Self(pos)
    }
}

impl Deref for InChunkCellPosition {
    type Target = RelativePosition;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for InChunkCellPosition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl InChunkCellPosition {
    #[inline(always)]
    pub fn new(x: u32, y: u32) -> Self {
        Self(RelativePosition { x, y })
    }

    /// Returns index of the cell in the chunk.
    #[inline(always)]
    pub fn to_index(self) -> usize {
        Chunk::SIZE.coords_to_index(*self)
    }

    #[inline(always)]
    pub fn from_index(index: usize) -> Self {
        Self(Chunk::SIZE.index_to_coords(index))
    }

    #[inline(always)]
    pub fn to_absolute(self) -> CellPosition {
        CellPosition(self.0.into())
    }
}

/// Absolute cell position in the world
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CellPosition(pub Position);

impl nohash_hasher::IsEnabled for CellPosition {}

impl From<Position> for CellPosition {
    fn from(pos: Position) -> Self {
        Self(pos)
    }
}

impl Deref for CellPosition {
    type Target = Position;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CellPosition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Add<CellPosition> for CellPosition {
    type Output = CellPosition;

    fn add(self, rhs: CellPosition) -> Self::Output {
        CellPosition(self.0 + rhs.0)
    }
}
impl std::ops::Add<Direction> for CellPosition {
    type Output = CellPosition;

    fn add(self, rhs: Direction) -> Self::Output {
        CellPosition(self.0 + rhs.to_offset())
    }
}

impl std::ops::Add<(i32, i32)> for CellPosition {
    type Output = CellPosition;

    fn add(self, rhs: (i32, i32)) -> Self::Output {
        CellPosition(self.0 + Position { x: rhs.0, y: rhs.1 })
    }
}

impl std::ops::Add<InChunkCellPosition> for CellPosition {
    type Output = CellPosition;

    fn add(self, rhs: InChunkCellPosition) -> Self::Output {
        CellPosition(self.0 + rhs.0)
    }
}

impl std::ops::Add<AreaSize> for CellPosition {
    type Output = CellPosition;

    fn add(self, rhs: AreaSize) -> Self::Output {
        CellPosition(self.0 + rhs)
    }
}

impl CellPosition {
    #[inline(always)]
    pub fn new(x: i32, y: i32) -> Self {
        Self(Position { x, y })
    }

    /// Get position of the chunk in which the cell is located.
    #[inline(always)]
    pub fn get_chunk_position(self) -> ChunkPosition {
        ChunkPosition(Position {
            x: downscale_cord(self.x, Chunk::SIZE.width),
            y: downscale_cord(self.y, Chunk::SIZE.height),
        })
    }

    /// Convert absolute cell position in the world to position inside chunk.
    #[inline(always)]
    pub fn get_in_chunk_pos(self) -> InChunkCellPosition {
        InChunkCellPosition(RelativePosition {
            x: mod_cord(self.x, Chunk::SIZE.width) as u32,
            y: mod_cord(self.y, Chunk::SIZE.height) as u32,
        })
    }
}

/// Convert absolute cord of the region part into region cord (e.g. convert
/// absolute cord of the cell in the world to the chunk's cord).
#[inline(always)]
fn downscale_cord(v: i32, size: usize) -> i32 {
    if v >= 0 {
        v / size as i32
    } else {
        (v - size as i32 + 1) / size as i32
    }
}

#[inline(always)]
fn mod_cord(v: i32, size: usize) -> usize {
    ((v % size as i32) + size as i32) as usize % size
}

#[test]
fn test_downscale_cord() {
    assert_eq!(downscale_cord(0, 100), 0);
    assert_eq!(downscale_cord(-1, 100), -1);
    assert_eq!(downscale_cord(-99, 100), -1);
    assert_eq!(downscale_cord(-100, 100), -1);
    assert_eq!(downscale_cord(-101, 100), -2);
    assert_eq!(downscale_cord(123, 100), 1);
    assert_eq!(downscale_cord(23, 100), 0);
    assert_eq!(downscale_cord(-23, 100), -1);
    assert_eq!(downscale_cord(-123, 100), -2);
}

#[test]
fn test_position_ord() {
    let pos0 = Position::new(0, 0);
    let pos1 = Position::new(1, 0);
    let pos2 = Position::new(0, 1);
    let pos3 = Position::new(1, 1);

    assert_eq!(pos0.cmp(&pos1), std::cmp::Ordering::Less);
    assert_eq!(pos0.cmp(&pos2), std::cmp::Ordering::Less);
    assert_eq!(pos0.cmp(&pos3), std::cmp::Ordering::Less);

    assert_eq!(pos1.cmp(&pos2), std::cmp::Ordering::Less);
    assert_eq!(pos1.cmp(&pos3), std::cmp::Ordering::Less);

    assert_eq!(pos2.cmp(&pos3), std::cmp::Ordering::Less);
}
