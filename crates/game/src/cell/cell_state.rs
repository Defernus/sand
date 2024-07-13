use crate::*;
use macroquad::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, strum::Display)]
pub enum Cell {
    Empty,
    Wall,
    Sand,
    Water,
    Gas,
    Seed,
    PlantHead,
    PlantStem,
    LeafBody,
    LeafSide,
}

impl Default for Cell {
    fn default() -> Self {
        Self::Empty
    }
}

impl Cell {
    pub fn from_prev_state(self, data: &Sandbox, x: u16, y: u16) -> Action {
        let neighbors = data.get_neighbors(x, y);
        let swap_seed = ::rand::random::<u16>();

        let ctx = RuleContext {
            current_cell_x: x,
            current_cell_y: y,
            current_cell: self,
            data,
            neighbors: &neighbors,
            swap_seed,
        };

        ctx.into_action()
    }
}
