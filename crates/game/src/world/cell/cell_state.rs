use crate::*;

pub type CellId = u32;

pub const CELL_REGISTERS_COUNT: usize = 14;

/// Register used to store cell's system information. Do not modify it manually.
pub const CELL_REGISTER_SYSTEM: usize = CELL_REGISTERS_COUNT - 1;

pub const CELL_REGISTER_SYSTEM_FLAGS: usize = 0;
pub const CELL_REGISTER_SYSTEM_BRIGHTNESS_VALUE: usize = 1;
pub const CELL_REGISTER_SYSTEM_FLAG_IS_BRIGHTNESS_SET: u8 = 1 << 0;

/// Register used to track cell's age if it's enabled, may be used for other purposes, but will be
/// incremented every tick
pub const CELL_REGISTER_AGE: usize = CELL_REGISTERS_COUNT - 2;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cell {
    pub id: CellId,
    /// Tick at which cell was last updated
    pub last_update: u32,
    pub registers: [u32; CELL_REGISTERS_COUNT],
}

impl Cell {
    #[inline(always)]
    pub fn new(cells_template: &CellsTemplate, id: CellId) -> Self {
        cells_template.get_cell_meta(id).init()
    }

    #[inline(always)]
    pub fn color(&self, cells_template: &CellsTemplate) -> CellColor {
        cells_template.get_cell_meta(self.id).color
    }

    #[inline(always)]
    pub fn meta<'a>(&self, config: &'a CellsTemplate) -> &'a CellMeta {
        config.get_cell_meta(self.id)
    }
}
