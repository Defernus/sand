use crate::*;
use macroquad::math::Vec2;
use nohash_hasher::IntMap;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

pub const UPDATE_DELTA_TIME: f32 = 1.0 / 20.0;

/// State of the physical simulation of the world.
pub struct WorldState {
    chunks: IntMap<ChunkPos, Chunk>,
    current_tick: u32,
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            chunks: Default::default(),
            current_tick: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    #[inline(always)]
    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn ensure_chunk(&mut self, pos: ChunkPos, cells_template: &CellsTemplate) -> &mut Chunk {
        self.chunks
            .entry(pos)
            .or_insert_with(|| Chunk::new(cells_template))
    }

    pub fn take_chunk(&mut self, pos: ChunkPos, cells_template: &CellsTemplate) -> Chunk {
        self.chunks
            .remove(&pos)
            .unwrap_or_else(|| Chunk::new(cells_template))
    }

    pub fn set_chunk(&mut self, pos: ChunkPos, chunk: Chunk) -> Option<Chunk> {
        self.chunks.insert(pos, chunk)
    }

    pub fn update_state(&mut self, cells_template: &CellsTemplate) -> usize {
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
                    if chunk.should_update()
                        && true_mod(pos.x, 3) == x_rem
                        && true_mod(pos.y, 3) == y_rem
                    {
                        Some(pos)
                    } else {
                        None
                    }
                })
                .collect();

            updates_count += chunk_group.len();
        }

        for group in update_groups {
            let mut update_contexts =
                Vec::<(ChunkUpdateContext, ChunkPos)>::with_capacity(group.len());

            for chunk_pos in group {
                update_contexts.push((
                    ChunkUpdateContext {
                        cells_template,
                        current_tick: self.current_tick,
                        center: self.take_chunk(chunk_pos, cells_template),
                        left: self.take_chunk(chunk_pos.left(), cells_template),
                        right: self.take_chunk(chunk_pos.right(), cells_template),
                        top: self.take_chunk(chunk_pos.top(), cells_template),
                        bottom: self.take_chunk(chunk_pos.bottom(), cells_template),
                        left_top: self.take_chunk(chunk_pos.left_top(), cells_template),
                        right_top: self.take_chunk(chunk_pos.right_top(), cells_template),
                        left_bottom: self.take_chunk(chunk_pos.left_bottom(), cells_template),
                        right_bottom: self.take_chunk(chunk_pos.right_bottom(), cells_template),
                        delta_time: UPDATE_DELTA_TIME,
                    },
                    chunk_pos,
                ));
            }

            update_contexts
                .par_iter_mut()
                .for_each(|(context, _chunk_pos)| {
                    context.process();
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

        self.current_tick += 1;

        updates_count
    }

    pub fn set_cell(&mut self, pos: GlobalCellPos, cell: Cell, cells_template: &CellsTemplate) {
        let chunk = self.ensure_chunk(pos.chunk, cells_template);
        chunk.set_cell(pos.cell, cell);

        chunk.set_should_update(true);
        chunk.set_should_redraw(true);
    }

    pub fn add_particle_rand_vel(
        &mut self,
        pos: GlobalCellPos,
        cell_meta: &CellMeta,
        cells_template: &CellsTemplate,
    ) {
        let vel = Vec2::new(
            rand::random::<f32>() * 2.0 - 1.0,
            rand::random::<f32>() * 2.0 - 1.0,
        ) * 10.0;

        self.add_particle(pos, vel, cell_meta, cells_template);
    }

    pub fn add_particle(
        &mut self,
        pos: GlobalCellPos,
        vel: Vec2,
        cell_meta: &CellMeta,
        cells_template: &CellsTemplate,
    ) {
        if cell_meta.replaceable_by_particles {
            // if vacuum or something just spawn as a cell
            self.set_cell(pos, cell_meta.init(), cells_template);

            return;
        }

        let chunk = self.ensure_chunk(pos.chunk, cells_template);
        let in_chunk_pos = pos.cell.to_vec();
        let color = cell_meta.color.calculate(chunk, pos.cell.to_index());

        let particle = Particle {
            cell_id: cell_meta.id,
            color,
            age: 0,
            gravity: cell_meta.particle_gravity,
            in_chunk_pos,
            vel,
        };

        chunk.particles.push(particle);
    }
}
