use crate::CellType;
use macroquad::prelude::*;

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct CellState {
    pub ty: CellType,
    pub last_update: u64,
}

impl From<CellType> for CellState {
    fn from(ty: CellType) -> Self {
        Self { ty, last_update: 0 }
    }
}

impl CellState {
    pub fn draw_to_image(&self, image: &mut Image, x: u16, y: u16) {
        let x = x as u32;
        let y = y as u32;

        let color = match self.ty {
            CellType::Empty => BLACK,
            CellType::Wall => DARKGRAY,
            CellType::Sand => GOLD,
            CellType::Water => BLUE,
            CellType::Gas => LIGHTGRAY,
        };

        image.set_pixel(x, image.height as u32 - y - 1, color);
    }
}
