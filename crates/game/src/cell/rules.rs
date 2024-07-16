use crate::*;
use macroquad::rand::ChooseRandom;

// TODO - Implement N rules
/// Check rules in random order
macro_rules! shuffle_order {
    ($swap_seed:expr, $rule0:expr, $rule1:expr) => {{
        let swap_seed: u16 = $swap_seed;

        if swap_seed & 1 == 0 {
            $rule1.or_else(|| $rule0)
        } else {
            $rule0.or_else(|| $rule1)
        }
    }};
    ($swap_seed:expr, $rule0:expr, $rule1:expr, $rule2:expr, $rule3:expr) => {{
        let swap_seed: u16 = $swap_seed;
        let sub_seed = swap_seed >> 1;

        shuffle_order!(
            swap_seed,
            shuffle_order!(sub_seed, $rule0, $rule1),
            shuffle_order!(sub_seed, $rule2, $rule3),
        )
    }};
    ($swap_seed:expr, $rule0:expr, $rule1:expr, $rule3:expr) => {{
        let swap_seed: u16 = $swap_seed;
        match swap_seed % 6 {
            0 => {
                $rule0.or_else(|| $rule1).or_else(|| $rule3)
            }
            1 => {
                $rule0.or_else(|| $rule3).or_else(|| $rule1)
            }
            2 => {
                $rule1.or_else(|| $rule0).or_else(|| $rule3)
            }
            3 => {
                $rule1.or_else(|| $rule3).or_else(|| $rule0)
            }
            4 => {
                $rule3.or_else(|| $rule0).or_else(|| $rule1)
            }
            5 => {
                $rule3.or_else(|| $rule1).or_else(|| $rule0)
            }
            value => unreachable!("Impossible `swap_seed % 6` value: {value}"),
        }
    }};
    ($($rule:expr,)*) => {{
        shuffle_order!($($rule),*)
    }};
}

pub struct RuleContext<'a> {
    pub current_cell_x: u16,
    pub current_cell_y: u16,
    pub current_cell: Cell,
    pub data: &'a Sandbox,
    pub neighbors: &'a CellNeighbors,
    pub swap_seed: u16,
}

impl RuleContext<'_> {
    /// Return swap action with given direction
    pub fn swap(&self, dir: Direction) -> Action {
        let index = self.data.size.relative_dir_to_index_wrapped(
            self.current_cell_x,
            self.current_cell_y,
            dir,
        );
        Action::Swap(index)
    }

    /// Checks rules in random order until one of them returns an action or all rules are checked
    pub fn shuffle_rules(&self, rules: &[Box<dyn Fn() -> Option<Action>>]) -> Option<Action> {
        let mut random_indices = (0..rules.len()).collect::<Vec<_>>();
        random_indices.shuffle();

        for i in random_indices {
            if let Some(action) = rules[i]() {
                return Some(action);
            }
        }

        None
    }

    /// Return swap action if neighbor at given direction is contained in expected neighbors list
    #[inline(always)]
    pub fn swap_on_any_neighbor<const N: usize>(
        &self,
        dir: Direction,
        expected_neighbors: [Cell; N],
    ) -> Option<Action> {
        if expected_neighbors.contains(&self.neighbors.get(dir)) {
            Some(self.swap(dir))
        } else {
            None
        }
    }

    /// Return swap action if neighbor at given direction is equal to expected neighbor
    #[inline(always)]
    pub fn swap_if_neighbor<const N: usize>(
        &self,
        dir: Direction,
        expected_neighbor: Cell,
    ) -> Option<Action> {
        if expected_neighbor == self.neighbors.get(dir) {
            Some(self.swap(dir))
        } else {
            None
        }
    }

    /// Check if neighbor at given direction is equal to expected neighbor
    #[inline(always)]
    pub fn has_neighbor(&self, dir: Direction, expected_neighbor: Cell) -> bool {
        expected_neighbor == self.neighbors.get(dir)
    }

    /// Check if neighbor at given direction is contained in expected neighbors list
    #[inline(always)]
    pub fn any_neighbor<const N: usize>(
        &self,
        dir: Direction,
        expected_neighbors: [Cell; N],
    ) -> bool {
        expected_neighbors.contains(&self.neighbors.get(dir))
    }

    #[inline(always)]
    pub fn into_action(self) -> Action {
        self.handle_sand()
            .or_else(|| self.handle_water())
            .or_else(|| self.handle_gas())
            .or_else(|| self.handle_plant())
            .unwrap_or(Action::Idle)
    }

    fn handle_plant(&self) -> Option<Action> {
        match self.current_cell {
            Cell::Seed => {
                // seed falls down
                self.swap_on_any_neighbor(Direction::Down, [Cell::Empty, Cell::Water])
                    .or_else(|| {
                        // seed grows into plant head if on ground
                        self.any_neighbor(Direction::Down, [Cell::Sand, Cell::Wall])
                            .then_some(Action::Transform(Cell::PlantHead))
                    })
            }
            Cell::PlantHead => {
                // plant head grows into plant stem if there is empty space above
                self.has_neighbor(Direction::Up, Cell::Empty)
                    .then_some(Action::Transform(Cell::PlantStem))
            }
            Cell::Empty => {
                // plant stem grows up
                self.has_neighbor(Direction::Down, Cell::PlantHead)
                    .then_some(Action::Transform(Cell::PlantHead))
            }
            _ => None,
        }
    }

    fn handle_sand(&self) -> Option<Action> {
        if self.current_cell != Cell::Sand {
            return None;
        }

        // Sand falls down
        self.swap_on_any_neighbor(Direction::Down, [Cell::Empty, Cell::Water, Cell::Gas])
            .or_else(|| {
                // Sand falls to the sides
                shuffle_order!(
                    self.swap_seed,
                    self.swap_on_any_neighbor(
                        Direction::DownLeft,
                        [Cell::Empty, Cell::Water, Cell::Gas]
                    ),
                    self.swap_on_any_neighbor(
                        Direction::DownRight,
                        [Cell::Empty, Cell::Water, Cell::Gas]
                    ),
                )
            })
    }

    fn handle_water(&self) -> Option<Action> {
        if self.current_cell != Cell::Water {
            return None;
        }

        // Water falls down
        self.swap_on_any_neighbor(Direction::Down, [Cell::Empty])
            .or_else(|| {
                // Water falls to the sides
                shuffle_order!(
                    self.swap_seed,
                    self.swap_on_any_neighbor(Direction::DownLeft, [Cell::Empty, Cell::Gas]),
                    self.swap_on_any_neighbor(Direction::DownRight, [Cell::Empty, Cell::Gas]),
                )
            })
            .or_else(|| {
                // Water flows to the sides
                shuffle_order!(
                    self.swap_seed,
                    self.swap_on_any_neighbor(Direction::Right, [Cell::Empty, Cell::Gas]),
                    self.swap_on_any_neighbor(Direction::Left, [Cell::Empty, Cell::Gas]),
                )
            })
    }

    fn handle_gas(&self) -> Option<Action> {
        if self.current_cell != Cell::Gas {
            return None;
        }

        // Gas flows in random direction
        shuffle_order!(
            self.swap_seed,
            self.swap_on_any_neighbor(Direction::Up, [Cell::Empty]),
            self.swap_on_any_neighbor(Direction::Down, [Cell::Empty]),
            self.swap_on_any_neighbor(Direction::Right, [Cell::Empty]),
            self.swap_on_any_neighbor(Direction::Left, [Cell::Empty]),
        )
    }
}
