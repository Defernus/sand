use crate::*;
use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

pub struct CellState {
    cell: Cell,
    action: Action,
    updated: bool,
}

pub const THREADS_COUNT: usize = 100;

impl Sandbox {
    /// Iterate over all cells and prepare actions for each cell.
    pub fn prepare_actions(&self) -> Vec<CellState> {
        let mut cells_actions = self
            .cells
            .clone()
            .into_iter()
            .map(|cell| CellState {
                cell,
                action: Action::Idle,
                updated: false,
            })
            .collect::<Vec<_>>();

        cells_actions
            .par_chunks_mut(THREADS_COUNT)
            .enumerate()
            .for_each(|(chunk_index, chunk)| {
                let index_offset = THREADS_COUNT * chunk_index;

                for (in_chunk_offset, CellState { action, cell, .. }) in
                    chunk.iter_mut().enumerate()
                {
                    let index = index_offset + in_chunk_offset;
                    let (x, y) = self.size.index_to_coords(index);

                    *action = cell.from_prev_state(&self, x, y);
                }
            });

        cells_actions
    }

    /// Apply actions to the cells.
    pub fn apply_actions(&mut self, mut cells_actions: Vec<CellState>) {
        let reverse_y = ::rand::random::<bool>();

        for y in 0..self.size.height {
            let y = if reverse_y {
                self.size.height - y - 1
            } else {
                y
            };

            let reverse_x = ::rand::random::<bool>();

            for x in 0..self.size.width {
                let x = if reverse_x {
                    self.size.width - x - 1
                } else {
                    x
                };

                self.apply_action_at(&mut cells_actions, x, y);
            }
        }
    }

    fn apply_action_at(&mut self, cells_actions: &mut [CellState], x: u16, y: u16) {
        let index = self.size.coords_to_index(x, y);

        let CellState {
            action, updated, ..
        } = cells_actions[index];

        if updated {
            return;
        }

        match action {
            Action::Transform(new_cell) => {
                cells_actions[index].updated = true;
                self.cells[index] = new_cell;
            }
            Action::Swap(neighbor_index) => {
                // Proceed with swap if neighbor cell is not updated yet.
                if cells_actions[neighbor_index].updated {
                    return;
                }
                cells_actions[index].updated = true;

                // We have to access neighbor again due to borrow checker.
                let CellState {
                    updated: neighbor_updated,
                    ..
                } = &mut cells_actions[neighbor_index];
                *neighbor_updated = true;

                let cell = self.cells[index];
                self.cells[index] = self.cells[neighbor_index];
                self.cells[neighbor_index] = cell;
            }
            Action::Idle => {}
        }
    }
}
