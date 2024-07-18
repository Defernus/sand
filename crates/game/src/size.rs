use macroquad::texture::Image;

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AreaSize {
    pub width: usize,
    pub height: usize,
}

impl AreaSize {
    pub const fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    pub const fn splat(size: usize) -> Self {
        Self::new(size, size)
    }

    #[inline(always)]
    pub const fn area(&self) -> usize {
        self.width * self.height
    }

    #[inline(always)]
    pub fn index_to_coords(&self, index: usize) -> RelativePosition {
        assert!(
            index < self.area(),
            "Index out of bounds: {} >= {}",
            index,
            self.area()
        );

        let x = (index % self.width) as u32;
        let y = (index / self.width) as u32;

        RelativePosition { x, y }
    }

    #[inline(always)]
    pub fn coords_to_index(&self, position: RelativePosition) -> usize {
        let x = position.x as usize;
        let y = position.y as usize;

        assert!(x < self.width, "X out of bounds: {} >= {}", x, self.width);
        assert!(y < self.height, "Y out of bounds: {} >= {}", y, self.height);

        y * self.width + x
    }

    #[inline(always)]
    pub fn is_intersects_with_rect(self, rect_pos: Position, rect_size: Self) -> bool {
        if rect_pos.x >= self.width as i32 || rect_pos.y >= self.height as i32 {
            return false;
        }

        if rect_pos.x + rect_size.width as i32 <= 0 || rect_pos.y + rect_size.height as i32 <= 0 {
            return false;
        }

        true
    }
}

pub trait GetAreaSize {
    fn get_area_size(&self) -> AreaSize;
}

impl GetAreaSize for Image {
    fn get_area_size(&self) -> AreaSize {
        AreaSize::new(self.width as usize, self.height as usize)
    }
}
