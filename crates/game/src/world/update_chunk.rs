use crate::*;

pub struct ChunkUpdateContext {
    pub current_tick: u32,
    pub center: Chunk,
    pub left: Chunk,
    pub right: Chunk,
    pub top: Chunk,
    pub bottom: Chunk,
    pub left_top: Chunk,
    pub right_top: Chunk,
    pub left_bottom: Chunk,
    pub right_bottom: Chunk,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelativeTransformation {
    pub mirror_x: bool,
    pub mirror_y: bool,
    pub mirror_diagonal: bool,
}

impl RelativeTransformation {
    pub const fn identity() -> Self {
        Self {
            mirror_x: false,
            mirror_y: false,
            mirror_diagonal: false,
        }
    }

    pub const fn mirror_x(self) -> Self {
        Self {
            mirror_x: !self.mirror_x,
            mirror_y: self.mirror_y,
            mirror_diagonal: self.mirror_diagonal,
        }
    }

    pub const fn mirror_y(self) -> Self {
        Self {
            mirror_x: self.mirror_x,
            mirror_y: !self.mirror_y,
            mirror_diagonal: self.mirror_diagonal,
        }
    }

    pub const fn mirror_diagonal(self) -> Self {
        Self {
            mirror_x: self.mirror_y,
            mirror_y: self.mirror_x,
            mirror_diagonal: !self.mirror_diagonal,
        }
    }
}

/// Cell relative position
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelativePos {
    pub x: i8,
    pub y: i8,
}

impl RelativePos {
    pub const fn self_pos() -> Self {
        Self { x: 0, y: 0 }
    }

    pub const fn new(x: i8, y: i8) -> Self {
        Self { x, y }
    }

    pub const fn down() -> Self {
        Self { x: 0, y: -1 }
    }

    pub const fn down_left() -> Self {
        Self { x: -1, y: -1 }
    }

    pub const fn down_right() -> Self {
        Self { x: 1, y: -1 }
    }

    #[inline(always)]
    pub const fn transform(mut self, transformation: RelativeTransformation) -> Self {
        if transformation.mirror_x {
            self.x = -self.x;
        }
        if transformation.mirror_y {
            self.y = -self.y;
        }
        if transformation.mirror_diagonal {
            core::mem::swap(&mut self.x, &mut self.y);
        }

        self
    }
}

impl ChunkUpdateContext {
    /// This function will process only central chunk, but it will also access the surrounding
    /// chunks and in some cases modify them (e.g. sand falling)
    pub fn process(&mut self) {
        self.center.should_update = false;

        let rev_row = ::rand::random::<bool>();
        let rev_col = ::rand::random::<bool>();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let index = CellPos {
                    y: if rev_col { CHUNK_SIZE - 1 - x } else { x } as u16,
                    x: if rev_row { CHUNK_SIZE - 1 - y } else { y } as u16,
                }
                .to_index();
                self.update_cell(index);
            }
        }

        // If chunk updated next frame we need to update surrounding chunks
        if self.center.should_update {
            self.bottom.should_update = true;
            self.top.should_update = true;
            self.left.should_update = true;
            self.right.should_update = true;
            self.left_top.should_update = true;
            self.right_top.should_update = true;
            self.left_bottom.should_update = true;
            self.right_bottom.should_update = true;
        }
    }

    #[inline(always)]
    fn update_cell(&mut self, cell_index: usize) {
        let cell = self.center.get_by_index(cell_index);
        if cell.last_update == self.current_tick {
            return;
        }
        let cell_config = cell.config();

        self.try_apply_rule(
            &cell_config.rule,
            cell_index,
            cell,
            RelativeTransformation::default(),
        );
    }

    fn try_apply_rule(
        &mut self,
        rule: &CellRule,
        cell_index: usize,
        cell: Cell,
        transformation: RelativeTransformation,
    ) -> bool {
        match rule {
            CellRule::FirstSuccess(list) => {
                for rule in *list {
                    if self.try_apply_rule(rule, cell_index, cell, transformation) {
                        return true;
                    }
                }

                false
            }
            CellRule::Conditioned { condition, action } => {
                if self.check_condition(condition, cell_index, transformation) {
                    self.apply_action(action, cell_index, transformation);
                    return true;
                }

                false
            }
            CellRule::RandomPair(rule_a, rule_b) => {
                let random_value = self.center.get_random_value(cell_index);

                if random_value & 1 == 0 {
                    self.apply_rule_pair(
                        cell_index,
                        cell,
                        rule_a,
                        transformation,
                        rule_b,
                        transformation,
                    )
                } else {
                    self.apply_rule_pair(
                        cell_index,
                        cell,
                        rule_b,
                        transformation,
                        rule_a,
                        transformation,
                    )
                }
            }
            CellRule::ApplyAndContinue(rule) => {
                self.try_apply_rule(rule, cell_index, cell, transformation);
                false
            }
            CellRule::SwapWithIds { pos, match_ids } => {
                let pos = get_absolute_cell_pos(cell_index, pos.transform(transformation));
                let cell = self.get_cell(pos);

                if match_ids.contains(&cell.id) {
                    self.swap_cells(AbsoluteCellPos::central(cell_index), pos);
                    return true;
                }

                false
            }
            CellRule::SymmetryX(rule) => {
                let random_value = self.center.get_random_value(cell_index);

                if random_value & 1 == 0 {
                    self.apply_rule_pair(
                        cell_index,
                        cell,
                        rule,
                        transformation,
                        rule,
                        transformation.mirror_x(),
                    )
                } else {
                    self.apply_rule_pair(
                        cell_index,
                        cell,
                        rule,
                        transformation.mirror_x(),
                        rule,
                        transformation,
                    )
                }
            }
            CellRule::SymmetryY(rule) => {
                let random_value = self.center.get_random_value(cell_index);

                if random_value & 1 == 0 {
                    self.apply_rule_pair(
                        cell_index,
                        cell,
                        rule,
                        transformation,
                        rule,
                        transformation.mirror_y(),
                    )
                } else {
                    self.apply_rule_pair(
                        cell_index,
                        cell,
                        rule,
                        transformation.mirror_y(),
                        rule,
                        transformation,
                    )
                }
            }
            CellRule::SymmetryDiagonal(rule) => {
                let random_value = self.center.get_random_value(cell_index);

                if random_value & 1 == 0 {
                    self.apply_rule_pair(
                        cell_index,
                        cell,
                        rule,
                        transformation,
                        rule,
                        transformation.mirror_diagonal(),
                    )
                } else {
                    self.apply_rule_pair(
                        cell_index,
                        cell,
                        rule,
                        transformation.mirror_diagonal(),
                        rule,
                        transformation,
                    )
                }
            }
            CellRule::Idle => true,
        }
    }

    fn apply_rule_pair(
        &mut self,
        cell_index: usize,
        cell: Cell,
        a: &CellRule,
        a_transformation: RelativeTransformation,
        b: &CellRule,
        b_transformation: RelativeTransformation,
    ) -> bool {
        if self.try_apply_rule(a, cell_index, cell, a_transformation) {
            return true;
        }

        self.try_apply_rule(b, cell_index, cell, b_transformation)
    }

    fn check_condition(
        &self,
        condition: &RuleCondition,
        cell_index: usize,
        transformation: RelativeTransformation,
    ) -> bool {
        match condition {
            RuleCondition::RelativeCell { pos, cell_id } => {
                let pos = get_absolute_cell_pos(cell_index, pos.transform(transformation));
                let cell = self.get_cell(pos);
                cell.id == *cell_id
            }
            RuleCondition::RelativeCellNot { pos, cell_id } => {
                let pos = get_absolute_cell_pos(cell_index, pos.transform(transformation));
                let cell = self.get_cell(pos);
                cell.id != *cell_id
            }
            RuleCondition::RelativeCellIn { pos, cell_id_list } => {
                let pos = get_absolute_cell_pos(cell_index, pos.transform(transformation));
                let cell = self.get_cell(pos);
                cell_id_list.contains(&cell.id)
            }
            RuleCondition::RelativeCellNotIn { pos, cell_id_list } => {
                let pos = get_absolute_cell_pos(cell_index, pos.transform(transformation));
                let cell = self.get_cell(pos);
                !cell_id_list.contains(&cell.id)
            }
            RuleCondition::And(conditions) => conditions
                .iter()
                .all(|c| self.check_condition(c, cell_index, transformation)),
            RuleCondition::Or(conditions) => conditions
                .iter()
                .any(|c| self.check_condition(c, cell_index, transformation)),
            RuleCondition::Not(condition) => {
                !self.check_condition(condition, cell_index, transformation)
            }
            RuleCondition::RegisterEq {
                pos,
                register,
                value,
            } => {
                let pos = get_absolute_cell_pos(cell_index, pos.transform(transformation));
                let cell = self.get_cell(pos);
                cell.registers[*register as usize] == *value
            }
            RuleCondition::RegisterNotEq {
                pos,
                register,
                value,
            } => {
                let pos = get_absolute_cell_pos(cell_index, pos.transform(transformation));
                let cell = self.get_cell(pos);
                cell.registers[*register as usize] != *value
            }
            RuleCondition::Always => true,
        }
    }

    fn apply_action(
        &mut self,
        action: &RuleAction,
        cell_index: usize,
        transformation: RelativeTransformation,
    ) {
        match action {
            RuleAction::OrderedActions(list) => {
                for action in *list {
                    self.apply_action(action, cell_index, transformation);
                }
            }
            RuleAction::InitCell { pos, cell_id } => {
                let pos = get_absolute_cell_pos(cell_index, pos.transform(transformation));
                self.set_cell(pos, Cell::new(*cell_id));
            }
            RuleAction::SwapWith { pos } => {
                let pos = get_absolute_cell_pos(cell_index, pos.transform(transformation));
                self.swap_cells(AbsoluteCellPos::central(cell_index), pos);
            }
            RuleAction::IncrementRegister { register, pos } => {
                let register_index = *register as usize;
                assert!(
                    register_index < CELL_REGISTERS_COUNT,
                    "Register out of bounds"
                );
                let pos = get_absolute_cell_pos(cell_index, pos.transform(transformation));
                let mut cell = self.get_cell(pos);
                cell.registers[register_index] = cell.registers[register_index].wrapping_add(1);
                self.set_cell(pos, cell);
            }
            RuleAction::DecrementRegister { register, pos } => {
                let register_index = *register as usize;
                assert!(
                    register_index < CELL_REGISTERS_COUNT,
                    "Register out of bounds"
                );
                let pos = get_absolute_cell_pos(cell_index, pos.transform(transformation));
                let mut cell = self.get_cell(pos);
                cell.registers[register_index] = cell.registers[register_index].wrapping_sub(1);
                self.set_cell(pos, cell);
            }
            RuleAction::SetRegister {
                register,
                value,
                pos,
            } => {
                let register_index = *register as usize;
                assert!(
                    register_index < CELL_REGISTERS_COUNT,
                    "Register out of bounds"
                );
                let pos = get_absolute_cell_pos(cell_index, pos.transform(transformation));
                let mut cell = self.get_cell(pos);
                cell.registers[register_index] = *value;
                self.set_cell(pos, cell);
            }
            RuleAction::MoveRegister {
                source_register,
                source_cell,
                target_register,
                target_cell,
            } => {
                let source_register_index = *source_register as usize;
                let target_register_index = *target_register as usize;
                assert!(
                    source_register_index < CELL_REGISTERS_COUNT,
                    "Register out of bounds"
                );
                assert!(
                    target_register_index < CELL_REGISTERS_COUNT,
                    "Register out of bounds"
                );

                let source_pos =
                    get_absolute_cell_pos(cell_index, source_cell.transform(transformation));
                let target_pos =
                    get_absolute_cell_pos(cell_index, target_cell.transform(transformation));

                let source_cell = self.get_cell(source_pos);
                let mut target_cell = self.get_cell(target_pos);

                target_cell.registers[target_register_index] =
                    source_cell.registers[source_register_index];

                self.set_cell(target_pos, target_cell);
            }
            RuleAction::SerRegisterRandomMasked {
                register,
                mask,
                pos,
            } => {
                let register_index = *register as usize;
                assert!(
                    register_index < CELL_REGISTERS_COUNT,
                    "Register out of bounds"
                );

                let pos = get_absolute_cell_pos(cell_index, pos.transform(transformation));
                let mut cell = self.get_cell(pos);
                cell.registers[register_index] =
                    self.center.get_random_value(cell_index) as u32 & *mask;
                self.set_cell(pos, cell);
            }
        }
    }

    #[inline(always)]
    fn get_chunk(&self, side: Side) -> &Chunk {
        match (side.horizontal, side.vertical) {
            (HorizontalSide::Left, VerticalSide::Top) => &self.left_top,
            (HorizontalSide::Right, VerticalSide::Top) => &self.right_top,
            (HorizontalSide::Left, VerticalSide::Bottom) => &self.left_bottom,
            (HorizontalSide::Right, VerticalSide::Bottom) => &self.right_bottom,
            (HorizontalSide::Left, VerticalSide::Center) => &self.left,
            (HorizontalSide::Right, VerticalSide::Center) => &self.right,
            (HorizontalSide::Center, VerticalSide::Top) => &self.top,
            (HorizontalSide::Center, VerticalSide::Bottom) => &self.bottom,
            (HorizontalSide::Center, VerticalSide::Center) => &self.center,
        }
    }

    #[inline(always)]
    fn get_chunk_mut(&mut self, side: Side) -> &mut Chunk {
        match (side.horizontal, side.vertical) {
            (HorizontalSide::Left, VerticalSide::Top) => &mut self.left_top,
            (HorizontalSide::Right, VerticalSide::Top) => &mut self.right_top,
            (HorizontalSide::Left, VerticalSide::Bottom) => &mut self.left_bottom,
            (HorizontalSide::Right, VerticalSide::Bottom) => &mut self.right_bottom,
            (HorizontalSide::Left, VerticalSide::Center) => &mut self.left,
            (HorizontalSide::Right, VerticalSide::Center) => &mut self.right,
            (HorizontalSide::Center, VerticalSide::Top) => &mut self.top,
            (HorizontalSide::Center, VerticalSide::Bottom) => &mut self.bottom,
            (HorizontalSide::Center, VerticalSide::Center) => &mut self.center,
        }
    }

    #[inline(always)]
    fn get_cell(&self, pos: AbsoluteCellPos) -> Cell {
        self.get_chunk(pos.side).get_by_index(pos.index)
    }

    #[inline(always)]
    fn set_cell(&mut self, pos: AbsoluteCellPos, mut cell: Cell) {
        // mark cells as updated
        cell.last_update = self.current_tick;
        self.get_chunk_mut(pos.side).set_by_index(pos.index, cell);
    }

    #[inline(always)]
    fn swap_cells(&mut self, a: AbsoluteCellPos, b: AbsoluteCellPos) {
        let cell_a = self.get_cell(a);
        let cell_b = self.get_cell(b);

        self.set_cell(a, cell_b);
        self.set_cell(b, cell_a);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum HorizontalSide {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum VerticalSide {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Side {
    horizontal: HorizontalSide,
    vertical: VerticalSide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct AbsoluteCellPos {
    index: usize,
    side: Side,
}

impl AbsoluteCellPos {
    fn central(index: usize) -> Self {
        Self {
            index,
            side: Side {
                horizontal: HorizontalSide::Center,
                vertical: VerticalSide::Center,
            },
        }
    }
}

fn get_absolute_cell_pos(cell_index: usize, dst_relative_pos: RelativePos) -> AbsoluteCellPos {
    let cell_pos = CellPos::from_index(cell_index);

    let x = cell_pos.x as i32 + dst_relative_pos.x as i32;
    let y = cell_pos.y as i32 + dst_relative_pos.y as i32;

    let (x, horizontal) = if x < 0 {
        ((x + CHUNK_SIZE as i32) as CellCord, HorizontalSide::Left)
    } else if x < CHUNK_SIZE as i32 {
        (x as CellCord, HorizontalSide::Center)
    } else {
        ((x - CHUNK_SIZE as i32) as CellCord, HorizontalSide::Right)
    };

    let (y, vertical) = if y < 0 {
        ((y + CHUNK_SIZE as i32) as CellCord, VerticalSide::Bottom)
    } else if y < CHUNK_SIZE as i32 {
        (y as CellCord, VerticalSide::Center)
    } else {
        ((y - CHUNK_SIZE as i32) as CellCord, VerticalSide::Top)
    };

    let index = CellPos::new(x, y).to_index();
    let side = Side {
        horizontal,
        vertical,
    };

    AbsoluteCellPos { index, side }
}

#[test]
fn test_get_absolute_cell_pos() {
    let center = get_absolute_cell_pos(0, RelativePos::self_pos());
    assert_eq!(center, AbsoluteCellPos::central(0));

    let left_top = get_absolute_cell_pos(0, RelativePos::new(-1, 1));
    assert_eq!(
        left_top,
        AbsoluteCellPos {
            index: CHUNK_SIZE - 1 + CHUNK_SIZE,
            side: Side {
                horizontal: HorizontalSide::Left,
                vertical: VerticalSide::Center
            }
        }
    );

    let right_bottom = get_absolute_cell_pos(0, RelativePos::new(1, -1));
    assert_eq!(
        right_bottom,
        AbsoluteCellPos {
            index: 1 + CHUNK_SIZE * (CHUNK_SIZE - 1),
            side: Side {
                horizontal: HorizontalSide::Center,
                vertical: VerticalSide::Bottom
            }
        }
    );

    let right_bottom = get_absolute_cell_pos(0, RelativePos::new(3, -1));
    assert_eq!(
        right_bottom,
        AbsoluteCellPos {
            index: 3 + CHUNK_SIZE * (CHUNK_SIZE - 1),
            side: Side {
                horizontal: HorizontalSide::Center,
                vertical: VerticalSide::Bottom
            }
        }
    );

    let left = get_absolute_cell_pos(0, RelativePos::new(-1, 0));
    assert_eq!(
        left,
        AbsoluteCellPos {
            index: CHUNK_SIZE - 1,
            side: Side {
                horizontal: HorizontalSide::Left,
                vertical: VerticalSide::Center
            }
        }
    );
}
