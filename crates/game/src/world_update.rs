use crate::*;
use nohash_hasher::IntMap;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

pub struct ChunkUpdateGroup {
    pub update_id: u64,
    pub chunk_to_update: Chunk,
    pub top_left: Option<Chunk>,
    pub top: Option<Chunk>,
    pub top_right: Option<Chunk>,
    pub left: Option<Chunk>,
    pub right: Option<Chunk>,
    pub bottom_left: Option<Chunk>,
    pub bottom: Option<Chunk>,
    pub bottom_right: Option<Chunk>,
}

// macro_rules! random_order_or {
//     ($rule0:expr, $rule1:expr) => {{
//         let order = ::rand::random::<bool>();

//         if order {
//             $rule0 || $rule1
//         } else {
//             $rule1 || $rule0
//         }
//     }};

//     ($rule0:expr, $rule1:expr, $rule2:expr, $rule3:expr) => {{
//         random_order_or!(
//             random_order_or!($rule0, $rule1),
//             random_order_or!($rule2, $rule3)
//         )
//     }};
// }

impl ChunkUpdateGroup {
    pub const SIZE: AreaSize = AreaSize::splat(3);

    pub const BOTTOM_LEFT_OFFSET: (i32, i32) = (0, 0);
    pub const BOTTOM_OFFSET: (i32, i32) = (1, 0);
    pub const BOTTOM_RIGHT_OFFSET: (i32, i32) = (2, 0);
    pub const LEFT_OFFSET: (i32, i32) = (0, 1);
    pub const CENTER_OFFSET: (i32, i32) = (1, 1);
    pub const RIGHT_OFFSET: (i32, i32) = (2, 1);
    pub const TOP_LEFT_OFFSET: (i32, i32) = (0, 2);
    pub const TOP_OFFSET: (i32, i32) = (1, 2);
    pub const TOP_RIGHT_OFFSET: (i32, i32) = (2, 2);

    pub fn len(&self) -> usize {
        1 + self.bottom.is_some() as usize
            + self.bottom_left.is_some() as usize
            + self.bottom_right.is_some() as usize
            + self.left.is_some() as usize
            + self.right.is_some() as usize
            + self.top.is_some() as usize
            + self.top_left.is_some() as usize
            + self.top_right.is_some() as usize
    }

    /// returns None if cell is not loaded yet or already updated.
    pub fn get_cell(&self, position: CellPosition) -> Option<CellType> {
        let position = position + Chunk::SIZE;
        let chunk_pos = position.get_chunk_position();

        let chunk = match (chunk_pos.x, chunk_pos.y) {
            Self::BOTTOM_LEFT_OFFSET => self.bottom_left.as_ref()?,
            Self::BOTTOM_OFFSET => self.bottom.as_ref()?,
            Self::BOTTOM_RIGHT_OFFSET => self.bottom_right.as_ref()?,
            Self::LEFT_OFFSET => self.left.as_ref()?,
            Self::CENTER_OFFSET => &self.chunk_to_update,
            Self::RIGHT_OFFSET => self.right.as_ref()?,
            Self::TOP_LEFT_OFFSET => self.top_left.as_ref()?,
            Self::TOP_OFFSET => self.top.as_ref()?,
            Self::TOP_RIGHT_OFFSET => self.top_right.as_ref()?,
            _ => return None,
        };

        let index = position.get_in_chunk_pos().to_index();
        let cell = chunk.get_at_index(index);

        if cell.last_update >= self.update_id {
            return None;
        }

        Some(cell.ty)
    }

    // Set cell at given position. Load chunk if it's not loaded yet.
    pub fn set_cell(&mut self, position: CellPosition, cell: CellType, updated: bool) {
        let position = position + Chunk::SIZE;
        let chunk_pos = position.get_chunk_position();

        let chunk = match (chunk_pos.x, chunk_pos.y) {
            Self::CENTER_OFFSET => {
                if updated {
                    self.chunk_to_update.set(
                        position.get_in_chunk_pos(),
                        CellState {
                            ty: cell,
                            last_update: self.update_id,
                        },
                    );
                } else {
                    self.chunk_to_update
                        .set_ty(position.get_in_chunk_pos(), cell);
                };

                return;
            }

            Self::BOTTOM_LEFT_OFFSET => &mut self.bottom_left,
            Self::BOTTOM_OFFSET => &mut self.bottom,
            Self::BOTTOM_RIGHT_OFFSET => &mut self.bottom_right,
            Self::LEFT_OFFSET => &mut self.left,
            Self::RIGHT_OFFSET => &mut self.right,
            Self::TOP_LEFT_OFFSET => &mut self.top_left,
            Self::TOP_OFFSET => &mut self.top,
            Self::TOP_RIGHT_OFFSET => &mut self.top_right,
            _ => unreachable!("Invalid chunk position {:?}", chunk_pos),
        };

        let chunk = chunk.get_or_insert_with(|| {
            let real_chunk_pos = chunk_pos + self.chunk_to_update.position + (-1, -1);
            Chunk::new(real_chunk_pos).into()
        });

        if updated {
            chunk.set(
                position.get_in_chunk_pos(),
                CellState {
                    ty: cell,
                    last_update: self.update_id,
                },
            );
        } else {
            chunk.set_ty(position.get_in_chunk_pos(), cell);
        }
    }

    pub fn handle_update(&mut self) {
        for cell_y in 0..Chunk::SIZE.height {
            for cell_x in 0..Chunk::SIZE.width {
                let cell_pos = InChunkCellPosition::new(cell_x as u32, cell_y as u32);
                let cell_index = cell_pos.to_index();

                if self.chunk_to_update.get_at_index(cell_index).last_update >= self.update_id {
                    return;
                }

                self.update_cell(cell_index);
            }
        }
    }

    fn swap_exact_neighbor(
        &mut self,
        pos: CellPosition,
        dir: Direction,
        expected_neighbor: CellType,
    ) -> bool {
        let neighbor_pos = pos + dir;
        if self.get_cell(neighbor_pos) == Some(expected_neighbor) {
            let current_cell = self.get_cell(pos).expect("Cell is not loaded");

            self.set_cell(pos, expected_neighbor, true);
            self.set_cell(neighbor_pos, current_cell, true);

            true
        } else {
            false
        }
    }

    fn swap_any_neighbor<const N: usize>(
        &mut self,
        pos: CellPosition,
        dir: Direction,
        expected_neighbors: [CellType; N],
    ) -> bool {
        let neighbor_pos = pos + dir;

        let Some(neighbor_cell) = self.get_cell(neighbor_pos) else {
            return false;
        };

        if expected_neighbors.contains(&neighbor_cell) {
            let current_cell = self.get_cell(pos).expect("Cell is not loaded");

            self.set_cell(pos, neighbor_cell, true);
            self.set_cell(neighbor_pos, current_cell, true);

            true
        } else {
            false
        }
    }

    fn update_cell(&mut self, cell_index: usize) {
        let pos = InChunkCellPosition::from_index(cell_index).to_absolute();

        let Some(current_cell) = self.get_cell(pos) else {
            return;
        };

        match current_cell {
            CellType::Sand => {
                let swap_with = [CellType::Empty, CellType::Water, CellType::Gas];

                if self.swap_any_neighbor(pos, Direction::Down, swap_with) {
                    return;
                }

                if self.swap_any_neighbor(pos, Direction::DownLeft, swap_with)
                    || self.swap_any_neighbor(pos, Direction::DownRight, swap_with)
                {
                    return;
                }
            }
            CellType::Gas => {
                if self.swap_exact_neighbor(pos, Direction::Up, CellType::Empty)
                    || self.swap_exact_neighbor(pos, Direction::Down, CellType::Empty)
                    || self.swap_exact_neighbor(pos, Direction::Left, CellType::Empty)
                    || self.swap_exact_neighbor(pos, Direction::Right, CellType::Empty)
                {
                    return;
                }
            }
            _ => {}
        }
    }
}

impl World {
    fn take_group_from_world(
        &mut self,
        group_offset: ChunkPosition,
        rest_chunks: &mut IntMap<ChunkPosition, Chunk>,
    ) -> Option<ChunkUpdateGroup> {
        let chunk_to_update = self
            .take_chunk(group_offset + ChunkUpdateGroup::CENTER_OFFSET)?
            .into();

        macro_rules! take_chunk {
            ($position:expr) => {{
                let position: ChunkPosition = $position;
                self.take_chunk(position)
                    .or_else(|| rest_chunks.remove(&position))
                    .map(Into::into)
            }};
        }

        ChunkUpdateGroup {
            update_id: self.update_id,
            chunk_to_update,

            bottom_left: take_chunk!(group_offset + ChunkUpdateGroup::BOTTOM_LEFT_OFFSET),
            bottom: take_chunk!(group_offset + ChunkUpdateGroup::BOTTOM_OFFSET),
            bottom_right: take_chunk!(group_offset + ChunkUpdateGroup::BOTTOM_RIGHT_OFFSET),
            left: take_chunk!(group_offset + ChunkUpdateGroup::LEFT_OFFSET),
            right: take_chunk!(group_offset + ChunkUpdateGroup::RIGHT_OFFSET),
            top_left: take_chunk!(group_offset + ChunkUpdateGroup::TOP_LEFT_OFFSET),
            top: take_chunk!(group_offset + ChunkUpdateGroup::TOP_OFFSET),
            top_right: take_chunk!(group_offset + ChunkUpdateGroup::TOP_RIGHT_OFFSET),
        }
        .into()
    }

    fn push_group_to_world(&mut self, group: ChunkUpdateGroup) {
        self.set_chunk(group.chunk_to_update);
        if let Some(chunk) = group.top_left {
            self.set_chunk(chunk);
        }
        if let Some(chunk) = group.top {
            self.set_chunk(chunk);
        }
        if let Some(chunk) = group.top_right {
            self.set_chunk(chunk);
        }
        if let Some(chunk) = group.left {
            self.set_chunk(chunk);
        }
        if let Some(chunk) = group.right {
            self.set_chunk(chunk);
        }
        if let Some(chunk) = group.bottom_left {
            self.set_chunk(chunk);
        }
        if let Some(chunk) = group.bottom {
            self.set_chunk(chunk);
        }
        if let Some(chunk) = group.bottom_right {
            self.set_chunk(chunk);
        }
    }

    /// Take all chunks from world and return them as groups.
    fn prepare_update_groups(&mut self, group_shift: Position) -> Vec<ChunkUpdateGroup> {
        let mut result = Vec::<ChunkUpdateGroup>::new();

        // Chunks that not fit into any group.
        let mut rest_chunks = IntMap::<ChunkPosition, Chunk>::default();

        while let Some(&chunk_position) = self.chunks.keys().next() {
            let group_offset = chunk_position.to_group_offset(group_shift);

            if let Some(group) = self.take_group_from_world(group_offset, &mut rest_chunks) {
                result.push(group);
            } else {
                let chunk = self.take_chunk(chunk_position).unwrap();
                rest_chunks.insert(chunk.position, chunk);
            }
        }

        // Push rest chunks back to world.
        for chunk in rest_chunks.into_values() {
            self.set_chunk(chunk);
        }

        result.sort_by(|a, b| {
            a.chunk_to_update
                .position
                .cmp(&b.chunk_to_update.position)
                .then(a.len().cmp(&b.len()))
        });

        result
    }

    /// Each update divided into 9 similar steps.
    ///
    /// Each step we dividing world into 3x3 chunks groups and updating them in parallel.
    ///
    /// In each group we only update chunk in the center of the group.
    pub fn update_state(&mut self) {
        self.update_id += 1;
        for group_y in -1..2 {
            for group_x in -1..2 {
                let group_shift = Position {
                    x: group_x as i32,
                    y: group_y as i32,
                };

                // Take chunks from world for update.
                let mut groups = self.prepare_update_groups(group_shift);

                // Handle updates in parallel.
                groups.par_iter_mut().for_each(|group| {
                    group.handle_update();
                });

                // Push chunks back to world.
                for group in groups {
                    self.push_group_to_world(group);
                }
            }
        }
    }
}

#[test]
fn test_sand_fall() {
    let mut world = World::default();

    world.ensure_chunk(ChunkPosition::new(0, 0));
    world.ensure_chunk(ChunkPosition::new(0, -1));

    world.set(CellPosition::new(0, 0), CellType::Sand);
    world.set(CellPosition::new(0, 1), CellType::Sand);
    world.set(CellPosition::new(0, 2), CellType::Sand);

    world.update_state();

    assert_eq!(
        world.get(CellPosition::new(0, -1)).unwrap().ty,
        CellType::Sand
    );
    assert_eq!(
        world.get(CellPosition::new(0, 0)).unwrap().ty,
        CellType::Sand
    );
    assert_eq!(
        world.get(CellPosition::new(0, 1)).unwrap().ty,
        CellType::Sand
    );
    assert_eq!(
        world.get(CellPosition::new(0, 2)).unwrap().ty,
        CellType::Empty
    );

    world.update_state();

    assert_eq!(
        world.get(CellPosition::new(0, -2)).unwrap().ty,
        CellType::Sand
    );
    assert_eq!(
        world.get(CellPosition::new(0, -1)).unwrap().ty,
        CellType::Sand
    );
    assert_eq!(
        world.get(CellPosition::new(0, 0)).unwrap().ty,
        CellType::Sand
    );
    assert_eq!(
        world.get(CellPosition::new(0, 1)).unwrap().ty,
        CellType::Empty
    );
}

#[test]
fn test_get_update_groups() {
    let mut world = World::default();

    world.ensure_chunk(ChunkPosition::new(0, 0));
    world.ensure_chunk(ChunkPosition::new(0, -1));

    let groups = world.prepare_update_groups(Position::new(-1, -1));

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].chunk_to_update.position, ChunkPosition::new(0, 0));
    assert_eq!(
        groups[0].bottom.as_ref().unwrap().position,
        ChunkPosition::new(0, -1)
    );
}

#[test]
fn test_take_group_from_world() {
    let mut world = World::default();

    world.ensure_chunk(ChunkPosition::new(0, 2));
    world.ensure_chunk(ChunkPosition::new(0, 3));

    assert_eq!(world.chunks.len(), 2);

    let group = world.take_group_from_world(ChunkPosition::new(-1, 2), &mut IntMap::default());

    let group = group.expect("Failed to take group from world");

    assert_eq!(group.chunk_to_update.position, ChunkPosition::new(0, 3));
    assert_eq!(group.len(), 2);

    assert_eq!(world.chunks.len(), 0);

    world.push_group_to_world(group);

    assert_eq!(world.chunks.len(), 2);
}

#[test]
fn test_take_group_from_world_and_rest_chunks() {
    let mut world = World::default();

    world.ensure_chunk(ChunkPosition::new(0, 2));
    world.ensure_chunk(ChunkPosition::new(0, 3));

    assert_eq!(world.chunks.len(), 2);

    let chunk = world
        .take_chunk(ChunkPosition::new(0, 2))
        .expect("Failed to take chunk");
    let mut rest_chunks = IntMap::<ChunkPosition, Chunk>::default();
    rest_chunks.insert(chunk.position, chunk);

    let group = world.take_group_from_world(ChunkPosition::new(-1, 2), &mut rest_chunks);

    let group = group.expect("Failed to take group from world");

    assert_eq!(group.chunk_to_update.position, ChunkPosition::new(0, 3));
    assert_eq!(group.len(), 2);

    assert_eq!(world.chunks.len(), 0);
}

#[test]
fn test_world_update_multiple_chunks() {
    let mut world = World::default();

    let chunk_x_range = -10..10;
    let chunk_y_range = -10..10;

    for chunk_x in chunk_x_range.clone() {
        for chunk_y in chunk_y_range.clone() {
            let chunk_pos = ChunkPosition(Position {
                x: chunk_x,
                y: chunk_y,
            });

            world.ensure_chunk(chunk_pos + Direction::Down);

            let pos: CellPosition = chunk_pos.to_cell_offset();
            world.set(pos, CellType::Sand);
            assert_eq!(
                world.get(pos),
                Some(CellState {
                    ty: CellType::Sand,
                    last_update: 0
                }),
                "Failed to set cell in {chunk_pos:?}"
            );
        }
    }

    world.update_state();

    for chunk_x in chunk_x_range {
        for chunk_y in chunk_y_range.clone() {
            let chunk_pos = ChunkPosition(Position {
                x: chunk_x,
                y: chunk_y,
            });
            let pos: CellPosition = chunk_pos.to_cell_offset() + Direction::Down;

            assert_eq!(
                world.get(pos),
                Some(CellState {
                    ty: CellType::Sand,
                    last_update: 1
                }),
                "Update failed in {chunk_pos:?}"
            );
        }
    }
}
