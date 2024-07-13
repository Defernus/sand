use crate::*;

impl Sandbox {
    pub fn with_bottom_wall(mut self) -> Self {
        self.set_bottom_wall();
        self
    }

    pub fn set_bottom_wall(&mut self) {
        for x in 0..self.size().width {
            self.set(x, 0, Cell::Wall);
        }
    }
}
