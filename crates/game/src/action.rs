use crate::*;

/// Action to be performed on the cell
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, strum::Display)]
pub enum Action {
    /// Do nothing
    Idle,
    /// Update state of the cell
    Transform(Cell),
    /// Swap current cell with cell at given index
    Swap(usize),
}
