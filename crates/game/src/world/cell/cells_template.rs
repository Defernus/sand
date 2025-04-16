use crate::*;
use macroquad::math::Vec2;

pub const CELL_VACUUM_LABEL: &str = "Vacuum";
pub const CELL_STONE_LABEL: &str = "Stone";
pub const CELL_SAND_LABEL: &str = "Sand";
pub const CELL_WET_SAND_LABEL: &str = "Wet Sand";
pub const CELL_WATER_LABEL: &str = "Water";

pub fn default_cells() -> CellsTemplate {
    const WATER_IS_INITIALIZED_REGISTER: u8 = 0;
    const WATER_DIR_REGISTER: u8 = 1;
    const WATER_DIRECTION_LEFT: u32 = 0;
    const WATER_DIRECTION_RIGHT: u32 = 1;

    let mut builder = CellTemplateBuilder::new();

    let vacuum_id = builder.ensure_id_by_label(CELL_VACUUM_LABEL);
    let _stone_id = builder.ensure_id_by_label(CELL_STONE_LABEL);
    let sand_id = builder.ensure_id_by_label(CELL_SAND_LABEL);
    let wet_sand_id = builder.ensure_id_by_label(CELL_WET_SAND_LABEL);
    let water_id = builder.ensure_id_by_label(CELL_WATER_LABEL);

    let default_gravity = Vec2::new(0.0, -10.0);

    builder.add_cell(CellMeta {
        color: CellColor::Plain([0, 0, 0, 0]),
        particle_gravity: default_gravity,
        replaceable_by_particles: true,
        label: CELL_VACUUM_LABEL.into(),
        rule: CellRule::Idle,

        count_age: false,
        initial_register_values: Default::default(),
        id: Default::default(),
    });

    builder.add_cell(CellMeta {
        color: CellColor::RandomizeBrightness([120, 120, 120, 255], 32),
        particle_gravity: default_gravity,
        replaceable_by_particles: false,
        label: CELL_STONE_LABEL.into(),
        rule: CellRule::Idle,

        count_age: false,
        initial_register_values: Default::default(),
        id: Default::default(),
    });

    builder.add_cell(CellMeta {
        id: sand_id,
        color: CellColor::RandomizeBrightness([190, 174, 110, 255], 16),
        particle_gravity: default_gravity,
        replaceable_by_particles: false,
        count_age: true,
        initial_register_values: [0; CELL_REGISTERS_COUNT],
        label: CELL_SAND_LABEL.into(),
        rule: CellRule::FirstSuccess(vec![
            CellRule::random_pair(
                CellRule::symmetry_diagonal(CellRule::symmetry_y(CellRule::if_then(
                    RuleCondition::RelativeCell {
                        pos: RelativePos::new(1, 1),
                        cell_id: water_id,
                    },
                    CellRule::TryAll(vec![
                        CellRule::InitCell {
                            pos: RelativePos::new(1, 1),
                            cell_id: vacuum_id,
                        },
                        CellRule::InitCell {
                            pos: RelativePos::new(0, 0),
                            cell_id: wet_sand_id,
                        },
                    ]),
                ))),
                CellRule::symmetry_diagonal(CellRule::symmetry_y(CellRule::if_then(
                    RuleCondition::RelativeCell {
                        pos: RelativePos::new(0, 1),
                        cell_id: water_id,
                    },
                    CellRule::TryAll(vec![
                        CellRule::InitCell {
                            pos: RelativePos::new(0, 1),
                            cell_id: vacuum_id,
                        },
                        CellRule::InitCell {
                            pos: RelativePos::new(0, 0),
                            cell_id: wet_sand_id,
                        },
                    ]),
                ))),
            ),
            CellRule::SwapWithIds {
                pos: RelativePos::down(),
                match_ids: vec![vacuum_id, water_id],
            },
            CellRule::symmetry_x(CellRule::SwapWithIds {
                pos: RelativePos::down_right(),
                match_ids: vec![vacuum_id, water_id],
            }),
        ]),
    });

    builder.add_cell(CellMeta {
        id: wet_sand_id,
        color: CellColor::RandomizeBrightness([130, 120, 77, 255], 16),
        particle_gravity: default_gravity,
        replaceable_by_particles: false,
        count_age: true,
        initial_register_values: [0; CELL_REGISTERS_COUNT],
        label: CELL_WET_SAND_LABEL.into(),
        rule: CellRule::FirstSuccess(vec![
            CellRule::SwapWithIds {
                pos: RelativePos::down(),
                match_ids: vec![vacuum_id, water_id],
            },
            CellRule::symmetry_x(CellRule::SwapWithIds {
                pos: RelativePos::down_right(),
                match_ids: vec![vacuum_id, water_id],
            }),
        ]),
    });

    builder.add_cell(CellMeta {
        id: water_id,
        color: CellColor::RandomizeBrightness([20, 20, 220, 255], 8),
        particle_gravity: default_gravity,
        replaceable_by_particles: false,
        count_age: false,
        initial_register_values: [0; CELL_REGISTERS_COUNT],
        label: CELL_WATER_LABEL.into(),
        rule: CellRule::FirstSuccess(vec![
            // set random direction to water on initialization
            CellRule::apply_and_continue(CellRule::if_then(
                RuleCondition::reg_eq(WATER_IS_INITIALIZED_REGISTER, 0),
                CellRule::TryAll(vec![
                    CellRule::set_reg_value(WATER_IS_INITIALIZED_REGISTER, 1),
                    CellRule::SerRegisterRandomMasked {
                        register: WATER_DIR_REGISTER,
                        mask: 1,
                        pos: RelativePos::self_pos(),
                    },
                ]),
            )),
            // go down
            CellRule::if_then(
                RuleCondition::RelativeCell {
                    pos: RelativePos::new(0, -1),
                    cell_id: vacuum_id,
                },
                CellRule::SwapWith {
                    pos: RelativePos::new(0, -1),
                },
            ),
            CellRule::random_pair(
                // go down left
                CellRule::if_then(
                    RuleCondition::RelativeCell {
                        pos: RelativePos::new(-1, -1),
                        cell_id: vacuum_id,
                    },
                    CellRule::SwapWith {
                        pos: RelativePos::new(-1, -1),
                    },
                ),
                // go down right
                CellRule::if_then(
                    RuleCondition::RelativeCell {
                        pos: RelativePos::new(1, -1),
                        cell_id: vacuum_id,
                    },
                    CellRule::SwapWith {
                        pos: RelativePos::new(1, -1),
                    },
                ),
            ),
            // go left if direction is set
            CellRule::if_then(
                RuleCondition::And(vec![
                    RuleCondition::reg_eq(WATER_DIR_REGISTER, WATER_DIRECTION_LEFT),
                    RuleCondition::RelativeCell {
                        pos: RelativePos::new(-1, 0),
                        cell_id: vacuum_id,
                    },
                ]),
                CellRule::SwapWith {
                    pos: RelativePos::new(-1, 0),
                },
            ),
            // change direction if left is blocked
            CellRule::if_then(
                RuleCondition::And(vec![
                    RuleCondition::reg_eq(WATER_DIR_REGISTER, WATER_DIRECTION_LEFT),
                    RuleCondition::RelativeCellNot {
                        pos: RelativePos::new(-1, 0),
                        cell_id: vacuum_id,
                    },
                ]),
                CellRule::SetRegister {
                    register: WATER_DIR_REGISTER,
                    value: WATER_DIRECTION_RIGHT,
                    pos: RelativePos::self_pos(),
                },
            ),
            // go right if direction is set
            CellRule::if_then(
                RuleCondition::And(vec![
                    RuleCondition::reg_not_eq(WATER_DIR_REGISTER, WATER_DIRECTION_LEFT),
                    RuleCondition::RelativeCell {
                        pos: RelativePos::new(1, 0),
                        cell_id: vacuum_id,
                    },
                ]),
                CellRule::SwapWith {
                    pos: RelativePos::new(1, 0),
                },
            ),
            // change direction if right is blocked
            CellRule::if_then(
                RuleCondition::And(vec![
                    RuleCondition::reg_not_eq(WATER_DIR_REGISTER, WATER_DIRECTION_LEFT),
                    RuleCondition::RelativeCellNot {
                        pos: RelativePos::new(1, 0),
                        cell_id: vacuum_id,
                    },
                ]),
                CellRule::SetRegister {
                    register: WATER_DIR_REGISTER,
                    value: WATER_DIRECTION_LEFT,
                    pos: RelativePos::self_pos(),
                },
            ),
        ]),
    });

    builder.build().expect("Failed to build cells")
}
