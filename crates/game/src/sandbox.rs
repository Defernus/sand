use crate::*;
use macroquad::prelude::*;

#[derive(Debug, Clone)]
pub struct Sandbox {
    pub size: SandboxSize,
    pub cells: Vec<Cell>,
}

impl Sandbox {
    #[inline(always)]
    pub fn new(width: u16, height: u16) -> Self {
        let size = SandboxSize { width, height };
        Self {
            size,
            cells: vec![Cell::default(); size.area()],
        }
    }

    #[inline(always)]
    pub fn size(&self) -> SandboxSize {
        self.size
    }

    pub fn spawn_cells(&mut self, cell: Cell, x: i16, y: i16, radius: usize, density: f32) {
        let r = radius as i16;
        for i in -r..=r {
            for j in -r..=r {
                if i * i + j * j <= r * r as i16 && ::rand::random::<f32>() < density {
                    self.set_wrapper(x + i, y + j, cell);
                }
            }
        }
    }

    #[inline(always)]
    pub fn set(&mut self, x: u16, y: u16, cell: Cell) {
        let index = self.size.coords_to_index(x, y);
        self.cells[index] = cell;
    }

    #[inline(always)]
    pub fn draw_to_image(&self, image: &mut Image) {
        for (index, cell) in self.cells.iter().enumerate() {
            let (x, y) = self.size.index_to_coords(index);

            cell.draw_to_image(image, x, y);
        }
    }

    #[inline(always)]
    pub fn tick(&mut self) {
        let cells_actions = self.prepare_actions();
        self.apply_actions(cells_actions);
    }

    #[inline(always)]
    pub fn get(&self, x: u16, y: u16) -> Option<&Cell> {
        self.cells.get(self.size.coords_to_index(x, y))
    }

    /// Set cell at absolute coordinates wrapping around the edges.
    pub fn set_wrapper(&mut self, x: i16, y: i16, cell: Cell) {
        let x = relative_to_abs_wrapped(x as u16, 0, self.size.width);
        let y = relative_to_abs_wrapped(y as u16, 0, self.size.height);

        let index = self.size.coords_to_index(x, y);

        self.cells[index] = cell;
    }

    /// Get cell relative to the origin cell, wrapping around the edges.
    pub fn get_relative_wrapped(
        &self,
        origin_x: u16,
        origin_y: u16,
        offset_x: i16,
        offset_y: i16,
    ) -> Cell {
        let x = relative_to_abs_wrapped(origin_x, offset_x, self.size.width);
        let y = relative_to_abs_wrapped(origin_y, offset_y, self.size.height);

        *self.get(x as u16, y as u16).unwrap()
    }

    pub fn get_at_dir(&self, origin_x: u16, origin_y: u16, dir: Direction) -> Cell {
        let (offset_x, offset_y) = dir.to_offset();
        self.get_relative_wrapped(origin_x, origin_y, offset_x, offset_y)
    }

    pub fn get_neighbors(&self, x: u16, y: u16) -> CellNeighbors {
        CellNeighbors {
            top: self.get_at_dir(x, y, Direction::Up),
            bottom: self.get_at_dir(x, y, Direction::Down),
            left: self.get_at_dir(x, y, Direction::Left),
            right: self.get_at_dir(x, y, Direction::Right),
            top_left: self.get_at_dir(x, y, Direction::UpLeft),
            top_right: self.get_at_dir(x, y, Direction::UpRight),
            bottom_left: self.get_at_dir(x, y, Direction::DownLeft),
            bottom_right: self.get_at_dir(x, y, Direction::DownRight),
        }
    }
}

pub struct CellNeighbors {
    pub top: Cell,
    pub bottom: Cell,
    pub left: Cell,
    pub right: Cell,
    pub top_left: Cell,
    pub top_right: Cell,
    pub bottom_left: Cell,
    pub bottom_right: Cell,
}

impl CellNeighbors {
    #[inline(always)]
    pub fn get(&self, dir: Direction) -> Cell {
        match dir {
            Direction::Up => self.top,
            Direction::Down => self.bottom,
            Direction::Left => self.left,
            Direction::Right => self.right,
            Direction::UpLeft => self.top_left,
            Direction::UpRight => self.top_right,
            Direction::DownLeft => self.bottom_left,
            Direction::DownRight => self.bottom_right,
        }
    }
}

/// Convert relative cord to absolute cord, wrapping around the edge.
pub fn relative_to_abs_wrapped(origin: u16, offset: i16, size: u16) -> u16 {
    let cord = origin as i16 + offset;
    let size = size as i16;
    let cord = (cord % size + size) % size;

    cord as u16
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SandboxSize {
    pub width: u16,
    pub height: u16,
}

impl SandboxSize {
    pub fn area(&self) -> usize {
        self.width as usize * self.height as usize
    }

    pub fn index_to_coords(&self, index: usize) -> (u16, u16) {
        let x = index % (self.width as usize);
        let y = index / (self.width as usize);

        (x as u16, y as u16)
    }

    pub fn coords_to_index(&self, x: u16, y: u16) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }

    pub fn relative_dir_wrapped(&self, x: u16, y: u16, dir: Direction) -> (u16, u16) {
        let (offset_x, offset_y) = dir.to_offset();
        let x = relative_to_abs_wrapped(x, offset_x, self.width);
        let y = relative_to_abs_wrapped(y, offset_y, self.height);

        (x, y)
    }

    pub fn relative_dir_to_index_wrapped(&self, x: u16, y: u16, dir: Direction) -> usize {
        let (x, y) = self.relative_dir_wrapped(x, y, dir);
        self.coords_to_index(x, y)
    }
}
