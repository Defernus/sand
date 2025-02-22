use crate::*;

pub const CELL_BORDER: CellConfig = CellConfig {
    id: 0,
    color: CellColor::Plain([40, 40, 40, 255]),
    count_age: false,
    initial_register_values: [0; CELL_REGISTERS_COUNT],
    name: "Border",
};

pub const CELL_VACUUM: CellConfig = CellConfig {
    id: 0,
    color: CellColor::Plain([0, 0, 0, 0]),
    count_age: false,
    initial_register_values: [0; CELL_REGISTERS_COUNT],
    name: "Vacuum",
};

pub const CELL_SAND: CellConfig = CellConfig {
    id: 1,
    color: CellColor::RandomizeBrightness([220, 220, 0, 255], 32),
    count_age: true,
    initial_register_values: [0; CELL_REGISTERS_COUNT],
    name: "Sand",
};

pub const WATER_IS_INITIALIZED_REGISTER: u8 = 0;
pub const WATER_DIR_REGISTER: u8 = 1;
pub const WATER_DIRECTION_LEFT: u32 = 0;
pub const WATER_DIRECTION_RIGHT: u32 = 1;
pub const CELL_WATER: CellConfig = CellConfig {
    id: 2,
    color: CellColor::RandomizeBrightness([20, 20, 220, 255], 8),
    count_age: false,
    initial_register_values: [0; CELL_REGISTERS_COUNT],
    name: "Water",
};

pub const CELL_STONE: CellConfig = CellConfig {
    id: 3,
    color: CellColor::RandomizeBrightness([120, 120, 120, 255], 32),
    count_age: false,
    initial_register_values: [0; CELL_REGISTERS_COUNT],
    name: "Stone",
};

pub const CELLS: &[CellConfig] = &[CELL_VACUUM, CELL_SAND, CELL_WATER, CELL_STONE];
