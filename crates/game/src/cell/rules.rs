use crate::*;

// TODO - Implement N rules
/// Check rules in random order
macro_rules! shuffle_order {
    ($swap_seed:expr, $rule0:expr, $rule1:expr) => {
        let swap_seed: u16 = $swap_seed;

        if swap_seed & 1 == 0 {
            $rule1;
            $rule0;
        } else {
            $rule0;
            $rule1;
        }
    };
    ($swap_seed:expr, $rule0:expr, $rule1:expr, $rule2:expr, $rule3:expr) => {
        let swap_seed: u16 = $swap_seed;
        let sub_seed = swap_seed >> 1;

        shuffle_order!(
            swap_seed,
            shuffle_order!(sub_seed, $rule0, $rule1),
            shuffle_order!(sub_seed, $rule2, $rule3),
        );
    };
    ($swap_seed:expr, $rule0:expr, $rule1:expr, $rule3:expr) => {
        let swap_seed: u16 = $swap_seed;
        match swap_seed % 6 {
            0 => {
                $rule0;
                $rule1;
                $rule3;
            }
            1 => {
                $rule0;
                $rule3;
                $rule1;
            }
            2 => {
                $rule1;
                $rule0;
                $rule3;
            }
            3 => {
                $rule1;
                $rule3;
                $rule0;
            }
            4 => {
                $rule3;
                $rule0;
                $rule1;
            }
            5 => {
                $rule3;
                $rule1;
                $rule0;
            }
            value => unreachable!("Impossible `swap_seed % 6` value: {value}"),
        }
    };
    ($($rule:expr,)*) => {
        shuffle_order!($($rule),*)
    };
}

macro_rules! swap_rule {
    ($ctx:expr, $direction:ident, $($current_is:tt),* => $($neighbor_is:tt),*) => {
            let direction = Direction::$direction;
            let neighbor = $ctx.neighbors.get(direction);

            if ($( matches!(neighbor, Cell::$neighbor_is) ||)* false) && ($(matches!($ctx.current_cell, Cell::$current_is) || false )*) {
                let neighbor_index = $ctx.data.size.relative_dir_to_index_wrapped(
                    $ctx.current_cell_x,
                    $ctx.current_cell_y,
                    direction,
                );
                return Some(Action::Swap(neighbor_index));
            }
    };
}

macro_rules! check_neighbors {
    ($ctx:expr, $direction:ident == $($neighbor_is:ident) ||*) => {{
        let direction = Direction::$direction;
        let neighbor = $ctx.neighbors.get(direction);

        $(matches!(neighbor, Cell::$neighbor_is) ||)* false
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
    pub fn into_action(self) -> Action {
        self.handle_sand()
            .or_else(|| self.handle_water())
            .or_else(|| self.handle_gas())
            .or_else(|| self.handle_plant())
            .unwrap_or(Action::Idle)
    }

    fn handle_plant(&self) -> Option<Action> {
        // seed falls down
        swap_rule!(self, Down, Seed => Empty, Water);

        // seed becomes plant head on the ground
        if self.current_cell == Cell::Seed && check_neighbors!(self, Down == Sand || Wall) {
            return Some(Action::Transform(Cell::PlantHead));
        }

        // plant head moves up if there is empty space above and leaves stem behind
        if self.current_cell == Cell::PlantHead && check_neighbors!(self, Up == Empty) {
            return Some(Action::Transform(Cell::PlantStem));
        }
        if self.current_cell == Cell::Empty && check_neighbors!(self, Down == PlantHead) {
            return Some(Action::Transform(Cell::PlantHead));
        }

        None
    }

    fn handle_sand(&self) -> Option<Action> {
        // Sand falls down
        swap_rule!(self, Down, Sand => Empty, Water);

        // Sand falls to the sides
        shuffle_order!(
            self.swap_seed,
            swap_rule!(self, DownLeft, Sand => Empty, Water, Gas),
            swap_rule!(self, DownRight, Sand => Empty, Water, Gas),
        );

        None
    }

    fn handle_water(&self) -> Option<Action> {
        // Water falls down
        swap_rule!(self, Down, Water => Empty);

        // Water falls to the sides
        shuffle_order!(
            self.swap_seed,
            swap_rule!(self, DownLeft, Water => Empty, Gas),
            swap_rule!(self, DownRight, Water => Empty, Gas),
        );

        // Water flows to the sides
        shuffle_order!(
            self.swap_seed,
            swap_rule!(self, Right, Water => Empty, Gas),
            swap_rule!(self, Left, Water => Empty, Gas),
        );

        None
    }

    fn handle_gas(&self) -> Option<Action> {
        // Gas flows in random direction
        shuffle_order!(
            self.swap_seed,
            swap_rule!(self, Up, Gas => Empty),
            swap_rule!(self, Down, Gas => Empty),
            swap_rule!(self, Right, Gas => Empty),
            swap_rule!(self, Left, Gas => Empty),
        );

        None
    }
}
