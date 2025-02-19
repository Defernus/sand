use macroquad::color::Color;

pub type CellId = u32;
pub type CellData = u32;

// TODO unhardcode cell types and load from assets
pub const CELL_BORDER: CellId = 0;
pub const CELL_VACUUM: CellId = 1;
pub const CELL_SAND: CellId = 2;
pub const CELL_WATER: CellId = 3;
pub const CELL_STONE: CellId = 4;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cell {
    pub id: CellId,
    pub data: CellData,

    /// Used to track if the cell was updated in the current iteration (if current iteration switch is equal to the cell's switch it means the cell was updated)
    pub updated_switch: bool,
}

impl Cell {
    pub fn new(id: CellId) -> Self {
        Self {
            id,
            data: 0,
            updated_switch: false,
        }
    }

    #[inline(always)]
    pub fn get_color(self) -> Color {
        match self.id {
            CELL_BORDER => Color::from_rgba(20, 20, 20, 255),
            CELL_VACUUM => Color::from_rgba(0, 0, 0, 0),
            CELL_SAND => Color::from_rgba(255, 255, 0, 255),
            CELL_WATER => Color::from_rgba(0, 0, 255, 255),
            CELL_STONE => Color::from_rgba(128, 128, 128, 255),
            _ => Color::from_rgba(255, 0, 255, 255),
        }
    }
}
