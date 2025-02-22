use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UpdateRule {
    /// Try to apply this rule in mirrored form too. Also apply order randomly changed each time.
    pub x_symmetry: bool,
    /// Same as[`UpdateRule::x_symmetry`], but for y axis.
    pub y_symmetry: bool,
    pub condition: RuleCondition,
    pub action: RuleAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuleCondition {
    /// Applied avery time.
    ///
    /// NOTE: be careful with this rule, it makes sense to use it only as last one.
    Any,
    MatchId([Option<CellId>; 4]),
    RegisterEq([Option<(usize, CellRegister)>; 4]),
    And(&'static [RuleCondition]),
    Or(&'static [RuleCondition]),
    Not(&'static RuleCondition),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuleAction {
    /// Keep the same cell group.
    Keep,
    /// Mirror pairs
    Mirror {
        /// If `true` mirror along y axis. if false, along x axis.
        mirror_y: bool,
    },
    /// Init cell if `Some`. skip if `None`.
    InitCells {
        cells: [Option<CellId>; 4],
    },
    Swap {
        group_index_a: usize,
        group_index_b: usize,
    },
    Multiple(&'static [RuleAction]),
}

pub const RULES: &[UpdateRule] = &[
    UpdateRule {
        x_symmetry: true,
        y_symmetry: false,
        condition: RuleCondition::MatchId([Some(CELL_VACUUM.id), None, Some(CELL_SAND.id), None]),
        action: RuleAction::Swap {
            group_index_a: 0,
            group_index_b: 2,
        },
    },
    UpdateRule {
        x_symmetry: true,
        y_symmetry: false,
        condition: RuleCondition::MatchId([None, Some(CELL_VACUUM.id), Some(CELL_SAND.id), None]),
        action: RuleAction::Swap {
            group_index_a: 2,
            group_index_b: 1,
        },
    },
];
