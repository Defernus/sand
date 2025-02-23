use crate::*;

pub const CELL_BORDER_ID: CellId = 0;
pub const CELL_VACUUM_ID: CellId = 1;
pub const CELL_SAND_ID: CellId = 2;
pub const CELL_WET_SAND_ID: CellId = 3;
pub const CELL_WATER_ID: CellId = 4;
pub const CELL_STONE_ID: CellId = 5;

pub const CELL_BORDER: CellConfig = CellConfig {
    id: CELL_BORDER_ID,
    color: CellColor::Plain([40, 40, 40, 255]),
    count_age: false,
    initial_register_values: [0; CELL_REGISTERS_COUNT],
    name: "Border",
    rule: CellRule::Idle,
};

pub const CELL_VACUUM: CellConfig = CellConfig {
    id: CELL_VACUUM_ID,
    color: CellColor::Plain([0, 0, 0, 0]),
    count_age: false,
    initial_register_values: [0; CELL_REGISTERS_COUNT],
    name: "Vacuum",
    rule: CellRule::Idle,
};

pub const CELL_SAND: CellConfig = CellConfig {
    id: CELL_SAND_ID,
    color: CellColor::RandomizeBrightness([190, 174, 110, 255], 16),
    count_age: true,
    initial_register_values: [0; CELL_REGISTERS_COUNT],
    name: "Sand",
    rule: CellRule::FirstSuccess(&[
        CellRule::RandomPair(
            &CellRule::SymmetryDiagonal(&CellRule::SymmetryY(&CellRule::Conditioned {
                condition: RuleCondition::RelativeCell {
                    pos: RelativePos::new(1, 1),
                    cell_id: CELL_WATER_ID,
                },
                action: RuleAction::OrderedActions(&[
                    RuleAction::InitCell {
                        pos: RelativePos::new(1, 1),
                        cell_id: CELL_VACUUM_ID,
                    },
                    RuleAction::InitCell {
                        pos: RelativePos::new(0, 0),
                        cell_id: CELL_WET_SAND_ID,
                    },
                ]),
            })),
            &CellRule::SymmetryDiagonal(&CellRule::SymmetryY(&CellRule::Conditioned {
                condition: RuleCondition::RelativeCell {
                    pos: RelativePos::new(0, 1),
                    cell_id: CELL_WATER_ID,
                },
                action: RuleAction::OrderedActions(&[
                    RuleAction::InitCell {
                        pos: RelativePos::new(0, 1),
                        cell_id: CELL_VACUUM_ID,
                    },
                    RuleAction::InitCell {
                        pos: RelativePos::new(0, 0),
                        cell_id: CELL_WET_SAND_ID,
                    },
                ]),
            })),
        ),
        CellRule::SwapWithIds {
            pos: RelativePos::down(),
            match_ids: &[CELL_VACUUM_ID, CELL_WATER_ID],
        },
        CellRule::SymmetryX(&CellRule::SwapWithIds {
            pos: RelativePos::down_right(),
            match_ids: &[CELL_VACUUM_ID, CELL_WATER_ID],
        }),
    ]),
};

pub const CELL_WET_SAND: CellConfig = CellConfig {
    id: CELL_WET_SAND_ID,
    color: CellColor::RandomizeBrightness([130, 120, 77, 255], 16),
    count_age: true,
    initial_register_values: [0; CELL_REGISTERS_COUNT],
    name: "Wet Sand",
    rule: CellRule::FirstSuccess(&[
        CellRule::SwapWithIds {
            pos: RelativePos::down(),
            match_ids: &[CELL_VACUUM_ID, CELL_WATER_ID],
        },
        CellRule::SymmetryX(&CellRule::SwapWithIds {
            pos: RelativePos::down_right(),
            match_ids: &[CELL_VACUUM_ID, CELL_WATER_ID],
        }),
    ]),
};

pub const WATER_IS_INITIALIZED_REGISTER: u8 = 0;
pub const WATER_DIR_REGISTER: u8 = 1;
pub const WATER_DIRECTION_LEFT: u32 = 0;
pub const WATER_DIRECTION_RIGHT: u32 = 1;
pub const CELL_WATER: CellConfig = CellConfig {
    id: 4,
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
                cell_id_list: &[CELL_VACUUM_ID],
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
                    cell_id_list: &[CELL_VACUUM_ID],
                },
                action: RuleAction::SwapWith {
                    pos: RelativePos::new(-1, -1),
                },
            },
            // go down right
            &CellRule::Conditioned {
                condition: RuleCondition::RelativeCellIn {
                    pos: RelativePos::new(1, -1),
                    cell_id_list: &[CELL_VACUUM_ID],
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
                    cell_id_list: &[CELL_VACUUM_ID],
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
                    cell_id_list: &[CELL_VACUUM_ID],
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
                    cell_id_list: &[CELL_VACUUM_ID],
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
                    cell_id_list: &[CELL_VACUUM_ID],
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
    id: CELL_STONE_ID,
    color: CellColor::RandomizeBrightness([120, 120, 120, 255], 32),
    count_age: false,
    initial_register_values: [0; CELL_REGISTERS_COUNT],
    name: "Stone",
    rule: CellRule::Idle,
};

pub const CELLS: &[CellConfig] = &[
    CELL_BORDER,
    CELL_VACUUM,
    CELL_SAND,
    CELL_WET_SAND,
    CELL_WATER,
    CELL_STONE,
];
