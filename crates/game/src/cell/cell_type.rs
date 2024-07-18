#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, strum::Display)]
pub enum CellType {
    Empty,
    Wall,
    Sand,
    Water,
    Gas,
}

impl Default for CellType {
    fn default() -> Self {
        Self::Empty
    }
}
