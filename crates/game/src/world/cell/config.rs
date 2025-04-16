use crate::*;
use macroquad::math::Vec2;

#[derive(Debug, Clone, PartialEq)]
pub struct CellsTemplate {
    pub cells: Vec<CellMeta>,
}

impl CellsTemplate {
    pub fn get_cell_meta(&self, id: CellId) -> &CellMeta {
        &self.cells[id as usize]
    }

    pub fn get_cell_meta_by_label(&self, label: &str) -> Option<&CellMeta> {
        self.cells.iter().find(|cell| cell.label == label)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CellMeta {
    pub id: CellId,
    pub color: CellColor,
    pub label: String,
    pub rule: CellRule,
    /// If true, AGE register will be incremented on each tick.
    pub count_age: bool,
    /// Gravity in particle mode
    pub particle_gravity: Vec2,
    pub replaceable_by_particles: bool,
    pub initial_register_values: [u32; CELL_REGISTERS_COUNT],
}

impl CellMeta {
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CellRule {
    /// Do nothing, always succeed.
    ///
    /// NOTE: can be used to randomly stop processing if used in [`CellRule::FirstSuccess`]
    Idle,
    /// If this `condition` is met, `action` will be executed
    If {
        condition: RuleCondition,
        action: Box<CellRule>,
        else_action: Option<Box<CellRule>>,
    },
    /// Swap current cell with cell at position if it has specific id
    SwapWithIds {
        pos: RelativePos,
        match_ids: Vec<CellId>,
    },
    /// Even if rule applied, continue processing other rules.
    ApplyAndContinue(Box<CellRule>),
    /// Rules will be checked in order they are provided and first matching rule will be executed.
    FirstSuccess(Vec<CellRule>),
    /// Pair of rules will be checked in random order and first matching rule will be executed.
    RandomPair(Box<(CellRule, CellRule)>),
    /// Try to apply same rule twice: as is and mirrored by X axis. Randomly choose which one to
    /// apply first.
    SymmetryX(Box<CellRule>),
    /// Same as [`CellRule::SymmetryX`] but mirrored by Y axis.
    SymmetryY(Box<CellRule>),
    /// Same as [`CellRule::SymmetryX`] but instead of mirroring by X axis, swap X and Y
    /// coordinates.
    SymmetryDiagonal(Box<CellRule>),
    /// apply underlying rule as is or mirrored by X axis depending on condition
    MirrorXIf {
        condition: RuleCondition,
        rule: Box<CellRule>,
    },
    /// same as [`CellRule::MirrorXIf`] but mirrored by Y axis
    MirrorYIf {
        condition: RuleCondition,
        rule: Box<CellRule>,
    },
    /// same as [`CellRule::MirrorXIf`] but swap X and Y coordinates instead of mirroring by X axis
    MirrorDiagonalIf {
        condition: RuleCondition,
        rule: Box<CellRule>,
    },

    /// Execute all rules even if some some of them succeed.
    TryAll(Vec<CellRule>),
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

impl CellRule {
    pub fn random_pair(first: CellRule, second: CellRule) -> Self {
        CellRule::RandomPair(Box::new((first, second)))
    }
    pub fn apply_and_continue(rule: CellRule) -> Self {
        CellRule::ApplyAndContinue(Box::new(rule))
    }
    pub fn symmetry_x(rule: CellRule) -> Self {
        CellRule::SymmetryX(Box::new(rule))
    }
    pub fn symmetry_y(rule: CellRule) -> Self {
        CellRule::SymmetryY(Box::new(rule))
    }
    pub fn symmetry_diagonal(rule: CellRule) -> Self {
        CellRule::SymmetryDiagonal(Box::new(rule))
    }

    pub fn if_then(condition: RuleCondition, action: CellRule) -> Self {
        CellRule::If {
            condition,
            action: Box::new(action),
            else_action: None,
        }
    }

    pub fn if_else(condition: RuleCondition, action: CellRule, else_action: CellRule) -> Self {
        CellRule::If {
            condition,
            action: Box::new(action),
            else_action: Some(Box::new(else_action)),
        }
    }

    pub fn mirror_x_if(condition: RuleCondition, rule: CellRule) -> Self {
        CellRule::MirrorXIf {
            condition,
            rule: Box::new(rule),
        }
    }
    pub fn mirror_y_if(condition: RuleCondition, rule: CellRule) -> Self {
        CellRule::MirrorYIf {
            condition,
            rule: Box::new(rule),
        }
    }
    pub fn mirror_diagonal_if(condition: RuleCondition, rule: CellRule) -> Self {
        CellRule::MirrorDiagonalIf {
            condition,
            rule: Box::new(rule),
        }
    }

    pub fn set_reg_value(register: u8, value: u32) -> Self {
        CellRule::SetRegister {
            register,
            value,
            pos: RelativePos::self_pos(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuleCondition {
    And(Vec<RuleCondition>),
    Or(Vec<RuleCondition>),
    Not(Box<RuleCondition>),
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
        cell_id_list: Vec<CellId>,
    },
    /// Check if cell at position does not have id from list
    RelativeCellNotIn {
        pos: RelativePos,
        cell_id_list: Vec<CellId>,
    },
    BinaryOp {
        op: ConditionBinaryOp,
        a: ConditionArg,
        b: ConditionArg,
    },
    Always,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConditionArg {
    Register { pos: RelativePos, register: u8 },
    Value(u32),
}

impl ConditionArg {
    pub fn register(register: u8) -> Self {
        ConditionArg::Register {
            pos: RelativePos::self_pos(),
            register,
        }
    }

    pub fn register_at(pos: RelativePos, register: u8) -> Self {
        ConditionArg::Register { pos, register }
    }

    pub fn value(value: u32) -> Self {
        ConditionArg::Value(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConditionBinaryOp {
    Eq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
}

impl RuleCondition {
    pub const fn reg_non_zero(register: u8) -> Self {
        RuleCondition::BinaryOp {
            op: ConditionBinaryOp::NotEq,
            a: ConditionArg::Register {
                pos: RelativePos::self_pos(),
                register,
            },
            b: ConditionArg::Value(0),
        }
    }

    pub const fn reg_neg(register: u8) -> Self {
        RuleCondition::BinaryOp {
            op: ConditionBinaryOp::Less,
            a: ConditionArg::Register {
                pos: RelativePos::self_pos(),
                register,
            },
            b: ConditionArg::Value(0),
        }
    }

    pub const fn reg_pos(register: u8) -> Self {
        RuleCondition::BinaryOp {
            op: ConditionBinaryOp::GreaterEq,
            a: ConditionArg::Register {
                pos: RelativePos::self_pos(),
                register,
            },
            b: ConditionArg::Value(0),
        }
    }

    pub const fn reg_not_eq(register: u8, value: u32) -> Self {
        RuleCondition::BinaryOp {
            op: ConditionBinaryOp::NotEq,
            a: ConditionArg::Register {
                pos: RelativePos::self_pos(),
                register,
            },
            b: ConditionArg::Value(value),
        }
    }

    pub const fn reg_eq(register: u8, value: u32) -> Self {
        RuleCondition::BinaryOp {
            op: ConditionBinaryOp::Eq,
            a: ConditionArg::Register {
                pos: RelativePos::self_pos(),
                register,
            },
            b: ConditionArg::Value(value),
        }
    }
}
