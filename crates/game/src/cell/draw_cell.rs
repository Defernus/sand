use crate::*;
use macroquad::prelude::*;

impl Cell {
    pub fn draw_to_image(&self, image: &mut Image, x: u16, y: u16) {
        let x = x as u32;
        let y = y as u32;

        let color = match *self {
            Self::Empty => BLACK,
            Self::Wall => DARKGRAY,
            Self::Sand => GOLD,
            Self::Water => BLUE,
            Self::Gas => LIGHTGRAY,
            Self::Seed => DARKBROWN,
            Self::PlantHead => PINK,
            Self::PlantStem => BROWN,
            Self::LeafBody => GREEN,
            Self::LeafSide => DARKGREEN,
        };

        image.set_pixel(x, image.height as u32 - y - 1, color);
    }
}
