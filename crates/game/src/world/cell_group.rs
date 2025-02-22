use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CellGroup {
    pub cells: [Cell; 4],
}

impl CellGroup {
    pub fn process(self, rules: &[UpdateRule], random_state: &mut u64) -> Option<Self> {
        for rule in rules {
            if let Some(result_group) = self.try_apply_rule(rule, random_state) {
                return Some(result_group);
            }
        }

        None
    }

    #[inline(always)]
    pub fn swap_x(&mut self) {
        self.cells.swap(0, 1);
        self.cells.swap(2, 3);
    }

    #[inline(always)]
    pub fn swap_y(&mut self) {
        self.cells.swap(0, 2);
        self.cells.swap(1, 3);
    }

    #[inline(always)]
    pub fn swap(&mut self, along_x: bool, along_y: bool) {
        if along_x {
            self.swap_x();
        }
        if along_y {
            self.swap_y();
        }
    }

    pub fn try_apply_rule(&self, rule: &UpdateRule, random_state: &mut u64) -> Option<Self> {
        let mut new_group = self.clone();
        let swapped_x = rule.x_symmetry && next_random_bool(random_state);
        let swapped_y = rule.y_symmetry && next_random_bool(random_state);
        // let swapped_x = false;
        // let swapped_y = false;

        new_group.swap(swapped_x, swapped_y);

        if new_group.check_rule_condition(&rule.condition) {
            let mut new_group = new_group;
            new_group.apply_action(&rule.action);
            new_group.swap(swapped_x, swapped_y);
            return Some(new_group);
        }

        if rule.x_symmetry {
            new_group.swap_x();
            if new_group.check_rule_condition(&rule.condition) {
                new_group.apply_action(&rule.action);
                new_group.swap_x();
                new_group.swap(swapped_x, swapped_y);
                return Some(new_group);
            }

            new_group.swap_x();
        }

        if rule.y_symmetry {
            new_group.swap_y();
            if new_group.check_rule_condition(&rule.condition) {
                new_group.apply_action(&rule.action);
                new_group.swap_y();
                new_group.swap(swapped_x, swapped_y);
                return Some(new_group);
            }

            new_group.swap_y();
        }

        if rule.x_symmetry && rule.y_symmetry {
            new_group.swap(true, true);
            if new_group.check_rule_condition(&rule.condition) {
                new_group.apply_action(&rule.action);
                new_group.swap(true, true);
                new_group.swap(swapped_x, swapped_y);
                return Some(new_group);
            }

            new_group.swap(true, true);
        }

        new_group.swap(swapped_x, swapped_y);

        None
    }

    pub fn apply_action(&mut self, action: &RuleAction) {
        match action {
            RuleAction::Keep => {}
            RuleAction::Mirror { mirror_y } => {
                if *mirror_y {
                    self.cells.swap(0, 2);
                    self.cells.swap(1, 3);
                } else {
                    self.cells.swap(0, 1);
                    self.cells.swap(2, 3);
                }
            }
            RuleAction::InitCells { cells } => {
                self.cells.iter_mut().enumerate().for_each(|(index, cell)| {
                    if let Some(new_id) = cells[index] {
                        *cell = Cell::new(new_id);
                    }
                });
            }
            RuleAction::Swap {
                group_index_a,
                group_index_b,
            } => {
                self.cells.swap(*group_index_a, *group_index_b);
            }
            RuleAction::Multiple(actions) => {
                for action in *actions {
                    self.apply_action(action);
                }
            }
        }
    }

    pub fn check_rule_condition(&self, condition: &RuleCondition) -> bool {
        match condition {
            RuleCondition::And(conditions) => {
                conditions.iter().all(|c| self.check_rule_condition(c))
            }
            RuleCondition::Or(conditions) => {
                conditions.iter().any(|c| self.check_rule_condition(c))
            }
            RuleCondition::Not(condition) => !self.check_rule_condition(condition),
            RuleCondition::Any => true,
            RuleCondition::MatchId(ids) => self
                .cells
                .iter()
                .enumerate()
                .all(|(index, c)| ids[index].is_none_or(|v| v == c.id)),
            RuleCondition::RegisterEq(registers) => {
                registers.iter().enumerate().all(|(index, register)| {
                    register.is_none_or(|(reg_index, value)| {
                        self.cells[index].registers[reg_index] == value
                    })
                })
            }
        }
    }
}
