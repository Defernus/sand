use crate::*;

pub const CELL_VACUUM_ID: CellId = 0;
pub const CELL_BORDER_ID: CellId = 1;
pub const CELL_SAND_ID: CellId = 2;
pub const CELL_WET_SAND_ID: CellId = 3;
pub const CELL_WATER_ID: CellId = 4;
pub const CELL_STONE_ID: CellId = 5;

pub fn default_cells() -> CellsTemplate {
    pub const WATER_IS_INITIALIZED_REGISTER: u8 = 0;
    pub const WATER_DIR_REGISTER: u8 = 1;
    pub const WATER_DIRECTION_LEFT: u32 = 0;
    pub const WATER_DIRECTION_RIGHT: u32 = 1;

    let cells = vec![
        CellMeta {
            id: CELL_VACUUM_ID,
            color: CellColor::Plain([0, 0, 0, 0]),
            count_age: false,
            initial_register_values: [0; CELL_REGISTERS_COUNT],
            label: "Vacuum".into(),
            rule: CellRule::Idle,
        },
        CellMeta {
            id: CELL_BORDER_ID,
            color: CellColor::Plain([40, 40, 40, 255]),
            count_age: false,
            initial_register_values: [0; CELL_REGISTERS_COUNT],
            label: "Border".into(),
            rule: CellRule::Idle,
        },
        CellMeta {
            id: CELL_SAND_ID,
            color: CellColor::RandomizeBrightness([190, 174, 110, 255], 16),
            count_age: true,
            initial_register_values: [0; CELL_REGISTERS_COUNT],
            label: "Sand".into(),
            rule: CellRule::FirstSuccess(vec![
                CellRule::random_pair(
                    CellRule::symmetry_diagonal(CellRule::symmetry_y(CellRule::Conditioned {
                        condition: RuleCondition::RelativeCell {
                            pos: RelativePos::new(1, 1),
                            cell_id: CELL_WATER_ID,
                        },
                        action: RuleAction::OrderedActions(vec![
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
                    CellRule::symmetry_diagonal(CellRule::symmetry_y(CellRule::Conditioned {
                        condition: RuleCondition::RelativeCell {
                            pos: RelativePos::new(0, 1),
                            cell_id: CELL_WATER_ID,
                        },
                        action: RuleAction::OrderedActions(vec![
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
                    match_ids: vec![CELL_VACUUM_ID, CELL_WATER_ID],
                },
                CellRule::symmetry_x(CellRule::SwapWithIds {
                    pos: RelativePos::down_right(),
                    match_ids: vec![CELL_VACUUM_ID, CELL_WATER_ID],
                }),
            ]),
        },
        CellMeta {
            id: CELL_WET_SAND_ID,
            color: CellColor::RandomizeBrightness([130, 120, 77, 255], 16),
            count_age: true,
            initial_register_values: [0; CELL_REGISTERS_COUNT],
            label: "Wet Sand".into(),
            rule: CellRule::FirstSuccess(vec![
                CellRule::SwapWithIds {
                    pos: RelativePos::down(),
                    match_ids: vec![CELL_VACUUM_ID, CELL_WATER_ID],
                },
                CellRule::symmetry_x(CellRule::SwapWithIds {
                    pos: RelativePos::down_right(),
                    match_ids: vec![CELL_VACUUM_ID, CELL_WATER_ID],
                }),
            ]),
        },
        CellMeta {
            id: CELL_WATER_ID,
            color: CellColor::RandomizeBrightness([20, 20, 220, 255], 8),
            count_age: false,
            initial_register_values: [0; CELL_REGISTERS_COUNT],
            label: "Water".into(),
            rule: CellRule::FirstSuccess(vec![
                // set random direction to water on initialization
                CellRule::apply_and_continue(CellRule::Conditioned {
                    condition: RuleCondition::RegisterEq {
                        pos: RelativePos::self_pos(),
                        register: WATER_IS_INITIALIZED_REGISTER,
                        value: 0,
                    },
                    action: RuleAction::OrderedActions(vec![
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
                    condition: RuleCondition::RelativeCell {
                        pos: RelativePos::new(0, -1),
                        cell_id: CELL_VACUUM_ID,
                    },
                    action: RuleAction::SwapWith {
                        pos: RelativePos::new(0, -1),
                    },
                },
                CellRule::random_pair(
                    // go down left
                    CellRule::Conditioned {
                        condition: RuleCondition::RelativeCell {
                            pos: RelativePos::new(-1, -1),
                            cell_id: CELL_VACUUM_ID,
                        },
                        action: RuleAction::SwapWith {
                            pos: RelativePos::new(-1, -1),
                        },
                    },
                    // go down right
                    CellRule::Conditioned {
                        condition: RuleCondition::RelativeCell {
                            pos: RelativePos::new(1, -1),
                            cell_id: CELL_VACUUM_ID,
                        },
                        action: RuleAction::SwapWith {
                            pos: RelativePos::new(1, -1),
                        },
                    },
                ),
                // go left if direction is set
                CellRule::Conditioned {
                    condition: RuleCondition::And(vec![
                        RuleCondition::RegisterEq {
                            pos: RelativePos::self_pos(),
                            register: WATER_DIR_REGISTER,
                            value: WATER_DIRECTION_LEFT,
                        },
                        RuleCondition::RelativeCell {
                            pos: RelativePos::new(-1, 0),
                            cell_id: CELL_VACUUM_ID,
                        },
                    ]),
                    action: RuleAction::SwapWith {
                        pos: RelativePos::new(-1, 0),
                    },
                },
                // change direction if left is blocked
                CellRule::Conditioned {
                    condition: RuleCondition::And(vec![
                        RuleCondition::RegisterEq {
                            pos: RelativePos::self_pos(),
                            register: WATER_DIR_REGISTER,
                            value: WATER_DIRECTION_LEFT,
                        },
                        RuleCondition::RelativeCellNot {
                            pos: RelativePos::new(-1, 0),
                            cell_id: CELL_VACUUM_ID,
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
                    condition: RuleCondition::And(vec![
                        RuleCondition::RegisterNotEq {
                            pos: RelativePos::self_pos(),
                            register: WATER_DIR_REGISTER,
                            value: WATER_DIRECTION_LEFT,
                        },
                        RuleCondition::RelativeCell {
                            pos: RelativePos::new(1, 0),
                            cell_id: CELL_VACUUM_ID,
                        },
                    ]),
                    action: RuleAction::SwapWith {
                        pos: RelativePos::new(1, 0),
                    },
                },
                // change direction if right is blocked
                CellRule::Conditioned {
                    condition: RuleCondition::And(vec![
                        RuleCondition::RegisterNotEq {
                            pos: RelativePos::self_pos(),
                            register: WATER_DIR_REGISTER,
                            value: WATER_DIRECTION_LEFT,
                        },
                        RuleCondition::RelativeCellNot {
                            pos: RelativePos::new(1, 0),
                            cell_id: CELL_VACUUM_ID,
                        },
                    ]),
                    action: RuleAction::SetRegister {
                        register: WATER_DIR_REGISTER,
                        value: WATER_DIRECTION_LEFT,
                        pos: RelativePos::self_pos(),
                    },
                },
            ]),
        },
        CellMeta {
            id: CELL_STONE_ID,
            color: CellColor::RandomizeBrightness([120, 120, 120, 255], 32),
            count_age: false,
            initial_register_values: [0; CELL_REGISTERS_COUNT],
            label: "Stone".into(),
            rule: CellRule::Idle,
        },
    ];

    CellsTemplate { cells }
}
