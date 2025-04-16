use crate::*;
use macroquad::math::Vec2;

pub struct ChunkUpdateContext<'a> {
    pub cells_template: &'a CellsTemplate,
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
    pub delta_time: f32,
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

impl<'a> ChunkUpdateContext<'a> {
    /// This function will process only central chunk, but it will also access the surrounding
    /// chunks and in some cases modify them (e.g. sand falling)
    pub fn process(&mut self) {
        self.center.set_should_update(false);

        let update_order_mask = ::rand::random::<u32>();
        let chunk_area_mask = CHUNK_AREA - 1;

        for index in 0..CHUNK_AREA {
            // randomize index
            let index = (index ^ update_order_mask as usize) & chunk_area_mask;

            self.update_cell(index);
        }

        if !self.center.particles.is_empty() {
            self.center.set_should_redraw(true);
            self.center.set_should_update(true);
        }
        for particle_index in (0..self.center.particles.len()).rev() {
            self.update_particle(particle_index);
        }

        // If chunk updated next frame we need to update surrounding chunks
        if self.center.should_update_neighbor() {
            self.bottom.set_should_update(true);
            self.top.set_should_update(true);
            self.left.set_should_update(true);
            self.right.set_should_update(true);
            self.left_top.set_should_update(true);
            self.right_top.set_should_update(true);
            self.left_bottom.set_should_update(true);
            self.right_bottom.set_should_update(true);
        }
    }

    /// Update particle and move it to another chunk if needed
    ///
    /// NOTE: This may affect particles order with indexes greater or equal to `particle_index`.
    /// **Iterate over particles in reverse order!**
    fn update_particle(&mut self, particle_index: usize) {
        debug_assert!(particle_index < self.center.particles.len());

        // TODO place technical cell placeholder on particle position to avoid conflict with other
        // particles and cells (e.g. sand falling on top of the particle)

        let particle = &mut self.center.particles[particle_index];

        let start_cell_pos = particle.get_cell_pos().unwrap_or_else(|| {
            panic!(
                "Particle is outside of the chunk. age: {}, pos: {:?}",
                particle.age, particle.in_chunk_pos
            )
        });

        particle.update_pos(self.delta_time);

        let mut prev_pos = AbsoluteCellPos::central(start_cell_pos.to_index());

        let target_poss = AbsoluteCellPos::from_vec(particle.in_chunk_pos);

        if prev_pos == target_poss {
            // particle didn't move
            return;
        }

        while prev_pos != target_poss {
            let next_pos = prev_pos.move_towards(target_poss);

            let cell = self.get_cell(next_pos);
            let cell_meta = self.cells_template.get_cell_meta(cell.id);

            let is_collided = !cell_meta.replaceable_by_particles;

            if is_collided {
                // replace particle with cell if collided
                let particle = self.center.particles.swap_remove(particle_index);
                let particle_cell = self.cells_template.get_cell_meta(particle.cell_id);
                self.set_cell(prev_pos, particle_cell.init());

                return;
            }

            prev_pos = next_pos;
        }

        // if particle is outside of the central chunk we need to move it to the new chunk
        if !target_poss.is_in_central() {
            let mut particle = self.center.particles.swap_remove(particle_index);
            particle.in_chunk_pos -= target_poss.side.chunk_pos_offset_vec();

            if particle.get_cell_pos().is_none() {
                // TODO fix this: some time particles has coordinates right on the chunk boundary
                // (e.g. x = 128.0). This is temporary fix to avoid panic.
                println!(
                    "!!!!!!!!!! BAD PARTICLE POSITION age: {}, pos: {:?}, target_pos: {:?}",
                    particle.age, particle.in_chunk_pos, target_poss
                );
                particle.in_chunk_pos = target_poss.cell_pos().to_vec();
            }

            self.get_chunk_mut(target_poss.side)
                .particles
                .push(particle);
        }
    }

    #[inline(always)]
    fn update_cell(&mut self, cell_index: usize) {
        let cell = self.center.get_by_index(cell_index);
        if cell.last_update == self.current_tick {
            return;
        }
        let cell_config = cell.meta(self.cells_template);

        self.try_apply_rule(
            &cell_config.rule,
            cell_index,
            RelativeTransformation::default(),
        );
    }

    fn calc_value(
        &self,
        value: &ConditionArg,
        cell_index: usize,
        transformation: RelativeTransformation,
    ) -> u32 {
        match value {
            ConditionArg::Value(v) => *v,
            ConditionArg::Register { pos, register } => {
                let pos = get_absolute_cell_pos(cell_index, pos.transform(transformation));
                self.get_cell(pos).registers[*register as usize]
            }
        }
    }

    fn try_apply_rule(
        &mut self,
        rule: &CellRule,
        cell_index: usize,
        transformation: RelativeTransformation,
    ) -> bool {
        match rule {
            CellRule::FirstSuccess(list) => {
                for rule in list {
                    if self.try_apply_rule(rule, cell_index, transformation) {
                        return true;
                    }
                }

                false
            }
            CellRule::If {
                condition,
                action,
                else_action,
            } => {
                if self.check_condition(condition, cell_index, transformation) {
                    return self.try_apply_rule(action.as_ref(), cell_index, transformation);
                }

                if let Some(else_action) = else_action {
                    return self.try_apply_rule(else_action, cell_index, transformation);
                }

                false
            }
            CellRule::RandomPair(pair) => {
                let (rule_a, rule_b) = pair.as_ref();
                let random_value = self.center.get_random_value(cell_index);

                if random_value & 1 == 0 {
                    self.apply_rule_pair(cell_index, rule_a, transformation, rule_b, transformation)
                } else {
                    self.apply_rule_pair(cell_index, rule_b, transformation, rule_a, transformation)
                }
            }
            CellRule::ApplyAndContinue(rule) => {
                self.try_apply_rule(rule, cell_index, transformation);
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
                        rule,
                        transformation,
                        rule,
                        transformation.mirror_x(),
                    )
                } else {
                    self.apply_rule_pair(
                        cell_index,
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
                        rule,
                        transformation,
                        rule,
                        transformation.mirror_y(),
                    )
                } else {
                    self.apply_rule_pair(
                        cell_index,
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
                        rule,
                        transformation,
                        rule,
                        transformation.mirror_diagonal(),
                    )
                } else {
                    self.apply_rule_pair(
                        cell_index,
                        rule,
                        transformation.mirror_diagonal(),
                        rule,
                        transformation,
                    )
                }
            }
            CellRule::TryAll(list) => {
                for action in list {
                    self.try_apply_rule(action, cell_index, transformation);
                }

                true
            }
            CellRule::InitCell { pos, cell_id } => {
                let pos = get_absolute_cell_pos(cell_index, pos.transform(transformation));
                self.set_cell(pos, Cell::new(&self.cells_template, *cell_id));

                true
            }
            CellRule::SwapWith { pos } => {
                let pos = get_absolute_cell_pos(cell_index, pos.transform(transformation));
                self.swap_cells(AbsoluteCellPos::central(cell_index), pos);

                true
            }
            CellRule::IncrementRegister { register, pos } => {
                let register_index = *register as usize;
                assert!(
                    register_index < CELL_REGISTERS_COUNT,
                    "Register out of bounds"
                );
                let pos = get_absolute_cell_pos(cell_index, pos.transform(transformation));
                let mut cell = self.get_cell(pos);
                cell.registers[register_index] = cell.registers[register_index].wrapping_add(1);
                self.set_cell(pos, cell);

                true
            }
            CellRule::DecrementRegister { register, pos } => {
                let register_index = *register as usize;
                assert!(
                    register_index < CELL_REGISTERS_COUNT,
                    "Register out of bounds"
                );
                let pos = get_absolute_cell_pos(cell_index, pos.transform(transformation));
                let mut cell = self.get_cell(pos);
                cell.registers[register_index] = cell.registers[register_index].wrapping_sub(1);
                self.set_cell(pos, cell);

                true
            }
            CellRule::SetRegister {
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

                true
            }
            CellRule::MoveRegister {
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

                true
            }
            CellRule::SerRegisterRandomMasked {
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

                true
            }
            CellRule::MirrorXIf { condition, rule } => {
                if self.check_condition(condition, cell_index, transformation) {
                    return self.try_apply_rule(rule, cell_index, transformation.mirror_x());
                }

                false
            }
            CellRule::MirrorYIf { condition, rule } => {
                if self.check_condition(condition, cell_index, transformation) {
                    return self.try_apply_rule(rule, cell_index, transformation.mirror_y());
                }

                false
            }
            CellRule::MirrorDiagonalIf { condition, rule } => {
                if self.check_condition(condition, cell_index, transformation) {
                    return self.try_apply_rule(rule, cell_index, transformation.mirror_diagonal());
                }

                false
            }
            CellRule::Idle => true,
        }
    }

    fn apply_rule_pair(
        &mut self,
        cell_index: usize,
        a: &CellRule,
        a_transformation: RelativeTransformation,
        b: &CellRule,
        b_transformation: RelativeTransformation,
    ) -> bool {
        if self.try_apply_rule(a, cell_index, a_transformation) {
            return true;
        }

        self.try_apply_rule(b, cell_index, b_transformation)
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
            RuleCondition::BinaryOp { op, a, b } => {
                let value_a = self.calc_value(a, cell_index, transformation);
                let value_b = self.calc_value(b, cell_index, transformation);

                match op {
                    ConditionBinaryOp::Eq => value_a == value_b,
                    ConditionBinaryOp::NotEq => value_a != value_b,
                    ConditionBinaryOp::Less => value_a < value_b,
                    ConditionBinaryOp::LessEq => value_a <= value_b,
                    ConditionBinaryOp::Greater => value_a > value_b,
                    ConditionBinaryOp::GreaterEq => value_a >= value_b,
                }
            }
            RuleCondition::Always => true,
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

impl Side {
    fn chunk_pos_offset(self) -> (i16, i16) {
        let x_offset = match self.horizontal {
            HorizontalSide::Left => -(CHUNK_SIZE as i16),
            HorizontalSide::Center => 0,
            HorizontalSide::Right => CHUNK_SIZE as i16,
        };
        let y_offset = match self.vertical {
            VerticalSide::Top => CHUNK_SIZE as i16,
            VerticalSide::Center => 0,
            VerticalSide::Bottom => -(CHUNK_SIZE as i16),
        };

        (x_offset, y_offset)
    }

    fn chunk_pos_offset_vec(self) -> Vec2 {
        let (x_offset, y_offset) = self.chunk_pos_offset();
        Vec2::new(x_offset as f32, y_offset as f32)
    }
}

/// Absolute cell position in the update region
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct AbsoluteCellPos {
    index: usize,
    side: Side,
}

impl AbsoluteCellPos {
    #[inline(always)]
    fn central(index: usize) -> Self {
        Self {
            index,
            side: Side {
                horizontal: HorizontalSide::Center,
                vertical: VerticalSide::Center,
            },
        }
    }

    fn cell_pos(self) -> CellPos {
        CellPos::from_index(self.index)
    }

    #[inline(always)]
    fn is_in_central(&self) -> bool {
        self.side.horizontal == HorizontalSide::Center && self.side.vertical == VerticalSide::Center
    }

    #[inline(always)]
    fn from_vec(pos: Vec2) -> Self {
        let x = if pos.x < 0.0 {
            pos.x.floor() as i16
        } else {
            pos.x as i16
        };
        let y = if pos.y < 0.0 {
            pos.y.floor() as i16
        } else {
            pos.y as i16
        };

        Self::from_cords(x, y)
    }

    #[inline(always)]
    fn from_cords(x: i16, y: i16) -> Self {
        const CHUNK_SIZE_I: i16 = CHUNK_SIZE as i16;
        assert!(
            x > -CHUNK_SIZE_I && x < CHUNK_SIZE_I * 2,
            "x out of bounds: {}",
            x
        );
        assert!(
            y > -CHUNK_SIZE_I && y < CHUNK_SIZE_I * 2,
            "y out of bounds: {}",
            y
        );

        let (norm_x, h_side) = if x < 0 {
            (x + CHUNK_SIZE_I, HorizontalSide::Left)
        } else if x < CHUNK_SIZE_I {
            (x, HorizontalSide::Center)
        } else {
            (x - CHUNK_SIZE_I, HorizontalSide::Right)
        };
        let (norm_y, v_size) = if y < 0 {
            (y + CHUNK_SIZE_I, VerticalSide::Bottom)
        } else if y < CHUNK_SIZE_I {
            (y, VerticalSide::Center)
        } else {
            (y - CHUNK_SIZE_I, VerticalSide::Top)
        };

        AbsoluteCellPos {
            index: CellPos::new(norm_x as CellCord, norm_y as CellCord).to_index(),
            side: Side {
                horizontal: h_side,
                vertical: v_size,
            },
        }
    }

    #[inline(always)]
    fn to_cord(self) -> (i16, i16) {
        let cell_pos = CellPos::from_index(self.index);
        let x = cell_pos.x as i16;
        let y = cell_pos.y as i16;

        let x = match self.side.horizontal {
            HorizontalSide::Left => x - CHUNK_SIZE as i16,
            HorizontalSide::Center => x,
            HorizontalSide::Right => x + CHUNK_SIZE as i16,
        };

        let y = match self.side.vertical {
            VerticalSide::Top => y + CHUNK_SIZE as i16,
            VerticalSide::Center => y,
            VerticalSide::Bottom => y - CHUNK_SIZE as i16,
        };

        (x, y)
    }

    #[inline(always)]
    fn move_towards(self, target: Self) -> Self {
        let (x, y) = self.to_cord();
        let (target_x, target_y) = target.to_cord();

        let new_x = move_towards(x, target_x);
        let new_y = move_towards(y, target_y);

        AbsoluteCellPos::from_cords(new_x, new_y)
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

/// Move value towards target by 1
#[inline(always)]
fn move_towards(value: i16, target: i16) -> i16 {
    if value < target {
        value + 1
    } else if value > target {
        value - 1
    } else {
        value
    }
}
