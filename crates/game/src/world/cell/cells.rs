use crate::*;

pub const CELL_BORDER: CellConfig = CellConfig {
    id: 0,
    color: CellColor::Plain([40, 40, 40, 255]),
    count_age: false,
    initial_register_values: [0; CELL_REGISTERS_COUNT],
    name: "Border",
    rule: CellRule::Idle,
};

pub const CELL_VACUUM: CellConfig = CellConfig {
    id: 1,
    color: CellColor::Plain([0, 0, 0, 0]),
    count_age: false,
    initial_register_values: [0; CELL_REGISTERS_COUNT],
    name: "Vacuum",
    rule: CellRule::Idle,
};

pub const CELL_SAND: CellConfig = CellConfig {
    id: 2,
    color: CellColor::RandomizeBrightness([220, 220, 0, 255], 32),
    count_age: true,
    initial_register_values: [0; CELL_REGISTERS_COUNT],
    name: "Sand",
    rule: CellRule::FirstSuccess(&[
        CellRule::Conditioned {
            condition: RuleCondition::RelativeCellIn {
                pos: RelativePos::new(0, -1),
                cell_id_list: &[CELL_VACUUM.id, CELL_WATER.id],
            },
            action: RuleAction::SwapWith {
                pos: RelativePos::new(0, -1),
            },
        },
        CellRule::RandomPair(
            &CellRule::Conditioned {
                condition: RuleCondition::RelativeCellIn {
                    pos: RelativePos::new(-1, -1),
                    cell_id_list: &[CELL_VACUUM.id, CELL_WATER.id],
                },
                action: RuleAction::SwapWith {
                    pos: RelativePos::new(-1, -1),
                },
            },
            &CellRule::Conditioned {
                condition: RuleCondition::RelativeCellIn {
                    pos: RelativePos::new(1, -1),
                    cell_id_list: &[CELL_VACUUM.id, CELL_WATER.id],
                },
                action: RuleAction::SwapWith {
                    pos: RelativePos::new(1, -1),
                },
            },
        ),
    ]),
};

pub const WATER_IS_INITIALIZED_REGISTER: u8 = 0;
pub const WATER_DIR_REGISTER: u8 = 1;
pub const WATER_DIRECTION_LEFT: u32 = 0;
pub const WATER_DIRECTION_RIGHT: u32 = 1;
pub const CELL_WATER: CellConfig = CellConfig {
    id: 3,
    color: CellColor::RandomizeBrightness([20, 20, 220, 255], 8),
    count_age: false,
    initial_register_values: [0; CELL_REGISTERS_COUNT],
    name: "Water",
    rule: CellRule::FirstSuccess(&[
        // set random direction to water on initialization
        CellRule::ApplyAndContinue(&CellRule::Conditioned {
            condition: RuleCondition::RegisterEq {
                pos: RelativePos::self_pos(),
                register: WATER_IS_INITIALIZED_REGISTER,
                value: 0,
            },
            action: RuleAction::OrderedActions(&[
                RuleAction::SetRegister {
                    register: WATER_IS_INITIALIZED_REGISTER,
                    value: 1,
                    pos: RelativePos::self_pos(),
                },
                RuleAction::SerRegisterRandomMasked {
                    register: WATER_DIR_REGISTER,
                    mask: 1,
                    pos: RelativePos::self_pos(),
                },
            ]),
        }),
        // go down
        CellRule::Conditioned {
            condition: RuleCondition::RelativeCellIn {
                pos: RelativePos::new(0, -1),
                cell_id_list: &[CELL_VACUUM.id],
            },
            action: RuleAction::SwapWith {
                pos: RelativePos::new(0, -1),
            },
        },
        CellRule::RandomPair(
            // go down left
            &CellRule::Conditioned {
                condition: RuleCondition::RelativeCellIn {
                    pos: RelativePos::new(-1, -1),
                    cell_id_list: &[CELL_VACUUM.id],
                },
                action: RuleAction::SwapWith {
                    pos: RelativePos::new(-1, -1),
                },
            },
            // go down right
            &CellRule::Conditioned {
                condition: RuleCondition::RelativeCellIn {
                    pos: RelativePos::new(1, -1),
                    cell_id_list: &[CELL_VACUUM.id],
                },
                action: RuleAction::SwapWith {
                    pos: RelativePos::new(1, -1),
                },
            },
        ),
        // go left if direction is set
        CellRule::Conditioned {
            condition: RuleCondition::And(&[
                RuleCondition::RegisterEq {
                    pos: RelativePos::self_pos(),
                    register: WATER_DIR_REGISTER,
                    value: WATER_DIRECTION_LEFT,
                },
                RuleCondition::RelativeCellIn {
                    pos: RelativePos::new(-1, 0),
                    cell_id_list: &[CELL_VACUUM.id],
                },
            ]),
            action: RuleAction::SwapWith {
                pos: RelativePos::new(-1, 0),
            },
        },
        // change direction if left is blocked
        CellRule::Conditioned {
            condition: RuleCondition::And(&[
                RuleCondition::RegisterEq {
                    pos: RelativePos::self_pos(),
                    register: WATER_DIR_REGISTER,
                    value: WATER_DIRECTION_LEFT,
                },
                RuleCondition::RelativeCellNotIn {
                    pos: RelativePos::new(-1, 0),
                    cell_id_list: &[CELL_VACUUM.id],
                },
            ]),
            action: RuleAction::SetRegister {
                register: WATER_DIR_REGISTER,
                value: WATER_DIRECTION_RIGHT,
                pos: RelativePos::self_pos(),
            },
        },
        // go right if direction is set
        CellRule::Conditioned {
            condition: RuleCondition::And(&[
                RuleCondition::RegisterNotEq {
                    pos: RelativePos::self_pos(),
                    register: WATER_DIR_REGISTER,
                    value: WATER_DIRECTION_LEFT,
                },
                RuleCondition::RelativeCellIn {
                    pos: RelativePos::new(1, 0),
                    cell_id_list: &[CELL_VACUUM.id],
                },
            ]),
            action: RuleAction::SwapWith {
                pos: RelativePos::new(1, 0),
            },
        },
        // change direction if right is blocked
        CellRule::Conditioned {
            condition: RuleCondition::And(&[
                RuleCondition::RegisterNotEq {
                    pos: RelativePos::self_pos(),
                    register: WATER_DIR_REGISTER,
                    value: WATER_DIRECTION_LEFT,
                },
                RuleCondition::RelativeCellNotIn {
                    pos: RelativePos::new(1, 0),
                    cell_id_list: &[CELL_VACUUM.id],
                },
            ]),
            action: RuleAction::SetRegister {
                register: WATER_DIR_REGISTER,
                value: WATER_DIRECTION_LEFT,
                pos: RelativePos::self_pos(),
            },
        },
    ]),
};

pub const CELL_STONE: CellConfig = CellConfig {
    id: 4,
    color: CellColor::RandomizeBrightness([120, 120, 120, 255], 32),
    count_age: false,
    initial_register_values: [0; CELL_REGISTERS_COUNT],
    name: "Stone",
    rule: CellRule::Idle,
};

pub const CELLS: &[CellConfig] = &[CELL_BORDER, CELL_VACUUM, CELL_SAND, CELL_WATER, CELL_STONE];
