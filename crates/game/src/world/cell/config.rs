use crate::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellConfig {
    pub id: CellId,
    pub color: CellColor,
    pub name: &'static str,
    pub rule: CellRule,
    /// If true, AGE register will be incremented on each tick.
    pub count_age: bool,
    pub initial_register_values: [u32; CELL_REGISTERS_COUNT],
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CellRule {
    /// Do nothing, always succeed.
    ///
    /// NOTE: can be used to randomly stop processing if used in [`CellRule::FirstSuccess`]
    Idle,
    /// If this `condition` is met, `action` will be executed
    Conditioned {
        condition: RuleCondition,
        action: RuleAction,
    },
    /// Swap current cell with cell at position if it has specific id
    SwapWithIds {
        pos: RelativePos,
        match_ids: &'static [CellId],
    },
    /// Even if rule applied, continue processing other rules.
    ApplyAndContinue(&'static CellRule),
    /// Rules will be checked in order they are provided and first matching rule will be executed.
    FirstSuccess(&'static [CellRule]),
    /// Pair of rules will be checked in random order and first matching rule will be executed.
    RandomPair(&'static CellRule, &'static CellRule),
    /// Try to apply same rule twice: as is and mirrored by X axis. Randomly choose which one to
    /// apply first.
    SymmetryX(&'static CellRule),
    /// Same as [`CellRule::SymmetryX`] but mirrored by Y axis.
    SymmetryY(&'static CellRule),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuleCondition {
    And(&'static [RuleCondition]),
    Or(&'static [RuleCondition]),
    Not(&'static RuleCondition),
    /// Check if cell at position has specific id
    RelativeCell {
        pos: RelativePos,
        cell_id: CellId,
    },
    /// Check if cell at position does not have specific id
    RelativeCellNot {
        pos: RelativePos,
        cell_id: CellId,
    },
    /// Check if cell at position has id from list
    RelativeCellIn {
        pos: RelativePos,
        cell_id_list: &'static [CellId],
    },
    /// Check if cell at position does not have id from list
    RelativeCellNotIn {
        pos: RelativePos,
        cell_id_list: &'static [CellId],
    },
    RegisterEq {
        pos: RelativePos,
        register: u8,
        value: u32,
    },
    RegisterNotEq {
        pos: RelativePos,
        register: u8,
        value: u32,
    },
    Always,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuleAction {
    /// Execute all actions from the list one by one in order they are provided
    OrderedActions(&'static [RuleAction]),
    /// Set cell to specific id and initialize it.
    InitCell {
        pos: RelativePos,
        cell_id: CellId,
    },
    SwapWith {
        pos: RelativePos,
    },
    IncrementRegister {
        register: u8,
        pos: RelativePos,
    },
    DecrementRegister {
        register: u8,
        pos: RelativePos,
    },
    SetRegister {
        register: u8,
        value: u32,
        pos: RelativePos,
    },
    SerRegisterRandomMasked {
        register: u8,
        mask: u32,
        pos: RelativePos,
    },
    MoveRegister {
        source_register: u8,
        source_cell: RelativePos,
        target_register: u8,
        target_cell: RelativePos,
    },
}
