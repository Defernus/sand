use crate::*;
use nohash_hasher::IntMap;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

/// State of the physical simulation of the world.
#[derive(Default)]
pub struct WorldState {
    chunks: IntMap<ChunkPos, Chunk>,
    current_tick: u32,
}

impl WorldState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    #[inline(always)]
    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn ensure_chunk(&mut self, pos: ChunkPos) -> &mut Chunk {
        self.chunks.entry(pos).or_insert_with(Chunk::new)
    }

    pub fn take_chunk(&mut self, pos: ChunkPos) -> Chunk {
        self.chunks.remove(&pos).unwrap_or_else(Chunk::new)
    }

    pub fn set_chunk(&mut self, pos: ChunkPos, chunk: Chunk) -> Option<Chunk> {
        self.chunks.insert(pos, chunk)
    }

    pub fn update_state(&mut self) -> usize {
        const GROUP_SIZE: usize = 3;
        let mut update_groups: [Vec<ChunkPos>; GROUP_SIZE * GROUP_SIZE] = Default::default();
        let mut updates_count = 0usize;

        for (group_index, chunk_group) in update_groups.iter_mut().enumerate() {
            let x_rem = group_index as i32 % 3;
            let y_rem = group_index as i32 / 3;

            *chunk_group = self
                .chunks
                .iter()
                .filter_map(|(&pos, chunk)| {
                    if chunk.should_update
                        && pos.x.rem_euclid(3) == x_rem
                        && pos.y.rem_euclid(3) == y_rem
                    {
                        Some(pos)
                    } else {
                        None
                    }
                })
                .collect();

            updates_count += chunk_group.len();
        }

        for update_variant in [
            UpdateVariant::A,
            UpdateVariant::B,
            UpdateVariant::C,
            UpdateVariant::D,
        ] {
            for group in &mut update_groups {
                let mut update_contexts =
                    Vec::<(ChunkUpdateContext, ChunkPos)>::with_capacity(group.len());

                for chunk_pos in group {
                    let chunk_pos = *chunk_pos;

                    update_contexts.push((
                        ChunkUpdateContext {
                            current_tick: self.current_tick,
                            update_variant,
                            center: self.take_chunk(chunk_pos),
                            left: self.take_chunk(chunk_pos.left()),
                            right: self.take_chunk(chunk_pos.right()),
                            top: self.take_chunk(chunk_pos.top()),
                            bottom: self.take_chunk(chunk_pos.bottom()),
                            left_top: self.take_chunk(chunk_pos.left_top()),
                            right_top: self.take_chunk(chunk_pos.right_top()),
                            left_bottom: self.take_chunk(chunk_pos.left_bottom()),
                            right_bottom: self.take_chunk(chunk_pos.right_bottom()),
                        },
                        chunk_pos,
                    ));
                }

                update_contexts
                    .par_iter_mut()
                    .for_each(|(context, _chunk_pos)| {
                        let prev_should_update =
                            update_variant != UpdateVariant::A && context.center.should_update;
                        context.process();
                        context.center.should_update |= prev_should_update;
                    });

                for (context, chunk_pos) in update_contexts {
                    self.set_chunk(chunk_pos, context.center);
                    self.set_chunk(chunk_pos.left(), context.left);
                    self.set_chunk(chunk_pos.right(), context.right);
                    self.set_chunk(chunk_pos.top(), context.top);
                    self.set_chunk(chunk_pos.bottom(), context.bottom);
                    self.set_chunk(chunk_pos.left_top(), context.left_top);
                    self.set_chunk(chunk_pos.right_top(), context.right_top);
                    self.set_chunk(chunk_pos.left_bottom(), context.left_bottom);
                    self.set_chunk(chunk_pos.right_bottom(), context.right_bottom);
                }
            }
        }

        self.current_tick += 1;

        updates_count
    }

    pub fn set_cell(&mut self, pos: GlobalCellPos, cell: Cell) {
        let chunk = self.ensure_chunk(pos.chunk);
        chunk.set_cell(pos.cell, cell);

        chunk.should_update = true;
        chunk.should_redraw = true;
    }
}
