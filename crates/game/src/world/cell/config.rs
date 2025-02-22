use crate::*;

pub type CellRegister = u32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellConfig {
    pub id: CellId,
    pub color: CellColor,
    pub name: &'static str,
    /// If true, AGE register will be incremented on each tick.
    pub count_age: bool,
    pub initial_register_values: [CellRegister; CELL_REGISTERS_COUNT],
}

impl CellConfig {
    pub fn init(&self) -> Cell {
        Cell {
            id: self.id,
            last_update: 0,
            registers: self.initial_register_values,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CellColor {
    Plain([u8; 4]),
    /// Randomize base color by adding random value to brightness. Second parameter is max
    /// brightness value.
    RandomizeBrightness([u8; 4], u8),
}

impl CellColor {
    #[inline(always)]
    pub fn calculate(&self, chunk: &mut Chunk, cell_index: usize) -> [u8; 4] {
        match self {
            CellColor::Plain(color) => *color,
            CellColor::RandomizeBrightness(base_color, max_value) => {
                let mut cell = chunk.get_by_index(cell_index);

                let mut system_reg = cell.registers[CELL_REGISTER_SYSTEM].to_le_bytes();
                let mut brightness = system_reg[CELL_REGISTER_SYSTEM_BRIGHTNESS_VALUE];
                let brightness_node_set = system_reg[CELL_REGISTER_SYSTEM_FLAGS]
                    & CELL_REGISTER_SYSTEM_FLAG_IS_BRIGHTNESS_SET
                    == 0;

                if brightness_node_set {
                    let random_value = chunk.get_random_value(cell_index);
                    brightness = (random_value % *max_value as u64) as u8;

                    system_reg[CELL_REGISTER_SYSTEM_BRIGHTNESS_VALUE] = brightness;
                    system_reg[CELL_REGISTER_SYSTEM_FLAGS] |=
                        CELL_REGISTER_SYSTEM_FLAG_IS_BRIGHTNESS_SET;

                    cell.registers[CELL_REGISTER_SYSTEM] = u32::from_le_bytes(system_reg);
                    chunk.set_by_index(cell_index, cell);
                }

                let mut color = *base_color;
                color[0] = color[0].saturating_add(brightness);
                color[1] = color[1].saturating_add(brightness);
                color[2] = color[2].saturating_add(brightness);

                color
            }
        }
    }
}
